//! Colours: Tokyo Night by default, or the current Omarchy theme
//! ("Omarchy colour sync", `[ui] omarchy_theme`).
//!
//! Omarchy keeps the theme in use in a folder: `~/.local/state/omarchy/current/theme`
//! (Omarchy 4), `~/.config/omarchy/current/theme` before that (a symlink to the
//! theme in Omarchy 2, a copy in 3). Its `colors.toml` names the palette
//! (`foreground`, `red`, `selection`… in 4; `color0`–`color15` in 3); themes
//! older than that only carry `alacritty.toml`. `omarchy theme set` swaps the
//! folder, so [`Watcher`] notices a new file (path, inode, size or time) and
//! reads it again.
//!
//! The interface is drawn with the Tokyo Night colours below and [`recolor`]
//! swaps them for the theme's in the finished frame, so drawing code keeps its
//! named colours and a theme is one table.

use std::path::{Path, PathBuf};
use std::time::SystemTime;

use ratatui::buffer::Buffer;
use ratatui::style::Color;
use toml::Value;

/// The colours the interface uses, by role.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Palette {
    /// The terminal background: animations fade in from it, and text on a
    /// coloured button uses it. Elsewhere the terminal's own background shows.
    pub background: Color,
    /// Text.
    pub fg: Color,
    /// Secondary text: hints, dates, labels.
    pub muted: Color,
    /// Background of the selected row.
    pub highlight: Color,
    /// Separator lines in dialogs.
    pub line: Color,
    /// Animations' flash.
    pub accent: Color,
    pub blue: Color,
    pub cyan: Color,
    pub magenta: Color,
    pub green: Color,
    pub orange: Color,
    pub yellow: Color,
    pub red: Color,
    pub teal: Color,
    pub purple: Color,
}

const fn rgb(hex: u32) -> Color {
    Color::Rgb((hex >> 16) as u8, (hex >> 8) as u8, hex as u8)
}

/// The built-in colours (Tokyo Night): what the interface is drawn with.
pub const TOKYO_NIGHT: Palette = Palette {
    background: rgb(0x1a1b26),
    fg: rgb(0xc0caf5),
    muted: rgb(0x565f89),
    highlight: rgb(0x292e42),
    line: rgb(0x3b4261),
    accent: rgb(0x7aa2f7),
    blue: rgb(0x7aa2f7),
    cyan: rgb(0x7dcfff),
    magenta: rgb(0xbb9af7),
    green: rgb(0x9ece6a),
    orange: rgb(0xff9e64),
    yellow: rgb(0xe0af68),
    red: rgb(0xf7768e),
    teal: rgb(0x73daca),
    purple: rgb(0x9d7cd8),
};

/// What the interface draws the accent with: next to Tokyo Night's blue,
/// which is its accent too, so [`recolor`] can tell them apart.
pub const ACCENT_DRAWN: Color = Color::Rgb(0x7a, 0xa2, 0xf6);

impl Default for Palette {
    fn default() -> Self {
        TOKYO_NIGHT
    }
}

impl Palette {
    /// `(drawn with, shown as)` for every colour of the interface.
    fn swaps(&self) -> [(Color, Color); 15] {
        let t = TOKYO_NIGHT;
        [
            (ACCENT_DRAWN, self.accent),
            (t.background, self.background),
            (t.fg, self.fg),
            (t.muted, self.muted),
            (t.highlight, self.highlight),
            (t.line, self.line),
            (t.blue, self.blue),
            (t.cyan, self.cyan),
            (t.magenta, self.magenta),
            (t.green, self.green),
            (t.orange, self.orange),
            (t.yellow, self.yellow),
            (t.red, self.red),
            (t.teal, self.teal),
            (t.purple, self.purple),
        ]
    }
}

