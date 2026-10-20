//! Rendering. `areas` is the single source of truth for the layout, so the
//! runtime places the embedded video exactly where `draw` leaves room for it.

use std::time::Duration;

use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Flex, Layout, Position, Rect};
use ratatui::style::{Color, Modifier, Style, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Clear, List, ListItem, ListState, Paragraph};

use crate::app::{
    Confirm, ContextMenu, Focus, Group, LOCAL_LATER, ListKind, Listing, MenuRow, Prompt, Section, Setting, State, Tab,
    YOUTUBE_LATER, language_label,
};
use crate::config::{BorderStyle, IconSet, donations};
use crate::doctor::{Manager, Tool};
use crate::domain::{Channel, Download, DownloadState, Live, Playlist, Size, TabId, Video, clock, date_labels};
use crate::i18n::Lang;
use crate::media::{LocalFile, MediaKind};
use crate::player::Info;
use crate::sponsorblock::Segment;
use crate::theme::{self, TOKYO_NIGHT};
use crate::update::{Install, Progress, Replace};
#[cfg(test)]
use crate::youtube::Watch;
use crate::youtube::{Items, Rating, compact};
use crate::{keymap, session};

pub const SIDEBAR_WIDTH: u16 = 26;
/// Columns a list keeps beside its preview; narrower, the preview goes
/// under the list.
const LIST_MIN_WIDTH: u16 = 50;
/// Rows a list keeps (borders included) above a stacked preview.
const LIST_MIN_ROWS: u16 = 10;
/// Rows a stacked preview takes when there is room, and the fewest it is
/// shown with (fewer and it is left out rather than squeezed).
const PREVIEW_STACKED_ROWS: u16 = 16;
const PREVIEW_STACKED_MIN_ROWS: u16 = 6;
const CHANNEL_WIDTH: usize = 20;
/// Columns of the channel lists; the @handle goes last.
const CHANNEL_NAME_WIDTH: usize = 26;
const FOLLOWERS_WIDTH: usize = 24;
/// Width of the date column: `"10/10/26 hoy   "`.
const DATE_WIDTH: usize = 14;

// Drawn in Tokyo Night; `theme::recolor` shows the frame in the Omarchy
// theme's colours when colour sync is on.
/// Text on a coloured background (the focused button).
const BG: Color = TOKYO_NIGHT.background;
const FG: Color = TOKYO_NIGHT.fg;
const MUTED: Color = TOKYO_NIGHT.muted;
const HIGHLIGHT: Color = TOKYO_NIGHT.highlight;
/// Separator lines inside dialogs: visible but quieter than text.
const LINE: Color = TOKYO_NIGHT.line;
const BLUE: Color = TOKYO_NIGHT.blue;
const CYAN: Color = TOKYO_NIGHT.cyan;
const MAGENTA: Color = TOKYO_NIGHT.magenta;
/// The menu row of what the main area shows, when the cursor is elsewhere.
const SHOWN: Color = MAGENTA;
const GREEN: Color = TOKYO_NIGHT.green;
const YELLOW: Color = TOKYO_NIGHT.yellow;
const RED: Color = TOKYO_NIGHT.red;
const TEAL: Color = TOKYO_NIGHT.teal;
const PURPLE: Color = TOKYO_NIGHT.purple;

/// One colour per menu entry, in `Section::ALL` order.
/// The menu's cursor row, sections and tabs alike: the theme's accent.
const SELECTED: Color = crate::theme::ACCENT_DRAWN;

pub struct Icons {
    /// In `Section::ALL` order.
    pub sections: [&'static str; 18],
    /// In `Setting::ALL` order.
    pub settings: [&'static str; 37],
    pub logo: &'static str,
    pub tab: &'static str,
    pub live: &'static str,
    pub upcoming: &'static str,
    pub later: &'static str,
    /// A channel I watch a lot.
    pub favorite: &'static str,
    pub subscribed: &'static str,
    pub liked: &'static str,
    pub disliked: &'static str,
    pub user: &'static str,
    pub language: &'static str,
    pub volume: &'static str,
    pub muted: &'static str,
    /// A video already watched.
    pub watched: &'static str,
    /// About: the latest version is installed (also: an update just installed).
    pub up_to_date: &'static str,
    /// A newer version is out.
    pub update: &'static str,
    /// The update is downloading.
    pub downloading: &'static str,
    /// More tabs above / below the menu's visible ones.
    pub more_above: &'static str,
    pub more_below: &'static str,
    /// The update failed.
    pub failed: &'static str,
    /// A file moved to the trash; also anything removed or deleted.
    pub trash: &'static str,
    /// Subscribed to a channel; new videos from my channels.
    pub notify: &'static str,
    /// Unsubscribed from a channel.
    pub unsubscribed: &'static str,
    /// Playback resumes where it was left.
    pub resume: &'static str,
    /// A SponsorBlock segment skipped.
    pub skip: &'static str,
    /// A video closed, a download cancelled.
    pub stop: &'static str,
    /// Sound only.
    pub audio: &'static str,
    /// Saved to tubevibe's Watch Later.
    pub saved: &'static str,
    /// Something needs a look (config.toml, missing tools).
    pub warning: &'static str,
    /// A thumbnail on its way.
    pub waiting: &'static str,
    /// The donation line on About.
    pub donate: &'static str,
    /// YouTube's AI summary: its heading in the preview and the video tab.
    pub summary: &'static str,
    /// A row marked with Space (My Videos, My Music, Downloads). One cell.
    pub marked: &'static str,
    /// The login in use, in the list of browsers.
    pub current: &'static str,
    /// System Check: an optional tool that is not installed.
    pub optional: &'static str,
}

impl Icons {
    fn section(&self, section: Section) -> &'static str {
        self.sections[Section::ALL.iter().position(|s| *s == section).unwrap_or(0)]
    }

    fn setting(&self, setting: Setting) -> &'static str {
        self.settings[Setting::ALL.iter().position(|s| *s == setting).unwrap_or(0)]
    }
}

const EMOJI: Icons = Icons {
    sections: [
        "🏠", "🕒", "📺", "📜", "📚", "⭐", "🔎", "📡", "🎯", "🧭", "💡", "🔖", "📼", "🎵", "📥", "🔧", "💜", "🚪",
    ],
    settings: [
        "👤", "📄", "🔄", "🚪", "🌐", "💬", "📐", "🎬", "🎧", "⏩", "👆", "🔤", "😀", "🔲", "📏", "🔭", "💭", "📑",
        "🎨", "📷", "🐭", "🎦", "✨", "🌊", "💥", "📼", "🎵", "📁", "📌", "🩺", "👀", "🕒", "🩳", "🔔", "⭐", "🆙",
        "📝",
    ],
    // Plain ▶ (no U+FE0F): one cell, drawn in the logo's red, not as the orange emoji.
    logo: "▶",
    tab: "🎬",
    live: "🔴",
    upcoming: "⏳",
    later: "🕒",
    favorite: "⭐",
    subscribed: "✓",
    liked: "👍",
    disliked: "👎",
    user: "👤",
    language: "🌐",
    volume: "🔊",
    muted: "🔇",
    watched: "👀",
    up_to_date: "✅",
    update: "⬆",
    downloading: "⬇",
    more_above: "🔼",
    more_below: "🔽",
    failed: "❌",
    trash: "🗑",
    notify: "🔔",
    unsubscribed: "🔕",
    resume: "⏯",
    skip: "⏩",
    stop: "⏹",
    audio: "🎧",
    saved: "🔖",
    warning: "⚠",
    waiting: "⏳",
    donate: "☕",
    summary: "✦",
    marked: "✖",
    current: "●",
    optional: "➖",
};

const NERD: Icons = Icons {
    sections: [
        "\u{f015}",
        "\u{f017}",
        "\u{f05c6}",
        "\u{f1da}",
        "\u{f0cb}",
        "\u{f005}",
        "\u{f002}",
        "\u{f2ce}",
        "\u{f140}",
        "\u{f14e}",
        "\u{f0eb}",
        "\u{f02e}",
        "\u{f03d}",
        "\u{f001}",
        "\u{f019}",
        "\u{f013}",
        "\u{f004}",
        "\u{f08b}",
    ],
    settings: [
        "\u{f007}",
        "\u{f15b}",
        "\u{f021}",
        "\u{f08b}",
        "\u{f1ab}",
        "\u{f075}",
        "\u{f1de}",
        "\u{f008}",
        "\u{f025}",
        "\u{f051}",
        "\u{f144}",
        "\u{f0ac}",
        "\u{f118}",
        "\u{f00ce}",
        // nf-fa-bars, nf-fa-eye, nf-fa-comments: the menu, preview and comments widths.
        "\u{f0c9}",
        "\u{f06e}",
        "\u{f086}",
        "\u{f2d2}",
        "\u{f1fc}",
        "\u{f03e}",
        "\u{f245}",
        "\u{f04b}",
        "\u{f0d0}",
        "\u{f0e7}",
        "\u{f0a1}",
        "\u{f03d}",
        "\u{f001}",
        "\u{f07b}",
        "\u{f02c}",
        "\u{f0f1}",
        "\u{f06e}",
        "\u{f017}",
        "\u{f10b}",
        "\u{f0f3}",
        "\u{f005}",
        "\u{f01b}",
        "\u{f044}",
    ],
    logo: "\u{f16a}",
    tab: "\u{f144}",
    live: "\u{f111}",
    upcoming: "\u{f252}",
    later: "\u{f017}",
    favorite: "\u{f005}",
    subscribed: "\u{f00c}",
    liked: "\u{f164}",
    disliked: "\u{f165}",
    user: "\u{f007}",
    language: "\u{f1ab}",
    volume: "\u{f028}",
    muted: "\u{f026}",
    watched: "\u{f06e}",
    // nf-fa-check_circle, arrow_circle_up, arrow_circle_down, times_circle.
    up_to_date: "\u{f058}",
    update: "\u{f0aa}",
    downloading: "\u{f0ab}",
    // nf-fa-angle_up, angle_down.
    more_above: "\u{f106}",
    more_below: "\u{f107}",
    failed: "\u{f057}",
    trash: "\u{f1f8}",
    // nf-fa-bell, bell_slash, nf-md-play_pause, nf-fa-forward, stop,
    // headphones, bookmark, warning, hourglass_half, coffee.
    notify: "\u{f0f3}",
    unsubscribed: "\u{f1f6}",
    resume: "\u{f040e}",
    skip: "\u{f04e}",
    stop: "\u{f04d}",
    audio: "\u{f025}",
    saved: "\u{f02e}",
    warning: "\u{f071}",
    waiting: "\u{f252}",
    donate: "\u{f0f4}",
    // nf-md-creation: sparkles.
    summary: "\u{f0674}",
    // nf-fa-times, circle, minus_circle.
    marked: "\u{f00d}",
    current: "\u{f111}",
    optional: "\u{f056}",
};

const UNICODE: Icons = Icons {
    sections: ["⌂", "◷", "▣", "↺", "≣", "★", "⌕", "⌁", "◈", "◎", "✧", "♦", "▷", "♫", "⇩", "⚙", "♥", "⏻"],
    settings: [
        "☺", "▤", "↻", "⏻", "⇄", "≡", "▦", "▶", "♪", "»", "▸", "⚑", "☻", "▢", "◧", "◨", "▥", "❐", "◐", "▨", "➚", "◆",
        "✦", "≈", "✸", "▷", "♫", "▭", "#", "✚", "◉", "◷", "▯", "♪", "★", "↑", "✎",
    ],
    logo: "▶",
    tab: "▶",
    live: "●",
    upcoming: "…",
    later: "◷",
    favorite: "★",
    subscribed: "✓",
    liked: "▲",
    disliked: "▼",
    user: "☺",
    language: "⇄",
    volume: "♪",
    muted: "✕",
    watched: "◉",
    up_to_date: "✔",
    update: "↑",
    downloading: "↓",
    more_above: "▴",
    more_below: "▾",
    failed: "✘",
    trash: "✕",
    // Text symbols only: none of these turns into a colour emoji.
    notify: "⚑",
    unsubscribed: "⚐",
    resume: "▸",
    skip: "»",
    stop: "■",
    audio: "♪",
    saved: "♦",
    warning: "△",
    waiting: "◌",
    donate: "♥",
    summary: "✦",
    marked: "✗",
    current: "●",
    optional: "·",
};

pub fn icons(set: IconSet) -> &'static Icons {
    match set {
        IconSet::Emoji => &EMOJI,
        IconSet::Nerd => &NERD,
        IconSet::Unicode => &UNICODE,
    }
}

/// Rendering options that do not belong to the app state.
pub struct View {
    pub embedded: bool,
    /// Unix time, for "hace 3 días".
    pub now: i64,
    /// How long the current list has been loading.
    pub loading: Option<Duration>,
    /// How long the video of the active tab has been loading, until it plays.
    pub opening: Option<Duration>,
    /// The preview panel leaves room for a thumbnail (the runtime draws it).
    pub thumbnails: bool,
    /// What mpv reports about the video of the active tab.
    pub playing: Option<Info>,
}

const SPINNER: [&str; 10] = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

/// `"⠹ Cargando Suscripciones… 3 s"`.
fn loading_text(title: &str, elapsed: Duration, lang: Lang) -> String {
    let frame = SPINNER[(elapsed.as_millis() / 100) as usize % SPINNER.len()];
    format!("{frame} {} {title}… {} s", lang.pick("Cargando", "Loading"), elapsed.as_secs())
}

/// A block bouncing inside `width` cells: progress without a known total.
fn bouncing_bar(width: usize, elapsed: Duration) -> String {
    let block = 6.min(width);
    let span = width - block;
    let step = (elapsed.as_millis() / 60) as usize % (2 * span.max(1));
    let left = if step <= span { step } else { 2 * span - step };
    format!("{}{}{}", "░".repeat(left), "█".repeat(block), "░".repeat(span - left))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Areas {
    pub sidebar: Option<Rect>,
    /// The main pane; open tabs are listed in the menu only (no tab bar).
    pub content: Rect,
    pub status: Rect,
    /// Keys that work right now, with what they do.
    pub hints: Rect,
}

pub fn areas(area: Rect, sidebar: bool) -> Areas {
    areas_with(area, if sidebar { SIDEBAR_WIDTH } else { 0 })
}

/// The screen's areas with the menu as wide as `state` says (Menu Width).
pub fn screen_areas(area: Rect, state: &State) -> Areas {
    areas_with(area, if state.sidebar { state.config.ui.menu_width } else { 0 })
}

/// `menu` columns of menu (0: none), never more than half the screen.
fn areas_with(area: Rect, menu: u16) -> Areas {
    let [body, status, hints] =
        Layout::vertical([Constraint::Fill(1), Constraint::Length(1), Constraint::Length(1)]).areas(area);
    let width = menu.min(area.width / 2);
    let [side, content] = Layout::horizontal([Constraint::Length(width), Constraint::Fill(1)]).areas(body);
    Areas { sidebar: (menu > 0).then_some(side), content, status, hints }
}

/// What is under the mouse pointer.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Spot {
    /// A section of the menu, in `Section::ALL` order.
    Section(usize),
    /// A tab in the menu (1 and up); 0 is the browsing view.
    Tab(usize),
    /// A row of the current list.
    Row(usize),
    /// The picture of the active video tab.
    Video,
    /// A second on the progress bar.
    Progress(u32),
    /// A line of the right-click menu.
    MenuItem(usize),
}

/// What is at cell (`x`, `y`), laid out as [`draw`] does it.
pub fn spot(state: &State, playing: Option<&Info>, screen: Rect, x: u16, y: u16) -> Option<Spot> {
    let at = Position::new(x, y);
    if state.splash || state.help || state.confirm.is_some() {
        return None;
    }
    if let Some(menu) = &state.context {
        let inner = Block::bordered().inner(context_area(menu, screen));
        return inner.contains(at).then(|| Spot::MenuItem(usize::from(y - inner.y)));
    }
    if state.fullscreen {
        return Some(Spot::Video);
    }
    let areas = screen_areas(screen, state);
    if let Some(side) = areas.sidebar.filter(|a| a.contains(at)) {
        let inner = Block::bordered().inner(side);
        let line = usize::from(y.checked_sub(inner.y)?);
        return sidebar_items(state, icons(state.config.ui.icons), inner).get(line)?.1;
    }
    if !areas.content.contains(at) {
        return None;
    }
    if let Some(tab) = state.active_tab() {
        if let Some((bar, duration)) = playing.and_then(|info| progress_bar(info, areas.content))
            && bar.contains(at)
        {
            let fraction = (f64::from(x - bar.x) + 0.5) / f64::from(bar.width);
            return Some(Spot::Progress((fraction * duration) as u32));
        }
        if state.details_shown(tab) {
            // The video is elsewhere (or not shown): here are its details and the related list.
            state.related_shown()?;
            let list = Block::bordered()
                .inner(tab_layout(tab, Block::bordered().inner(areas.content), state.config.ui.comments_width).related);
            return list_row(list, tab.related, tab.related_videos().len(), at);
        }
        let inner = Block::bordered().inner(areas.content);
        return (!tab.playback.audio_only && inner.contains(at)).then_some(Spot::Video);
    }
    if state.prompt == Some(Prompt::CookiesFile) {
        return None;
    }
    let listing = state.listing()?;
    let (_, rest) = split_prompt(state, areas.content);
    let list = match listing.items {
        Items::Videos(_) | Items::Channels(_) => split_preview(rest, state.config.ui.preview_width).0,
        Items::About | Items::System => return None,
        _ => rest,
    };
    list_row(Block::bordered().inner(list), listing.selected, listing.items.len(), at)
}

/// The row of a list (drawn in `inner`, `selected` of `len`) at `at`.
fn list_row(inner: Rect, selected: usize, len: usize, at: Position) -> Option<Spot> {
    if !inner.contains(at) || inner.height == 0 {
        return None;
    }
    // A list drawn from offset 0 scrolls just enough to show the selection last.
    let offset = selected.saturating_sub(usize::from(inner.height) - 1);
    let row = offset + usize::from(at.y - inner.y);
    (row < len).then_some(Spot::Row(row))
}

/// The cells the embedded video may cover: the active video tab, unless a
/// dialog is open (kitty draws images above text, it would hide the dialog).
pub fn video_area(state: &State, area: Rect) -> Option<Rect> {
    if state.splash || state.help || state.confirm.is_some() {
        return None;
    }
    state.active_tab().filter(|t| !t.playback.audio_only)?;
    let cells = if state.fullscreen { area } else { Block::bordered().inner(screen_areas(area, state).content) };
    under_context_menu(state, area, cells)
}

/// `cells`, unless the right-click menu falls on them (an image would hide it).
fn under_context_menu(state: &State, screen: Rect, cells: Rect) -> Option<Rect> {
    let covered = state.context.as_ref().is_some_and(|menu| context_area(menu, screen).intersects(cells));
    (!covered).then_some(cells)
}

/// Where the right-click menu is drawn: at the pointer, inside the screen.
pub fn context_area(menu: &ContextMenu, screen: Rect) -> Rect {
    let widest = menu.items.iter().map(|i| Span::raw(i.label).width() + Span::raw(i.key).width()).max();
    let width = (widest.unwrap_or(0) as u16 + 7).min(screen.width);
    let height = (menu.items.len() as u16 + 2).min(screen.height);
    // One row below the pointer, so what was clicked stays in sight.
    let (x, y) = menu.at;
    Rect {
        x: x.min(screen.right().saturating_sub(width)),
        y: (y + 1).min(screen.bottom().saturating_sub(height)),
        width,
        height,
    }
}

pub fn draw(frame: &mut Frame, state: &State, view: &View) {
    BORDERS.set(state.config.ui.border_style);
    draw_screen(frame, state, view);
    theme::recolor(frame.buffer_mut(), &state.palette);
}

fn draw_screen(frame: &mut Frame, state: &State, view: &View) {
    let icons = icons(state.config.ui.icons);
    let lang = state.lang();
    if state.splash {
        draw_splash(frame, lang);
        return;
    }
    if state.fullscreen {
        // Nothing but the video (embedded) or a hint (window mode).
        if !view.embedded {
            let note = lang.pick(
                "Pantalla completa en la ventana de mpv · f o Esc para salir",
                "Fullscreen in the mpv window · f or Esc to leave",
            );
            frame.render_widget(Paragraph::new(note).fg(MUTED).alignment(Alignment::Center), frame.area());
        }
        if state.help {
            draw_help(frame, lang);
        }
        return;
    }
    let areas = screen_areas(frame.area(), state);
    if let Some(side) = areas.sidebar {
        draw_sidebar(frame, state, icons, side);
    }
    match state.active_tab() {
        None => draw_inicio(frame, state, icons, view, areas.content),
        Some(tab) => {
            let playback = &tab.playback;
            let title = Line::from(vec![
                Span::raw(format!(" {} ", tab_icon(icons, tab))),
                Span::styled(playback.title.clone(), Style::new().fg(FG).bold()),
                Span::styled(format!(" · {} ", playback.subtitle), MAGENTA),
            ]);
            let body = if state.details_shown(tab) {
                Paragraph::default() // the details, drawn below
            } else if tab.ended {
                // Not started yet (autoplay off): nothing plays until Enter.
                Paragraph::new(vec![
                    Line::from(""),
                    Line::from(playback.title.clone()).fg(FG).bold(),
                    Line::from(""),
                    Line::from(lang.pick("▶ Enter lo reproduce", "▶ Enter plays it")).fg(YELLOW),
                ])
                .alignment(Alignment::Center)
            } else if playback.audio_only {
                let songs = playback.queue.len();
                Paragraph::new(vec![
                    Line::from(""),
                    Line::from(format!("{}  {}", icons.section(Section::MiMusica), playback.title)).fg(FG).bold(),
                    Line::from(playback.subtitle.clone()).fg(MAGENTA),
                    Line::from(""),
                    Line::from(format!(
                        "{songs} {}",
                        lang.pick(
                            "canciones en la cola · . siguiente · , anterior",
                            "songs queued · . next · , previous"
                        )
                    ))
                    .fg(MUTED),
                ])
                .alignment(Alignment::Center)
            } else if let Some(elapsed) = view.opening {
                // In embedded mode the first video frame covers it.
                Paragraph::new(vec![
                    Line::from(""),
                    Line::from(loading_text(lang.pick("el video", "the video"), elapsed, lang)).fg(YELLOW),
                    Line::from(bouncing_bar(30, elapsed)).fg(BLUE),
                ])
                .alignment(Alignment::Center)
            } else if view.embedded {
                Paragraph::default() // the video
            } else {
                Paragraph::new(lang.pick("Se reproduce en una ventana de mpv", "Playing in an mpv window")).fg(MUTED)
            };
            // Lit while it has the keys; dim while Up Next has them.
            let mut block = focus_panel(title, state.key_context().in_tab && !state.key_context().related);
            if let Some(info) = &view.playing {
                block = block.title_top(Line::from(playing_label(info, icons)).fg(CYAN).right_aligned());
                if let Some(progress) = progress_line(info, &tab.segments, areas.content.width) {
                    block = block.title_bottom(progress);
                }
            }
            frame.render_widget(body.block(block), areas.content);
            if state.details_shown(tab) {
                draw_watch(frame, state, icons, view, tab, Block::bordered().inner(areas.content));
            }
        }
    }
    draw_status(frame, state, icons, view, areas.status);
    draw_hints(frame, &hints(state), areas.hints);
    if state.help {
        draw_help(frame, lang);
    }
    if let Some(confirm) = &state.confirm {
        draw_confirm(frame, confirm, lang);
    }
    if let Some(menu) = &state.context {
        draw_context_menu(frame, menu, lang);
    }
}

/// Cells of a video tab whose video plays in its own window.
struct WatchLayout {
    thumbnail: Rect,
    details: Rect,
    /// The AI summary, under the thumbnail and details, the left column's width.
    summary: Rect,
    related: Rect,
    /// Beside "Up next" when there is room, else under it.
    comments: Option<Rect>,
    /// The column with the thumbnail, details, summary and "Up next".
    left: Rect,
}

/// Columns of the thumbnail on a video tab.
const WATCH_THUMBNAIL_WIDTH: u16 = 36;
/// Columns the details and "Up next" keep beside the comments; narrower,
/// the comments go under them, across the tab.
const WATCH_LEFT_MIN_WIDTH: u16 = 50;
/// Rows stacked comments take at least (else they are left out).
const COMMENTS_STACKED_MIN_ROWS: u16 = 6;

/// The tab, its inside, and the rows its details and AI summary took.
type DrawnCard = (TabId, Rect, (u16, u16));

thread_local! {
    /// Rows the details card of a video tab took when last drawn, with the
    /// area it was drawn in: it grows to hold the AI summary, so a click on
    /// "Up next" must find the rows where they were drawn.
    static CARD_ROWS: std::cell::Cell<Option<DrawnCard>> = const { std::cell::Cell::new(None) };
}

fn watch_layout(inner: Rect, comments: u16) -> WatchLayout {
    watch_layout_with(inner, comments, (0, 0))
}

/// The layout of `tab` as last drawn in `inner`.
fn tab_layout(tab: &Tab, inner: Rect, comments: u16) -> WatchLayout {
    let drawn = CARD_ROWS.with(std::cell::Cell::get).filter(|(id, area, _)| *id == tab.id && *area == inner);
    watch_layout_with(inner, comments, drawn.map_or((0, 0), |(_, _, rows)| rows))
}

/// `detail_rows` / `summary_rows`: what the details (beside the thumbnail,
/// when more than its height) and the AI summary under them need.
/// Wide enough, the comments take the right side from top to bottom; the
/// left column has the thumbnail and details, then the AI summary across it,
/// then "Up next" right below.
/// `width`: Comments Width, in columns; the left column keeps
/// [`WATCH_LEFT_MIN_WIDTH`] columns whatever it says.
fn watch_layout_with(inner: Rect, width: u16, (detail_rows, summary_rows): (u16, u16)) -> WatchLayout {
    let (left, comments) = if inner.width >= WATCH_LEFT_MIN_WIDTH + width {
        let [left, comments] = Layout::horizontal([Constraint::Fill(1), Constraint::Length(width)]).areas(inner);
        (left, Some(comments))
    } else {
        // Under the details and "Up next", across the tab: two fifths of the
        // rows, when the rest still holds the card and "Up next".
        let card = (u32::from(WATCH_THUMBNAIL_WIDTH.min(inner.width * 2 / 5)) * 9 / 32) as u16;
        let rows = (inner.height * 2 / 5).max(COMMENTS_STACKED_MIN_ROWS);
        if inner.height >= rows + card + 1 + UP_NEXT_MIN_ROWS {
            let [left, comments] = Layout::vertical([Constraint::Fill(1), Constraint::Length(rows)]).areas(inner);
            (left, Some(comments))
        } else {
            (inner, None)
        }
    };
    // Two fifths of the left column, at most 36 columns.
    let width = WATCH_THUMBNAIL_WIDTH.min(left.width * 2 / 5);
    let rows = (u32::from(width) * 9 / 32) as u16;
    let card_rows = rows.max(detail_rows);
    let [card, summary, _, related] = Layout::vertical([
        Constraint::Length(card_rows),
        Constraint::Length(summary_rows),
        Constraint::Length(1),
        Constraint::Fill(1),
    ])
    .areas(left);
    let [thumbnail, _, details] =
        Layout::horizontal([Constraint::Length(width), Constraint::Length(2), Constraint::Fill(1)]).areas(card);
    let thumbnail = Rect { height: rows.min(card.height), ..thumbnail };
    WatchLayout { thumbnail, details, summary, related, comments, left }
}

/// Rows "Up next" keeps at least (its border and five videos), however long
/// the summary: past that the summary is cut with "…".
const UP_NEXT_MIN_ROWS: u16 = 7;

/// `text` broken into lines of at most `width` columns: its own line breaks
/// kept, words kept whole when they fit, longer ones cut.
fn wrap_text(text: &str, width: u16) -> Vec<String> {
    let width = usize::from(width.max(1));
    let mut lines = Vec::new();
    for source in text.lines() {
        let mut line = String::new();
        let mut used = 0;
        for word in source.split_whitespace() {
            let size = Span::raw(word).width();
            if used > 0 && used + 1 + size > width {
                lines.push(std::mem::take(&mut line));
                used = 0;
            }
            if used > 0 {
                line.push(' ');
                used += 1;
            }
            for c in word.chars() {
                let w = Span::raw(c.to_string()).width();
                if used + w > width && used > 0 {
                    lines.push(std::mem::take(&mut line));
                    used = 0;
                }
                line.push(c);
                used += w;
            }
        }
        lines.push(line);
    }
    lines
}