/// Shows a frame drawn in Tokyo Night in `palette` instead. Colours that are
/// not part of the palette (YouTube red, white, the terminal's own) stay.
pub fn recolor(buffer: &mut Buffer, palette: &Palette) {
    // Tokyo Night too: the accent is drawn with a stand-in.
    let swaps = palette.swaps();
    let swap = |color: &mut Color| {
        if let Some((_, to)) = swaps.iter().find(|(from, _)| from == color) {
            *color = *to;
        }
    };
    for cell in &mut buffer.content {
        swap(&mut cell.fg);
        swap(&mut cell.bg);
    }
}

/// `#rrggbb`, `0xrrggbb` (alacritty) or `rrggbb`.
fn hex(text: &str) -> Option<(u8, u8, u8)> {
    let digits = text.trim().trim_start_matches('#').trim_start_matches("0x").trim_start_matches("0X");
    if digits.len() != 6 {
        return None;
    }
    let n = u32::from_str_radix(digits, 16).ok()?;
    Some(((n >> 16) as u8, (n >> 8) as u8, n as u8))
}

fn color((r, g, b): (u8, u8, u8)) -> Color {
    Color::Rgb(r, g, b)
}

/// Relative luminance (WCAG).
fn luminance((r, g, b): (u8, u8, u8)) -> f64 {
    let channel = |c: u8| {
        let c = f64::from(c) / 255.0;
        if c <= 0.03928 { c / 12.92 } else { ((c + 0.055) / 1.055).powf(2.4) }
    };
    0.2126 * channel(r) + 0.7152 * channel(g) + 0.0722 * channel(b)
}

fn contrast(a: (u8, u8, u8), b: (u8, u8, u8)) -> f64 {
    let (la, lb) = (luminance(a), luminance(b));
    (la.max(lb) + 0.05) / (la.min(lb) + 0.05)
}

/// The named colours a theme file gives, read into roles.
#[derive(Default)]
struct Named {
    background: Option<(u8, u8, u8)>,
    fg: Option<(u8, u8, u8)>,
    muted: Option<(u8, u8, u8)>,
    selection: Option<(u8, u8, u8)>,
    /// Omarchy 4's `lighter_background`.
    lighter: Option<(u8, u8, u8)>,
    line: Option<(u8, u8, u8)>,
    accent: Option<(u8, u8, u8)>,
    /// black red green yellow blue magenta cyan white.
    normal: [Option<(u8, u8, u8)>; 8],
    bright: [Option<(u8, u8, u8)>; 8],
    orange: Option<(u8, u8, u8)>,
}

const RED: usize = 1;
const GREEN: usize = 2;
const YELLOW: usize = 3;
const BLUE: usize = 4;
const MAGENTA: usize = 5;
const CYAN: usize = 6;

impl Named {
    /// The palette, with what is missing worked out from what is there and
    /// secondary text kept readable on the background.
    fn palette(self) -> Option<Palette> {
        let background = self.background.or(self.normal[0])?;
        let fg = self.fg.or(self.normal[7])?;
        let pick = |i: usize| self.normal[i].or(self.bright[i]);
        let bright = |i: usize| self.bright[i].or(self.normal[i]);
        let red = pick(RED)?;
        let green = pick(GREEN)?;
        let yellow = pick(YELLOW)?;
        let blue = pick(BLUE)?;
        let magenta = pick(MAGENTA)?;
        let cyan = pick(CYAN)?;
        // Only colours the theme has, never a blend: the first that fits.
        let first = |candidates: &[Option<(u8, u8, u8)>], fits: &dyn Fn(&(u8, u8, u8)) -> bool| {
            candidates.iter().flatten().copied().find(|c| fits(c))
        };
        let blacks = [self.bright[0], self.normal[0]];
        let readable = |c: &(u8, u8, u8)| contrast(*c, background) >= 2.0;
        // Secondary text: the theme's dim text, its muted, a readable black, else the text.
        let muted = first(&[self.muted, self.line, blacks[0], blacks[1]], &readable).unwrap_or(fg);
        let near = |c: &(u8, u8, u8)| *c != background && contrast(*c, background) < 2.0 && contrast(*c, fg) >= 3.0;
        // The selected row: the selection, the lighter background, a black near the background.
        let highlight = first(&[self.selection, self.lighter, blacks[0], blacks[1]], &near)
            .or(first(&[self.lighter, blacks[0], blacks[1]], &|c| *c != background))
            .unwrap_or(background);
        let line = first(&[self.line, blacks[0], blacks[1]], &|c| *c != background).unwrap_or(muted);
        Some(Palette {
            background: color(background),
            fg: color(fg),
            muted: color(muted),
            highlight: color(highlight),
            line: color(line),
            accent: color(self.accent.unwrap_or(blue)),
            blue: color(blue),
            cyan: color(cyan),
            magenta: color(bright(MAGENTA).unwrap_or(magenta)),
            green: color(green),
            orange: color(self.orange.unwrap_or(yellow)),
            yellow: color(yellow),
            red: color(red),
            teal: color(bright(CYAN).unwrap_or(cyan)),
            purple: color(magenta),
        })
    }
}

const NAMES: [&str; 8] = ["black", "red", "green", "yellow", "blue", "magenta", "cyan", "white"];

/// An Omarchy `colors.toml`: the named palette of Omarchy 4 (`foreground`,
/// `dark_foreground`, `selection`, `red`, `bright_red`…) or the terminal one of
/// Omarchy 3 (`color0`–`color15`, `selection_background`).
pub fn parse_colors_toml(text: &str) -> Option<Palette> {
    let table: toml::Table = text.parse().ok()?;
    let get = |key: &str| table.get(key).and_then(Value::as_str).and_then(hex);
    let mut named = Named {
        background: get("background"),
        fg: get("bright_foreground").or(get("foreground")),
        muted: get("dark_foreground"),
        selection: get("selection").or(get("selection_background")),
        lighter: get("lighter_background"),
        line: get("muted"),
        accent: get("accent"),
        orange: get("orange"),
        ..Named::default()
    };
    for (i, name) in NAMES.iter().enumerate() {
        named.normal[i] = get(name).or(get(&format!("color{i}")));
        named.bright[i] = get(&format!("bright_{name}")).or(get(&format!("color{}", i + 8)));
    }
    named.palette()
}

/// An alacritty theme: `[colors.primary]`, `[colors.normal]`, `[colors.bright]`
/// and `[colors.selection]` (the only file of Omarchy 2 themes).
pub fn parse_alacritty(text: &str) -> Option<Palette> {
    let table: toml::Table = text.parse().ok()?;
    let colors = table.get("colors")?;
    let get = |group: &str, key: &str| colors.get(group)?.get(key)?.as_str().and_then(hex);
    let mut named = Named {
        background: get("primary", "background"),
        fg: get("primary", "foreground"),
        muted: get("primary", "dim_foreground"),
        selection: get("selection", "background"),
        ..Named::default()
    };
    for (i, name) in NAMES.iter().enumerate() {
        named.normal[i] = get("normal", name);
        named.bright[i] = get("bright", name);
    }
    named.palette()
}

/// Where Omarchy keeps the current theme, newest layout first.
pub fn theme_dirs(home: &Path, state_home: Option<&Path>) -> Vec<PathBuf> {
    let state = state_home.map_or_else(|| home.join(".local/state"), Path::to_path_buf);
    vec![state.join("omarchy/current/theme"), home.join(".config/omarchy/current/theme")]
}

/// The theme's palette file: `colors.toml`, else `alacritty.toml`, in the
/// first theme folder that has one.
pub fn theme_file(dirs: &[PathBuf]) -> Option<PathBuf> {
    dirs.iter().find_map(|dir| ["colors.toml", "alacritty.toml"].iter().map(|f| dir.join(f)).find(|f| f.is_file()))
}

/// The palette in a theme file (`None`: unreadable, or not a palette).
pub fn read(file: &Path) -> Option<Palette> {
    let text = std::fs::read_to_string(file).ok()?;
    if file.file_name().is_some_and(|n| n == "alacritty.toml") {
        parse_alacritty(&text)
    } else {
        parse_colors_toml(&text)
    }
}