/// A blank line, "✦ AI summary" and the summary wrapped at `width`, in at
/// most `rows` lines: the last one ends in "…" when it does not all fit.
/// Without one (`None`), the heading says so.
fn summary_lines(text: Option<&str>, width: u16, rows: usize, lang: Lang, icon: &str) -> Vec<Line<'static>> {
    if rows < 2 {
        return Vec::new();
    }
    let note = match text {
        Some(_) => lang.pick(" · la IA puede equivocarse", " · AI can make mistakes"),
        None => lang.pick(" · no disponible para este video", " · not available for this video"),
    };
    let heading = Line::from(vec![
        Span::styled(format!("{icon} {}", lang.pick("Resumen IA", "AI Summary")), Style::new().fg(BLUE).bold()),
        Span::styled(note, MUTED),
    ]);
    let Some(text) = text else { return vec![Line::from(""), heading] };
    let mut wrapped = wrap_text(text, width);
    let room = rows - 2;
    if wrapped.len() > room {
        wrapped.truncate(room);
        if let Some(last) = wrapped.last_mut() {
            // Room for the mark: drop characters until it fits.
            while !last.is_empty() && Span::raw(last.as_str()).width() + 1 > usize::from(width) {
                last.pop();
            }
            *last = format!("{}…", last.trim_end());
        }
    }
    [Line::from(""), heading].into_iter().chain(wrapped.into_iter().map(|l| Line::from(l).fg(FG))).collect()
}

/// The video plays in its own window: the tab shows its details and what to watch next.
fn draw_watch(frame: &mut Frame, state: &State, icons: &Icons, view: &View, tab: &Tab, inner: Rect) {
    let lang = state.lang();
    let Some(video) = &tab.playback.video else { return };
    let comments_width = state.config.ui.comments_width;
    let mut layout = watch_layout(inner, comments_width);
    let details = if view.thumbnails {
        let middle = Rect { y: layout.thumbnail.y + layout.thumbnail.height / 2, height: 1, ..layout.thumbnail };
        frame.render_widget(Paragraph::new(icons.waiting).fg(MUTED).alignment(Alignment::Center), middle);
        layout.details
    } else {
        // No thumbnail: the details take the left column's whole width.
        let left = layout.related;
        Rect { x: left.x, width: left.width, ..layout.details }
    };
    let watch = tab.watch.as_ref();
    let mut channel = vec![Span::styled(video.channel.as_str(), MAGENTA)];
    if let Some(subscribers) = watch.and_then(|w| w.subscribers.as_deref()) {
        channel.push(Span::styled(format!(" · {subscribers} {}", lang.pick("suscriptores", "subscribers")), MUTED));
    }
    if video.channel_id.as_ref().is_some_and(|c| state.subs.contains(c)) {
        channel.push(Span::styled(format!("  {} {}", icons.subscribed, lang.pick("Suscripto", "Subscribed")), GREEN));
    }
    let mut numbers = Vec::new();
    if let Some(views) = watch.and_then(|w| w.views) {
        numbers.push(format!("{} {}", compact(views, lang), lang.pick("vistas", "views")));
    }
    if let Some(likes) = watch.and_then(|w| w.likes) {
        numbers.push(format!("{} {}", icons.liked, compact(likes, lang)));
    }
    let mut when = vec![length_label(state, video)];
    if let Some(ts) = watch.and_then(|w| w.published).or(video.published) {
        let (date, age) = date_labels(ts, view.now, lang);
        when.push(format!("{age} ({date})"));
    }
    let mut lines = vec![Line::from(video.title.as_str()).fg(FG).bold(), Line::from(""), Line::from(channel)];
    for row in [numbers, when] {
        let row: Vec<String> = row.into_iter().filter(|p| !p.is_empty()).collect();
        if !row.is_empty() {
            lines.push(Line::from(row.join(" · ")).fg(MUTED));
        }
    }
    lines.extend(later_lines(state, icons, &video.id, lang));
    lines.extend(rating_line(state, icons, &video.id));
    lines.extend(size_line(state, video));
    if tab.playback.audio_only {
        let note = format!(
            "{} {}",
            icons.audio,
            lang.pick("Solo audio · a vuelve al video", "Audio Only · a brings the video back")
        );
        lines.push(Line::from(note).fg(TEAL));
    }
    if tab.ended {
        let note = if tab.started {
            format!(
                "{} {}",
                icons.stop,
                lang.pick("Video cerrado · Enter lo vuelve a ver", "Video closed · Enter plays it again")
            )
        } else {
            lang.pick("▶ Enter lo reproduce", "▶ Enter plays it").to_owned()
        };
        lines.push(Line::from(note).fg(YELLOW));
    }
    if let Some(elapsed) = view.opening {
        lines.push(Line::from(loading_text(lang.pick("el video", "the video"), elapsed, lang)).fg(YELLOW));
    }
    // The details card grows to hold them all, up to what leaves "Up next" its rows.
    let paragraph = Paragraph::new(lines).wrap(ratatui::widgets::Wrap { trim: true });
    let most_card = layout.left.height.saturating_sub(1 + UP_NEXT_MIN_ROWS);
    let detail_rows = (paragraph.line_count(details.width) as u16).min(most_card);
    layout = watch_layout_with(inner, comments_width, (detail_rows, 0));
    let details = Rect { y: layout.details.y, height: layout.details.height, ..details };
    frame.render_widget(paragraph, details);
    // The AI summary across the left column, all of it, as long as "Up next"
    // keeps its rows (then cut with "…").
    let mut summary_rows = 0;
    if let Some(summary) = state.tab_summary_lookup(tab) {
        let width = layout.related.width;
        let most = layout.left.height.saturating_sub(layout.details.height + 1 + UP_NEXT_MIN_ROWS);
        let lines = summary_lines(summary, width, usize::from(most), lang, icons.summary);
        let wrapped = Paragraph::new(lines.clone()).wrap(ratatui::widgets::Wrap { trim: true });
        summary_rows = (wrapped.line_count(width) as u16).min(most);
        layout = watch_layout_with(inner, comments_width, (detail_rows, summary_rows));
        frame.render_widget(wrapped, layout.summary);
    }
    CARD_ROWS.with(|c| c.set(Some((tab.id, inner, (detail_rows, summary_rows)))));

    let related = tab.related_videos();
    let focused = state.key_context().related;
    let title = match watch {
        None => format!(" {} ", lang.pick("Buscando videos relacionados…", "Looking for Related Videos…")),
        Some(_) if related.is_empty() => format!(" {} ", lang.pick("Sin videos relacionados", "No Related Videos")),
        Some(_) if focused => format!(" {} · {} ", lang.pick("A continuación", "Up Next"), related.len()),
        Some(_) => format!(" {} · {} · Tab ", lang.pick("A continuación", "Up Next"), related.len()),
    };
    let highlight = if focused { Style::new().bg(HIGHLIGHT).add_modifier(Modifier::BOLD) } else { Style::new() };
    // Inside the borders, less the cursor column.
    let row_width = usize::from(layout.related.width.saturating_sub(3));
    let list = List::new(related.iter().map(|v| video_row_dated(state, icons, view, v, false, row_width)))
        .block(focus_panel(title, focused))
        .highlight_symbol(if focused { "▌" } else { " " })
        .highlight_style(highlight);
    let selected = (!related.is_empty()).then_some(tab.related);
    frame.render_stateful_widget(list, layout.related, &mut ListState::default().with_selected(selected));
    if let Some(area) = layout.comments {
        draw_comments(frame, tab, icons, lang, area);
    }
}

/// The video's top comments, from the one scrolled to (PgUp / PgDn).
fn draw_comments(frame: &mut Frame, tab: &Tab, icons: &Icons, lang: Lang, area: Rect) {
    let title = match &tab.comments {
        None if tab.watch.is_none() || tab.watch.as_ref().is_some_and(|w| w.comments_token.is_some()) => {
            format!(" {} ", lang.pick("Buscando comentarios…", "Looking for Comments…"))
        }
        None => format!(" {} ", lang.pick("Comentarios desactivados", "Comments Are Off")),
        Some(c) if c.is_empty() => format!(" {} ", lang.pick("Sin comentarios", "No Comments")),
        Some(c) => format!(
            " {} {}/{} · PgUp PgDn ",
            lang.pick("Comentarios", "Comments"),
            (tab.comments_from + 1).min(c.len()),
            c.len()
        ),
    };
    let mut lines = Vec::new();
    for comment in tab.comments.iter().flatten().skip(tab.comments_from) {
        let mut head = vec![Span::styled(comment.author.clone(), Style::new().fg(MAGENTA).bold())];
        if let Some(likes) = &comment.likes {
            head.push(Span::styled(format!("  {} {likes}", icons.liked), MUTED));
        }
        if let Some(published) = &comment.published {
            head.push(Span::styled(format!("  {published}"), MUTED));
        }
        lines.push(Line::from(head));
        lines.extend(comment.text.lines().map(|l| Line::from(l.to_owned()).fg(FG)));
        lines.push(Line::from(""));
    }
    let comments = Paragraph::new(lines).wrap(ratatui::widgets::Wrap { trim: true }).block(panel(title, MUTED));
    frame.render_widget(comments, area);
}

fn tab_icon<'a>(icons: &'a Icons, tab: &Tab) -> &'a str {
    if tab.playback.audio_only { icons.section(Section::MiMusica) } else { icons.tab }
}

/// `" 🔊 85 %  720p  H.264 "`: what the video keys change, as it is now.
fn playing_label(info: &Info, icons: &Icons) -> String {
    let mut parts = Vec::new();
    if let Some(title) = info.chapter_title() {
        parts.push(format!("§ {}", truncate(title, 32)));
    }
    if let Some(volume) = info.volume {
        let icon = if info.muted { icons.muted } else { icons.volume };
        parts.push(format!("{icon} {volume:.0} %"));
    }
    if let Some(height) = info.height {
        parts.push(format!("{height}p"));
    }
    if let Some(codec) = &info.codec {
        parts.push(match codec.as_str() {
            "h264" => "H.264".into(),
            "hevc" => "H.265".into(),
            other => other.to_uppercase(),
        });
    }
    if parts.is_empty() { String::new() } else { format!(" {} ", parts.join("  ")) }
}

/// Where the progress bar sits in the bottom border of the video tab:
/// the cells of the bar itself and the seconds it spans.
pub fn progress_bar(info: &Info, content: Rect) -> Option<(Rect, f64)> {
    info.position?;
    let duration = info.duration.filter(|d| *d > 0.0)?;
    let clock_width = clock(duration as u32).len() as u16;
    // ` ▶ 1:23 ` before the bar and ` 4:56 ` after it, inside the corners.
    let before = 2 + 2 + clock_width + 1;
    let after = 1 + clock_width + 1;
    let width = content.width.checked_sub(2 + before + after).filter(|w| *w >= 10)?;
    Some((Rect::new(content.x + 1 + before, content.bottom().saturating_sub(1), width, 1), duration))
}

/// ` ▶ 1:23 ━━━━━━━━━━━━──┃───── 4:56 `, the width of the tab's bottom border:
/// played in red, downloaded in grey, the rest dim; the parts SponsorBlock
/// skips in green and where chapters start as `┃`.
fn progress_line(info: &Info, segments: &[Segment], width: u16) -> Option<Line<'static>> {
    let (bar, duration) = progress_bar(info, Rect::new(0, 0, width, 1))?;
    let position = info.position.unwrap_or_default().clamp(0.0, duration);
    let cells = |seconds: f64| ((seconds.clamp(0.0, duration) / duration) * f64::from(bar.width)).round() as usize;
    let done = cells(position);
    let loaded = cells(info.buffered.unwrap_or_default()).max(done);
    let width = usize::from(bar.width);
    // The seconds each cell spans.
    let span = |i: usize| (i as f64 * duration / width as f64, (i + 1) as f64 * duration / width as f64);
    let mut track: Vec<Span<'static>> = Vec::new();
    for i in 0..width {
        let (from, to) = span(i);
        let mut symbol = if i < loaded { "━" } else { "─" };
        let mut color = if i < done { RED } else { MUTED };
        if segments.iter().any(|s| s.start_secs() < to && s.end_secs() > from) {
            (symbol, color) = ("━", GREEN);
        }
        if info.chapters.iter().any(|(start, _)| *start > 0.0 && *start >= from && *start < to) {
            symbol = "┃";
        }
        match track.last_mut() {
            Some(last) if last.style.fg == Some(color) && last.content.starts_with(symbol) => {
                last.content.to_mut().push_str(symbol);
            }
            _ => track.push(Span::styled(symbol.to_owned(), Style::new().fg(color))),
        }
    }
    let clock_width = clock(duration as u32).len();
    // One cell each: the emoji pause sign takes two in most terminals.
    let icon = if info.paused { "‖" } else { "▶" };
    let mut spans =
        vec![Span::styled(format!(" {icon} {:>clock_width$} ", clock(position as u32)), Style::new().fg(FG))];
    spans.extend(track);
    spans.push(Span::styled(format!(" {} ", clock(duration as u32)), Style::new().fg(FG)));
    Some(Line::from(spans))
}

thread_local! {
    /// `[ui] border_style` of the frame being drawn (set by [`draw`]).
    static BORDERS: std::cell::Cell<BorderStyle> = const { std::cell::Cell::new(BorderStyle::Plain) };
}

/// A border of blanks: no lines, but the same room, so nothing moves.
const NO_BORDER: ratatui::symbols::border::Set<'static> = ratatui::symbols::border::Set {
    top_left: " ",
    top_right: " ",
    bottom_left: " ",
    bottom_right: " ",
    vertical_left: " ",
    vertical_right: " ",
    horizontal_top: " ",
    horizontal_bottom: " ",
};

/// The lines of `style`.
pub fn border_set(style: BorderStyle) -> ratatui::symbols::border::Set<'static> {
    match style {
        BorderStyle::Plain => BorderType::Plain.to_border_set(),
        BorderStyle::Rounded => BorderType::Rounded.to_border_set(),
        BorderStyle::Thick => BorderType::Thick.to_border_set(),
        BorderStyle::Double => BorderType::Double.to_border_set(),
        BorderStyle::None => NO_BORDER,
    }
}

/// Every pane's frame, in the chosen border style.
fn framed<'a>() -> Block<'a> {
    Block::bordered().border_set(border_set(BORDERS.get()))
}

fn panel<'a>(title: impl Into<Line<'a>>, color: Color) -> Block<'a> {
    framed().border_style(Style::new().fg(color)).title(title)
}

/// A pane that takes the keys in turn (the menu, the list, Up next): its
/// border lights up in blue while it has them. With no border to light,
/// its title does.
fn focus_panel<'a>(title: impl Into<Line<'a>>, focused: bool) -> Block<'a> {
    let mut title = title.into();
    if focused && BORDERS.get() == BorderStyle::None {
        title = title.patch_style(Style::new().fg(BLUE).bg(HIGHLIGHT).add_modifier(Modifier::BOLD));
    }
    panel(title, if focused { BLUE } else { MUTED })
}

/// Whether the keys go to the menu: the browsing view, with the menu in focus.
/// Over the browsing view or a video tab still shown.
fn menu_focused(state: &State) -> bool {
    state.focus == Focus::Menu && state.sidebar
}

/// The menu's lines, each with what a click on it does, for a menu whose
/// inside is `inner`: Settings, About and Quit pinned to its bottom under a
/// short rule; when there is no room for that, right below the rest.
fn sidebar_items(state: &State, icons: &Icons, inner: Rect) -> Vec<(ListItem<'static>, Option<Spot>)> {
    let lang = state.lang();
    let mut top: Vec<(ListItem, Option<Spot>)> = Vec::new();
    let mut bottom: Vec<(ListItem, Option<Spot>)> = Vec::new();
    let mut group = None;
    for (i, section) in Section::ALL.iter().enumerate() {
        let pinned = section.group() == Group::App;
        if group != Some(section.group()) && !pinned {
            // A blank row above every group but the first, right under the logo.
            if group.is_some() {
                top.push((ListItem::new(""), None));
            }
            group = Some(section.group());
            top.push((ListItem::new(Line::from(format!(" {}", section.group().label(lang))).fg(MUTED).bold()), None));
        }
        let mut label = section.label(lang).to_owned();
        // Only a count YouTube's whole list backs up.
        if *section == Section::VerDespues && state.later_complete && !state.later.is_empty() {
            label.push_str(&format!(" ({})", state.later.len()));
        }
        if *section == Section::Suscripciones && !state.fresh.is_empty() {
            label.push_str(&format!(" ({} {})", state.fresh.len(), lang.pick("nuevos", "new")));
        }
        if *section == Section::Descargas && state.downloading() > 0 {
            label.push_str(&format!(" ({} {})", state.downloading(), icons.downloading));
        }
        if *section == Section::AcercaDe && state.update.is_some() {
            label.push_str(&format!(" {}", icons.update));
        }
        let focused = state.menu_row() == MenuRow::Section(i) && menu_focused(state);
        let shown = state.shown_row() == Some(MenuRow::Section(i));
        let color = SELECTED;
        let mut style = Style::new().fg(FG);
        if focused {
            style = style.fg(color).bg(HIGHLIGHT).add_modifier(Modifier::BOLD);
        } else if shown {
            style = style.fg(SHOWN);
        }
        let line = Line::from(vec![
            Span::styled(if focused { "▌" } else { " " }, color),
            Span::raw(format!("{} ", icons.sections[i])),
            Span::raw(label),
        ]);
        let row = (ListItem::new(line).style(style), Some(Spot::Section(i)));
        if pinned { bottom.push(row) } else { top.push(row) }
    }
    let mut tabs: Vec<(ListItem, Option<Spot>)> = Vec::new();
    for (i, tab) in state.tabs.iter().enumerate() {
        // The cursor's row has the focus highlight; the tab shown, the "shown" mark.
        let active = state.menu_row() == MenuRow::Tab(i) && menu_focused(state);
        let shown = state.shown_row() == Some(MenuRow::Tab(i));
        let style = if active {
            Style::new().fg(SELECTED).bg(HIGHLIGHT).bold()
        } else if shown {
            Style::new().fg(SHOWN)
        } else {
            Style::new().fg(FG)
        };
        let line = format!("{}{} {}", if active { "▌" } else { " " }, tab_icon(icons, tab), tab.playback.title);
        tabs.push((ListItem::new(line).style(style), Some(Spot::Tab(i + 1))));
    }
    // A blank row, then the rule: inset two columns on each side.
    let rule = || {
        let width = usize::from(inner.width).saturating_sub(2 * SEPARATOR_INSET.len());
        (ListItem::new(Line::from(format!("{SEPARATOR_INSET}{}", "─".repeat(width))).fg(MUTED)), None)
    };
    let room = usize::from(inner.height);
    // Above the tabs: a blank row and the TABS title. Below: a blank row, the rule, Settings…
    let (tabs_head, tail) = (if tabs.is_empty() { 0 } else { 2 }, 2 + bottom.len());
    let tab_rows = room.saturating_sub(top.len() + tabs_head + tail);
    if room >= top.len() + tabs_head + tail + usize::from(!tabs.is_empty()) {
        // The top and the bottom stay put; the tabs take what is left and scroll.
        let mut items = top;
        let mut more_below = false;
        if !tabs.is_empty() {
            let total = tabs.len();
            let title = lang.pick(" PESTAÑAS", " TABS");
            let shown = tab_rows.min(total);
            let offset = tab_offset(state, total, shown);
            let chevron = |icon: &str| format!("{}{icon}", " ".repeat(Span::raw(title).width() + 1));
            let mut head = Line::from(title.to_owned()).fg(MUTED).bold();
            if offset > 0 {
                head = Line::from(vec![Span::raw(title.to_owned()), Span::raw(format!(" {}", icons.more_above))])
                    .fg(MUTED)
                    .bold();
            }
            items.push((ListItem::new(""), None));
            items.push((ListItem::new(head), None));
            items.extend(tabs.into_iter().skip(offset).take(shown));
            // More below: the chevron takes the blank row above the rule.
            more_below = offset + shown < total;
            if more_below {
                items.push((ListItem::new(Line::from(chevron(icons.more_below)).fg(MUTED).bold()), None));
            }
        }
        let gap = room.saturating_sub(items.len() + 1 + bottom.len()).max(usize::from(!more_below));
        items.extend((0..gap).map(|_| (ListItem::new(""), None)));
        items.push(rule());
        items.extend(bottom);
        return items;
    }
    // Too short for even one tab row: one list, scrolled so the cursor's row
    // shows (or, without the menu's keys, the tab shown).
    let mut items = top;
    if !tabs.is_empty() {
        items.push((ListItem::new(""), None));
        items.push((ListItem::new(Line::from(lang.pick(" PESTAÑAS", " TABS")).fg(MUTED).bold()), None));
        items.extend(tabs);
    }
    items.push((ListItem::new(""), None));
    items.push(rule());
    items.extend(bottom);
    let row = match state.shown_row() {
        Some(MenuRow::Tab(i)) if !menu_focused(state) => MenuRow::Tab(i),
        _ => state.menu_row(),
    };
    let spot = match row {
        MenuRow::Section(i) => Spot::Section(i),
        MenuRow::Tab(i) => Spot::Tab(i + 1),
    };
    let chosen = items.iter().position(|(_, s)| *s == Some(spot)).unwrap_or(0);
    let skip = (chosen + 1).saturating_sub(room);
    items.drain(..skip);
    items
}

/// The first of `total` tabs to show in `rows` rows. It follows the cursor's
/// tab (with the menu's keys) or else the tab shown; after the wheel it
/// stays where the wheel left it until that tab changes.
fn tab_offset(state: &State, total: usize, rows: usize) -> usize {
    let anchor = match state.menu_row() {
        MenuRow::Tab(i) if menu_focused(state) => Some(i),
        _ => state.active.checked_sub(1),
    };
    let mut scroll = state.tab_scroll.get();
    if anchor != scroll.anchor {
        scroll.wheeled = false;
    }
    if let Some(i) = anchor.filter(|_| !scroll.wheeled) {
        if i < scroll.offset {
            scroll.offset = i;
        } else if i >= scroll.offset + rows {
            scroll.offset = i + 1 - rows;
        }
    }
    scroll.anchor = anchor;
    scroll.offset = scroll.offset.min(total - rows.min(total));
    state.tab_scroll.set(scroll);
    scroll.offset
}

/// How far the rule above Settings stays from each side of the menu.
const SEPARATOR_INSET: &str = "  ";

fn draw_sidebar(frame: &mut Frame, state: &State, icons: &Icons, area: Rect) {
    let items = sidebar_items(state, icons, framed().inner(area)).into_iter().map(|(item, _)| item);
    let title = Line::from(vec![
        Span::styled(format!(" {} ", icons.logo), Style::new().fg(RED).bold()),
        Span::styled("tube", Style::new().fg(RED).bold()),
        Span::styled("vibe ", Style::new().fg(FG).bold()),
    ]);
    frame.render_widget(List::new(items).block(focus_panel(title, menu_focused(state))), area);
}

/// "In YouTube Watch Later" / "In tubevibe Watch Later" lines of a video's preview.
fn later_lines(state: &State, icons: &Icons, id: &str, lang: Lang) -> Vec<Line<'static>> {
    let mut lines = Vec::new();
    if state.later.iter().any(|l| l.id == id) {
        let label = lang.pick("En Ver después de YouTube", "In YouTube Watch Later");
        lines.push(Line::from(format!("{} {label}", icons.later)).fg(CYAN));
    }
    if state.saved.iter().any(|v| v.id == id) {
        let label = lang.pick("En Ver después de tubevibe", "In tubevibe Watch Later");
        lines.push(Line::from(format!("{} {label}", icons.section(Section::Guardados))).fg(CYAN));
    }
    lines
}

/// The search box and the rest of Inicio below it.
fn split_prompt(state: &State, area: Rect) -> (Option<Rect>, Rect) {
    match state.prompt {
        Some(_) => {
            let [input, rest] = Layout::vertical([Constraint::Length(3), Constraint::Fill(1)]).areas(area);
            (Some(input), rest)
        }
        None => (None, area),
    }
}

/// A video list and, when there is room, the preview panel beside it.
/// `width`: Preview Width. The list keeps the columns it had beside the
/// default preview: narrower than that, no preview.
/// The list and its preview: side by side (`width` columns) when the list
/// keeps [`LIST_MIN_WIDTH`]; else the preview under the list, across it;
/// with too few rows for that, no preview.
fn split_preview(area: Rect, width: u16) -> (Rect, Option<Rect>) {
    if area.width >= LIST_MIN_WIDTH + width {
        let [list, preview] = Layout::horizontal([Constraint::Fill(1), Constraint::Length(width)]).areas(area);
        return (list, Some(preview));
    }
    let rows = PREVIEW_STACKED_ROWS.min(area.height.saturating_sub(LIST_MIN_ROWS));
    if rows < PREVIEW_STACKED_MIN_ROWS {
        return (area, None);
    }
    let [list, preview] = Layout::vertical([Constraint::Fill(1), Constraint::Length(rows)]).areas(area);
    (list, Some(preview))
}

/// The cells of the thumbnail inside a preview panel: 16:9, taking terminal
/// cells as about twice as tall as wide.
fn thumbnail_cells(preview: Rect) -> Rect {
    let inner = Block::bordered().inner(preview);
    // At most half the rows (a preview under the list is wide and short),
    // narrowed to stay 16:9 when that cuts it.
    let full = (inner.width as u32 * 9 / 32) as u16;
    let rows = if full > inner.height / 2 && inner.width > inner.height {
        (inner.height / 2).max(1)
    } else {
        full.min(inner.height)
    };
    let width = if rows < full { ((u32::from(rows) * 32 / 9) as u16).min(inner.width) } else { inner.width };
    Rect { height: rows, width, ..inner }
}

/// Where the thumbnail of the selected video goes, with that video's id.
pub fn thumbnail_area(state: &State, screen: Rect) -> Option<(String, Rect)> {
    if state.splash || state.help || state.confirm.is_some() || state.prompt == Some(Prompt::CookiesFile) {
        return None;
    }
    if let Some(tab) = state.active_tab() {
        let video = tab.playback.video.as_ref().filter(|_| state.details_shown(tab) && !state.fullscreen)?;
        let inner = Block::bordered().inner(screen_areas(screen, state).content);
        let cells = watch_layout(inner, state.config.ui.comments_width).thumbnail;
        return under_context_menu(state, screen, cells).filter(|c| c.height > 0).map(|c| (video.id.clone(), c));
    }
    let video = state.videos_from_selection(0).first()?;
    let (_, list) = split_prompt(state, screen_areas(screen, state).content);
    let (_, preview) = split_preview(list, state.config.ui.preview_width);
    Some((video.id.clone(), under_context_menu(state, screen, thumbnail_cells(preview?))?))
}

fn draw_inicio(frame: &mut Frame, state: &State, icons: &Icons, view: &View, area: Rect) {
    let lang = state.lang();
    let (input, rest) = split_prompt(state, area);
    let area = match (&state.prompt, input) {
        (Some(prompt), Some(input)) => {
            let title = match prompt {
                Prompt::Videos { scope, .. } => Line::from(vec![
                    Span::raw(format!(
                        " {} {} ",
                        icons.section(Section::BuscarVideos),
                        lang.pick("Buscar videos en", "Search Videos in")
                    )),
                    Span::styled(scope.label(lang), Style::new().fg(YELLOW).bold()),
                    Span::raw(lang.pick(" · Tab cambia ", " · Tab changes ")).fg(MUTED),
                ]),
                Prompt::Channels => Line::from(format!(
                    " {} {} ",
                    icons.section(Section::BuscarCanales),
                    lang.pick("Buscar canales", "Search Channels")
                )),
                Prompt::CookiesFile => Line::from(format!(
                    " {} {} ",
                    icons.setting(Setting::ImportCookies),
                    lang.pick("Ruta del cookies.txt", "cookies.txt Path")
                )),
                Prompt::DownloadDir => Line::from(format!(
                    " {} {} ",
                    icons.setting(Setting::DownloadDir),
                    lang.pick("Carpeta de descargas", "Downloads Folder")
                )),
                Prompt::MediaDirs(kind) => {
                    let (icon, what) = match kind {
                        MediaKind::Video => {
                            (icons.setting(Setting::VideoDirs), lang.pick("Carpetas de videos", "Video Folders"))
                        }
                        MediaKind::Music => {
                            (icons.setting(Setting::MusicDirs), lang.pick("Carpetas de música", "Music Folders"))
                        }
                    };
                    let how = lang.pick(
                        "separadas por coma (vacío = la del sistema)",
                        "comma separated (empty = the system one)",
                    );
                    Line::from(format!(" {icon} {what}, {how} "))
                }
                Prompt::Topics => Line::from(format!(
                    " {} {} ",
                    icons.setting(Setting::DiscoverTopics),
                    lang.pick("Temas de Descubrir, separados por coma", "Discover Topics, Comma Separated")
                )),
            };
            frame.render_widget(Paragraph::new(state.query.as_str()).block(panel(title, CYAN)), input);
            let x = input.x + 1 + state.query.chars().count() as u16;
            frame.set_cursor_position(Position::new(x.min(input.right().saturating_sub(2)), input.y + 1));
            if *prompt == Prompt::CookiesFile {
                draw_cookies_help(frame, rest, lang);
                return;
            }
            rest
        }
        _ => area,
    };
    match state.listing() {
        Some(listing) => draw_listing(frame, state, icons, view, listing, area),
        None => draw_welcome(frame, state, icons, view, area),
    }
}