/// What tells one theme file from another without reading it.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Stamp {
    file: PathBuf,
    modified: Option<SystemTime>,
    len: u64,
    inode: u64,
}

fn stamp(file: &Path) -> Option<Stamp> {
    let real = std::fs::canonicalize(file).ok()?;
    let meta = std::fs::metadata(&real).ok()?;
    #[cfg(unix)]
    let inode = std::os::unix::fs::MetadataExt::ino(&meta);
    #[cfg(not(unix))]
    let inode = 0;
    Some(Stamp { file: real, modified: meta.modified().ok(), len: meta.len(), inode })
}

/// Follows the current Omarchy theme: [`Watcher::poll`] answers the palette
/// to show when it changed since the last call.
pub struct Watcher {
    dirs: Vec<PathBuf>,
    /// The file seen last time (`Some(None)`: none was found).
    last: Option<Option<Stamp>>,
}

impl Watcher {
    pub fn new(dirs: Vec<PathBuf>) -> Self {
        Self { dirs, last: None }
    }

    /// The user's Omarchy folders.
    pub fn system() -> Option<Self> {
        let home = std::env::var_os("HOME").filter(|h| !h.is_empty()).map(PathBuf::from)?;
        let state = std::env::var_os("XDG_STATE_HOME").filter(|s| !s.is_empty()).map(PathBuf::from);
        Some(Self::new(theme_dirs(&home, state.as_deref())))
    }

    /// Forget what was seen: the next poll answers in any case.
    pub fn reset(&mut self) {
        self.last = None;
    }