fn draw_cookies_help(frame: &mut Frame, area: Rect, lang: Lang) {
    let line = |es, en, color| Line::from(lang.pick(es, en)).fg(color);
    let lines = vec![
        Line::from(""),
        line(
            "Para usar tu cuenta en una máquina sin navegador (servidor, SSH):",
            "To use your account on a machine without a browser (server, SSH):",
            FG,
        ),
        Line::from(""),
        line(
            "  1. En cualquier PC con navegador, entrá a youtube.com con tu cuenta.",
            "  1. On any computer with a browser, log in to youtube.com.",
            FG,
        ),
        line(
            "  2. Exportá las cookies en formato Netscape (cookies.txt), por ejemplo con",
            "  2. Export the cookies in Netscape format (cookies.txt), for example with",
            FG,
        ),
        line(
            "     la extensión «Get cookies.txt LOCALLY», o con:",
            "     the “Get cookies.txt LOCALLY” extension, or with:",
            FG,
        ),
        Line::from("     yt-dlp --cookies-from-browser firefox --cookies cookies.txt --skip-download :ytsubs").fg(CYAN),
        line(
            "  3. Copialo a esta máquina (scp cookies.txt servidor:~/) y escribí la ruta arriba.",
            "  3. Copy it to this machine (scp cookies.txt server:~/) and type its path above.",
            FG,
        ),
        Line::from(""),
        line(
            "tubevibe comprueba que tenga una sesión de YouTube y guarda una copia privada (permisos 600).",
            "tubevibe checks it holds a YouTube login and keeps a private copy (mode 600).",
            MUTED,
        ),
        line(
            "Las cookies dan acceso a tu cuenta: no las compartas.",
            "Cookies give access to your account: never share them.",
            YELLOW,
        ),
    ];
    let title = lang.pick(" Importar sesión ", " Import Login ");
    frame.render_widget(Paragraph::new(lines).block(panel(title, MUTED)), area);
}

fn draw_welcome(frame: &mut Frame, state: &State, icons: &Icons, view: &View, area: Rect) {
    let lang = state.lang();
    let mut lines = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled(format!("{} ", icons.logo), Style::new().fg(RED).bold()),
            Span::styled("tube", Style::new().fg(RED).bold()),
            Span::styled("vibe", Style::new().fg(FG).bold()),
        ]),
        Line::from(lang.pick("YouTube en tu terminal", "YouTube in your terminal")).fg(MUTED),
        Line::from(""),
        Line::from(
            lang.pick("Elegí una sección del menú con ↑ ↓ y Enter", "Pick a section from the menu with ↑ ↓ and Enter"),
        )
        .fg(FG),
        Line::from(lang.pick(
            "Abajo siempre ves las teclas que sirven en cada momento",
            "The bottom line always shows the keys that work right now",
        ))
        .fg(MUTED),
        Line::from(lang.pick(
            "Shift+/  muestra todos los atajos explicados · L  English",
            "Shift+/  shows every shortcut, explained · L  español",
        ))
        .fg(MUTED),
    ];
    if let Some(title) = &state.loading {
        lines.push(Line::from(""));
        lines.push(Line::from(loading_text(title, view.loading.unwrap_or_default(), lang)).fg(YELLOW));
        lines.push(Line::from(bouncing_bar(30, view.loading.unwrap_or_default())).fg(BLUE));
    }
    frame.render_widget(Paragraph::new(lines).alignment(Alignment::Center).block(panel("", MUTED)), area);
}

fn draw_listing(frame: &mut Frame, state: &State, icons: &Icons, view: &View, listing: &Listing, area: Rect) {
    if listing.items == Items::About {
        draw_about(frame, state, icons, area);
        return;
    }
    if listing.items == Items::System {
        draw_system(frame, state, area);
        return;
    }
    let breadcrumb: Vec<&str> = state.stack.iter().map(|l| l.title.as_str()).collect();
    let title = match &state.loading {
        Some(loading) => format!(" {} ", loading_text(loading, view.loading.unwrap_or_default(), state.lang())),
        None if listing.items == Items::Settings => format!(" {} ", breadcrumb.join(" › ")),
        None => match &listing.items {
            Items::Videos(videos) if listing.all.is_some() => {
                let mut note = match (state.show_watched, listing.hidden_watched) {
                    (true, _) => {
                        format!(" · {} {}", icons.watched, state.lang().pick("con los vistos", "watched included"))
                    }
                    (false, 0) => String::new(),
                    (false, n) => {
                        format!(" · {} {n} {}", icons.watched, state.lang().pick("vistos ocultos", "watched hidden"))
                    }
                };
                if listing.hidden_later > 0 {
                    let label = state.lang().pick("de Ver después de YouTube ocultos", "in YouTube Watch Later hidden");
                    note.push_str(&format!(" · {} {} {label}", icons.later, listing.hidden_later));
                }
                format!(" {} · {}{note} ", breadcrumb.join(" › "), videos.len())
            }
            Items::Downloads(_) if !state.marked_download_ids().is_empty() => {
                let marked = state.marked_download_ids().len();
                let label = state.lang().pick("marcadas", "marked");
                let mark = icons.marked;
                format!(" {} · {} · {mark} {marked} {label} ", breadcrumb.join(" › "), listing.items.len())
            }
            Items::Files(files, _) if files.iter().any(|f| state.marked.contains(&f.path)) => {
                let marked = files.iter().filter(|f| state.marked.contains(&f.path)).count();
                let label = state.lang().pick("marcados", "marked");
                let mark = icons.marked;
                format!(" {} · {} · {mark} {marked} {label} ", breadcrumb.join(" › "), listing.items.len())
            }
            _ => format!(" {} · {} ", breadcrumb.join(" › "), listing.items.len()),
        },
    };
    // Rows fit inside the borders, less the cursor column.
    let list_width = match (&listing.items, split_preview(area, state.config.ui.preview_width)) {
        (Items::Videos(_) | Items::Channels(_), (left, Some(_))) => left.width,
        _ => area.width,
    };
    let row_width = usize::from(list_width.saturating_sub(3));
    let items: Vec<ListItem> = match &listing.items {
        Items::Videos(videos) => {
            let history = listing.kind == ListKind::History;
            videos.iter().map(|v| video_row_dated(state, icons, view, v, history, row_width)).collect()
        }
        Items::Channels(channels) => {
            // The description goes in the panel when there is one, other details in the row.
            let panel = split_preview(area, state.config.ui.preview_width).1.is_some();
            channels.iter().map(|c| channel_row(state, icons, c, panel)).collect()
        }
        Items::Profiles(profiles) => profiles.iter().map(|p| profile_row(state, icons, p)).collect(),
        Items::Playlists(playlists) => {
            playlists.iter().map(|p| playlist_row(icons, p, state.lang(), row_width)).collect()
        }
        Items::Settings => {
            Setting::ALL.iter().enumerate().map(|(i, s)| setting_row(state, icons.settings[i], *s)).collect()
        }
        Items::Files(files, kind) => {
            files.iter().map(|f| file_row(icons, f, *kind, state.marked.contains(&f.path), row_width)).collect()
        }
        Items::Downloads(downloads) => downloads
            .iter()
            .map(|d| download_row(state, d, state.lang(), state.marked_downloads.contains(&d.id), row_width))
            .collect(),
        Items::About | Items::System => Vec::new(),
    };
    let focused = state.focus == Focus::List;
    let highlight = if focused { Style::new().bg(HIGHLIGHT).add_modifier(Modifier::BOLD) } else { Style::new() };
    let list = List::new(items)
        .block(focus_panel(title, focused))
        .highlight_symbol(if focused { "▌" } else { " " })
        .highlight_style(highlight);
    let selected = (!listing.items.is_empty()).then_some(listing.selected);
    let area = match (&listing.items, split_preview(area, state.config.ui.preview_width)) {
        (Items::Videos(videos), (left, Some(preview))) => {
            draw_preview(frame, state, icons, view, videos.get(listing.selected), preview);
            left
        }
        (Items::Channels(channels), (left, Some(preview))) => {
            draw_channel_preview(frame, state, icons, channels.get(listing.selected), preview);
            left
        }
        _ => area,
    };
    frame.render_stateful_widget(list, area, &mut ListState::default().with_selected(selected));
}

/// The selected video: thumbnail (drawn by the runtime over the empty cells
/// left here) and its details.
fn draw_preview(frame: &mut Frame, state: &State, icons: &Icons, view: &View, video: Option<&Video>, area: Rect) {
    let lang = state.lang();
    let block = panel(lang.pick(" Vista previa ", " Preview "), MUTED);
    let inner = block.inner(area);
    frame.render_widget(block, area);
    let Some(v) = video else { return };
    let text_area = if view.thumbnails {
        let image = thumbnail_cells(area);
        frame.render_widget(
            Paragraph::new(icons.waiting).fg(MUTED).alignment(Alignment::Center),
            Rect { y: image.y + image.height / 2, height: 1, ..image },
        );
        Rect { y: image.bottom() + 1, height: inner.bottom().saturating_sub(image.bottom() + 1), ..inner }
    } else {
        inner
    };
    let mut details = vec![length_label(state, v)];
    if let Some(ts) = v.published {
        let (date, age) = date_labels(ts, view.now, lang);
        details.push(format!("{age} ({date})"));
    }
    let mut lines = vec![
        Line::from(v.title.as_str()).fg(FG).bold(),
        Line::from(""),
        Line::from(v.channel.as_str()).fg(MAGENTA),
        Line::from(details.into_iter().filter(|d| !d.is_empty()).collect::<Vec<_>>().join(" · ")).fg(MUTED),
    ];
    lines.extend(size_line(state, v));
    match v.live {
        Live::Now => lines.push(Line::from(format!("{} {}", icons.live, lang.pick("en vivo", "live"))).fg(RED)),
        Live::Upcoming => {
            lines.push(Line::from(format!("{} {}", icons.upcoming, lang.pick("próximo", "upcoming"))).fg(YELLOW))
        }
        Live::No => {}
    }
    if let Some(at) = state.resume_at(&v.id) {
        lines.push(
            Line::from(format!("{} {} {}", icons.resume, lang.pick("Sigue desde", "Resumes at"), clock(at))).fg(YELLOW),
        );
    }
    lines.extend(later_lines(state, icons, &v.id, lang));
    if v.channel_id.as_ref().is_some_and(|c| state.subs.contains(c)) {
        lines.push(Line::from(format!("{} {}", icons.subscribed, lang.pick("Suscripto", "Subscribed"))).fg(GREEN));
    }
    lines.extend(rating_line(state, icons, &v.id));
    // The AI summary in all the room left, cut with "…" if longer (or that it has none).
    if let Some(summary) = state.summary_lookup(v) {
        let used =
            Paragraph::new(lines.clone()).wrap(ratatui::widgets::Wrap { trim: true }).line_count(text_area.width);
        let room = usize::from(text_area.height).saturating_sub(used);
        lines.extend(summary_lines(summary, text_area.width, room, lang, icons.summary));
    }
    frame.render_widget(Paragraph::new(lines).wrap(ratatui::widgets::Wrap { trim: true }), text_area);
}

/// "Te gusta" / "No te gusta" when I voted on the video.
fn rating_line(state: &State, icons: &Icons, id: &str) -> Option<Line<'static>> {
    let lang = state.lang();
    match state.ratings.get(id)? {
        Rating::Like => Some(Line::from(format!("{} {}", icons.liked, lang.pick("Te gusta", "Liked"))).fg(GREEN)),
        Rating::Dislike => {
            Some(Line::from(format!("{} {}", icons.disliked, lang.pick("No te gusta", "Disliked"))).fg(RED))
        }
        Rating::None => None,
    }
}

/// The selected channel: name, subscribers and its whole description.
fn draw_channel_preview(frame: &mut Frame, state: &State, icons: &Icons, channel: Option<&Channel>, area: Rect) {
    let lang = state.lang();
    let block = panel(lang.pick(" Canal ", " Channel "), MUTED);
    let inner = block.inner(area);
    frame.render_widget(block, area);
    let Some(c) = channel else { return };
    let mut lines = vec![Line::from(c.name.as_str()).fg(MAGENTA).bold()];
    if let Some(handle) = &c.handle {
        lines.push(Line::from(handle.as_str()).fg(CYAN));
    }
    if let Some(followers) = &c.followers {
        lines.push(Line::from(followers.as_str()).fg(MUTED));
    }
    if state.subs.contains(&c.id) {
        lines.push(Line::from(format!("{} {}", icons.subscribed, lang.pick("Suscripto", "Subscribed"))).fg(GREEN));
    }
    lines.push(Line::from(""));
    match &c.description {
        Some(description) => lines.extend(description.lines().map(|l| Line::from(l).fg(FG))),
        None => lines.push(Line::from(lang.pick("Sin descripción", "No description")).fg(MUTED)),
    }
    frame.render_widget(Paragraph::new(lines).wrap(ratatui::widgets::Wrap { trim: true }), inner);
}

/// `watched_date`: the date column says when I watched it (Historial).
/// A video row `width` cells wide (every video list and Up Next): a long
/// title is cut with `…` so the length always shows whole at the right edge.
fn video_row_dated<'a>(
    state: &State,
    icons: &Icons,
    view: &View,
    v: &'a Video,
    watched_date: bool,
    width: usize,
) -> ListItem<'a> {
    let (head, title, tail) = video_row_parts(state, icons, view, v, watched_date);
    ListItem::new(Line::from(fit_row(head, title, tail, width)))
}

/// `head` + `title` + `tail` in `width` cells, `tail` flush right. The title
/// gives way first, then the head; the tail only when nothing else is left.
fn fit_row<'a>(head: Vec<Span<'a>>, title: Span<'a>, tail: Vec<Span<'a>>, width: usize) -> Vec<Span<'a>> {
    let cells = |spans: &[Span]| spans.iter().map(Span::width).sum::<usize>();
    let tail_width = cells(&tail);
    if tail_width >= width {
        let mut out = head;
        out.push(title);
        out.extend(tail);
        return out;
    }
    let room = width - tail_width;
    let mut out = Vec::new();
    let mut used = 0;
    for span in head.into_iter().chain(std::iter::once(title)) {
        let span = Span::styled(plain_cells(&span.content), span.style);
        let w = span.width();
        if used + w <= room {
            used += w;
            out.push(span);
            continue;
        }
        let (text, w) = cut_cells(&span.content, room - used);
        used += w;
        out.push(Span::styled(text, span.style));
        break;
    }
    out.push(Span::raw(" ".repeat(room - used)));
    out.extend(tail);
    out
}

/// The text cut to at most `max` cells, ending in `…` when it was cut; and
/// the cells it takes.
fn cut_cells(text: &str, max: usize) -> (String, usize) {
    use unicode_width::UnicodeWidthStr;
    let whole = UnicodeWidthStr::width(text);
    if whole <= max {
        return (text.to_owned(), whole);
    }
    if max == 0 {
        return (String::new(), 0);
    }
    // Measured as a whole string each time, the way ratatui measures the
    // line (ligatures like Arabic lam-alef take one cell, not two).
    let mut out = String::new();
    for c in text.chars() {
        out.push(c);
        if UnicodeWidthStr::width(out.as_str()) > max - 1 {
            out.pop();
            break;
        }
    }
    out.push('…');
    let used = UnicodeWidthStr::width(out.as_str());
    (out, used)
}

/// Text for a fitted cell: no line breaks, and none of the invisible
/// direction marks (RLM, embeddings, isolates) that terminals draw
/// differently, so right-to-left titles keep the columns in place.
fn plain_cells(text: &str) -> String {
    text.chars()
        .filter(
            |c| !matches!(c, '\u{200E}' | '\u{200F}' | '\u{061C}' | '\u{202A}'..='\u{202E}' | '\u{2066}'..='\u{2069}'),
        )
        .map(|c| if c.is_control() { ' ' } else { c })
        .collect()
}

type RowParts<'a> = (Vec<Span<'a>>, Span<'a>, Vec<Span<'a>>);

fn video_row_parts<'a>(state: &State, icons: &Icons, view: &View, v: &'a Video, watched_date: bool) -> RowParts<'a> {
    let lang = state.lang();
    let watched_at = watched_date.then(|| state.watched_at(&v.id)).flatten();
    let live = if watched_at.is_some() { Live::No } else { v.live };
    let mut spans = match (live, watched_at.or(v.published)) {
        (Live::Now, _) => {
            let live = format!("{} {}", icons.live, lang.pick("en vivo", "live"));
            vec![Span::styled(pad_cells(&live, DATE_WIDTH), RED)]
        }
        (Live::Upcoming, _) => {
            let upcoming = format!("{} {}", icons.upcoming, lang.pick("próximo", "upcoming"));
            vec![Span::styled(pad_cells(&upcoming, DATE_WIDTH), YELLOW)]
        }
        (Live::No, Some(ts)) => {
            let (date, age) = date_labels(ts, view.now, lang);
            vec![Span::styled(date, GREEN), Span::styled(format!(" {age:<5}"), MUTED)]
        }
        (Live::No, None) => vec![Span::raw(" ".repeat(DATE_WIDTH))],
    };
    spans.push(Span::styled(
        format!(" {} ", pad_cells(&cut_cells(&plain_cells(&v.channel), CHANNEL_WIDTH).0, CHANNEL_WIDTH)),
        MAGENTA,
    ));
    if state.fresh_shown.contains(&v.id) {
        spans.push(Span::styled(format!("{} ", lang.pick("NUEVO", "NEW")), Style::new().fg(GREEN).bold()));
    }
    if state.is_favorite(v) {
        spans.push(Span::raw(format!("{} ", icons.favorite)));
    }
    if state.later.iter().any(|l| l.id == v.id) {
        spans.push(Span::raw(format!("{} ", icons.later)));
    }
    if state.watched.contains(&v.id) {
        // Already seen: marked and dimmed.
        spans.push(Span::styled(format!("{} ", icons.watched), MUTED));
    }
    let title = Span::styled(v.title.clone(), if state.watched.contains(&v.id) { MUTED } else { FG });
    let mut tail = vec![Span::styled(format!("  {}", v.duration_label()), MUTED)];
    if let Some(at) = state.resume_at(&v.id) {
        tail.extend(resume_bar(at, v.duration));
    }
    (spans, title, tail)
}

/// How far I got: `  ━━━━─────` (or `  ▸ 12:34` when the length is unknown).
fn resume_bar(at: u32, duration: Option<u32>) -> Vec<Span<'static>> {
    const CELLS: usize = 10;
    match duration.filter(|d| *d > 0) {
        Some(d) => {
            let done = ((u64::from(at) * CELLS as u64 / u64::from(d)) as usize).clamp(1, CELLS);
            vec![Span::raw("  "), Span::styled("━".repeat(done), RED), Span::styled("─".repeat(CELLS - done), MUTED)]
        }
        None => vec![Span::styled(format!("  ▸ {}", clock(at)), YELLOW)],
    }
}

/// Name, subscribers and @handle in columns; the description too unless
/// the side panel shows it.
fn channel_row<'a>(state: &State, icons: &Icons, c: &'a Channel, panel: bool) -> ListItem<'a> {
    let mark = if state.subs.contains(&c.id) { icons.subscribed } else { " " };
    let column = |text: &str, width: usize| pad_cells(&truncate(text, width - 2), width);
    let mut spans = vec![
        Span::styled(format!(" {mark} "), GREEN),
        Span::styled(column(&c.name, CHANNEL_NAME_WIDTH), MAGENTA),
        Span::styled(column(c.followers.as_deref().unwrap_or(""), FOLLOWERS_WIDTH), MUTED),
    ];
    if let Some(n) = state.channel_views.get(&c.id).filter(|_| state.config.ui.favorites_first) {
        let star = if state.favorites.contains(&c.id) { icons.favorite } else { " " };
        let word = if *n == 1 { state.lang().pick("visto", "watched") } else { state.lang().pick("vistos", "watched") };
        let seen = format!("{star} {n} {word}");
        spans.push(Span::styled(pad_cells(&seen, 14), YELLOW));
    }
    if let Some(handle) = &c.handle {
        spans.push(Span::styled(handle.as_str(), CYAN));
    }
    if let Some(description) = c.description.as_ref().filter(|_| !panel) {
        spans.push(Span::styled(
            format!("  {}", description.replace('\n', " ")),
            Style::new().fg(FG).add_modifier(Modifier::DIM),
        ));
    }
    ListItem::new(Line::from(spans))
}

/// Every row starts with one icon from the icon set: YouTube's own lists
/// their own (liked, Watch Later), the rest the playlists one, so the names
/// line up.
fn playlist_row<'a>(icons: &Icons, playlist: &'a Playlist, lang: Lang, width: usize) -> ListItem<'a> {
    let icon = match playlist.id.as_str() {
        crate::app::LIKED => icons.liked,
        crate::youtube::WATCH_LATER => icons.later,
        _ => icons.section(Section::MisListas),
    };
    // A long name is cut with `…`; the count sits at the right edge.
    let name = playlist.name(lang).to_owned();
    let tail = playlist.count.iter().map(|c| Span::styled(c.as_str(), MUTED)).collect();
    let head = vec![Span::raw(format!(" {icon} "))];
    ListItem::new(Line::from(fit_row(head, Span::styled(name, FG), tail, width)))
}

/// `⇩  45.3% ━━━━━─────  2.1MiB/s  00:12   title · channel`.
/// How long `video` is, from the list or, when it did not say, from yt-dlp.
fn length_label(state: &State, video: &Video) -> String {
    video.duration.or_else(|| state.sizes.get(&video.id).and_then(|s| s.duration)).map(clock).unwrap_or_default()
}

/// `"📥 Download ≈ 30 MB (720p)"` (the Downloads icon of the icon set), or
/// that it is being worked out.
fn size_line(state: &State, video: &Video) -> Option<Line<'static>> {
    let lang = state.lang();
    let icon = icons(state.config.ui.icons).section(Section::Descargas);
    let text = match state.sizes.get(&video.id) {
        Some(size) if size.bytes.is_some() => format!("{icon} {} {}", lang.pick("Descarga", "Download"), size.label()),
        Some(_) => return None,
        None if video.live == Live::No => {
            format!("{icon} {}", lang.pick("Calculando el peso…", "Working out the size…"))
        }
        None => return None,
    };
    Some(Line::from(text).fg(MUTED))
}

fn download_row(state: &State, download: &Download, lang: Lang, marked: bool, width: usize) -> ListItem<'static> {
    const BAR: usize = 12;
    let icons = icons(state.config.ui.icons);
    // The same mark as My Videos / My Music.
    let mut spans = vec![Span::styled(if marked { icons.marked } else { " " }, Style::new().fg(RED).bold())];
    // Every state's symbol takes two cells, whatever the icon set.
    let symbol = |icon: &str| pad_cells(icon, 2);
    spans.extend(match &download.state {
        DownloadState::Running { permille, speed, eta } => {
            let done = usize::from(*permille) * BAR / 1000;
            vec![
                Span::styled(
                    format!(" {}{:>5.1}% ", symbol(icons.downloading), f64::from(*permille) / 10.0),
                    Style::new().fg(YELLOW).bold(),
                ),
                Span::styled("━".repeat(done), RED),
                Span::styled("─".repeat(BAR - done), MUTED),
                Span::styled(format!(" {speed:>10} {eta:>8}  "), MUTED),
            ]
        }
        DownloadState::Done(_) => {
            vec![Span::styled(
                pad_cells(&format!(" {}{}", symbol(icons.up_to_date), lang.pick("lista", "done")), 41),
                GREEN,
            )]
        }
        DownloadState::Failed(_) => {
            vec![Span::styled(
                pad_cells(&format!(" {}{}", symbol(icons.failed), lang.pick("falló", "failed")), 41),
                RED,
            )]
        }
        DownloadState::Cancelled => {
            vec![Span::styled(
                pad_cells(&format!(" {}{}", symbol(icons.stop), lang.pick("cancelada", "cancelled")), 41),
                MUTED,
            )]
        }
    });
    spans.push(Span::styled(download.video.title.clone(), FG));
    let channel = cut_cells(&plain_cells(&download.video.channel), CHANNEL_WIDTH).0;
    spans.push(Span::styled(format!(" · {channel}"), MAGENTA));
    let size = match &download.state {
        DownloadState::Done(path) => std::fs::metadata(path).ok().map(|m| crate::domain::bytes_label(m.len())),
        _ => state.sizes.get(&download.video.id).map(Size::label).filter(|l| !l.is_empty()),
    };
    if let Some(size) = size {
        spans.push(Span::styled(format!("  {size}"), YELLOW));
    }
    // The path or error gives way first, then the title.
    let last = match &download.state {
        DownloadState::Done(path) => Span::styled(format!("  {}", path.display()), MUTED),
        DownloadState::Failed(error) => Span::styled(format!("  {error}"), RED),
        _ => Span::raw(""),
    };
    ListItem::new(Line::from(fit_row(spans, last, Vec::new(), width)))
}

fn profile_row<'a>(state: &State, icons: &Icons, spec: &'a str) -> ListItem<'a> {
    let current = if spec == state.config.session.browser && state.config.session.cookies_file.is_empty() {
        format!("{} ", icons.current)
    } else {
        "  ".into()
    };
    let label = if spec.is_empty() {
        state.lang().pick("Sin sesión", "Not Logged In").to_owned()
    } else {
        session::label(spec)
    };
    ListItem::new(Line::from(vec![
        Span::styled(format!(" {current}"), GREEN),
        Span::raw(format!("{} ", icons.user)),
        Span::styled(label, FG),
    ]))
}

fn file_row<'a>(icons: &Icons, file: &'a LocalFile, kind: MediaKind, marked: bool, width: usize) -> ListItem<'a> {
    let icon = match kind {
        MediaKind::Video => icons.section(Section::MisVideos),
        MediaKind::Music => icons.section(Section::MiMusica),
    };
    let folder = pad_cells(&cut_cells(&plain_cells(&file.folder), 28).0, 28);
    let head = vec![
        Span::styled(if marked { icons.marked } else { " " }, Style::new().fg(RED).bold()),
        Span::raw(format!("{icon} ")),
        Span::styled(format!("{folder} "), MAGENTA),
    ];
    ListItem::new(Line::from(fit_row(head, Span::styled(file.name.as_str(), FG), Vec::new(), width)))
}

/// Width of the settings' label column: the longest label and a gap, so
/// the values line up.
fn setting_label_width(lang: Lang) -> usize {
    Setting::ALL.iter().map(|s| Span::raw(s.label(lang)).width()).max().unwrap_or(0) + 3
}

fn setting_row(state: &State, icon: &str, setting: Setting) -> ListItem<'static> {
    let value = state.setting_value(setting);
    let lang = state.lang();
    let mut spans = vec![
        Span::raw(format!(" {} ", pad_cells(icon, 2))),
        Span::styled(pad_cells(setting.label(lang), setting_label_width(lang)), FG),
    ];
    if !value.is_empty() {
        spans.push(Span::styled(value, Style::new().fg(YELLOW).bold()));
    }
    ListItem::new(Line::from(spans))
}

fn draw_status(frame: &mut Frame, state: &State, icons: &Icons, view: &View, area: Rect) {
    let right = format!(
        "{} {}  {} {} ",
        icons.user,
        state.session_label(),
        icons.language,
        language_label(state.config.player.language, state.lang())
    );
    let right_width = Span::raw(right.as_str()).width() as u16 + 1;
    let [left_area, right_area] =
        Layout::horizontal([Constraint::Fill(1), Constraint::Length(right_width)]).areas(area);
    let left = match &state.loading {
        Some(title) => {
            Paragraph::new(format!(" {}", loading_text(title, view.loading.unwrap_or_default(), state.lang())))
                .fg(YELLOW)
        }
        None => Paragraph::new(format!(" {}", state.status)).fg(FG),
    };
    // A shaded bar, so it reads apart from the keys below it.
    frame.render_widget(Block::new().bg(HIGHLIGHT), area);
    frame.render_widget(left, left_area);
    frame.render_widget(Paragraph::new(right).fg(BLUE).alignment(Alignment::Right), right_area);
}