    /// The palette to show, when the theme file changed (or on the first call):
    /// the theme's, or Tokyo Night when there is no Omarchy theme or it can't be
    /// read.
    pub fn poll(&mut self) -> Option<Palette> {
        let file = theme_file(&self.dirs);
        let now = file.as_deref().and_then(stamp);
        if self.last.as_ref() == Some(&now) {
            return None;
        }
        self.last = Some(now);
        Some(file.as_deref().and_then(read).unwrap_or(TOKYO_NIGHT))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture(theme: &str, file: &str) -> String {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/omarchy").join(theme).join(file);
        std::fs::read_to_string(path).unwrap()
    }

    #[test]
    fn gruvbox_from_omarchy_4() {
        let p = parse_colors_toml(&fixture("gruvbox", "colors.toml")).unwrap();
        assert_eq!(p.background, rgb(0x282828));
        assert_eq!(p.fg, rgb(0xd4be98));
        assert_eq!(p.muted, rgb(0x7c6f64), "dark_foreground");
        assert_eq!(p.highlight, rgb(0x504945), "selection");
        assert_eq!(p.line, rgb(0x665c54), "muted");
        assert_eq!(p.accent, rgb(0x7daea3));
        assert_eq!((p.red, p.green, p.yellow), (rgb(0xea6962), rgb(0xa9b665), rgb(0xd8a657)));
        assert_eq!((p.blue, p.cyan, p.magenta), (rgb(0x7daea3), rgb(0x89b482), rgb(0xd3869b)));
        assert_eq!(p.orange, rgb(0xe1875c));
    }

    #[test]
    fn tokyo_night_theme_is_close_to_the_built_in_one() {
        let p = parse_colors_toml(&fixture("tokyo-night", "colors.toml")).unwrap();
        assert_eq!(p.fg, TOKYO_NIGHT.fg);
        assert_eq!(p.muted, TOKYO_NIGHT.muted);
        assert_eq!(p.highlight, TOKYO_NIGHT.highlight);
        assert_eq!(p.background, TOKYO_NIGHT.background);
        assert_eq!(p.magenta, TOKYO_NIGHT.magenta);
    }

    #[test]
    fn light_themes_stay_readable() {
        for theme in ["catppuccin-latte", "rose-pine"] {
            let p = parse_colors_toml(&fixture(theme, "colors.toml")).unwrap();
            let Color::Rgb(r, g, b) = p.background else { panic!() };
            assert!(luminance((r, g, b)) > 0.5, "{theme} is light");
            let c = |x: Color| match x {
                Color::Rgb(r, g, b) => (r, g, b),
                _ => panic!(),
            };
            assert!(contrast(c(p.fg), c(p.background)) > 4.5, "{theme} text");
            assert!(contrast(c(p.muted), c(p.background)) >= 2.0, "{theme} muted");
            assert!(contrast(c(p.fg), c(p.highlight)) >= 3.0, "{theme} selected row");
        }
        let latte = parse_colors_toml(&fixture("catppuccin-latte", "colors.toml")).unwrap();
        assert_eq!(latte.fg, rgb(0x4c4f69));
        assert_eq!(latte.highlight, rgb(0xccd0da));
    }

    #[test]
    fn older_omarchy_files_work_too() {
        // Omarchy 3: color0–color15; its selection is orange, too loud for a row.
        let v3 = parse_colors_toml(&fixture("gruvbox-v3", "colors.toml")).unwrap();
        assert_eq!((v3.background, v3.fg, v3.red), (rgb(0x282828), rgb(0xd4be98), rgb(0xea6962)));
        assert_ne!(v3.highlight, rgb(0xd65d0e));
        assert_ne!(v3.muted, rgb(0x3c3836), "color8 is too dark to read on gruvbox");
        // Omarchy 2: only alacritty.toml.
        let v2 = parse_alacritty(&fixture("gruvbox-v2", "alacritty.toml")).unwrap();
        assert_eq!((v2.background, v2.fg, v2.green), (rgb(0x282828), rgb(0xd4be98), rgb(0xa9b665)));
        assert_eq!((v2.highlight, v2.muted), (v3.highlight, v3.muted), "same theme, same result");
        let latte = parse_alacritty(&fixture("catppuccin-latte-v2", "alacritty.toml")).unwrap();
        assert_eq!((latte.background, latte.fg), (rgb(0xeff1f5), rgb(0x4c4f69)));
        assert_eq!(latte.muted, rgb(0x8c8fa1), "dim_foreground");
    }

    #[test]
    fn junk_is_no_palette() {
        assert_eq!(parse_colors_toml("not toml ["), None);
        assert_eq!(parse_colors_toml("background = \"#000000\""), None, "too little to go on");
        assert_eq!(parse_alacritty("[font]\nsize = 9"), None);
        assert_eq!(hex("0x282828"), Some((0x28, 0x28, 0x28)));
        assert_eq!(hex("#28282"), None);
    }

    #[test]
    fn the_frame_is_shown_in_the_theme() {
        use ratatui::layout::Rect;
        let gruvbox = parse_colors_toml(&fixture("gruvbox", "colors.toml")).unwrap();
        let mut buffer = Buffer::empty(Rect::new(0, 0, 3, 1));
        buffer.content[0].fg = TOKYO_NIGHT.fg;
        buffer.content[0].bg = TOKYO_NIGHT.highlight;
        buffer.content[1].fg = Color::Rgb(255, 0, 51);
        buffer.content[2].fg = TOKYO_NIGHT.green;
        let mut shown = buffer.clone();
        recolor(&mut shown, &TOKYO_NIGHT);
        assert_eq!(shown, buffer, "Tokyo Night: nothing to swap");
        recolor(&mut shown, &gruvbox);
        assert_eq!((shown.content[0].fg, shown.content[0].bg), (gruvbox.fg, gruvbox.highlight));
        assert_eq!(shown.content[1].fg, Color::Rgb(255, 0, 51), "YouTube red stays");
        assert_eq!(shown.content[2].fg, gruvbox.green);
        assert_eq!(shown.content[0].fg, Color::Rgb(0xd4, 0xbe, 0x98));
    }

    #[test]
    fn the_watcher_follows_theme_switches() {
        let root = std::env::temp_dir().join(format!("tubevibe-theme-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let themes = root.join("themes");
        for theme in ["gruvbox", "rose-pine"] {
            std::fs::create_dir_all(themes.join(theme)).unwrap();
            std::fs::write(themes.join(theme).join("colors.toml"), fixture(theme, "colors.toml")).unwrap();
        }
        let home = root.join("home");
        let dirs = theme_dirs(&home, None);
        assert_eq!(dirs[0], home.join(".local/state/omarchy/current/theme"));
        let mut watcher = Watcher::new(dirs);
        assert_eq!(watcher.poll(), Some(TOKYO_NIGHT), "not on Omarchy");
        assert_eq!(watcher.poll(), None, "nothing changed");
        // Omarchy 2's layout: ~/.config/omarchy/current/theme → the theme folder.
        let legacy = home.join(".config/omarchy/current");
        std::fs::create_dir_all(&legacy).unwrap();
        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(themes.join("gruvbox"), legacy.join("theme")).unwrap();
            assert_eq!(watcher.poll().map(|p| p.background), Some(rgb(0x282828)));
            assert_eq!(watcher.poll(), None);
            std::fs::remove_file(legacy.join("theme")).unwrap();
            std::os::unix::fs::symlink(themes.join("rose-pine"), legacy.join("theme")).unwrap();
            assert_eq!(watcher.poll().map(|p| p.background), Some(rgb(0xfaf4ed)), "switched");
        }
        // Omarchy 4's folder wins; a broken file falls back to Tokyo Night.
        let current = home.join(".local/state/omarchy/current/theme");
        std::fs::create_dir_all(&current).unwrap();
        std::fs::write(current.join("colors.toml"), "broken = [").unwrap();
        assert_eq!(watcher.poll(), Some(TOKYO_NIGHT));
        watcher.reset();
        assert_eq!(watcher.poll(), Some(TOKYO_NIGHT), "reset answers again");
        let _ = std::fs::remove_dir_all(&root);
    }

    const FIXTURES: [(&str, &str); 7] = [
        ("gruvbox", "colors.toml"),
        ("gruvbox-v3", "colors.toml"),
        ("gruvbox-v2", "alacritty.toml"),
        ("catppuccin-latte", "colors.toml"),
        ("catppuccin-latte-v2", "alacritty.toml"),
        ("rose-pine", "colors.toml"),
        ("tokyo-night", "colors.toml"),
    ];

    fn load(theme: &str, file: &str) -> Palette {
        let text = fixture(theme, file);
        if file == "alacritty.toml" { parse_alacritty(&text) } else { parse_colors_toml(&text) }.unwrap()
    }

    #[test]
    fn every_colour_is_one_the_theme_file_has() {
        for (theme, file) in FIXTURES {
            let text = fixture(theme, file).to_lowercase();
            let p = load(theme, file);
            for (_, c) in p.swaps() {
                let Color::Rgb(r, g, b) = c else { panic!() };
                let hex = format!("{r:02x}{g:02x}{b:02x}");
                assert!(text.contains(&hex), "{theme}: {hex} is not in {file} (a blend?)");
            }
        }
        // No orange: the theme's yellow, not red and yellow mixed.
        let v3 = load("gruvbox-v3", "colors.toml");
        assert_eq!(v3.orange, v3.yellow);
    }

    #[test]
    fn the_accent_reads_on_the_selected_row() {
        let c = |x: Color| match x {
            Color::Rgb(r, g, b) => (r, g, b),
            _ => panic!(),
        };
        let mut report = Vec::new();
        for (theme, file) in FIXTURES {
            let p = load(theme, file);
            let ratio = contrast(c(p.accent), c(p.highlight));
            report.push(format!("{theme}: {ratio:.2}"));
            // Light themes are the weak ones: Rosé Pine's own accent on its own
            // selection is 2.47, Catppuccin Latte from alacritty.toml (no accent, so
            // blue, on its bright black) 2.27. The row is bold and marked with ▌.
            assert!(ratio >= 2.2, "{theme}: accent on the selected row {ratio:.2}");
        }
        let built_in = contrast(c(TOKYO_NIGHT.accent), c(TOKYO_NIGHT.highlight));
        assert!(built_in >= 4.5, "{built_in}");
        println!("{}", report.join("\n"));
    }
}