/// Keys that work right now, each with what it does; English labels in
/// Title Case.
pub fn hints(state: &State) -> Vec<(String, String)> {
    let english = state.lang().pick(false, true);
    hint_rows(state).into_iter().map(|(k, d)| (k, if english { title_case(d) } else { d.to_owned() })).collect()
}

/// `"back to the video"` → `"Back to the Video"`. Short joining words stay
/// lowercase (but never the first one); `tubevibe` and words that already
/// carry capitals (YouTube, PgUp) are left as they are.
fn title_case(text: &str) -> String {
    const SMALL: &[&str] = &["a", "an", "the", "to", "of", "from", "in", "on", "and", "or", "for", "at", "by", "as"];
    let mut out = String::with_capacity(text.len());
    let mut first = true;
    for (i, word) in text.split(' ').enumerate() {
        if i > 0 {
            out.push(' ');
        }
        let bare = word.trim_start_matches(|c: char| !c.is_alphanumeric());
        let keep = bare.is_empty()
            || bare.starts_with("tubevibe")
            || bare.chars().any(char::is_uppercase)
            || (!first
                && !word.starts_with('(')
                && SMALL.contains(&bare.trim_end_matches(|c: char| !c.is_alphanumeric())));
        if keep {
            out.push_str(word);
        } else {
            let lead = &word[..word.len() - bare.len()];
            let mut chars = bare.chars();
            out.push_str(lead);
            out.extend(chars.next().map(|c| c.to_uppercase()).into_iter().flatten());
            out.push_str(chars.as_str());
        }
        if !bare.is_empty() {
            first = false;
        }
    }
    out
}

fn hint_rows(state: &State) -> Vec<(String, &'static str)> {
    let lang = state.lang();
    let rows = |table| keymap::rows(table, lang);
    let key = |k: &str, es, en| (k.to_string(), lang.pick(es, en));
    // Every shortcut, explained: the one hint always shown.
    let all_keys = key("Shift+/", "Ayuda", "Help");
    // Moving around (arrows, j k, Tab, Esc) is in the help, not the bar.
    let normal = state.confirm.is_none() && !state.help && state.context.is_none() && state.prompt.is_none();
    // `s` asks before unsubscribing; it says so when it will.
    let subscribe = if state.selected_channel().is_some_and(|(id, ..)| state.subs.contains(&id)) {
        key("s", "desuscribirse (pide confirmar)", "unsubscribe (asks first)")
    } else {
        key("s", "suscribirse", "subscribe")
    };
    let mut hints = if let Some(confirm) = &state.confirm {
        let yes = match confirm {
            Confirm::Unsubscribe { .. } => key(lang.pick("s", "y"), "sí, desuscribirme", "yes, unsubscribe"),
            Confirm::TrashDownloads { cancel, remove, .. } if !cancel.is_empty() || !remove.is_empty() => key(
                lang.pick("s", "y"),
                "sí (las otras marcadas se cancelan o se quitan)",
                "yes (the other marked ones get cancelled or taken off)",
            ),
            Confirm::Trash(_) | Confirm::TrashDownload { .. } | Confirm::TrashDownloads { .. } => {
                key(lang.pick("s", "y"), "sí, a la papelera", "yes, to the trash")
            }
            Confirm::Quit(0) => key(lang.pick("s", "y"), "sí, salir", "yes, quit"),
            Confirm::Quit(_) => key(lang.pick("s", "y"), "sí, salir y cortarlas", "yes, quit and stop them"),
            Confirm::ForgetHistory(_) => key(lang.pick("s", "y"), "sí, quitarlo", "yes, remove it"),
            Confirm::Download { running: true, .. } => key(lang.pick("s", "y"), "sí, cancelarla", "yes, cancel it"),
            Confirm::Download { .. } => key(lang.pick("s", "y"), "sí, quitarla", "yes, remove it"),
            Confirm::CloseAndRemove { .. } => key(lang.pick("s", "y"), "sí, cerrar y quitar", "yes, close and remove"),
            Confirm::DeletePlaylist { .. } => key(lang.pick("s", "y"), "sí, borrarla", "yes, delete it"),
            Confirm::RemoveFromLater(_) | Confirm::RemoveFromPlaylist { .. } | Confirm::RemoveFromSaved(_) => {
                key(lang.pick("s", "y"), "sí, quitarlo", "yes, remove it")
            }
        };
        vec![yes, key(lang.pick("Otra tecla", "Other Key"), "no", "no")]
    } else if state.help {
        vec![key(lang.pick("Cualquier tecla", "Any Key"), "cerrar la ayuda", "close the help")]
    } else if state.context.is_some() {
        vec![
            key("↑ ↓", "elegir", "choose"),
            key(lang.pick("Enter / Clic", "Enter / Click"), "hacerlo", "do it"),
            key("Esc", "cerrar", "close"),
        ]
    } else if let Some(prompt) = &state.prompt {
        match prompt {
            Prompt::Videos { .. } => rows(keymap::IN_SEARCH),
            _ => vec![key("Enter", "aceptar", "accept"), key("Esc", "cancelar", "cancel")],
        }
    } else if state.focus == Focus::Menu && state.active_tab().is_some() {
        let mut keys = rows(keymap::IN_MENU);
        if matches!(state.menu_row(), MenuRow::Tab(_)) {
            keys.push(key("x", "Cerrar pestaña", "Close Tab"));
        }
        keys
    } else if state.key_context().related {
        vec![
            key("Enter", "ver", "play"),
            key("↑ ↓", "elegir", "choose"),
            key("Tab", "volver al video", "back to the video"),
            key("w", "ver después YouTube", "YouTube Watch Later"),
            key("l", "ver después tubevibe", "tubevibe Watch Later"),
            key("c", "canal", "channel"),
            if state.active_tab().is_some_and(|t| t.ended && !t.started) {
                key(keymap::space(lang), "reproducir", "play")
            } else if state.active_tab().is_some_and(|t| t.ended) {
                key(keymap::space(lang), "volver a verlo", "play it again")
            } else {
                key(keymap::space(lang), "pausa", "pause")
            },
        ]
    } else if state.active_tab().is_some() {
        let mut keys = Vec::new();
        if state.active_tab().is_some_and(|t| t.ended && !t.started) {
            keys.push(key("Enter", "reproducir", "play"));
        } else if state.active_tab().is_some_and(|t| t.ended) {
            keys.push(key("Enter", "volver a verlo", "play it again"));
        }
        keys.push(key(keymap::space(lang), "pausa", "pause"));
        if state.related_shown().is_some() {
            keys.push(key("Tab", "A continuación", "Up Next"));
        }
        keys.push(key("x", "cerrar la pestaña", "close the tab"));
        if state.active_tab().is_some_and(|t| t.source.is_some()) {
            keys.push(key("X", "cerrar y quitar de la lista", "close and remove from the list"));
        }
        keys.push(key("v", "Cambiar calidad", "Quality Change"));
        keys.push(key("b", "Cambiar códec", "Codec Change"));
        // The rest of the tab's keys, but the ones above and Up Next's own,
        // which the help explains.
        let shown = ["Espacio", "Enter", "v", "b", "Tab", "j k", "Esc", "x", "X"];
        keys.extend(
            keymap::IN_TAB
                .iter()
                .filter(|(k, _)| !shown.contains(k))
                .map(|(k, d)| (keymap::key_name(k, lang), d.get(lang))),
        );
        keys.extend(rows(keymap::ON_VIDEO));
        keys
    } else if state.focus == Focus::Menu || state.listing().is_none() {
        let mut keys = rows(keymap::IN_MENU);
        if state.focus == Focus::Menu && matches!(state.menu_row(), MenuRow::Tab(_)) {
            keys.push(key("x", "Cerrar pestaña", "Close Tab"));
        }
        if state.listing().is_some_and(|l| l.items == Items::About) && about_button_ready(state) {
            keys.insert(0, key("Tab", "elegir Actualizar Ahora", "select Update Now"));
        }
        keys
    } else {
        let back = key("Esc", "volver", "back");
        match state.listing().map(|l| &l.items) {
            Some(Items::Videos(_)) if state.listing().is_some_and(|l| l.all.is_some()) => vec![
                key("Enter", "ver", "play"),
                if state.show_watched {
                    key("v", "ocultar vistos", "hide watched")
                } else {
                    key("v", "mostrar vistos", "show watched")
                },
                key("w", "ver después YouTube", "YouTube Watch Later"),
                key("W", "ver después tubevibe", "tubevibe Watch Later"),
                key("a", "solo audio", "audio only"),
                subscribe.clone(),
                key("g", "me gusta", "like"),
                key("d", "descargar", "download"),
                key("c", "canal", "channel"),
                key("/", "buscar", "search"),
                back,
            ],
            Some(Items::Videos(_)) if state.listing().is_some_and(|l| l.kind == ListKind::Saved) => vec![
                key("Enter", "ver", "play"),
                key("X", "quitar de la lista", "remove from the list"),
                key("w", "ver después YouTube", "YouTube Watch Later"),
                key("a", "solo audio", "audio only"),
                key("c", "canal", "channel"),
                back,
            ],
            Some(Items::Videos(_)) if state.listing().is_some_and(|l| l.kind == ListKind::History) => vec![
                key("Enter", "ver (sigue donde lo dejaste)", "play (resumes where you left)"),
                key("X", "quitar del historial", "remove from history"),
                key("w", "ver después YouTube", "YouTube Watch Later"),
                key("W", "ver después tubevibe", "tubevibe Watch Later"),
                key("c", "canal", "channel"),
                back,
            ],
            Some(Items::Videos(_))
                if state.listing().is_some_and(|l| matches!(l.kind, ListKind::Later | ListKind::Playlist(_))) =>
            {
                vec![
                    key("Enter", "ver", "play"),
                    key("X", "quitar de la lista", "remove from the list"),
                    key("W", "ver después tubevibe", "tubevibe Watch Later"),
                    key("a", "solo audio", "audio only"),
                    key("d", "descargar", "download"),
                    key("c", "canal", "channel"),
                    back,
                ]
            }
            Some(Items::Videos(_)) => vec![
                key("Enter", "ver", "play"),
                key("w", "ver después YouTube", "YouTube Watch Later"),
                key("W", "ver después tubevibe", "tubevibe Watch Later"),
                key("a", "solo audio", "audio only"),
                subscribe.clone(),
                key("g", "me gusta", "like"),
                key("d", "descargar", "download"),
                key("c", "canal", "channel"),
                key("/", "buscar", "search"),
                back,
            ],
            Some(Items::Channels(_)) => vec![key("Enter", "ver sus videos", "its videos"), subscribe, back],
            Some(Items::Profiles(_)) => vec![key("Enter", "usar esta sesión", "use this login"), back],
            Some(Items::Playlists(_)) => vec![
                key("Enter", "ver sus videos", "its videos"),
                key("X", "borrar la lista (pide confirmar)", "delete the playlist (asks first)"),
                back,
            ],
            Some(Items::Files(_, kind)) => vec![
                match kind {
                    MediaKind::Music => key("Enter", "escuchar desde esta canción", "play from this song"),
                    MediaKind::Video => key("Enter", "ver", "play"),
                },
                // Several files: mark them first, then `X` sends them all.
                if state.marked_files().is_empty() {
                    key(keymap::space(lang), "marcar varios", "mark several")
                } else {
                    key(keymap::space(lang), "marcar / desmarcar", "mark / unmark")
                },
                if state.marked_files().is_empty() {
                    key("X", "este a la papelera (pide confirmar)", "this one to the trash (asks first)")
                } else {
                    key("X", "los marcados a la papelera (pide confirmar)", "the marked ones to the trash (asks first)")
                },
                back,
            ],
            Some(Items::Downloads(_)) if !state.marked_download_ids().is_empty() => vec![
                key(keymap::space(lang), "marcar / desmarcar", "mark / unmark"),
                key("X", "las marcadas: papelera (pide confirmar)", "the marked ones: trash (asks first)"),
                back,
            ],
            Some(Items::Downloads(downloads)) => {
                let selected = state.listing().and_then(|l| downloads.get(l.selected)).map(|d| &d.state);
                let mut keys = Vec::new();
                match selected {
                    Some(DownloadState::Running { .. }) => {
                        keys.push(key("X", "cancelar la descarga (pide confirmar)", "cancel the download (asks first)"))
                    }
                    Some(DownloadState::Done(_)) => {
                        keys.push(key("Enter", "ver", "play"));
                        keys.push(key("X", "a la papelera (pide confirmar)", "move to trash (asks first)"));
                    }
                    Some(_) => {
                        keys.push(key("X", "quitar de la lista (pide confirmar)", "take off the list (asks first)"))
                    }
                    None => {}
                }
                if selected.is_some() {
                    keys.push(key(keymap::space(lang), "marcar varios", "mark several"));
                }
                keys.push(back);
                keys
            }
            Some(Items::About) if about_button_ready(state) => {
                let press = if restart_ready(state) {
                    key("Enter", "reiniciar ahora", "restart now")
                } else {
                    key("Enter", "actualizar ahora", "update now")
                };
                vec![press, back]
            }
            Some(Items::About) => vec![back],
            Some(Items::System) => vec![key("Enter", "revisar de nuevo", "check again"), back],
            _ => vec![key("Enter", "cambiar / abrir", "change / open"), back],
        }
    };
    if normal {
        // The About button is chosen with Tab: that one stays.
        const MOVING: &[&str] = &["↑ ↓", "↑ ↓  j k", "j k", "Esc", "Esc  ←  h", "←  Esc"];
        let menu = state.focus == Focus::Menu;
        hints.retain(|(k, d)| {
            let select_button = k == "Tab"
                && (d.contains("Actualizar")
                    || d.contains("Update")
                    || d.contains("Reiniciar")
                    || d.contains("Restart"));
            !(MOVING.contains(&k.as_str()) || (k == "Tab" && !select_button) || (menu && k == "Enter"))
        });
    }
    hints.push(all_keys);
    hints
}

/// As many hints as fit in the line, whole; the last one (`Shift+/`, every
/// shortcut) is always kept.
fn draw_hints(frame: &mut Frame, hints: &[(String, String)], area: Rect) {
    let hint = |(key, description): &(String, String)| {
        [Span::styled(key.clone(), Style::new().fg(CYAN).bold()), Span::styled(format!(" {description}   "), MUTED)]
    };
    let width = |spans: &[Span]| spans.iter().map(Span::width).sum::<usize>();
    let Some((last, rest)) = hints.split_last() else { return };
    let last = hint(last);
    let mut spans = vec![Span::raw(" ")];
    let room = (area.width as usize).saturating_sub(width(&last) + 1);
    for h in rest {
        let next = hint(h);
        if width(&spans) + width(&next) > room {
            break;
        }
        spans.extend(next);
    }
    spans.extend(last);
    frame.render_widget(Paragraph::new(Line::from(spans)), area);
}

/// Where the help or the confirmation dialog is drawn, if one is open.
pub fn dialog_area(state: &State, screen: Rect) -> Option<Rect> {
    if state.help {
        Some(help_area(screen, state.lang()))
    } else {
        state.confirm.as_ref().map(|_| centered(screen, 56, 5))
    }
}

/// The help screen's two balanced columns, so everything fits on a laptop screen.
type HelpSection = (&'static str, Vec<(String, &'static str)>);

/// The help's sections in two columns of about the same height.
fn help_columns(lang: Lang) -> (Vec<HelpSection>, Vec<HelpSection>) {
    let mut sections = keymap::help(lang);
    let sizes: Vec<usize> = sections.iter().map(|(_, rows)| help_box_height(rows.len()) as usize).collect();
    let split =
        (1..sections.len()).min_by_key(|&k| sizes[..k].iter().sum::<usize>().max(sizes[k..].iter().sum())).unwrap_or(1);
    let right = sections.split_off(split);
    (sections, right)
}

/// A section's box: its rows between two border lines.
fn help_box_height(rows: usize) -> u16 {
    rows as u16 + 2
}

fn help_column_height(sections: &[HelpSection]) -> u16 {
    sections.iter().map(|(_, rows)| help_box_height(rows.len())).sum()
}

/// The help's size: the taller column inside the outer border.
fn help_area(screen: Rect, lang: Lang) -> Rect {
    let (left, right) = help_columns(lang);
    centered(screen, 118, help_column_height(&left).max(help_column_height(&right)) + 2)
}

fn draw_help(frame: &mut Frame, lang: Lang) {
    let (left, right) = help_columns(lang);
    let area = help_area(frame.area(), lang);
    frame.render_widget(Clear, area);
    let title =
        lang.pick(" ⌨  Atajos de teclado · cualquier tecla cierra ", " ⌨  Keyboard Shortcuts · Any Key Closes ");
    let block = panel(title, MAGENTA);
    let inner = block.inner(area);
    frame.render_widget(block, area);
    let columns: [Rect; 2] = Layout::horizontal([Constraint::Fill(1), Constraint::Fill(1)]).spacing(1).areas(inner);
    for (sections, column) in [left, right].iter().zip(columns) {
        let heights = sections.iter().map(|(_, rows)| Constraint::Length(help_box_height(rows.len())));
        for ((title, rows), cell) in sections.iter().zip(Layout::vertical(heights).split(column).iter()) {
            draw_help_section(frame, title, rows, *cell);
        }
    }
}

/// One section of the help in its own box: keys aligned, then what they do.
fn draw_help_section(frame: &mut Frame, title: &str, rows: &[(String, &'static str)], area: Rect) {
    let width = rows.iter().map(|(keys, _)| Span::raw(keys.as_str()).width()).max().unwrap_or(0) + 2;
    let lines: Vec<Line> = rows
        .iter()
        .map(|(keys, description)| {
            Line::from(vec![
                Span::styled(format!(" {}", pad_cells(keys, width)), Style::new().fg(CYAN).bold()),
                Span::styled(*description, FG),
            ])
        })
        .collect();
    let block = framed().border_style(Style::new().fg(LINE)).title(Line::from(format!(" {title} ")).fg(BLUE).bold());
    frame.render_widget(Paragraph::new(lines).block(block), area);
}

fn draw_confirm(frame: &mut Frame, confirm: &Confirm, lang: Lang) {
    let area = centered(frame.area(), 56, 5);
    let question = match confirm {
        Confirm::Unsubscribe { name, .. } => Line::from(vec![
            Span::raw(lang.pick("¿Desuscribirte de ", "Unsubscribe from ")),
            Span::styled(name.as_str(), Style::new().fg(MAGENTA).bold()),
            Span::raw("?"),
        ]),
        Confirm::Quit(0) => Line::from(lang.pick("¿Salir de tubevibe?", "Quit tubevibe?")),
        Confirm::ForgetHistory(video) => Line::from(vec![
            Span::raw(lang.pick("¿Quitar ", "Remove ")),
            Span::styled(truncate(&video.title, 24), Style::new().fg(MAGENTA).bold()),
            Span::raw(lang.pick(" del historial?", " from history?")),
        ]),
        Confirm::Download { title, running } => Line::from(vec![
            Span::raw(match (running, lang) {
                (true, Lang::Es) => "¿Cancelar la descarga de ",
                (true, _) => "Cancel the download of ",
                (false, Lang::Es) => "¿Quitar de la lista ",
                (false, _) => "Take off the list ",
            }),
            Span::styled(truncate(title, 24), Style::new().fg(MAGENTA).bold()),
            Span::raw("?"),
        ]),
        Confirm::CloseAndRemove { source, video, .. } => Line::from(vec![
            Span::raw(lang.pick("¿Cerrar y quitar ", "Close and remove ")),
            Span::styled(truncate(&video.title, 20), Style::new().fg(MAGENTA).bold()),
            Span::raw(lang.pick(" de ", " from ")),
            Span::styled(
                match source {
                    crate::app::TabSource::YouTubeLater => lang.pick(YOUTUBE_LATER.0, YOUTUBE_LATER.1).to_owned(),
                    crate::app::TabSource::Saved => lang.pick(LOCAL_LATER.0, LOCAL_LATER.1).to_owned(),
                    crate::app::TabSource::Playlist { title, .. } => truncate(title, 20),
                },
                Style::new().fg(CYAN).bold(),
            ),
            Span::raw("?"),
        ]),
        Confirm::Quit(n) => Line::from(vec![
            Span::raw(lang.pick("Hay ", "There are ")),
            Span::styled(n.to_string(), Style::new().fg(MAGENTA).bold()),
            Span::raw(lang.pick(" descargas en curso. ¿Salir igual?", " downloads running. Quit anyway?")),
        ]),
        Confirm::DeletePlaylist { title, .. } => Line::from(vec![
            Span::raw(lang.pick("¿Borrar de YouTube la lista ", "Delete the playlist ")),
            Span::styled(truncate(title, 24), Style::new().fg(MAGENTA).bold()),
            Span::raw(lang.pick("?", " from YouTube?")),
        ]),
        Confirm::RemoveFromLater(video) => Line::from(vec![
            Span::raw(lang.pick("¿Quitar ", "Remove ")),
            Span::styled(truncate(&video.title, 24), Style::new().fg(MAGENTA).bold()),
            Span::raw(lang.pick(" de ", " from ")),
            Span::styled(lang.pick(YOUTUBE_LATER.0, YOUTUBE_LATER.1), Style::new().fg(CYAN).bold()),
            Span::raw("?"),
        ]),
        Confirm::RemoveFromSaved(video) => Line::from(vec![
            Span::raw(lang.pick("¿Quitar ", "Remove ")),
            Span::styled(truncate(&video.title, 24), Style::new().fg(MAGENTA).bold()),
            Span::raw(lang.pick(" de ", " from ")),
            Span::styled(lang.pick(LOCAL_LATER.0, LOCAL_LATER.1), Style::new().fg(CYAN).bold()),
            Span::raw("?"),
        ]),
        Confirm::RemoveFromPlaylist { playlist, video } => Line::from(vec![
            Span::raw(lang.pick("¿Quitar ", "Remove ")),
            Span::styled(truncate(&video.title, 24), Style::new().fg(MAGENTA).bold()),
            Span::raw(lang.pick(" de ", " from ")),
            Span::styled(truncate(&playlist.title, 20), Style::new().fg(CYAN).bold()),
            Span::raw("?"),
        ]),
        Confirm::TrashDownloads { files, .. } => Line::from(vec![
            Span::raw(lang.pick("¿Mandar ", "Move ")),
            Span::styled(files.len().to_string(), Style::new().fg(MAGENTA).bold()),
            Span::raw(match (files.len(), lang) {
                (1, Lang::Es) => " archivo a la papelera?",
                (_, Lang::Es) => " archivos a la papelera?",
                (1, _) => " file to the trash?",
                _ => " files to the trash?",
            }),
        ]),
        Confirm::TrashDownload { path, .. } => Line::from(vec![
            Span::raw(lang.pick("¿Mandar a la papelera ", "Move to the trash ")),
            Span::styled(
                truncate(&path.file_name().map_or_else(String::new, |n| n.to_string_lossy().into_owned()), 30),
                Style::new().fg(MAGENTA).bold(),
            ),
            Span::raw("?"),
        ]),
        Confirm::Trash(paths) => {
            let what = match paths.as_slice() {
                [one] => one.file_name().map_or_else(String::new, |n| n.to_string_lossy().into_owned()),
                many => format!("{} {}", many.len(), lang.pick("archivos", "files")),
            };
            Line::from(vec![
                Span::raw(lang.pick("¿Mandar a la papelera ", "Move to the trash ")),
                Span::styled(truncate(&what, 30), Style::new().fg(MAGENTA).bold()),
                Span::raw("?"),
            ])
        }
    };
    let lines = vec![
        Line::from(""),
        question,
        Line::from(lang.pick("s = sí · cualquier otra tecla = no", "y = yes · any other key = no")).fg(MUTED),
    ];
    let title = lang.pick(" Confirmar ", " Confirm ");
    frame.render_widget(Clear, area);
    frame.render_widget(Paragraph::new(lines).alignment(Alignment::Center).block(panel(title, RED)), area);
}

/// "tubevibe" in big letters (toilet's smmono12 font, 5 cells a letter).
const LOGO: [&str; 7] = [
    "          ▗▖               █  ▗▖        ",
    " ▐▌       ▐▌               ▀  ▐▌        ",
    "▐███ ▐▌ ▐▌▐▙█▙  ▟█▙ ▐▌ ▐▌ ██  ▐▙█▙  ▟█▙ ",
    " ▐▌  ▐▌ ▐▌▐▛ ▜▌▐▙▄▟▌▝█ █▘  █  ▐▛ ▜▌▐▙▄▟▌",
    " ▐▌  ▐▌ ▐▌▐▌ ▐▌▐▛▀▀▘ █ █   █  ▐▌ ▐▌▐▛▀▀▘",
    " ▐▙▄ ▐▙▄█▌▐█▄█▘▝█▄▄▌ ▐█▌ ▗▄█▄▖▐█▄█▘▝█▄▄▌",
    "  ▀▀  ▀▀▝▘▝▘▀▘  ▝▀▀   ▀  ▝▀▀▀▘▝▘▀▘  ▝▀▀ ",
];
/// YouTube's red, for the play button.
const YOUTUBE_RED: Color = Color::Rgb(255, 0, 51);

/// The red play button: a rounded box with a white triangle, drawn with half blocks.
fn play_button() -> Vec<Line<'static>> {
    const WIDTH: usize = 20;
    const START: usize = 6;
    // Width of the triangle on each half row, top to bottom.
    const HALVES: [usize; 10] = [2, 4, 6, 8, 9, 9, 8, 6, 4, 2];
    let edge = |c: &str| Line::from(format!(" {} ", c.repeat(WIDTH - 2))).fg(YOUTUBE_RED);
    let mut lines = vec![edge("▄")];
    for row in 0..5 {
        let cells: String = (0..WIDTH)
            .map(|c| {
                let inside = |half: usize| c >= START && c - START < HALVES[half];
                match (inside(row * 2), inside(row * 2 + 1)) {
                    (true, true) => '█',
                    (true, false) => '▀',
                    (false, true) => '▄',
                    (false, false) => ' ',
                }
            })
            .collect();
        lines.push(Line::from(cells).fg(Color::White).bg(YOUTUBE_RED));
    }
    lines.push(edge("▀"));
    lines
}

/// The logo for a moment when the app opens.
fn draw_splash(frame: &mut Frame, lang: Lang) {
    let area = frame.area();
    let tagline = lang.pick("YouTube en tu terminal", "YouTube in your terminal");
    let version = format!("v{}", env!("CARGO_PKG_VERSION"));
    let skip = lang.pick("cualquier tecla para seguir", "any key to go on");
    let text_width = LOGO.iter().map(|l| Span::raw(*l).width()).max().unwrap_or(0);
    let width = 20 + 2 + text_width as u16;
    if area.width < width || area.height < 12 {
        let line = Line::from(vec![
            Span::styled(" ▶ ", Style::new().fg(Color::White).bg(YOUTUBE_RED)),
            Span::raw(" tubevibe").bold(),
        ]);
        frame.render_widget(Paragraph::new(line).alignment(Alignment::Center), centered(area, area.width, 1));
        return;
    }
    let mut lines: Vec<Line> = play_button()
        .into_iter()
        .zip(LOGO)
        .map(|(button, text)| {
            // "tube" in the theme's text color, "vibe" in red, like YouTube's logo.
            let cut = text.char_indices().nth(20).map_or(text.len(), |(i, _)| i);
            let mut spans = button.spans.into_iter().map(|s| s.patch_style(button.style)).collect::<Vec<_>>();
            spans.push(Span::raw("  "));
            spans.push(Span::styled(&text[..cut], Style::new().fg(FG).bold()));
            spans.push(Span::styled(&text[cut..], Style::new().fg(YOUTUBE_RED).bold()));
            Line::from(spans)
        })
        .collect();
    lines.push(Line::from(""));
    lines.push(
        Line::from(vec![
            Span::styled(tagline, Style::new().fg(BLUE)),
            Span::styled(format!("  {version}"), Style::new().fg(MUTED)),
        ])
        .centered(),
    );
    lines.push(Line::from(""));
    lines.push(Line::from(skip).fg(MUTED).centered());
    let height = lines.len() as u16;
    frame.render_widget(Paragraph::new(lines), centered(area, width, height));
}

/// The right-click menu: each option with its key, the chosen one lit.
fn draw_context_menu(frame: &mut Frame, menu: &ContextMenu, lang: Lang) {
    let area = context_area(menu, frame.area());
    let block = panel(lang.pick(" Opciones ", " Options "), MAGENTA);
    let width = usize::from(block.inner(area).width);
    let lines: Vec<Line> = menu
        .items
        .iter()
        .enumerate()
        .map(|(i, item)| {
            let gap = width.saturating_sub(Span::raw(item.label).width() + Span::raw(item.key).width() + 2);
            let line = Line::from(vec![
                Span::raw(format!(" {}{}", item.label, " ".repeat(gap))),
                Span::styled(format!("{} ", item.key), Style::new().fg(MUTED)),
            ]);
            if i == menu.selected { line.style(Style::new().bg(HIGHLIGHT).fg(BLUE).bold()) } else { line }
        })
        .collect();
    frame.render_widget(Clear, area);
    frame.render_widget(Paragraph::new(lines).block(block), area);
}

/// What the system has and lacks, with the command that installs it.
fn draw_system(frame: &mut Frame, state: &State, area: Rect) {
    let lang = state.lang();
    let icons = icons(state.config.ui.icons);
    let title = format!(" {} ", lang.pick("Revisión del sistema", "System Check"));
    let block = panel(title, BLUE);
    let inner = block.inner(area);
    frame.render_widget(block, area);
    let Some((manager, checks)) = &state.system else {
        let wait = lang.pick("Revisando…", "Checking…");
        frame.render_widget(Paragraph::new(wait).fg(YELLOW).alignment(Alignment::Center), inner);
        return;
    };
    let mut lines = vec![Line::from("")];
    let width = checks.iter().map(|c| c.tool.program().len()).max().unwrap_or(0) + 2;
    for check in checks {
        let (mark, color, note) = match (&check.found, check.outdated) {
            (Some(_), false) => (icons.up_to_date, GREEN, String::new()),
            (Some((keyring, _)), true) if check.tool == Tool::Keyring => {
                let missing = if keyring.starts_with("KWALLET") { "kwallet-query" } else { "secretstorage" };
                (
                    icons.warning,
                    YELLOW,
                    format!(
                        "{}: {missing}",
                        lang.pick("yt-dlp no lo puede abrir, falta", "yt-dlp can't open it, missing")
                    ),
                )
            }
            (Some(_), true) => (
                icons.warning,
                YELLOW,
                lang.pick("viejo: YouTube cambia seguido", "too old: YouTube changes often").into(),
            ),
            (None, _) if check.tool.required() => {
                (icons.failed, RED, lang.pick("falta · necesario", "missing · needed").into())
            }
            (None, _) => (icons.optional, MUTED, lang.pick("no está · opcional", "not installed · optional").into()),
        };
        let version = check.found.as_ref().and_then(|(_, v)| v.clone()).unwrap_or_default();
        let mut row = vec![
            // Two cells for every mark, so the names line up in any icon set.
            Span::styled(format!("  {} ", pad_cells(mark, 2)), Style::new().fg(color).bold()),
            Span::styled(pad(check.tool.program(), width), Style::new().fg(FG).bold()),
            Span::styled(pad(&version, 16), MUTED),
            Span::styled(check.tool.purpose().get(lang), FG),
        ];
        if !note.is_empty() {
            row.push(Span::styled(format!("  ({note})"), color));
        }
        lines.push(Line::from(row));
        if let Some(fix) = &check.fix {
            lines.push(Line::from(vec![Span::raw(" ".repeat(width + 5)), Span::styled(fix.as_str(), CYAN)]));
        } else if check.found.is_none() {
            let how =
                lang.pick("instalalo con el gestor de paquetes de tu sistema", "install it with your package manager");
            lines.push(Line::from(vec![Span::raw(" ".repeat(width + 5)), Span::styled(how, MUTED)]));
        }
    }
    lines.push(Line::from(""));
    let footer = match manager {
        Manager::Unknown => lang.pick("Gestor de paquetes no reconocido", "Package manager not recognized").to_owned(),
        manager => format!("{}: {}", lang.pick("Comandos para", "Commands for"), manager.name()),
    };
    lines.push(
        Line::from(format!(
            "  {footer} · {}",
            lang.pick("Shift + arrastrar copia · Enter revisa de nuevo", "Shift + drag copies · Enter checks again",)
        ))
        .fg(MUTED),
    );
    frame.render_widget(Paragraph::new(lines).wrap(ratatui::widgets::Wrap { trim: false }), inner);
}

/// Whose the donation addresses are (they came with tubetui, unchanged).
const DONATIONS_NOTE: (&str, &str) = (
    "Estas direcciones son de polblancoo, el desarrollador original de tubetui: las donaciones van para él, \
     no para el mantenedor del fork tubevibe (mortbrew).",
    "These addresses belong to polblancoo, the original tubetui developer: donations go to him, \
     not to the tubevibe fork maintainer (mortbrew).",
);

/// Version, links and the donation addresses from `donate.toml`.
fn draw_about(frame: &mut Frame, state: &State, icons: &Icons, area: Rect) {
    let lang = state.lang();
    let label = |text: &'static str| Span::styled(format!("{text:>12}  "), MUTED);
    let mut lines = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled(format!("{} ", icons.logo), Style::new().fg(RED).bold()),
            Span::styled("tube", Style::new().fg(RED).bold()),
            Span::styled("vibe", Style::new().fg(FG).bold()),
            Span::styled(format!("  v{}", env!("CARGO_PKG_VERSION")), MUTED),
        ]),
        Line::from(lang.pick(
            "YouTube y tus videos y música, en la terminal",
            "YouTube and your own videos and music, in the terminal",
        ))
        .fg(FG),
        Line::from(""),
        Line::from(vec![label("GitHub"), Span::styled(env!("CARGO_PKG_REPOSITORY"), CYAN)]),
        Line::from(vec![
            label(lang.pick("Basado en", "Based on")),
            Span::styled(lang.pick("tubetui de polblancoo · ", "tubetui by polblancoo · "), FG),
            Span::styled("https://github.com/polblancoo/tubetui", CYAN),
        ]),
        Line::from(vec![label(lang.pick("Licencia", "License")), Span::styled("MIT / Apache-2.0", FG)]),
        Line::from(vec![
            label(lang.pick("Errores", "Bugs")),
            Span::styled(concat!(env!("CARGO_PKG_REPOSITORY"), "/issues"), CYAN),
        ]),
    ];
    if let Some(update) = &state.update {
        lines.push(Line::from(""));
        lines.push(
            Line::from(format!(
                "{} {} {}",
                icons.update,
                lang.pick("Hay una versión nueva:", "A new version is out:"),
                update.version
            ))
            .fg(GREEN)
            .bold(),
        );
        // The Update Now button: focused with Tab (like a list), pressed with Enter.
        let ready = about_button_ready(state);
        let focused = state.active == 0 && state.focus == Focus::List;
        let label = if restart_ready(state) {
            lang.pick("Reiniciar Ahora", "Restart Now")
        } else {
            lang.pick("Actualizar Ahora", "Update Now")
        };
        let button = format!("  {} {label}  ", icons.update);
        let style = match (ready, focused) {
            (true, true) => Style::new().fg(BG).bg(GREEN).bold(),
            (true, false) => Style::new().fg(GREEN).bg(HIGHLIGHT).bold(),
            (false, _) => Style::new().fg(MUTED).bg(HIGHLIGHT),
        };
        let (left, right) = if ready && focused { ("▶ ", " ◀") } else { ("  ", "  ") };
        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::styled(left, Style::new().fg(GREEN).bold()),
            Span::styled(button, style),
            Span::styled(right, Style::new().fg(GREEN).bold()),
        ]));
        if ready && !focused {
            lines.push(
                Line::from(lang.pick("(Tab para elegirlo, Enter para pulsarlo)", "(Tab to Select, Enter to Press)"))
                    .fg(MUTED),
            );
        }
        match &state.self_update {
            Progress::Downloading(percent) => {
                let text = lang.pick("Descargando…", "Downloading…");
                lines.push(Line::from(format!("{} {text} {percent}%", icons.downloading)).fg(YELLOW).bold());
            }
            Progress::Done(version) => {
                let text = lang.pick(
                    "instalada · Reiniciar Ahora la abre (tus pestañas vuelven)",
                    "Installed · Restart Now Opens It (Your Tabs Come Back)",
                );
                lines.push(Line::from(format!("{} tubevibe {version} {text}", icons.up_to_date)).fg(GREEN).bold());
            }
            Progress::Failed(error) => {
                lines.push(
                    Line::from(format!(
                        "{} {} {error}",
                        icons.failed,
                        lang.pick("No se pudo actualizar:", "Could Not Update:")
                    ))
                    .fg(RED),
                );
            }
            Progress::Idle => {}
        }
        if let Some(why) = match update.replace {
            Replace::FromSource => Some(lang.pick(
                "Compilado desde el código: git pull && cargo build --release",
                "Built From Source: Run git pull && cargo build --release",
            )),
            Replace::Cargo => Some(lang.pick("Instalado con cargo install", "Installed with cargo install")),
            Replace::Package => Some(lang.pick(
                "Instalado con un paquete: actualizalo con el paquete",
                "Installed From a Package: Update It With the Package",
            )),
            Replace::ReadOnly => {
                Some(lang.pick("La carpeta de tubevibe no se puede escribir", "tubevibe's Folder Isn't Writable"))
            }
            Replace::NoAsset => Some(lang.pick("No hay descarga para este sistema", "No Download for This System")),
            Replace::Ready(_) => None,
        } {
            lines.push(Line::from(why).fg(MUTED));
        }
        // By hand only where the button can't do it (or just failed to).
        let by_hand = !matches!(update.replace, Replace::Ready(_)) || matches!(state.self_update, Progress::Failed(_));
        if by_hand {
            lines.push(Line::from(""));
            let (how, value) = match &update.how {
                Install::Command(command) => (lang.pick("Actualizá con:", "Update with:"), command),
                Install::Link(url) => (lang.pick("Bajá el instalador:", "Get the installer:"), url),
            };
            lines.push(Line::from(how).fg(FG));
            lines.push(Line::from(value.as_str()).fg(CYAN));
            lines.push(
                Line::from(
                    lang.pick("(Shift + arrastrar para seleccionar y copiar)", "(Shift + drag to select and copy)"),
                )
                .fg(MUTED),
            );
        }
    } else if state.up_to_date {
        lines.push(Line::from(""));
        let text = lang.pick("Tenés la última versión", "You're on the Latest Version");
        lines.push(Line::from(format!("{} {text}", icons.up_to_date)).fg(GREEN).bold());
    }
    let wallets = donations();
    if !wallets.is_empty() {
        lines.push(Line::from(""));
        lines.push(
            Line::from(format!(
                "{} {}",
                icons.donate,
                lang.pick(
                    "¿Te sirve? Podés apoyar a polblancoo, el autor original de tubetui, con una donación:",
                    "Find it useful? You can support polblancoo, tubetui's original author, with a donation:",
                )
            ))
            .fg(YELLOW)
            .bold(),
        );
        lines.push(Line::from(lang.pick(DONATIONS_NOTE.0, DONATIONS_NOTE.1)).fg(MUTED));
        for wallet in wallets {
            lines.push(Line::from(vec![
                Span::styled(format!("{:>12}  ", wallet.network), MAGENTA),
                Span::styled(wallet.address, Style::new().fg(GREEN).bold()),
            ]));
        }
    }
    let title = format!(" {} {} ", icons.section(Section::AcercaDe), Section::AcercaDe.label(lang));
    // Wrapped: the update command can be longer than the panel.
    let about = Paragraph::new(lines).alignment(Alignment::Center).wrap(ratatui::widgets::Wrap { trim: false });
    frame.render_widget(about.block(panel(title, PURPLE)), area);
}

/// The About page's Update Now button can be pressed: a new version is out,
/// this copy can be replaced, and no update is running or done.
fn about_button_ready(state: &State) -> bool {
    state.update.as_ref().is_some_and(|u| matches!(u.replace, Replace::Ready(_)))
        && matches!(state.self_update, Progress::Idle | Progress::Failed(_) | Progress::Done(_))
}

/// The update is in: the button restarts tubevibe into it.
fn restart_ready(state: &State) -> bool {
    matches!(state.self_update, Progress::Done(_))
}

fn centered(area: Rect, width: u16, height: u16) -> Rect {
    let [row] = Layout::vertical([Constraint::Length(height.min(area.height))]).flex(Flex::Center).areas(area);
    let [cell] = Layout::horizontal([Constraint::Length(width.min(area.width))]).flex(Flex::Center).areas(row);
    cell
}

fn truncate(text: &str, max: usize) -> String {
    match text.char_indices().nth(max) {
        Some((cut, _)) => format!("{}…", &text[..cut]),
        None => text.to_owned(),
    }
}

/// Truncates or pads to exactly `width` characters, for aligned columns.
fn pad(text: &str, width: usize) -> String {
    let text: String = text.chars().take(width).collect();
    format!("{text:<width$}")
}

/// Pads to `width` terminal cells; emoji take two cells, so `{:<w}` is not enough.
fn pad_cells(text: &str, width: usize) -> String {
    let used = Span::raw(text).width();
    format!("{text}{}", " ".repeat(width.saturating_sub(used)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::{Action, Effect};
    use crate::config::Config;
    use crate::domain::{channel, video};
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;

    const VIEW: View =
        View { embedded: true, now: 1_791_590_400, loading: None, opening: None, thumbnails: true, playing: None };

    fn render_with(state: &State, view: &View) -> String {
        render_sized(state, view, 130)
    }

    fn render_sized(state: &State, view: &View, width: u16) -> String {
        let mut terminal = Terminal::new(TestBackend::new(width, 40)).unwrap();
        terminal.draw(|f| draw(f, state, view)).unwrap();
        let buffer = terminal.backend().buffer();
        buffer
            .content
            .chunks(buffer.area.width as usize)
            .map(|row| row.iter().map(|c| c.symbol()).collect::<String>())
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn render(state: &State) -> String {
        render_with(state, &VIEW)
    }

    /// Plain-Unicode icons keep every symbol one cell wide in the test buffer.
    fn state() -> State {
        let mut config = Config::default();
        config.ui.icons = IconSet::Unicode;
        config.player.autoplay = true;
        State::new(config, Vec::new())
    }

    /// Opens "Suscripciones" from the menu and fills it with `items`.
    fn with_items(items: Items) -> State {
        let mut s = state();
        s.menu = Section::ALL.iter().position(|x| *x == Section::Suscripciones).unwrap();
        let id = match s.update(Action::Enter).as_slice() {
            [Effect::Load { id, .. }] => *id,
            other => panic!("{other:?}"),
        };
        s.update(Action::Loaded(id, Ok(items)));
        s
    }

    fn with_tab() -> State {
        let mut s = with_items(Items::Videos(vec![video("a"), video("b")]));
        s.update(Action::Enter);
        s
    }

    fn open_settings() -> State {
        let mut s = state();
        s.menu = Section::ALL.iter().position(|x| *x == Section::Configuracion).unwrap();
        s.update(Action::Enter);
        s
    }

    #[test]
    fn settings_values_line_up() {
        for lang in [Lang::Es, Lang::En] {
            let mut s = open_settings();
            s.config.ui.lang = Some(lang);
            for set in [IconSet::Emoji, IconSet::Nerd, IconSet::Unicode] {
                let icons = icons(set);
                let starts: Vec<usize> = Setting::ALL
                    .iter()
                    .enumerate()
                    .map(|(i, setting)| {
                        // Icon cell and label column: the value starts right after.
                        Span::raw(format!(" {} ", pad_cells(icons.settings[i], 2))).width()
                            + Span::raw(pad_cells(setting.label(lang), setting_label_width(lang))).width()
                    })
                    .collect();
                assert!(starts.windows(2).all(|w| w[0] == w[1]), "{set:?} {lang:?}: {starts:?}");
            }
            // The longest label still leaves a gap before its value.
            let longest = Setting::ALL.iter().map(|x| Span::raw(x.label(lang)).width()).max().unwrap();
            assert!(setting_label_width(lang) >= longest + 2);
        }
        let mut s = open_settings();
        s.config.ui.lang = Some(Lang::En);
        let screen = render(&s);
        let mut columns = Vec::new();
        for setting in Setting::ALL {
            let (label, value) = (setting.label(Lang::En), s.setting_value(setting));
            let Some(line) = screen.lines().find(|l| l.contains(label)) else { continue };
            let after = line.find(label).unwrap() + label.len();
            if value.is_empty() || !line[after..].contains(value.as_str()) {
                continue;
            }
            let at = after + line[after..].find(value.as_str()).unwrap();
            assert!(line[after..at].chars().all(|c| c == ' ') && at - after >= 2, "{label}: {line}");
            columns.push((label, line[..at].chars().count()));
        }
        assert!(columns.len() > 10, "{screen}");
        assert!(columns.iter().all(|(_, c)| *c == columns[0].1), "{columns:?}\n{screen}");
    }

    /// Emoji whose width terminals disagree on (text-presentation ones like
    /// 🖼 or 👁, or any with U+FE0F) push the rest of the row over; every
    /// emoji that starts a column is one terminals always draw two cells wide.
    #[test]
    fn column_emoji_are_always_two_cells() {
        let column: Vec<&str> =
            EMOJI.settings.iter().chain(EMOJI.sections.iter()).copied().chain([EMOJI.watched, EMOJI.later]).collect();
        for icon in column {
            assert_eq!(icon.chars().count(), 1, "one code point, no variation selector: {icon:?}");
            assert_eq!(Span::raw(icon).width(), 2, "{icon} is not wide everywhere");
            let c = icon.chars().next().unwrap() as u32;
            // Text-presentation emoji in the ranges used here (2-wide only in some terminals).
            let text_style = [0x23EF, 0x23F9, 0x2B06, 0x2B07, 0x1F441, 0x1F5BC, 0x1F5B1, 0x1F5E3, 0x1F39E, 0x1F3F7];
            assert!(!text_style.contains(&c), "{icon} draws 1 or 2 cells depending on the terminal");
        }
    }

    fn hint_keys(state: &State) -> Vec<String> {
        hints(state).into_iter().map(|(k, _)| k).collect()
    }

    #[test]
    fn layout_reserves_status_and_hint_lines() {
        let area = Rect::new(0, 0, 100, 30);
        let with = areas(area, true);
        assert_eq!(with.sidebar.unwrap().width, SIDEBAR_WIDTH);
        assert_eq!(with.content.x, SIDEBAR_WIDTH);
        let without = areas(area, false);
        assert_eq!(without.sidebar, None);
        assert_eq!(without.content, Rect::new(0, 0, 100, 28), "no tab bar: the pane starts at the top");
        assert_eq!(with.content.y, 0);
        assert_eq!(with.content.height, with.sidebar.unwrap().height, "as tall as the menu");
        assert_eq!(without.status, Rect::new(0, 28, 100, 1));
        assert_eq!(without.hints, Rect::new(0, 29, 100, 1));
    }

    #[test]
    fn video_area_only_for_video_tabs_without_dialogs() {
        let area = Rect::new(0, 0, 100, 30);
        let mut s = with_tab();
        assert_eq!(video_area(&s, area), Some(Block::bordered().inner(areas(area, true).content)));
        s.update(Action::ToggleHelp);
        assert_eq!(video_area(&s, area), None);
        s.update(Action::ToggleHelp);
        s.update(Action::GoTab(0));
        assert_eq!(video_area(&s, area), None);
    }

    #[test]
    fn menu_shows_every_section_with_icons_and_later_count() {
        let mut s = state();
        s.later = vec![video("x"), video("y")];
        s.later_complete = true;
        let screen = render(&s);
        for (section, icon) in Section::ALL.iter().zip(UNICODE.sections) {
            assert!(screen.contains(&format!("{icon} {}", section.label(Lang::Es))), "{section:?}\n{screen}");
        }
        assert!(screen.contains("Ver después (2)"));
        assert!(screen.contains("Elegí una sección"));
    }

    #[test]
    fn icon_set_follows_the_config() {
        let mut s = state();
        s.config.ui.icons = IconSet::Nerd;
        assert!(render(&s).contains(&format!("{} Mis recomendados", NERD.sections[0])));
    }

    #[test]
    fn playlist_rows_take_their_icon_from_the_set_and_line_up() {
        let playlists =
            ["LL", "WL", "PLf"].map(|id| Playlist { id: id.into(), title: "Favorites".into(), count: None }).to_vec();
        for set in [IconSet::Emoji, IconSet::Nerd, IconSet::Unicode] {
            let mut s = with_items(Items::Playlists(playlists.clone()));
            s.config.ui.icons = set;
            s.config.ui.lang = Some(Lang::En);
            let screen = render(&s);
            let i = icons(set);
            let rows = [
                (i.liked, "Liked Videos"),
                (i.later, "Watch Later (YouTube)"),
                (i.section(Section::MisListas), "Favorites"),
            ];
            let mut columns = Vec::new();
            for (icon, name) in rows {
                // The list pane: right of the menu's border.
                let line = screen
                    .lines()
                    .map(|l| l.splitn(3, '│').nth(2).unwrap_or(""))
                    .find(|l| l.contains(name) && l.contains(icon))
                    .unwrap_or_else(|| panic!("{icon} {name}\n{screen}"));
                let at = line.find(name).unwrap();
                assert!(line[..at].contains(icon), "{set:?}: {line}");
                columns.push(Span::raw(&line[..at]).width());
            }
            assert!(columns.windows(2).all(|w| w[0] == w[1]), "{set:?}: {columns:?}");
            if set != IconSet::Emoji {
                assert!(!screen.contains('👍') && !screen.contains('🕒'), "{set:?}\n{screen}");
            }
        }
    }

    #[test]
    fn video_rows_show_date_channel_title_duration_and_later_mark() {
        let dated = Video { published: Some(VIEW.now - 3 * 86_400), ..video("a") };
        let live = Video { live: Live::Now, ..video("b") };
        let mut s = with_items(Items::Videos(vec![dated, live]));
        s.later = vec![video("a")];
        let screen = render(&s);
        assert!(screen.contains("07/10/26 3d"), "{screen}");
        assert!(screen.contains("● en vivo"));
        assert!(screen.contains("chan a"));
        assert!(screen.contains("◷ title a"));
        assert!(screen.contains("1:00"));
        assert!(screen.contains("Suscripciones · 2"));
    }

    #[test]
    fn channel_rows_mark_my_channels() {
        let followed = Channel {
            followers: Some("358 mil suscriptores".into()),
            handle: Some("@x".into()),
            description: Some("Música\ny charlas".into()),
            ..channel("UCx")
        };
        let mut s = with_items(Items::Channels(vec![followed, channel("UCy")]));
        s.subs.insert("UCx".into());
        let name = pad_cells("name UCx", CHANNEL_NAME_WIDTH);
        let row = format!("✓ {name}{}@x", pad_cells("358 mil suscriptores", FOLLOWERS_WIDTH));
        let wide = render(&s);
        let line = wide.lines().find(|l| l.contains("✓ name UCx")).unwrap();
        assert!(line.contains(&row), "{line}");
        assert!(!line.contains("Música"), "the panel shows it: {line}");
        let narrow = render_sized(&s, &VIEW, 110);
        assert!(narrow.contains(&row) && narrow.contains("┌ Canal"), "under the list: {narrow}");
        assert!(wide.contains("   name UCy"));
    }

    #[test]
    fn profile_rows_mark_current_session() {
        let mut s = with_items(Items::Profiles(vec!["brave:Default".into(), String::new()]));
        s.config.session.browser = "brave:Default".into();
        let screen = render(&s);
        assert!(screen.contains("● ☺ brave · Default"), "{screen}");
        assert!(screen.contains("☺ Sin sesión"));
    }

    #[test]
    fn settings_screen_shows_labels_and_values() {
        // Taller than the default screen: the list scrolls past 36 rows.
        let screen = render_screen(&open_settings(), 130, 46).join("\n");
        for setting in Setting::ALL {
            assert!(screen.contains(setting.label(Lang::Es)), "{setting:?}\n{screen}");
        }
        assert!(screen.contains("720p"));
        assert!(screen.contains("~/Videos"));
        assert!(screen.contains("Original + subtítulos"));
    }

    #[test]
    fn search_box_shows_scope_and_query() {
        let mut s = state();
        s.update(Action::FocusSearch);
        for c in "lofi".chars() {
            s.update(Action::Input(c));
        }
        let screen = render(&s);
        assert!(screen.contains("Buscar videos en todo YouTube"), "{screen}");
        assert!(screen.contains("lofi"));
        s.update(Action::CycleScope);
        assert!(render(&s).contains("Buscar videos en mis canales"));
    }

    #[test]
    fn cookies_prompt_explains_how_to_export() {
        let mut s = open_settings();
        s.stack[0].selected = Setting::ALL.iter().position(|x| *x == Setting::ImportCookies).unwrap();
        s.update(Action::Enter);
        let screen = render(&s);
        assert!(screen.contains("Ruta del cookies.txt"));
        assert!(screen.contains("scp cookies.txt"));
    }

    #[test]
    fn tabs_appear_in_the_menu_only() {
        let screen = render(&with_tab());
        assert!(screen.contains("PESTAÑAS"));
        assert_eq!(screen.matches("│ ▶ title a ").count(), 1, "the menu, no tab bar: {screen}");
        assert!(!screen.contains("Inicio"), "{screen}");
        // The tab's pane starts on the first row, next to the menu's title.
        assert!(screen.lines().next().unwrap().contains("▶ title a · chan a"), "{screen}");
    }

    #[test]
    fn video_tab_bar_is_short_and_ordered() {
        let mut s = with_tab();
        s.update(Action::ToggleUiLang);
        let keys: Vec<(String, String)> = hints(&s);
        let at = |k: &str, d: &str| keys.iter().position(|(key, desc)| key == k && *desc == d);
        assert_eq!(
            keys.iter().filter(|(k, _)| k == "Enter").count(),
            keys.iter().filter(|(_, d)| d.starts_with("Play")).count()
        );
        assert!(!keys.iter().any(|(_, d)| d.contains("in Up Next")), "{keys:?}");
        assert!(at("v", "Quality Change").is_some() && at("b", "Codec Change").is_some(), "{keys:?}");
        assert!(at("Space", "Pause") < at("x", "Close the Tab"));
        assert_eq!(keys.last().unwrap(), &("Shift+/".to_string(), "Help".to_string()));
        s.update(Action::ToggleUiLang);
        let keys = hints(&s);
        assert!(keys.iter().any(|(k, d)| k == "v" && *d == "Cambiar calidad"));
        assert!(keys.iter().any(|(k, d)| k == "b" && *d == "Cambiar códec"));
        assert_eq!(keys.last().unwrap(), &("Shift+/".to_string(), "Ayuda".to_string()));
    }

    #[test]
    fn menu_cursor_lights_the_tab_it_is_on() {
        let mut s = with_tab();
        s.update(Action::FocusMenu);
        assert_eq!(s.menu_row(), MenuRow::Tab(0));
        let screen = render(&s);
        assert!(screen.contains("▌▶ title a"), "{screen}");
        s.update(Action::Down);
        let screen = render(&s);
        assert!(!screen.contains("▌▶ title a") && screen.contains("▌⚙"), "on Settings now: {screen}");
    }

    #[test]
    fn window_mode_tab_explains_where_video_is() {
        assert!(render_with(&with_tab(), &View { embedded: false, ..VIEW }).contains("ventana de mpv"));
        assert!(!render(&with_tab()).contains("ventana de mpv"));
    }

    #[test]
    fn an_idle_tab_says_enter_plays_it() {
        // Embedded (kitty) or overlay: no details pane, just the hint.
        let mut s = with_items(Items::Videos(vec![video("a")]));
        s.config.player.autoplay = false;
        s.update(Action::Enter);
        let screen = render(&s);
        assert!(screen.contains("▶ Enter lo reproduce"), "{screen}");
        assert!(!screen.contains("volver a verlo"), "{screen}");
        assert!(hints(&s).iter().any(|(k, d)| k == "Enter" && *d == "reproducir"));
        // Video in its own window: the details say it.
        s.video_window = true;
        let screen = render_with(&s, &View { embedded: false, ..VIEW });
        assert!(screen.contains("▶ Enter lo reproduce") && !screen.contains("Video cerrado"), "{screen}");
    }

    #[test]
    fn the_logo_shows_alone_when_the_app_opens() {
        let mut s = state();
        s.splash = true;
        let screen = render(&s);
        assert!(screen.contains("YouTube en tu terminal") && screen.contains("▐███"), "{screen}");
        assert!(!screen.contains("Inicio"), "only the logo");
        assert_eq!(spot(&s, None, Rect::new(0, 0, 120, 40), 5, 5), None);
        s.update(Action::EndSplash);
        assert!(render(&s).contains("Mis recomendados"));
    }

    #[test]
    fn help_shows_every_section_with_explanations() {
        let mut s = state();
        s.update(Action::ToggleHelp);
        let screen = render(&s);
        for (title, rows) in keymap::help(Lang::Es) {
            assert!(screen.contains(title), "{title}\n{screen}");
            for (_, description) in rows {
                assert!(screen.contains(description), "{description}\n{screen}");
            }
        }
    }

    #[test]
    fn hints_follow_the_context() {
        let mut s = state();
        assert_eq!(hint_keys(&s), ["/", "Shift+/"], "the menu: no moving keys");
        s.update(Action::FocusSearch);
        s.update(Action::Back);
        let tab = with_tab();
        assert!(hint_keys(&tab).contains(&"Espacio".to_string()) && hint_keys(&tab).contains(&"[ ]".to_string()));
        let list = with_items(Items::Videos(vec![video("a")]));
        assert_eq!(
            hint_keys(&list),
            ["Enter", "v", "w", "W", "a", "s", "g", "d", "c", "/", "Shift+/"],
            "Inicio: a feed"
        );
        let mut search = state();
        search.update(Action::FocusSearch);
        assert!(hint_keys(&search).contains(&"Tab".to_string()));
    }

    #[test]
    fn hints_explain_two_step_keys() {
        let hint = |s: &State, k: &str| hints(s).into_iter().find(|(key, _)| key == k).map(|(_, d)| d);
        let mut list = with_items(Items::Videos(vec![video("a")]));
        assert_eq!(hint(&list, "s").as_deref(), Some("suscribirse"));
        list.subs.insert("UCa".into());
        assert_eq!(hint(&list, "s").as_deref(), Some("desuscribirse (pide confirmar)"));
        let file = |n: &str| LocalFile { path: format!("/m/{n}.mp3").into(), name: n.into(), folder: "D".into() };
        let mut files = with_items(Items::Files(vec![file("a"), file("b")], MediaKind::Music));
        assert_eq!(hint(&files, "Espacio").as_deref(), Some("marcar varios"));
        assert_eq!(hint(&files, "X").as_deref(), Some("este a la papelera (pide confirmar)"));
        files.update(Action::ToggleMark);
        assert_eq!(hint(&files, "X").as_deref(), Some("los marcados a la papelera (pide confirmar)"));
        files.update(Action::Trash);
        assert_eq!(hint(&files, "s").as_deref(), Some("sí, a la papelera"));
    }

    #[test]
    fn space_shows_on_downloads_my_videos_and_my_music() {
        let hint = |s: &State, k: &str| hints(s).into_iter().find(|(key, _)| key == k).map(|(_, d)| d);
        let file = |n: &str| LocalFile { path: format!("/m/{n}").into(), name: n.into(), folder: "D".into() };
        for kind in [MediaKind::Video, MediaKind::Music] {
            let s = with_items(Items::Files(vec![file("a")], kind));
            assert_eq!(hint(&s, "Espacio").as_deref(), Some("marcar varios"), "{kind:?}");
            assert!(render_sized(&s, &VIEW, 100).contains("Espacio marcar varios"), "{kind:?} fits at 100 columns");
        }
        let states = [
            DownloadState::Done("/v/a.mp4".into()),
            DownloadState::Running { permille: 1, speed: String::new(), eta: String::new() },
            DownloadState::Failed("x".into()),
        ];
        for state in states {
            let mut s = with_items(Items::Downloads(vec![Download { id: 0, video: video("a"), state }]));
            assert_eq!(hint(&s, "Espacio").as_deref(), Some("marcar varios"));
            s.update(Action::ToggleUiLang);
            let screen = render_sized(&s, &VIEW, 100);
            assert!(screen.contains("Space Mark Several"), "{screen}");
            assert!(screen.contains("Shift+/ Help"), "whole hints only, the last one kept");
        }
    }

    #[test]
    fn marked_downloads_show_the_mark_the_count_and_one_question() {
        let hint = |s: &State, k: &str| hints(s).into_iter().find(|(key, _)| key == k).map(|(_, d)| d);
        let done = |id: u64, n: &str| Download {
            id,
            video: video(n),
            state: DownloadState::Done(format!("/v/{n}.mp4").into()),
        };
        let mut s = with_items(Items::Downloads(vec![done(0, "a"), done(1, "b"), done(2, "c")]));
        s.update(Action::ToggleUiLang);
        s.marked_downloads.extend([0, 2]);
        let screen = render(&s);
        assert!(screen.contains("· 3 · ✗ 2 marked"), "{screen}");
        assert_eq!(screen.matches("✗ ✔ done").count(), 2, "{screen}");
        assert_eq!(hint(&s, "X").as_deref(), Some("The Marked Ones: Trash (Asks First)"));
        assert_eq!(hint(&s, "Space").as_deref(), Some("Mark / Unmark"));
        let files = vec![(0, "/v/a.mp4".into()), (1, "/v/b.mp4".into()), (2, "/v/c.mp4".into())];
        s.confirm = Some(Confirm::TrashDownloads { files: files.clone(), cancel: Vec::new(), remove: Vec::new() });
        assert!(render(&s).contains("Move 3 files to the trash?"));
        assert_eq!(hint(&s, "y").as_deref(), Some("Yes, to the Trash"));
        s.confirm = Some(Confirm::TrashDownloads { files, cancel: vec![7], remove: Vec::new() });
        assert_eq!(hint(&s, "y").as_deref(), Some("Yes (The Other Marked Ones Get Cancelled or Taken Off)"));
        s.update(Action::ToggleUiLang);
        assert!(render(&s).contains("¿Mandar 3 archivos a la papelera?"));
    }

    #[test]
    fn downloads_hint_and_ask_before_the_trash() {
        let hint = |s: &State, k: &str| hints(s).into_iter().find(|(key, _)| key == k).map(|(_, d)| d);
        let done = Download { id: 0, video: video("a"), state: DownloadState::Done("/v/a.mp4".into()) };
        let mut s = with_items(Items::Downloads(vec![done]));
        assert_eq!(hint(&s, "X").as_deref(), Some("a la papelera (pide confirmar)"));
        s.confirm = Some(Confirm::TrashDownload { id: 0, path: "/v/a.mp4".into() });
        assert_eq!(hint(&s, "s").as_deref(), Some("sí, a la papelera"));
        s.update(Action::ToggleUiLang);
        let screen = render(&s);
        assert!(screen.contains("Move to the trash a.mp4?"), "{screen}");
        assert_eq!(hint(&s, "x").as_deref(), None, "the question owns the bar");
    }

    #[test]
    fn hint_bar_shows_keys_with_descriptions() {
        let screen = render(&with_items(Items::Videos(vec![video("a")])));
        assert!(screen.contains("w ver después YouTube"), "{screen}");
        assert!(screen.contains("W ver después tubevibe"), "{screen}");
        assert!(screen.contains("Shift+/ Ayuda"));
    }

    #[test]
    fn confirm_dialog_names_the_channel() {
        let mut s = with_items(Items::Videos(vec![video("a")]));
        s.subs.insert("UCa".into());
        s.update(Action::ToggleSubscription);
        let screen = render(&s);
        assert!(screen.contains("¿Desuscribirte de chan a?"));
        assert!(screen.contains("sí, desuscribirme"));
    }

    #[test]
    fn status_bar_shows_session_and_language() {
        let mut s = state();
        s.config.session.browser = "brave:Default".into();
        let screen = render(&s);
        assert!(screen.contains("brave · Default"), "{screen}");
        assert!(screen.contains("Original + subtítulos"));
    }

    #[test]
    fn menu_shows_group_headers() {
        let screen = render(&state());
        assert!(screen.contains("YOUTUBE"));
        assert!(screen.contains("EN TU EQUIPO"));
        assert!(screen.contains("▷ Mis videos") && screen.contains("♫ Mi música"));
    }

    #[test]
    fn music_tab_shows_the_song_and_takes_no_video_area() {
        let file = |n: &str| LocalFile { path: format!("/m/{n}.mp3").into(), name: n.into(), folder: "Disco".into() };
        let mut s = with_items(Items::Files(vec![file("Tema uno"), file("Tema dos")], MediaKind::Music));
        let screen = render(&s);
        assert!(screen.contains("♫ Disco"), "{screen}");
        assert!(screen.contains("escuchar desde esta canción"));
        s.update(Action::Enter);
        let screen = render(&s);
        assert!(screen.contains("2 canciones en la cola"), "{screen}");
        assert_eq!(video_area(&s, Rect::new(0, 0, 100, 30)), None);
    }

    #[test]
    fn fullscreen_gives_the_whole_terminal_to_the_video() {
        let mut s = with_tab();
        s.update(Action::ToggleFullscreen);
        let area = Rect::new(0, 0, 100, 30);
        assert_eq!(video_area(&s, area), Some(area));
        assert!(!render(&s).contains("tubevibe"), "no sidebar or bars");
    }

    #[test]
    fn loading_shows_spinner_seconds_and_a_moving_bar() {
        let mut s = state();
        s.menu = Section::ALL.iter().position(|x| *x == Section::Suscripciones).unwrap();
        s.update(Action::Enter); // Suscripciones starts loading
        let view = View { loading: Some(Duration::from_millis(3_250)), ..VIEW };
        let screen = render_with(&s, &view);
        assert!(screen.contains("Cargando Suscripciones… 3 s"), "{screen}");
        assert!(screen.contains("█"));
        let later = View { loading: Some(Duration::from_millis(3_550)), ..VIEW };
        let (now, then) = (view.loading.unwrap(), later.loading.unwrap());
        assert_ne!(bouncing_bar(30, now), bouncing_bar(30, then), "the bar moves");
        assert_eq!(bouncing_bar(30, now).chars().count(), 30);
    }

    #[test]
    fn video_tab_shows_a_loading_bar_until_the_video_plays() {
        let opening = View { opening: Some(Duration::from_millis(2_400)), ..VIEW };
        let screen = render_with(&with_tab(), &opening);
        assert!(screen.contains("Cargando el video… 2 s"), "{screen}");
        assert!(screen.contains("█"));
        assert!(!render(&with_tab()).contains("Cargando"), "playing: nothing under the video");
        let window = View { embedded: false, ..VIEW };
        assert!(render_with(&with_tab(), &window).contains("ventana de mpv"));
    }

    #[test]
    fn english_interface() {
        let mut s = with_tab();
        s.update(Action::ToggleUiLang);
        s.update(Action::GoTab(0));
        let screen = render(&s);
        for text in
            ["My Recommendations", "Watch Later", "ON THIS COMPUTER", "Settings", "About", "TABS", "Shift+/ Help"]
        {
            assert!(screen.contains(text), "{text} missing\n{screen}");
        }
        assert!(!screen.contains("Ver después"), "{screen}");
        s.update(Action::ToggleHelp);
        assert!(render(&s).contains("Keyboard Shortcuts"));
    }

    #[test]
    fn about_shows_version_and_links() {
        let mut s = state();
        s.menu = Section::ALL.iter().position(|x| *x == Section::AcercaDe).unwrap();
        s.update(Action::Enter);
        let screen = render(&s);
        assert!(screen.contains(env!("CARGO_PKG_VERSION")), "{screen}");
        assert!(screen.contains("github.com/polblancoo/tubetui"), "{screen}");
        assert!(screen.contains("tubetui de polblancoo"), "credits the original: {screen}");
    }

    #[test]
    fn hints_fit_whole_and_keep_the_help_key() {
        let mut terminal = Terminal::new(TestBackend::new(60, 1)).unwrap();
        let hints = hints(&with_tab());
        terminal.draw(|f| draw_hints(f, &hints, f.area())).unwrap();
        let line: String = terminal.backend().buffer().content.iter().map(|c| c.symbol()).collect();
        assert!(line.contains("Shift+/ Ayuda"), "{line}");
        assert!(line.contains("Espacio pausa"), "{line}");
        assert!(!line.contains("silenciar"), "does not fit, left out whole: {line}");
    }

    #[test]
    fn wide_video_lists_show_a_preview_of_the_selection() {
        let mut s = with_items(Items::Videos(vec![video("a"), video("b")]));
        s.update(Action::Down);
        let screen = render(&s); // 130 columns: room for the panel
        assert!(screen.contains("Vista previa"), "{screen}");
        let screen_area = Rect::new(0, 0, 130, 40);
        let (id, cells) = thumbnail_area(&s, screen_area).unwrap();
        assert_eq!(id, "b", "the selected video");
        assert_eq!(cells.width, 46 - 2);
        assert_eq!(cells.height, (46 - 2) * 9 / 32);
        s.update(Action::ToggleHelp);
        assert_eq!(thumbnail_area(&s, screen_area), None, "kitty would draw it over the help");
        s.update(Action::ToggleHelp);
        let (_, under) = thumbnail_area(&s, Rect::new(0, 0, 100, 40)).expect("narrow: the preview goes under the list");
        assert!(under.y > 20 && under.height <= 7, "{under:?}");
        assert_eq!(thumbnail_area(&s, Rect::new(0, 0, 100, 16)), None, "too short to stack: no preview");
        assert!(!render_with(&s, &View { thumbnails: false, ..VIEW }).contains("⏳"));
    }

    #[test]
    fn wide_channel_lists_show_the_whole_description() {
        let described = Channel { description: Some("Primera línea\nSegunda línea".into()), ..channel("UCx") };
        let s = with_items(Items::Channels(vec![described, channel("UCy")]));
        let screen = render(&s);
        assert!(screen.contains(" Canal "), "{screen}");
        assert!(screen.contains("│Primera línea"), "{screen}");
        assert!(screen.contains("│Segunda línea"), "kept on its own line: {screen}");
        assert_eq!(thumbnail_area(&s, Rect::new(0, 0, 130, 40)), None, "channels have no thumbnail");
    }

    #[test]
    fn video_tab_shows_volume_quality_and_codec() {
        let mut s = with_items(Items::Videos(vec![video("a")]));
        s.update(Action::Enter);
        let info =
            Info { volume: Some(85.0), muted: false, height: Some(480), codec: Some("h264".into()), ..Info::default() };
        let screen = render_with(&s, &View { playing: Some(info.clone()), ..VIEW });
        assert!(screen.contains("♪ 85 %  480p  H.264"), "{screen}");
        let muted = Info { muted: true, codec: Some("vp9".into()), ..info };
        assert_eq!(playing_label(&muted, icons(IconSet::Unicode)), " ✕ 85 %  480p  VP9 ");
        assert_eq!(playing_label(&Info::default(), icons(IconSet::Unicode)), "");
    }

    #[test]
    fn video_tab_shows_a_progress_bar() {
        let mut s = with_items(Items::Videos(vec![video("a")]));
        s.update(Action::Enter);
        let info = Info { position: Some(30.0), duration: Some(120.0), buffered: Some(60.0), ..Info::default() };
        let screen = render_with(&s, &View { playing: Some(info.clone()), ..VIEW });
        let bottom = screen.lines().find(|l| l.contains("2:00")).unwrap_or_default();
        assert!(bottom.contains("▶ 0:30 ━"), "{screen}");
        let content = areas(Rect::new(0, 0, 130, 40), s.sidebar).content;
        let (bar, duration) = progress_bar(&info, content).unwrap();
        assert_eq!(duration, 120.0);
        let track = bottom.split("0:30 ").nth(1).and_then(|t| t.split(" 2:00").next()).unwrap();
        assert_eq!(track.chars().count(), bar.width as usize, "{bottom}");
        let thick = track.chars().filter(|c| *c == '━').count();
        assert_eq!(thick, (f64::from(bar.width) / 2.0).round() as usize, "a quarter played, another downloaded");
        let line = progress_line(&info, &[], content.width).unwrap();
        let played = &line.spans[1];
        assert_eq!(played.style.fg, Some(RED));
        assert_eq!(played.content.chars().count(), (f64::from(bar.width) / 4.0).round() as usize);
        assert_eq!(line.spans[2].style.fg, Some(MUTED));
        assert_eq!(bar.y, content.bottom() - 1);
        let paused = progress_line(&Info { paused: true, ..info.clone() }, &[], 60).unwrap();
        assert!(paused.to_string().starts_with(" ‖ 0:30 "));
        assert!(progress_line(&Info { duration: None, ..info }, &[], 60).is_none(), "live: no total");
    }

    #[test]
    fn progress_bar_marks_sponsors_and_chapters() {
        let info = Info {
            position: Some(0.0),
            duration: Some(100.0),
            chapters: vec![(0.0, "Intro".into()), (50.0, "Second half".into())],
            chapter: Some(1),
            ..Info::default()
        };
        let sponsor = Segment { start: 10_000, end: 20_000, category: "sponsor".into() };
        let line = progress_line(&info, std::slice::from_ref(&sponsor), 40).unwrap();
        let (bar, _) = progress_bar(&info, Rect::new(0, 0, 40, 1)).unwrap();
        let track: String = line.spans[1..line.spans.len() - 1].iter().map(|s| s.content.as_ref()).collect();
        assert_eq!(track.chars().count(), bar.width as usize);
        let green: usize =
            line.spans.iter().filter(|s| s.style.fg == Some(GREEN)).map(|s| s.content.chars().count()).sum();
        assert!(green >= 2, "{line:?}");
        assert_eq!(track.matches('┃').count(), 1, "only the chapter that starts after 0: {track}");
        assert!(playing_label(&info, icons(IconSet::Unicode)).contains("§ Second half"));
    }

    /// The first cell whose row contains `text`, at the start of `text`.
    fn find(screen: &str, text: &str) -> (u16, u16) {
        let (y, line) = screen.lines().enumerate().find(|(_, l)| l.contains(text)).unwrap();
        let x = line[..line.find(text).unwrap()].chars().count();
        (x as u16, y as u16)
    }

    #[test]
    fn the_menu_starts_under_the_logo_and_clicks_hit_every_entry() {
        let screen_area = Rect::new(0, 0, 130, 40);
        for set in [IconSet::Emoji, IconSet::Nerd, IconSet::Unicode] {
            let mut s = state();
            s.config.ui.icons = set;
            let screen = render(&s);
            let (_, title) = find(&screen, "tube");
            let (_, header) = find(&screen, " YOUTUBE");
            assert_eq!(header, title + 1, "YOUTUBE right under the logo:\n{screen}");
            assert_eq!(spot(&s, None, screen_area, 3, header), None, "nor the header");
            let icons = icons(set);
            for (i, section) in Section::ALL.iter().enumerate() {
                // The sidebar row that reads exactly `<icon> <label>`.
                let entry = format!("{} {}", icons.section(*section), section.label(Lang::Es));
                let y = screen
                    .lines()
                    .position(|l| {
                        l.split('│').nth(1).is_some_and(|cell| {
                            cell.trim_start_matches(['▌', ' ']).split_whitespace().collect::<Vec<_>>().join(" ")
                                == entry
                        })
                    })
                    .unwrap_or_else(|| panic!("{entry:?} not in the menu:\n{screen}"));
                assert_eq!(spot(&s, None, screen_area, 3, y as u16), Some(Spot::Section(i)), "{section:?}");
            }
            assert!(screen.contains(Section::Salir.label(Lang::Es)), "the whole menu fits:\n{screen}");
        }
    }

    #[test]
    fn the_donation_addresses_say_whose_they_are() {
        for lang in [Lang::Es, Lang::En] {
            let mut s = state();
            s.config.ui.lang = Some(lang);
            let about = Section::ALL.iter().position(|x| *x == Section::AcercaDe).unwrap();
            s.update(Action::OpenSection(about));
            let mut terminal = Terminal::new(TestBackend::new(200, 60)).unwrap();
            terminal.draw(|f| draw(f, &s, &VIEW)).unwrap();
            let screen: String = terminal
                .backend()
                .buffer()
                .content()
                .chunks(200)
                .map(|row| row.iter().map(|c| c.symbol()).collect::<String>() + "\n")
                .collect();
            let note = lang.pick(DONATIONS_NOTE.0, DONATIONS_NOTE.1);
            assert!(screen.contains(note), "{screen}");
            assert!(note.contains("polblancoo") && note.contains("mortbrew"));
            let (note_y, _) = screen.lines().enumerate().find(|(_, l)| l.contains(note)).unwrap();
            let (wallet_y, _) = screen.lines().enumerate().find(|(_, l)| l.contains(&donations()[0].address)).unwrap();
            assert!(note_y < wallet_y, "said before the addresses");
        }
    }

    #[test]
    fn the_logo_is_a_plain_red_triangle() {
        assert_eq!(EMOJI.logo, "▶");
        assert!(!EMOJI.logo.contains('\u{FE0F}'));
        let mut s = state();
        s.config.ui.icons = IconSet::Emoji;
        let mut terminal = Terminal::new(TestBackend::new(130, 40)).unwrap();
        terminal.draw(|f| draw(f, &s, &VIEW)).unwrap();
        let buffer = terminal.backend().buffer();
        // Sidebar title: `┌ ▶ tubevibe ┐`.
        let row: String = (0..14).map(|x| buffer[(x, 0)].symbol().to_owned()).collect();
        assert!(row.starts_with("┌ ▶ tubevibe "), "{row:?}");
        assert_eq!(buffer[(2, 0)].fg, RED, "▶ in the red of \"tube\"");
        assert_eq!(buffer[(4, 0)].fg, RED);
        // Welcome screen and About: `▶ tubevibe`, one space.
        let screen = render(&s);
        assert!(screen.contains("▶ tubevibe"), "{screen}");
        let about = Section::ALL.iter().position(|x| *x == Section::AcercaDe).unwrap();
        s.update(Action::OpenSection(about));
        assert!(render(&s).contains(&format!("▶ tubevibe  v{}", env!("CARGO_PKG_VERSION"))));
    }

    fn render_screen(state: &State, width: u16, height: u16) -> Vec<String> {
        let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
        terminal.draw(|f| draw(f, state, &VIEW)).unwrap();
        let buffer = terminal.backend().buffer();
        buffer.content.chunks(width as usize).map(|row| row.iter().map(|c| c.symbol()).collect()).collect()
    }

    fn row_of(screen: &[String], text: &str) -> usize {
        screen.iter().position(|l| l.contains(text)).unwrap_or_else(|| panic!("{text}\n{}", screen.join("\n")))
    }

    #[test]
    fn settings_about_and_quit_sit_at_the_bottom_under_a_short_rule() {
        let s = state();
        let screen = render_screen(&s, 130, 40);
        // 40 rows: the menu's frame ends at row 37 (status and hints below), its inside at 36.
        let quit = row_of(&screen, Section::Salir.label(Lang::Es));
        assert_eq!(quit, 36, "{}", screen.join("\n"));
        assert_eq!(row_of(&screen, Section::AcercaDe.label(Lang::Es)), 35);
        assert_eq!(row_of(&screen, Section::Configuracion.label(Lang::Es)), 34);
        let rule: String = screen[33].chars().take(usize::from(SIDEBAR_WIDTH)).collect();
        let inner: String = rule.chars().skip(1).take(usize::from(SIDEBAR_WIDTH) - 2).collect();
        assert!(inner.starts_with("  ─") && inner.ends_with("─  "), "inset on both sides: {rule:?}");
        assert!(inner.trim().chars().all(|c| c == '─'));
        // Downloads stays with its group, a gap below it.
        let downloads = row_of(&screen, Section::Descargas.label(Lang::Es));
        assert!(downloads < 30, "{downloads}");
        // A click on the pinned rows opens them.
        let area = Rect::new(0, 0, 130, 40);
        let salir = Section::ALL.iter().position(|x| *x == Section::Salir).unwrap();
        assert_eq!(spot(&s, None, area, 5, quit as u16), Some(Spot::Section(salir)));
        assert_eq!(spot(&s, None, area, 5, 33), None, "the rule");
    }

    #[test]
    fn a_short_menu_puts_settings_right_below_and_scrolls_to_the_choice() {
        let mut s = state();
        // 28 rows: one row short of the whole menu, no room for a gap.
        let screen = render_screen(&s, 130, 28);
        assert!(!screen.iter().any(|l| l.contains(Section::Salir.label(Lang::Es))), "cut at the bottom");
        let downloads = row_of(&screen, Section::Descargas.label(Lang::Es));
        assert!(screen[downloads + 2].contains("──"), "blank, then the rule: {}", screen.join("\n"));
        // Up from the first section wraps to Quit, which is scrolled into view.
        s.update(Action::Up);
        assert_eq!(Section::ALL[s.menu], Section::Salir);
        let screen = render_screen(&s, 130, 28);
        let quit = row_of(&screen, Section::Salir.label(Lang::Es));
        assert_eq!(quit, 24, "the menu's last row: {}", screen.join("\n"));
        let salir = Section::ALL.iter().position(|x| *x == Section::Salir).unwrap();
        assert_eq!(spot(&s, None, Rect::new(0, 0, 130, 28), 5, quit as u16), Some(Spot::Section(salir)));
        // And down from it back to the top.
        s.update(Action::Down);
        assert_eq!(s.menu, 0);
    }

    #[test]
    fn the_menu_border_lights_up_when_the_menu_has_the_keys() {
        let gruvbox = crate::theme::parse_colors_toml(
            &std::fs::read_to_string(
                std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/omarchy/gruvbox/colors.toml"),
            )
            .unwrap(),
        )
        .unwrap();
        for palette in [crate::theme::TOKYO_NIGHT, gruvbox] {
            let (blue, muted, highlight) = if palette == crate::theme::TOKYO_NIGHT {
                (BLUE, MUTED, HIGHLIGHT)
            } else {
                (palette.blue, palette.muted, palette.highlight)
            };
            for style in BorderStyle::ALL {
                let mut s = state();
                s.palette = palette;
                s.config.ui.border_style = style;
                let size = Rect::new(0, 0, 130, 40);
                let side = areas(size, true).sidebar.unwrap();
                let (left, right, mid) = (side.x, side.right() - 1, side.y + side.height / 2);
                let draw = |s: &State| {
                    let mut terminal = Terminal::new(TestBackend::new(130, 40)).unwrap();
                    terminal.draw(|f| draw(f, s, &VIEW)).unwrap();
                    terminal.backend().buffer().clone()
                };
                let rule_row = |buffer: &ratatui::buffer::Buffer| {
                    (0..side.height)
                        .find(|&y| buffer[(left + 4, y)].symbol() == "─" && buffer[(left + 1, y)].symbol() == " ")
                        .unwrap()
                };
                s.focus = Focus::Menu;
                let menu = draw(&s);
                s.focus = Focus::List;
                let list = draw(&s);
                if style == BorderStyle::None {
                    // No border to light: the title does, on the highlight.
                    assert_eq!(menu[(left + 6, 0)].bg, highlight, "{style:?}: tube… on the highlight");
                    assert_ne!(list[(left + 6, 0)].bg, highlight, "{style:?}");
                } else {
                    for (x, y) in [(left, 0), (left, mid), (right, mid), (left + 1, side.bottom() - 1)] {
                        assert_eq!(menu[(x, y)].fg, blue, "{style:?} focused at {x},{y}");
                        assert_eq!(list[(x, y)].fg, muted, "{style:?} unfocused at {x},{y}");
                    }
                }
                // The rule above Settings, About and Quit stays dim either way.
                for buffer in [&menu, &list] {
                    let y = rule_row(buffer);
                    assert_eq!(buffer[(left + 4, y)].fg, muted, "{style:?}: the rule");
                }
            }
        }
    }

    #[test]
    fn every_border_style_frames_the_panes_and_none_keeps_their_room() {
        let corners = [
            (BorderStyle::Plain, '┌'),
            (BorderStyle::Rounded, '╭'),
            (BorderStyle::Thick, '┏'),
            (BorderStyle::Double, '╔'),
        ];
        for (style, corner) in corners {
            let mut s = state();
            s.config.ui.border_style = style;
            let screen = render_screen(&s, 130, 40);
            assert!(screen[0].starts_with(corner), "{style:?}: {}", screen[0]);
        }
        let mut s = state();
        let plain = render_screen(&s, 130, 40);
        s.config.ui.border_style = BorderStyle::None;
        let none = render_screen(&s, 130, 40);
        assert!(!none.iter().any(|l| l.contains('│') || l.contains('┌')), "{}", none.join("\n"));
        // Same rows and columns for everything inside.
        for label in ["tubevibe", Section::Inicio.label(Lang::Es), Section::Salir.label(Lang::Es)] {
            assert_eq!(row_of(&plain, label), row_of(&none, label), "{label}");
            let col = |screen: &[String]| {
                let line = &screen[row_of(screen, label)];
                Span::raw(&line[..line.find(label).unwrap()]).width()
            };
            assert_eq!(col(&plain), col(&none), "{label}");
        }
    }

    #[test]
    fn spots_under_the_pointer() {
        let screen_area = Rect::new(0, 0, 130, 40);
        let videos: Vec<Video> = (0..60).map(|i| video(&i.to_string())).collect();
        let mut s = with_items(Items::Videos(videos));
        let screen = render(&s);
        let spot_at = |s: &State, (x, y): (u16, u16)| spot(s, None, screen_area, x, y);
        let musica = Section::ALL.iter().position(|x| *x == Section::MiMusica).unwrap();
        assert_eq!(spot_at(&s, find(&screen, Section::MiMusica.label(Lang::Es))), Some(Spot::Section(musica)));
        assert_eq!(spot_at(&s, find(&screen, "title 2 ")), Some(Spot::Row(2)));
        assert_eq!(spot_at(&s, (0, 39)), None, "the key hints");
        // Scrolled: the selection is the last visible row.
        for _ in 0..50 {
            s.update(Action::Down);
        }
        let screen = render(&s);
        assert_eq!(spot_at(&s, find(&screen, "▌")), Some(Spot::Row(50)));
        assert_eq!(spot_at(&s, find(&screen, "title 30 ")), Some(Spot::Row(30)));
        s.update(Action::Enter); // plays video 50 in tab 1
        let screen = render(&s);
        let (x, y) = find(&screen, "│ ▶ title 50 ");
        assert_eq!(spot_at(&s, (x + 2, y)), Some(Spot::Tab(1)));
        assert!(!screen.contains("Inicio"), "no tab bar to click");
        assert_eq!(spot_at(&s, (80, 20)), Some(Spot::Video));
        let info = Info { position: Some(0.0), duration: Some(100.0), ..Info::default() };
        let content = areas(screen_area, s.sidebar).content;
        let (bar, _) = progress_bar(&info, content).unwrap();
        assert_eq!(spot(&s, Some(&info), screen_area, bar.x, bar.y), Some(Spot::Progress(0)));
        let middle = spot(&s, Some(&info), screen_area, bar.x + bar.width / 2, bar.y);
        assert!(matches!(middle, Some(Spot::Progress(49..=51))), "{middle:?}");
        s.update(Action::ToggleHelp);
        assert_eq!(spot_at(&s, (80, 20)), None, "the help covers everything");
    }

    #[test]
    fn wrapping_keeps_line_breaks_words_and_cuts_long_ones() {
        assert_eq!(wrap_text("one two three\n\nfour", 7), ["one two", "three", "", "four"]);
        assert_eq!(wrap_text("https://example.com/abc", 10), ["https://ex", "ample.com/", "abc"]);
        assert_eq!(wrap_text("a  b", 10), ["a b"]);
    }

    #[test]
    fn summary_lines_fit_their_rows_and_mark_the_cut() {
        let lines = |rows| summary_lines(Some("uno dos tres cuatro cinco seis"), 9, rows, Lang::En, "✦");
        let text = |l: &[Line]| l.iter().map(ToString::to_string).collect::<Vec<_>>();
        assert!(lines(0).is_empty());
        assert!(lines(1).is_empty(), "no room for more than the blank line");
        assert_eq!(text(&lines(9))[2..], ["uno dos", "tres", "cuatro", "cinco", "seis"]);
        assert_eq!(text(&lines(4))[2..], ["uno dos", "tres…"]);
        assert_eq!(text(&lines(4))[0], "", "a blank line above the heading");
        assert!(text(&lines(4))[1].starts_with("✦ AI Summary"));
        let none = summary_lines(None, 40, 9, Lang::En, "✦");
        assert_eq!(text(&none), ["", "✦ AI Summary · not available for this video"]);
        assert!(summary_lines(None, 40, 1, Lang::En, "✦").is_empty());
    }

    /// The row above `(x, y)` is empty there and the one above it is not:
    /// exactly one blank line.
    fn blank_above(screen: &str, x: u16, y: u16) -> bool {
        let cells = |row: u16| -> String {
            screen.lines().nth(usize::from(row)).unwrap().chars().skip(usize::from(x)).take(12).collect()
        };
        cells(y - 1).trim().is_empty() && !cells(y - 2).trim().is_empty()
    }

    /// An idle video tab with its details (the video plays in its own window).
    fn details_tab(summary: Option<&str>) -> (State, String) {
        let mut s = with_items(Items::Videos(vec![video("a")]));
        s.config.player.autoplay = false;
        s.video_window = true;
        s.update(Action::Enter);
        let tab = s.active_tab().unwrap().id;
        let watch =
            Watch { related: vec![video("r1"), video("r2")], summary: summary.map(String::from), ..Watch::default() };
        s.update(Action::Watched(tab, Ok(watch)));
        let screen = render_with(&s, &View { embedded: false, ..VIEW });
        (s, screen)
    }

    #[test]
    fn video_tab_shows_the_ai_summary_right_under_its_details() {
        let summary = "UFD Tech explores the updated features of the XREAL Air 1 Pro+ glasses, highlighting \
                       improvements in display brightness, refresh rates, and comfort. The demonstration \
                       showcases how these spatial display specs function as a portable multi-monitor \
                       workspace and entertainment hub, providing flexibility for those working from home \
                       or traveling. A last sentence that ends it all: THE END.";
        let (s, screen) = details_tab(Some(summary));
        let (_, status) = find(&screen, "▶ Enter lo reproduce");
        let (x, heading) = find(&screen, "✦ Resumen IA");
        assert_eq!(heading, status + 2, "one blank line under the status line:\n{screen}");
        // Across the left column, under the thumbnail, all of it; Up next one
        // blank row below its last line.
        let (up_next_x, up_next) = find(&screen, "┌ A continuación");
        assert_eq!(x, up_next_x, "the left column's width:\n{screen}");
        assert!(screen.contains("UFD Tech explores") && screen.contains("THE END."), "whole: {screen}");
        assert_eq!(find(&screen, "THE END.").1 + 2, up_next, "Up next right below it:\n{screen}");
        // The comments run the whole height of the tab, on the right.
        let lines: Vec<&str> = screen.lines().collect();
        let (cx, top) = find(&screen, "┌ Comentarios");
        assert_eq!(top, 1, "from the tab's top:\n{screen}");
        let bottom = lines.iter().rposition(|l| l.chars().nth(usize::from(cx)) == Some('└')).unwrap();
        assert_eq!(bottom, lines.len() - 4, "to its bottom:\n{screen}");
        assert!(cx > up_next_x + 40, "on the right");
        // Clicks on Up next find the rows where they were drawn.
        let screen_area = Rect::new(0, 0, 130, 40);
        let (x, y) = find(&screen, "title r2 ");
        assert_eq!(spot(&s, None, screen_area, x, y), Some(Spot::Row(1)));
        // Without one: the heading says so, Up next right below.
        let (plain, plain_screen) = details_tab(None);
        let (hx, hy) = find(&plain_screen, "✦ Resumen IA · no disponible para este video");
        assert_eq!(hy, find(&plain_screen, "▶ Enter lo reproduce").1 + 2, "{plain_screen}");
        assert_eq!(hy + 2, find(&plain_screen, "┌ A continuación").1, "{plain_screen}");
        assert_eq!(hx, find(&plain_screen, "┌ A continuación").0);
        let (x, y2) = find(&plain_screen, "title r2 ");
        assert!(y2 < y, "Up next moves up without a summary");
        assert_eq!(spot(&plain, None, screen_area, x, y2), Some(Spot::Row(1)));
    }

    #[test]
    fn video_tab_says_nothing_of_the_summary_while_loading_nor_for_live_videos() {
        let mut s = with_items(Items::Videos(vec![video("a")]));
        s.config.player.autoplay = false;
        s.video_window = true;
        s.update(Action::Enter);
        let view = View { embedded: false, ..VIEW };
        assert!(!render_with(&s, &view).contains("✦"), "watch page not here yet");
        let mut s = with_items(Items::Videos(vec![Video { live: Live::Now, ..video("l") }]));
        s.video_window = true;
        s.update(Action::Enter);
        let tab = s.active_tab().unwrap().id;
        s.update(Action::Watched(tab, Ok(Watch { related: vec![video("r1")], ..Watch::default() })));
        let screen = render_with(&s, &view);
        assert!(!screen.contains("✦"), "live: never asked: {screen}");
        let (x, y) = find(&screen, "title r1 ");
        assert_eq!(spot(&s, None, Rect::new(0, 0, 130, 40), x, y), Some(Spot::Row(0)));
    }

    /// The tab drawn at `width` × `height`, window mode.
    fn tab_screen(s: &State, width: u16, height: u16) -> Vec<String> {
        let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
        terminal.draw(|f| draw(f, s, &View { embedded: false, ..VIEW })).unwrap();
        let buffer = terminal.backend().buffer();
        buffer.content.chunks(width as usize).map(|row| row.iter().map(|c| c.symbol()).collect()).collect()
    }

    #[test]
    fn up_next_keeps_seven_rows_under_a_long_summary_at_any_height() {
        let long = "palabra ".repeat(800);
        let (s, _) = details_tab(Some(&long));
        for height in [24, 30, 40, 60] {
            let screen = tab_screen(&s, 130, height);
            let top = row_of(&screen, "┌ A continuación");
            // The tab's inside ends 4 rows above the screen's: border, status, hints.
            let bottom = usize::from(height) - 4;
            assert_eq!(bottom - top + 1, 7, "{height} rows:\n{}", screen.join("\n"));
            assert!(screen.iter().any(|l| l.contains("palabra…")), "cut with the mark");
            // A click on Up next, after the layout changed size.
            let (x, y) = (screen[top + 1].find("title r1").unwrap(), top + 1);
            let x = screen[y][..x].chars().count() as u16;
            assert_eq!(spot(&s, None, Rect::new(0, 0, 130, height), x, y as u16), Some(Spot::Row(0)));
        }
    }

    #[test]
    fn a_short_summary_takes_only_its_rows_and_narrow_tabs_stack_the_comments() {
        let (s, _) = details_tab(Some("Corto."));
        let screen = tab_screen(&s, 130, 40);
        let summary = row_of(&screen, "Corto.");
        assert_eq!(row_of(&screen, "┌ A continuación"), summary + 2, "{}", screen.join("\n"));
        // Narrow: the comments go under Up next, which takes the whole width.
        let screen = tab_screen(&s, 100, 40);
        assert!(row_of(&screen, "┌ Comentarios") > row_of(&screen, "┌ A continuación"), "{}", screen.join("\n"));
        assert!(screen[row_of(&screen, "┌ A continuación")].trim_end().ends_with("┐│"));
        // Too short to stack them: none.
        let screen = tab_screen(&s, 100, 22);
        assert!(!screen.iter().any(|l| l.contains("Comentarios")), "{}", screen.join("\n"));
    }

    #[test]
    fn a_long_summary_leaves_up_next_some_rows() {
        let long = "palabra ".repeat(800);
        let (s, screen) = details_tab(Some(&long));
        assert!(screen.contains("A continuación") && screen.contains("title r1"), "{screen}");
        assert!(screen.contains("palabra…"), "cut with the mark: {screen}");
        let (x, y) = find(&screen, "title r1 ");
        assert_eq!(spot(&s, None, Rect::new(0, 0, 130, 40), x, y), Some(Spot::Row(0)));
    }

    #[test]
    fn preview_shows_the_ai_summary_in_all_its_room() {
        let mut s = with_items(Items::Videos(vec![video("a"), video("b")]));
        let short = "Uno dos tres. Fin corto.";
        s.update(Action::Summarized("a".into(), Ok(Some(short.into()))));
        let screen = render(&s);
        assert!(screen.contains("✦ Resumen IA") && screen.contains("Fin corto."), "{screen}");
        let (x, y) = find(&screen, "✦ Resumen IA");
        assert!(blank_above(&screen, x, y), "one blank line after the last detail: {screen}");
        let long: String = (1..=400).map(|n| format!("w{n} ")).collect();
        s.update(Action::Summarized("a".into(), Ok(Some(long))));
        let screen = render(&s);
        let shown = screen.lines().filter(|l| l.contains(" w")).count();
        assert!(shown > 5, "not capped at 5 lines: {shown}\n{screen}");
        assert!(screen.contains('…') && !screen.contains("w400"), "cut at the bottom: {screen}");
        // Not asked yet: nothing (but "a" in the list keeps its mark).
        s.update(Action::Down);
        assert!(!render(&s).contains("Resumen IA"));
        // Asked, it has none (or it could not be had): said so.
        for result in [Ok(None), Err("falló".to_owned())] {
            s.update(Action::Summarized("b".into(), result));
            let screen = render(&s);
            let (x, y) = find(&screen, "✦ Resumen IA · no disponible para este video");
            assert!(blank_above(&screen, x, y), "{screen}");
        }
    }

    #[test]
    fn preview_of_a_live_video_says_nothing_of_the_summary() {
        let mut s = with_items(Items::Videos(vec![Video { live: Live::Now, ..video("l") }]));
        s.summaries.insert("l".into(), None);
        assert!(!render(&s).contains("✦"));
    }

    #[test]
    fn only_the_pane_with_the_keys_is_lit_in_a_video_tab() {
        let mut s = with_items(Items::Videos(vec![video("a")]));
        s.video_window = true;
        s.update(Action::Enter);
        let tab = s.active_tab().unwrap().id;
        let watch = Watch {
            views: None,
            likes: None,
            published: None,
            subscribers: None,
            related: vec![video("r1"), video("r2")],
            rating: None,
            comments_token: None,
            summary: None,
        };
        s.update(Action::Watched(tab, Ok(watch)));
        let content = areas(Rect::new(0, 0, 130, 40), true).content;
        let corner = |s: &State| {
            let mut terminal = Terminal::new(TestBackend::new(130, 40)).unwrap();
            terminal.draw(|f| draw(f, s, &View { embedded: false, ..VIEW })).unwrap();
            terminal.backend().buffer()[(content.right() - 1, content.y)].fg
        };
        assert_eq!(corner(&s), BLUE, "the video pane has the keys");
        s.update(Action::ToggleFocus);
        assert!(s.key_context().related);
        assert_eq!(corner(&s), MUTED, "Up Next has them: the video pane is dim");
        let esc = keymap::map(
            ratatui::crossterm::event::KeyEvent::from(ratatui::crossterm::event::KeyCode::Esc),
            s.key_context(),
        )
        .unwrap();
        s.update(esc);
        assert_eq!(corner(&s), BLUE, "Esc in Up Next: back to the video, lit again");
        assert_eq!(s.active, 1, "still on the tab");
        // Esc again: the menu has the keys, the tab stays shown and goes dim.
        let esc = keymap::map(
            ratatui::crossterm::event::KeyEvent::from(ratatui::crossterm::event::KeyCode::Esc),
            s.key_context(),
        )
        .unwrap();
        s.update(esc);
        assert_eq!((s.active, s.focus), (1, Focus::Menu));
        assert_eq!(corner(&s), MUTED, "the video pane is dim while the menu has the keys");
        let screen = render_with(&s, &View { embedded: false, ..VIEW });
        assert!(screen.contains("▌▶ title a"), "the cursor's tab is lit: {screen}");
        assert!(screen.contains("A continuación · 2 · Tab"), "Up Next not lit either: {screen}");
    }

    #[test]
    fn video_tab_in_window_mode_shows_details_and_related() {
        let mut s = with_items(Items::Videos(vec![video("a")]));
        s.video_window = true;
        s.update(Action::Enter);
        let tab = s.active_tab().unwrap().id;
        let screen = render_with(&s, &View { embedded: false, ..VIEW });
        assert!(screen.contains("Buscando videos relacionados"), "{screen}");
        let watch = Watch {
            views: Some(5771),
            likes: Some(739),
            published: None,
            subscribers: Some("559K".into()),
            related: vec![video("r1"), video("r2")],
            rating: Some(Rating::Like),
            comments_token: Some("T".into()),
            summary: None,
        };
        s.update(Action::Watched(tab, Ok(watch)));
        assert!(render_with(&s, &View { embedded: false, ..VIEW }).contains("Buscando comentarios"));
        let comment = crate::youtube::Comment {
            author: "@ana".into(),
            text: "Qué temazo".into(),
            likes: Some("12".into()),
            published: Some("hace 1 año".into()),
        };
        s.update(Action::CommentsLoaded(tab, Ok(vec![comment])));
        let view = View { embedded: false, ..VIEW };
        let screen = render_with(&s, &view);
        for text in
            ["title a", "chan a · 559K suscriptores", "5 k vistas", "Te gusta", "A continuación · 2 · Tab", "title r2"]
                .into_iter()
                .chain(["Comentarios 1/1", "@ana  ▲", "12  hace 1 año", "Qué temazo"])
        {
            assert!(screen.contains(text), "{text}\n{screen}");
        }
        assert!(!screen.contains("ventana de mpv"));
        let screen_area = Rect::new(0, 0, 130, 40);
        let (id, cells) = thumbnail_area(&s, screen_area).unwrap();
        assert_eq!(id, "a");
        assert_eq!((cells.width, cells.height), (22, 22 * 9 / 32), "two fifths of the left column, at most 36");
        let (x, y) = find(&screen, "title r2 ");
        assert_eq!(spot(&s, None, screen_area, x, y), Some(Spot::Row(1)));
        assert_eq!(
            (hints(&s)[0].0.as_str(), hints(&s)[1].0.as_str()),
            ("Espacio", "x"),
            "Space, then x (Tab to Up Next is in the help)"
        );
        s.update(Action::ToggleFocus);
        let screen = render_with(&s, &view);
        assert!(screen.contains("A continuación · 2 "), "{screen}");
        assert_eq!(hints(&s)[0].1, "ver");
        s.video_window = false;
        assert_eq!(thumbnail_area(&s, screen_area), None, "embedded: the video covers the tab");
    }

    #[test]
    fn a_new_version_is_announced_with_how_to_get_it() {
        let mut s = state();
        let update = crate::update::Update {
            version: "9.9.9".into(),
            how: Install::Command("cargo install --locked --git https://github.com/mortbrew/tubevibe".into()),
            replace: Replace::Package,
        };
        s.update(Action::UpdateFound(update));
        assert!(s.status.contains("↑ tubevibe 9.9.9 disponible"), "{}", s.status);
        let screen = render(&s);
        assert!(screen.contains("Acerca de ↑"), "{screen}");
        let about = Section::ALL.iter().position(|x| *x == Section::AcercaDe).unwrap();
        s.update(Action::OpenSection(about));
        let screen = render(&s);
        assert!(screen.contains("↑ Hay una versión nueva: 9.9.9"), "the icon set's arrow: {screen}");
        assert!(screen.contains("cargo install --locked --git https://github.com/mortbrew/tubevibe"), "{screen}");
        assert!(screen.contains("Actualizá con:"), "greyed out: the command by hand stays: {screen}");
        assert!(screen.contains("Instalado con un paquete"), "{screen}");
        assert!(screen.contains("Actualizar Ahora"), "shown, but greyed out: {screen}");
        assert!(!about_button_ready(&s), "a package is updated by its manager");
        assert!(s.update(Action::Enter).is_empty(), "the greyed-out button does nothing");
        assert!(s.status.contains("Instalado con un paquete"), "{}", s.status);
        let mut ready = s.update.clone().unwrap();
        ready.replace = Replace::Ready("https://example.com/t.tar.gz".into());
        s.update(Action::UpdateFound(ready));
        assert!(s.status.contains("9.9.9 disponible · mirá Acerca de"), "{}", s.status);
        s.update(Action::ToggleUiLang);
        s.update(Action::OpenSection(about));
        assert_eq!(s.focus, Focus::List, "opening About selects the button");
        let screen = render(&s);
        assert!(screen.contains("▶   ↑ Update Now   ◀"), "focused: {screen}");
        assert!(!screen.contains("Update with:") && !screen.contains("Shift + drag"), "the button does it: {screen}");
        assert_eq!(hints(&s)[0], ("Enter".to_string(), "Update Now".to_string()));
        s.update(Action::ToggleFocus);
        assert_eq!(s.focus, Focus::Menu);
        let screen = render(&s);
        assert!(!screen.contains("◀") && screen.contains("↑ Update Now"), "{screen}");
        assert!(screen.contains("(Tab to Select, Enter to Press)"), "{screen}");
        assert_eq!(hints(&s)[0], ("Tab".to_string(), "Select Update Now".to_string()));
        s.update(Action::ToggleFocus);
        assert_eq!(
            s.update(Action::Enter),
            [Effect::SelfUpdate { url: "https://example.com/t.tar.gz".into(), version: "9.9.9".into() }]
        );
        s.update(Action::SelfUpdated(Err("download: timeout".into())));
        let screen = render(&s);
        assert!(screen.contains("✘ Could Not Update: download: timeout"), "{screen}");
        assert!(screen.contains("Update with:"), "after a failure, by hand too: {screen}");
        s.update(Action::Enter);
        s.update(Action::UpdateProgress(30));
        let screen = render(&s);
        assert!(screen.contains("↓ Downloading… 30%") && !screen.contains("◀"), "busy: {screen}");
        assert!(s.update(Action::Enter).is_empty(), "one at a time");
        s.update(Action::SelfUpdated(Ok("9.9.9".into())));
        let screen = render(&s);
        assert!(screen.contains("✔ tubevibe 9.9.9 Installed · Restart Now Opens It"), "{screen}");
        assert!(!screen.contains("Update with:"), "no command once done: {screen}");
        // The button is now Restart Now: Enter quits so the runtime starts the new one.
        assert!(screen.contains("Restart Now") && !screen.contains("Update Now  "), "{screen}");
        assert_eq!(hints(&s)[0], ("Enter".to_string(), "Restart Now".to_string()));
        assert!(s.update(Action::Enter).is_empty());
        assert!(s.restart && s.quit);
    }

    #[test]
    fn the_screen_takes_the_omarchy_theme_colours() {
        let colours = |state: &State| {
            let mut terminal = Terminal::new(TestBackend::new(130, 40)).unwrap();
            terminal.draw(|f| draw(f, state, &VIEW)).unwrap();
            let buffer = terminal.backend().buffer();
            buffer.content.iter().flat_map(|c| [c.fg, c.bg]).collect::<std::collections::HashSet<_>>()
        };
        let mut s = state();
        let tokyo = colours(&s);
        assert!(tokyo.contains(&FG) && tokyo.contains(&HIGHLIGHT));
        let gruvbox = theme::parse_colors_toml(include_str!("../tests/fixtures/omarchy/gruvbox/colors.toml")).unwrap();
        s.update(Action::Palette(gruvbox));
        let shown = colours(&s);
        assert!(shown.contains(&Color::Rgb(0xd4, 0xbe, 0x98)), "gruvbox text");
        assert!(shown.contains(&Color::Rgb(0x50, 0x49, 0x45)), "gruvbox selection");
        assert!(!shown.contains(&FG) && !shown.contains(&HIGHLIGHT), "no Tokyo Night left");
    }

    #[test]
    fn about_says_up_to_date_only_once_github_confirmed_it() {
        let mut s = state();
        let about = Section::ALL.iter().position(|x| *x == Section::AcercaDe).unwrap();
        s.update(Action::OpenSection(about));
        assert!(!render(&s).contains("última versión"), "still checking, or the check failed");
        s.update(Action::UpToDate);
        assert!(render(&s).contains("✔ Tenés la última versión"));
        s.update(Action::ToggleUiLang);
        s.update(Action::OpenSection(about));
        assert!(render(&s).contains("✔ You're on the Latest Version"));
        s.update(Action::UpdateFound(crate::update::Update {
            version: "9.9.9".into(),
            how: Install::Command("x".into()),
            replace: Replace::Package,
        }));
        assert!(!render(&s).contains("Latest Version"), "a newer one wins");
        // Every icon set has its own glyphs for the update lines.
        for set in [IconSet::Emoji, IconSet::Nerd, IconSet::Unicode] {
            let i = icons(set);
            assert!([i.up_to_date, i.update, i.downloading, i.failed].iter().all(|g| !g.is_empty()));
        }
        assert_eq!(icons(IconSet::Emoji).up_to_date, "✅");
        assert_eq!(icons(IconSet::Nerd).up_to_date, "\u{f058}");
    }

    /// Every glyph of a set, menu and settings included.
    fn all_glyphs(i: &Icons) -> Vec<&'static str> {
        let mut glyphs = vec![
            i.logo,
            i.tab,
            i.live,
            i.upcoming,
            i.later,
            i.favorite,
            i.subscribed,
            i.liked,
            i.disliked,
            i.user,
            i.language,
            i.volume,
            i.muted,
            i.watched,
            i.up_to_date,
            i.update,
            i.downloading,
            i.failed,
            i.trash,
            i.notify,
            i.unsubscribed,
            i.resume,
            i.skip,
            i.stop,
            i.audio,
            i.saved,
            i.warning,
            i.waiting,
            i.donate,
        ];
        glyphs.extend(i.sections);
        glyphs.extend(i.settings);
        glyphs
    }

    /// Shown as a colour emoji by default, or by many terminals anyway
    /// (⏩–⏺ media keys, ⬅–⬇ arrows, ⚠ ✉ with an emoji form). The menu's ⚙
    /// is text by default and stays.
    fn looks_like_emoji(c: char) -> bool {
        matches!(c as u32, 0x1F000..=0x1FAFF | 0x23E9..=0x23FA | 0x2B05..=0x2B55 | 0xFE0F)
            || "⌚⌛☔☕♿⚓⚡⚪⚫⚽⚾⛄⛅⛎⛔⛪⛲⛳⛵⛺⛽✅✊✋✨❌❎❓❔❕❗➕➖➗➰➿⚠✉".contains(c)
    }

    #[test]
    fn every_set_has_every_icon_and_only_emoji_has_emoji() {
        for set in [IconSet::Emoji, IconSet::Nerd, IconSet::Unicode] {
            assert!(all_glyphs(icons(set)).iter().all(|g| !g.is_empty()), "{set:?}");
        }
        for set in [IconSet::Nerd, IconSet::Unicode] {
            for glyph in all_glyphs(icons(set)) {
                assert!(!glyph.chars().any(looks_like_emoji), "{set:?}: {glyph}");
            }
        }
        // Nerd Fonts v3: Font Awesome (U+F000–U+F2FF) or Material Design (U+F0001 and up).
        for glyph in all_glyphs(&NERD).into_iter().skip(1) {
            let c = glyph.chars().next().unwrap() as u32;
            assert!((0xF000..=0xF2FF).contains(&c) || (0xF0001..=0xF1AF0).contains(&c), "{c:x}");
        }
        let (emoji, nerd, plain) = (icons(IconSet::Emoji), icons(IconSet::Nerd), icons(IconSet::Unicode));
        assert_eq!(
            [emoji.notify, emoji.unsubscribed, emoji.resume, emoji.skip, emoji.stop],
            ["🔔", "🔕", "⏯", "⏩", "⏹"]
        );
        assert_eq!([nerd.notify, nerd.resume, nerd.audio], ["\u{f0f3}", "\u{f040e}", "\u{f025}"]);
        assert_eq!([plain.notify, plain.unsubscribed, plain.stop, plain.audio], ["⚑", "⚐", "■", "♪"]);
    }

    #[test]
    fn status_messages_follow_the_icon_set() {
        for set in [IconSet::Emoji, IconSet::Nerd, IconSet::Unicode] {
            let mut s = state();
            s.config.ui.icons = set;
            s.config.ui.lang = Some(Lang::En);
            let (config, warnings) =
                Config::parse("[ui]\nlang = \"en\"\nicons = \"emoji\"\nfuture_glow = true\n").unwrap();
            drop(config);
            s.update(Action::ConfigWarnings(warnings));
            assert!(s.status.starts_with(&format!("{} Ignored", icons(set).warning)), "{set:?}: {}", s.status);
            if set != IconSet::Emoji {
                assert!(!s.status.chars().any(looks_like_emoji), "{set:?}: {}", s.status);
            }
        }
    }

    #[test]
    fn feeds_say_how_many_watched_videos_they_hide() {
        let mut s = state();
        s.watched.insert("b".into());
        s.menu = Section::ALL.iter().position(|x| *x == Section::Suscripciones).unwrap();
        let id = match s.update(Action::Enter).as_slice() {
            [Effect::Load { id, .. }, ..] => *id,
            other => panic!("{other:?}"),
        };
        s.update(Action::Loaded(id, Ok(Items::Videos(vec![video("a"), video("b")]))));
        let screen = render(&s);
        assert!(screen.contains("Suscripciones · 1 · ◉ 1 vistos ocultos"), "{screen}");
        assert!(hints(&s).iter().any(|(k, d)| k == "v" && *d == "mostrar vistos"));
        s.update(Action::ToggleWatched);
        let screen = render(&s);
        assert!(screen.contains("◉ con los vistos"), "{screen}");
        assert!(screen.contains("◉ title b"), "marked: {screen}");
    }

    #[test]
    fn system_check_page_says_what_to_install() {
        use crate::doctor::Check;
        let mut s = state();
        let missing = Check { tool: Tool::Mpv, found: None, outdated: false, fix: Some("sudo apt install mpv".into()) };
        let old = Check {
            tool: Tool::YtDlp,
            found: Some(("/usr/bin/yt-dlp".into(), Some("2024.04.09".into()))),
            outdated: true,
            fix: Some("sudo apt install pipx && pipx install yt-dlp".into()),
        };
        let fine = Check {
            tool: Tool::Kitty,
            found: Some(("/usr/bin/kitty".into(), Some("0.39.1".into()))),
            outdated: false,
            fix: None,
        };
        let keyring = Check {
            tool: Tool::Keyring,
            found: Some(("GNOMEKEYRING".into(), Some("GNOMEKEYRING".into()))),
            outdated: true,
            fix: Some("pipx inject yt-dlp secretstorage".into()),
        };
        s.update(Action::SystemChecked(Manager::Apt, vec![missing, old, fine, keyring]));
        let screen = render(&s);
        for text in [
            "✘  mpv",
            "falta · necesario",
            "sudo apt install mpv",
            "△  yt-dlp",
            "2024.04.09",
            "pipx install yt-dlp",
            "✔  kitty",
            "△  keyring",
            "falta: secretstorage",
            "pipx inject yt-dlp secretstorage",
            "Comandos para: apt",
        ] {
            assert!(screen.contains(text), "{text}\n{screen}");
        }
    }

    #[test]
    fn helpers() {
        assert_eq!(truncate("ñandú", 3), "ñan…");
        assert_eq!(truncate("abc", 3), "abc");
        assert_eq!(pad("ab", 4), "ab  ");
        assert_eq!(pad("abcdef", 4), "abcd");
        assert_eq!(pad_cells("🔴 en vivo", 14), "🔴 en vivo    ");
        assert_eq!(pad_cells("● en vivo", 14), "● en vivo     ");
    }

    /// The text of each sidebar row drawn in the "shown" mark (not the cursor's highlight).
    fn shown_rows(s: &State) -> Vec<String> {
        let mut terminal = Terminal::new(TestBackend::new(130, 40)).unwrap();
        terminal.draw(|f| draw(f, s, &VIEW)).unwrap();
        let buffer = terminal.backend().buffer();
        let side = areas(Rect::new(0, 0, 130, 40), true).sidebar.unwrap();
        (side.y + 1..side.bottom() - 1)
            .filter(|&y| {
                (side.x + 1..side.right() - 1).any(|x| {
                    let c = &buffer[(x, y)];
                    c.symbol().chars().any(char::is_alphanumeric) && c.fg == SHOWN && c.bg != HIGHLIGHT
                })
            })
            .map(|y| {
                (side.x + 1..side.right() - 1).map(|x| buffer[(x, y)].symbol()).collect::<String>().trim().to_owned()
            })
            .collect()
    }

    fn at(section: Section) -> usize {
        Section::ALL.iter().position(|x| *x == section).unwrap()
    }

    #[test]
    fn one_row_is_marked_shown() {
        let en = |mut s: State| {
            s.config.ui.lang = Some(Lang::En);
            s
        };
        // A list shown, the cursor moved off it: its row keeps the mark.
        let mut s = en(with_items(Items::Videos(vec![video("a")])));
        let shown = s.shown_row();
        s.focus = Focus::Menu;
        s.menu = at(Section::Configuracion);
        let rows = shown_rows(&s);
        assert_eq!(rows.len(), 1, "{rows:?}");
        assert!(shown.is_some());
        // The cursor on the shown row: its focus highlight, no second mark.
        if let Some(crate::app::MenuRow::Section(i)) = shown {
            s.menu = i;
            assert_eq!(shown_rows(&s), Vec::<String>::new());
        }
        // A tab shown, the cursor on a section: the tab's row has the mark, alone.
        let mut s = en(with_items(Items::Videos(vec![video("a")])));
        s.update(Action::Enter);
        assert!(s.active > 0);
        s.focus = Focus::Menu;
        s.menu_tab = None;
        s.menu = at(Section::Descargas);
        let rows = shown_rows(&s);
        assert_eq!(rows.len(), 1, "{rows:?}");
        assert!(!rows[0].contains(Section::Descargas.label(Lang::En)), "{rows:?}");
        // Settings shown: Settings has the mark.
        let mut s = en(open_settings());
        s.focus = Focus::Menu;
        s.menu = at(Section::Descargas);
        assert_eq!(shown_rows(&s).len(), 1);
        assert!(shown_rows(&s)[0].contains(Section::Configuracion.label(Lang::En)), "{:?}", shown_rows(&s));
    }

    /// 35 video tabs (titles "title 0" … "title 34"), the last one shown.
    fn with_35_tabs() -> State {
        let mut s = with_items(Items::Videos((0..35).map(|i| video(&i.to_string())).collect()));
        for i in 0..35 {
            s.update(Action::GoTab(0));
            s.focus = Focus::List;
            s.stack[0].selected = i;
            s.update(Action::Enter);
        }
        assert_eq!((s.tabs.len(), s.active), (35, 35));
        s
    }

    fn tab_rows_shown(screen: &[String]) -> Vec<usize> {
        (0..35).filter(|i| screen.iter().any(|l| l.contains(&format!(" title {i} ")) && l.starts_with('│'))).collect()
    }

    #[test]
    fn thirty_five_tabs_scroll_between_a_fixed_top_and_bottom() {
        let mut s = with_35_tabs();
        let empty = render_screen(&state(), 130, 40);
        let screen = render_screen(&s, 130, 40);
        let at = |screen: &[String], text: &str| row_of(screen, text);
        // The top and the bottom are where they are with no tabs.
        for label in [Section::Inicio, Section::Descargas, Section::Configuracion, Section::Salir] {
            let label = label.label(Lang::Es);
            assert_eq!(at(&screen, label), at(&empty, label), "{label}\n{}", screen.join("\n"));
        }
        // No rule above the tabs, just a blank row; the title's chevron says more
        // are above, and with the last tab in view no chevron row below it.
        let (up, down) = (icons(s.config.ui.icons).more_above, icons(s.config.ui.icons).more_below);
        let title = at(&screen, "PESTAÑAS");
        assert!(screen[title - 1].trim_matches(['│', ' ']).is_empty(), "{}", screen.join("\n"));
        assert!(screen[title].contains(up) && !screen.iter().any(|l| l.contains(down)), "{}", screen.join("\n"));
        let last = at(&screen, "│ ▶ title 34 ");
        assert!(screen[last + 1].trim_matches(['│', ' ']).is_empty(), "the gap above the rule");
        // The tab shown is in view, the last rows of the tab area.
        let rows = tab_rows_shown(&screen);
        assert_eq!(*rows.last().unwrap(), 34, "{}", screen.join("\n"));
        assert!(rows.len() < 35 && rows.windows(2).all(|w| w[1] == w[0] + 1), "{rows:?}");
        let count = rows.len();
        // Alt+. wraps to the first tab: the list follows.
        s.update(Action::NextTab);
        let screen = render_screen(&s, 130, 40);
        // As many rows: the "more below" chevron takes the blank row above the rule.
        assert_eq!(tab_rows_shown(&screen), (0..count).collect::<Vec<_>>());
        // At the top: no chevron by the title, a down one in its own row below
        // the last tab shown, in the title chevron's column.
        let title = at(&screen, "PESTAÑAS");
        assert!(!screen[title].contains(up), "{}", screen[title]);
        let below = at(&screen, down);
        assert_eq!(below, at(&screen, &format!("│ ▶ title {} ", count - 1)) + 1);
        assert!(screen[below + 1].contains("──"), "the rule right under it:\n{}", screen.join("\n"));
        let column = |line: &str, icon: &str| line[..line.find(icon).unwrap()].chars().count();
        assert_eq!(column(&screen[below], down), "│ PESTAÑAS ".chars().count(), "{}", screen[below]);
        // In the middle: both.
        s.update(Action::FocusMenu);
        for _ in 0..17 {
            s.update(Action::Down);
        }
        let screen = render_screen(&s, 130, 40);
        let title = at(&screen, "PESTAÑAS");
        assert!(screen[title].contains(up) && screen.iter().any(|l| l.contains(down)), "{}", screen.join("\n"));
        assert_eq!(column(&screen[title], up), column(&screen[at(&screen, down)], down));
        s.update(Action::GoTab(1));
        // Alt+, back to the last.
        s.update(Action::PrevTab);
        assert_eq!(*tab_rows_shown(&render_screen(&s, 130, 40)).last().unwrap(), 34);
        // The menu's cursor walks the tabs up; the list keeps it in view, then
        // down past the last tab to Settings while the tab shown stays put.
        s.update(Action::FocusMenu);
        for _ in 0..34 {
            s.update(Action::Up);
        }
        assert_eq!(s.menu_row(), MenuRow::Tab(0));
        let screen = render_screen(&s, 130, 40);
        assert_eq!(tab_rows_shown(&screen)[0], 0);
        assert!(screen.iter().any(|l| l.contains("▌▶ title 0 ")), "{}", screen.join("\n"));
        // Exactly one shown mark, on tab 35 — out of view now, so none drawn.
        assert!(shown_rows(&s).is_empty(), "{:?}", shown_rows(&s));
        // The wheel scrolls the tabs on its own, and no further than the end.
        for _ in 0..100 {
            s.update(Action::ScrollTabs(1));
        }
        let screen = render_screen(&s, 130, 40);
        assert_eq!(*tab_rows_shown(&screen).last().unwrap(), 34);
        s.update(Action::ScrollTabs(-3));
        let screen = render_screen(&s, 130, 40);
        assert_eq!(*tab_rows_shown(&screen).last().unwrap(), 31);
        // Settings, About and Quit are still at the bottom, clickable.
        let quit = at(&screen, Section::Salir.label(Lang::Es));
        assert_eq!(quit, at(&empty, Section::Salir.label(Lang::Es)));
    }

    #[test]
    fn a_short_menu_keeps_one_tab_row_then_falls_back_to_one_scrolled_list() {
        let mut s = with_35_tabs();
        // The shortest height that still pins both ends has one tab row.
        let empty = render_screen(&state(), 130, 40);
        let quit_row = row_of(&empty, Section::Salir.label(Lang::Es));
        let mut pinned = None;
        for height in (10..40).rev() {
            let screen = render_screen(&s, 130, height);
            let rows = tab_rows_shown(&screen);
            if rows.len() == 1 {
                pinned = Some(height);
                assert_eq!(rows, [34], "the tab shown");
                // The first tab shown instead: one tab row, the down chevron right
                // above the rule (no blank row between).
                let mut first = s.clone();
                first.update(Action::GoTab(1));
                let screen = render_screen(&first, 130, height);
                assert_eq!(tab_rows_shown(&screen), [0]);
                let down = icons(first.config.ui.icons).more_below;
                let below = row_of(&screen, down);
                assert!(screen[below + 1].contains("──"), "{}", screen.join("\n"));
                assert!(screen.iter().any(|l| l.contains(Section::Salir.label(Lang::Es))));
                assert!(screen.iter().any(|l| l.contains("YOUTUBE")), "the top stays");
                break;
            }
        }
        let pinned = pinned.expect("a height with one tab row");
        assert!(pinned < 40 && quit_row > 0);
        // One row shorter: one list, scrolled to the cursor (Quit after Up from the first section).
        let screen = render_screen(&s, 130, pinned - 1);
        assert!(tab_rows_shown(&screen).contains(&34), "the tab shown: {}", screen.join("\n"));
        s.update(Action::FocusMenu);
        s.menu_tab = None;
        s.menu = Section::ALL.iter().position(|x| *x == Section::Salir).unwrap();
        let screen = render_screen(&s, 130, pinned - 1);
        assert!(screen.iter().any(|l| l.contains(Section::Salir.label(Lang::Es))), "{}", screen.join("\n"));
    }

    #[test]
    fn tab_rows_have_no_number() {
        let screen = render(&with_tab());
        assert!(screen.contains("│ ▶ title a ") && !screen.contains("1 ▶ title a"), "{screen}");
    }

    #[test]
    fn youtube_is_right_under_the_logo() {
        let screen = render_screen(&state(), 130, 40);
        assert!(screen[0].contains("▶ tubevibe") && screen[1].contains("│ YOUTUBE "), "{}", screen.join("\n"));
        let tubevibe = row_of(&screen, "│ TUBEVIBE ");
        assert!(screen[tubevibe - 1].trim_matches(['│', ' ']).is_empty(), "later groups keep their blank row");
    }

    #[test]
    fn few_tabs_have_no_chevrons_and_no_rule_above() {
        for set in [IconSet::Emoji, IconSet::Nerd, IconSet::Unicode] {
            let mut s = with_tab();
            s.config.ui.icons = set;
            let screen = render_screen(&s, 130, 40);
            let i = icons(set);
            assert!(!screen.iter().any(|l| l.contains(i.more_above) || l.contains(i.more_below)), "{set:?}");
            let title = row_of(&screen, "PESTAÑAS");
            assert!(!screen[title - 1].contains("──"), "{}", screen.join("\n"));
            assert_eq!(
                screen.iter().filter(|l| l.contains("──") && l.starts_with("│ ")).count(),
                1,
                "only above Settings"
            );
        }
    }

    #[test]
    fn x_in_the_menu_closes_the_tab_under_the_cursor() {
        let mut s = with_35_tabs();
        s.update(Action::FocusMenu);
        assert_eq!(s.menu_row(), MenuRow::Tab(34));
        assert!(hint_keys(&s).contains(&"x".to_string()), "x Close Tab on a tab row");
        let x = |s: &State| {
            keymap::map(
                ratatui::crossterm::event::KeyEvent::from(ratatui::crossterm::event::KeyCode::Char('x')),
                s.key_context(),
            )
        };
        let shown = s.tabs[34].id;
        assert_eq!(s.update(x(&s).unwrap()), vec![Effect::Close(shown)]);
        // The shown tab is gone: the previous one is shown, like x in it, and has the cursor.
        assert_eq!((s.tabs.len(), s.active, s.menu_row()), (34, 34, MenuRow::Tab(33)));
        // Not the shown tab: the next one gets the cursor, the shown one stays.
        s.update(Action::Up);
        s.update(Action::Up);
        let closed = s.tabs[31].id;
        assert_eq!(s.update(x(&s).unwrap()), vec![Effect::Close(closed)]);
        assert_eq!((s.tabs.len(), s.active, s.menu_row()), (33, 33, MenuRow::Tab(31)));
        let screen = render_screen(&s, 130, 40);
        assert!(screen.iter().any(|l| l.contains("▌▶ title 32 ")), "{}", screen.join("\n"));
        // Not on a tab: nothing, and no x in the bar.
        s.menu_tab = None;
        assert!(s.update(x(&s).unwrap()).is_empty());
        assert_eq!(s.tabs.len(), 33);
        assert!(!hint_keys(&s).contains(&"x".to_string()));
        // The last tab: the cursor goes back to the sections, the browsing view shows.
        let mut s = with_tab();
        s.update(Action::FocusMenu);
        s.update(x(&s).unwrap());
        assert!(s.tabs.is_empty() && s.active == 0);
        assert!(matches!(s.menu_row(), MenuRow::Section(_)));
        render(&s);
    }

    #[test]
    fn menu_width_min_and_max_and_never_more_than_half() {
        for (width, screen_width, expect) in [(20, 130, 20u16), (40, 130, 40), (26, 130, 26), (40, 60, 30)] {
            let mut s = state();
            s.config.ui.menu_width = width;
            let screen = render_screen(&s, screen_width, 40);
            // The menu's right border, then the content's left one.
            let top: Vec<char> = screen[0].chars().collect();
            assert_eq!((top[usize::from(expect) - 1], top[usize::from(expect)]), ('┐', '┌'), "{width}: {}", screen[0]);
            assert!(screen.iter().any(|l| l.contains("Suscripciones")), "{width} at {screen_width}");
        }
    }

    #[test]
    fn preview_width_min_and_max_keep_the_list_its_columns() {
        let mut s = with_items(Items::Videos(vec![video("a"), video("b")]));
        // (preview width, screen width, beside the list?)
        for (width, screen_width, beside) in [(30, 130, true), (46, 130, true), (70, 130, false), (70, 200, true)] {
            s.config.ui.preview_width = width;
            let screen = render_screen(&s, screen_width, 40);
            let content = screen_areas(Rect::new(0, 0, screen_width, 40), &s).content;
            let area = split_prompt(&s, content).1;
            let (list, preview) = split_preview(area, width);
            let preview = preview.expect("shown, beside or under");
            if beside {
                assert_eq!(preview.width, width);
                assert_eq!(preview.y, list.y);
                assert!(list.width >= LIST_MIN_WIDTH, "the list keeps its columns");
            } else {
                assert_eq!(
                    (preview.width, preview.x),
                    (area.width, area.x),
                    "under, across: {width} at {screen_width}"
                );
                assert_eq!(preview.y, list.bottom());
            }
            let row: Vec<char> = screen[usize::from(preview.y)].chars().collect();
            assert_eq!(row[usize::from(preview.x)], '┌', "{}", screen.join("\n"));
        }
    }

    #[test]
    fn comments_width_beside_up_next_or_under_it() {
        let (mut s, _) = details_tab(Some("Corto."));
        // (columns, screen width, beside?)
        for (columns, screen_width, beside) in
            [(30, 130, true), (46, 130, true), (78, 130, false), (78, 200, true), (46, 120, false)]
        {
            s.config.ui.comments_width = columns;
            let screen = tab_screen(&s, screen_width, 40);
            let inner = Block::bordered().inner(screen_areas(Rect::new(0, 0, screen_width, 40), &s).content);
            let layout = watch_layout(inner, columns);
            let c = layout.comments.expect("room for comments");
            if beside {
                assert_eq!((c.width, c.height), (columns, inner.height), "{columns} at {screen_width}");
                assert!(layout.left.width >= WATCH_LEFT_MIN_WIDTH, "Up next keeps its columns");
            } else {
                assert_eq!((c.x, c.width), (inner.x, inner.width), "under, across: {columns} at {screen_width}");
                assert_eq!(c.y, layout.left.bottom());
                assert!(c.y >= layout.related.bottom(), "below Up Next");
            }
            let row: Vec<char> = screen[usize::from(c.y)].chars().collect();
            assert_eq!(row[usize::from(c.x)], '┌', "{}", screen.join("\n"));
            assert!(screen.iter().any(|l| l.contains("Comentarios")));
        }
    }

    #[test]
    fn every_menu_cursor_row_is_in_the_theme_accent() {
        let gruvbox = crate::theme::parse_colors_toml(
            &std::fs::read_to_string(
                std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/omarchy/gruvbox/colors.toml"),
            )
            .unwrap(),
        )
        .unwrap();
        assert_eq!(gruvbox.accent, Color::Rgb(0x7d, 0xae, 0xa3), "colors.toml's accent");
        assert_eq!(crate::theme::TOKYO_NIGHT.accent, Color::Rgb(0x7a, 0xa2, 0xf7));
        for palette in [crate::theme::TOKYO_NIGHT, gruvbox] {
            let mut s = with_tab();
            s.palette = palette;
            s.update(Action::FocusMenu);
            let rows: Vec<MenuRow> = s.menu_rows();
            for _ in 0..rows.len() {
                s.update(Action::Down);
                let mut terminal = Terminal::new(TestBackend::new(130, 40)).unwrap();
                terminal.draw(|f| draw(f, &s, &VIEW)).unwrap();
                let buffer = terminal.backend().buffer();
                // The cursor's "▌" and its text, in the palette's accent.
                let (x, y) = (0..40u16)
                    .flat_map(|y| (1..25u16).map(move |x| (x, y)))
                    .find(|&(x, y)| buffer[(x, y)].symbol() == "▌")
                    .unwrap_or_else(|| panic!("{:?}", s.menu_row()));
                assert_eq!(buffer[(x, y)].fg, palette.accent, "{:?}", s.menu_row());
                let text = (x + 1..25).find(|&c| buffer[(c, y)].symbol().chars().any(char::is_alphabetic)).unwrap();
                assert_eq!(buffer[(text, y)].fg, palette.accent, "{:?}", s.menu_row());
                assert_eq!(buffer[(text, y)].bg, palette.highlight);
            }
        }
        // The shown mark stays pink.
        let mut s = with_tab();
        s.update(Action::FocusMenu);
        s.update(Action::Down);
        assert_eq!(shown_rows(&s).len(), 1);
    }

    #[test]
    fn a_background_tab_joins_the_scrolled_tab_list() {
        let mut s = with_35_tabs();
        s.update(Action::GoTab(0));
        s.focus = Focus::List;
        let mut more = with_items(Items::Videos(vec![video("new")]));
        std::mem::swap(&mut s.stack, &mut more.stack);
        s.update(Action::OpenInBackground);
        assert_eq!((s.tabs.len(), s.active), (36, 0));
        let up = icons(s.config.ui.icons).more_above;
        let down = icons(s.config.ui.icons).more_below;
        let screen = render_screen(&s, 130, 40);
        assert!(!screen.iter().any(|l| l.contains("▶ title new")), "out of view, below");
        assert!(screen.iter().any(|l| l.contains(down)), "the down chevron says so:\n{}", screen.join("\n"));
        // Walking the menu down to it scrolls it in, the chevron goes.
        s.update(Action::FocusMenu);
        while s.menu_row() != MenuRow::Tab(35) {
            s.update(Action::Down);
        }
        let screen = render_screen(&s, 130, 40);
        assert!(screen.iter().any(|l| l.contains("▌▶ title new")), "{}", screen.join("\n"));
        assert!(!screen.iter().any(|l| l.contains(down)) && screen.iter().any(|l| l.contains(up)));
        s.update(Action::Back);
        assert!(render(&s).contains("title new "), "the list stays in front");
    }

    #[test]
    fn english_hints_are_title_case() {
        assert_eq!(title_case("back to the video"), "Back to the Video");
        assert_eq!(title_case("tubevibe Watch Later"), "tubevibe Watch Later");
        assert_eq!(title_case("the marked ones: trash (asks first)"), "The Marked Ones: Trash (Asks First)");
        assert_eq!(title_case("change / open"), "Change / Open");
    }

    #[test]
    fn up_next_rows_cut_the_title_and_keep_the_length() {
        let text = |spans: Vec<Span>| spans.iter().map(|s| s.content.to_string()).collect::<String>();
        let row = |title: &'static str, w| {
            text(fit_row(vec![Span::raw("head ")], Span::raw(title), vec![Span::raw("  12:34")], w))
        };
        assert_eq!(row("short", 20), "head short     12:34");
        assert_eq!(row("a very long title indeed", 20), "head a very …  12:34");
        let wide = row("日本語のとても長いタイトル", 20);
        assert!(wide.ends_with("…  12:34") || wide.ends_with("…   12:34"), "{wide}");
        assert_eq!(Line::from(wide.clone()).width(), 20, "{wide}");
        assert_eq!(row("title", 8), "…  12:34");
        assert!(row("title", 7).ends_with("  12:34"));
        assert_eq!(row("title", 3), "head title  12:34", "too narrow: left for the screen to clip");
    }

    /// Where `1:00` starts on each screen row that has one, in cells.
    fn length_columns(state: &State, width: u16) -> Vec<usize> {
        let mut terminal = Terminal::new(TestBackend::new(width, 40)).unwrap();
        terminal.draw(|f| draw(f, state, &VIEW)).unwrap();
        let buffer = terminal.backend().buffer();
        let rows: Vec<Vec<&str>> =
            buffer.content.chunks(width as usize).map(|r| r.iter().map(|c| c.symbol()).collect()).collect();
        // List rows: a channel column and a length (the preview has them on separate lines).
        rows.iter()
            .filter(|r| r.concat().contains("chan") || r.concat().contains("A Channel"))
            .filter_map(|r| r.windows(4).position(|w| w.concat() == "1:00"))
            .collect()
    }

    #[test]
    fn list_rows_cut_long_titles_and_line_up_the_length() {
        let mut long = video("long");
        long.title = "A very long title that would never fit in the list at all, not even close, really not".repeat(2);
        let mut wide = video("wide");
        wide.title = "日本語のとても長いタイトル😀😀 ".repeat(8);
        let mut rtl = video("rtl");
        rtl.title = "\u{200F}عنوان طويل جدا باللغة العربية لا يتسع في السطر ".repeat(4);
        let mut chan = video("chan");
        chan.channel = "A Channel Name Much Longer Than Its Column".into();
        let s = with_items(Items::Videos(vec![long, wide, rtl, chan, video("short")]));
        for width in [80, 130, 200] {
            let columns = length_columns(&s, width);
            assert_eq!(columns.len(), 5, "every row shows its length at {width}");
            assert!(columns.iter().all(|c| *c == columns[0]), "lined up at {width}: {columns:?}");
        }
        let screen = render_sized(&s, &VIEW, 130);
        assert!(screen.contains("A Channel Name Much…"), "{screen}");
        assert!(screen.contains("A very long …  1:00"), "{screen}");
    }

    #[test]
    fn watch_later_sits_above_subscriptions() {
        let at = |x| Section::ALL.iter().position(|s| *s == x).unwrap();
        assert_eq!(at(Section::Inicio) + 1, at(Section::VerDespues));
        assert_eq!(at(Section::VerDespues) + 1, at(Section::Suscripciones));
        assert_eq!(at(Section::Suscripciones) + 1, at(Section::Historial));
        let s = state();
        let screen = render(&s);
        let row = |label: &str| screen.lines().position(|l| l.contains(label)).unwrap();
        assert!(row(Section::Inicio.label(Lang::Es)) < row(Section::VerDespues.label(Lang::Es)));
        assert!(row(Section::VerDespues.label(Lang::Es)) < row(Section::Suscripciones.label(Lang::Es)));
        // The icons moved with them.
        let icons = icons(IconSet::Unicode);
        assert_eq!((icons.section(Section::VerDespues), icons.section(Section::Suscripciones)), ("◷", "▣"));
    }

    /// `inner` holds `r` whole, and its border corners are on the screen.
    fn drawn_whole(screen: &[String], r: Rect, inner: Rect) {
        assert!(
            r.x >= inner.x && r.y >= inner.y && r.right() <= inner.right() && r.bottom() <= inner.bottom(),
            "{r:?} in {inner:?}"
        );
        let cell = |x: u16, y: u16| screen[usize::from(y)].chars().nth(usize::from(x)).unwrap();
        assert_eq!(cell(r.x, r.y), '┌', "{r:?}\n{}", screen.join("\n"));
        assert_eq!(cell(r.right() - 1, r.bottom() - 1), '┘', "{r:?}\n{}", screen.join("\n"));
    }

    #[test]
    fn list_and_preview_side_by_side_when_wide_stacked_when_narrow() {
        let s = with_items(Items::Videos(vec![video("a"), video("b")]));
        for (w, h, beside) in [(80, 30, false), (120, 40, false), (200, 50, true)] {
            let screen = render_screen(&s, w, h);
            let area = split_prompt(&s, screen_areas(Rect::new(0, 0, w, h), &s).content).1;
            let (list, preview) = split_preview(area, s.config.ui.preview_width);
            let preview = preview.expect("shown");
            assert_eq!(preview.x > list.x, beside, "{w}x{h}");
            assert!(list.intersection(preview).is_empty(), "no overlap at {w}x{h}");
            if !beside {
                assert!(list.height >= LIST_MIN_ROWS && preview.width == area.width, "{w}x{h}");
            }
            drawn_whole(&screen, list, area);
            drawn_whole(&screen, preview, area);
            assert!(thumbnail_cells(preview).bottom() <= preview.bottom());
        }
        // Too short to stack: the list alone, never squeezed together.
        let area = Rect::new(0, 0, 60, 14);
        assert_eq!(split_preview(area, 46), (area, None));
        for (w, h) in [(1, 1), (20, 5), (40, 10)] {
            render_screen(&s, w, h);
        }
    }

    #[test]
    fn comments_beside_when_wide_under_up_next_when_narrow() {
        let (s, _) = details_tab(Some("Corto."));
        for (w, h, beside) in [(80, 30, false), (120, 40, false), (200, 50, true)] {
            let screen = tab_screen(&s, w, h);
            let inner = Block::bordered().inner(screen_areas(Rect::new(0, 0, w, h), &s).content);
            let layout = tab_layout(s.active_tab().unwrap(), inner, s.config.ui.comments_width);
            let comments = layout.comments.expect("shown");
            assert_eq!(comments.x > inner.x, beside, "{w}x{h}");
            for pane in [layout.related, layout.details, layout.thumbnail, layout.summary] {
                assert!(pane.intersection(comments).is_empty(), "no overlap at {w}x{h}");
            }
            assert!(layout.related.height >= UP_NEXT_MIN_ROWS, "{w}x{h}");
            drawn_whole(&screen, comments, inner);
            drawn_whole(&screen, layout.related, inner);
        }
        for (w, h) in [(1, 1), (30, 8), (60, 15)] {
            tab_screen(&s, w, h);
        }
    }
}
