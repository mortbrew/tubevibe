//! Keys → actions. Global commands live behind the `Ctrl+Y` prefix
//! (tmux style: the prefix waits for exactly one more key); plain keys go to
//! whatever has focus, so the video tab can use space and arrows freely.
//! The help screen is generated from the same tables.

use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

use crate::app::Action;
use crate::i18n::{Lang, Text};
use crate::youtube::Rating;

/// The slice of state that decides how a key is read.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct KeyContext {
    /// `Ctrl+Y` was pressed and the next key is a command.
    pub prefix: bool,
    pub help: bool,
    /// A yes/no question is open.
    pub confirm: bool,
    /// The search box has focus.
    pub typing: bool,
    /// A video tab is active.
    pub in_tab: bool,
    /// ... and its related list has the keys.
    pub related: bool,
    /// The right-click menu is open.
    pub context_menu: bool,
    /// The opening logo is shown: any key skips it.
    pub splash: bool,
    /// The left menu has the keys (browsing view, no search box).
    pub menu: bool,
}

pub const SEEK_SECONDS: i32 = 5;
pub const VOLUME_STEP: i32 = 5;

/// `Ctrl+Y` commands. Digits (`0`–`9`, go to tab) are handled apart.
pub const PREFIX: &[(char, Action, Text)] = &[
    ('b', Action::ToggleSidebar, Text::new("mostrar / ocultar el menú", "show / hide the menu")),
    ('m', Action::FocusMenu, Text::new("ir al menú", "go to the menu")),
    ('f', Action::ToggleFullscreen, Text::new("pantalla completa", "fullscreen")),
    (
        'x',
        Action::CloseTab,
        Text::new(
            "cerrar la pestaña · en listas: quitar el video",
            "close tab · in Watch Later lists/playlists: remove video",
        ),
    ),
    ('/', Action::FocusSearch, Text::new("buscar videos", "search videos")),
    ('l', Action::ToggleUiLang, Text::new("idioma: pasar a English", "language: switch to español")),
    ('?', Action::ToggleHelp, Text::new("esta ayuda", "this help")),
    ('q', Action::Quit, Text::new("salir", "quit")),
];

/// Keys that act on the selected video, in lists and in video tabs.
pub const ON_VIDEO: &[(&str, Text)] = &[
    ("w", Text::new("a Ver después de YouTube (ya está: lo dice)", "save to YouTube Watch Later (if there: says so)")),
    (
        "l",
        Text::new("a Ver después de tubevibe (ya está: lo dice)", "save to tubevibe Watch Later (if there: says so)"),
    ),
    ("s", Text::new("suscribirse / desuscribirse (pide confirmar)", "subscribe / unsubscribe (asks first)")),
    ("g", Text::new("me gusta (otra vez: lo quita)", "like (again: take it back)")),
    ("G", Text::new("no me gusta (otra vez: lo quita)", "dislike (again: take it back)")),
    ("d", Text::new("descargar (con subtítulos)", "download (with subtitles)")),
    ("a", Text::new("escuchar solo el audio / volver al video", "audio only / back to the video")),
    ("c", Text::new("ver los videos del canal", "the channel's videos")),
];

pub const IN_MENU: &[(&str, Text)] = &[
    ("↑ ↓", Text::new("elegir sección o pestaña", "choose a section or tab")),
    ("Enter", Text::new("abrir", "open")),
    ("Tab", Text::new("ir a la lista", "go to the list")),
    ("/", Text::new("buscar videos", "search videos")),
];

pub const IN_LISTS: &[(&str, Text)] = &[
    ("↑ ↓  j k", Text::new("moverse", "move")),
    ("Enter", Text::new("abrir video · abrir canal · cambiar opción", "open video / channel · change option")),
    ("Esc  ←  h", Text::new("volver (también Retroceso)", "back (Backspace too)")),
    ("Tab", Text::new("ir al menú", "go to the menu")),
    ("/", Text::new("buscar (dentro de un canal: en ese canal)", "search (inside a channel: in that channel)")),
    ("v", Text::new("mostrar / ocultar los ya vistos (Inicio…)", "show / hide watched videos (My Recommendations…)")),
    ("Espacio", Text::new("marcar varios (videos, música, descargas)", "mark several (videos, music, downloads)")),
    ("f", Text::new("pantalla completa del video que se reproduce", "fullscreen the video playing")),
    (
        "x  Supr",
        Text::new("a la papelera (pide s) · Historial: quitarlo", "to the trash (asks y) · History: remove it"),
    ),
];

pub const WITH_MOUSE: &[(&str, Text)] = &[
    ("Clic", Text::new("elegir · en el menú y las pestañas: abrir", "select · menu and tabs: open")),
    ("Doble clic", Text::new("abrir video · abrir canal · cambiar opción", "open video / channel · change option")),
    ("Rueda", Text::new("moverse en la lista · en el video: volumen", "move in the list · on the video: volume")),
    ("Clic video", Text::new("pausa (doble clic: pantalla completa)", "pause (double click: fullscreen)")),
    ("Clic der.", Text::new("opciones de lo que está debajo", "options for what is under it")),
    ("Clic barra", Text::new("saltar a ese momento del video", "jump to that point of the video")),
    ("Shift", Text::new("mantenela para seleccionar y copiar texto", "hold it to select and copy text")),
];

pub const IN_SEARCH: &[(&str, Text)] = &[
    ("Enter", Text::new("buscar / aceptar", "search / accept")),
    ("Tab", Text::new("dónde buscar: todo / mis canales / canal", "where: everywhere / my channels / channel")),
    ("Esc", Text::new("cancelar", "cancel")),
];

pub const IN_TAB: &[(&str, Text)] = &[
    ("Espacio", Text::new("pausa", "pause")),
    ("Enter", Text::new("reproducir · en A continuación: ese video", "play · in Up Next: the selected video")),
    ("v", Text::new("calidad: 360p → 480p → 720p → 1080p …", "quality: 360p → 480p → 720p → 1080p …")),
    ("b", Text::new("compresión: H.264 → VP9 → AV1 → cualquiera", "codec: H.264 → VP9 → AV1 → any")),
    ("f  Esc", Text::new("pantalla completa / salir", "fullscreen / leave")),
    ("←  Esc", Text::new("ir al menú", "go to the menu")),
    ("Shift+← →", Text::new("retroceder / avanzar 5 s (o solo →)", "back / forward 5 s (or → alone)")),
    ("[ ]", Text::new("capítulo anterior / siguiente", "previous / next chapter")),
    ("↑ ↓  m", Text::new("subir / bajar volumen · silenciar", "volume up / down · mute")),
    ("j", Text::new("subtítulos sí / no", "subtitles on / off")),
    ("i", Text::new("modo: subtítulos → original → doblado", "mode: subtitles → original → dubbed")),
    ("o", Text::new("otro idioma (es → en → pt …)", "another language (es → en → pt …)")),
    (". ,", Text::new("música: siguiente / anterior", "music: next / previous")),
    ("Tab", Text::new("ir a A continuación (en modo ventana)", "go to Up Next (window mode)")),
    ("j k", Text::new("moverse en A continuación", "move in Up Next")),
    ("Esc", Text::new("en A continuación: volver al video", "in Up Next: back to the video")),
    ("PgUp PgDn", Text::new("recorrer los comentarios (en ventana)", "scroll the comments (in a window)")),
];

/// `(keys, description)` rows of a table, in `lang`.
pub fn rows(table: &[(&str, Text)], lang: Lang) -> Vec<(String, &'static str)> {
    table.iter().map(|(k, d)| (key_name(k, lang), d.get(lang))).collect()
}

/// Keys and mouse actions with a name of their own, in Spanish and English.
const KEY_NAMES: &[(&str, &str)] = &[
    ("Espacio", "Space"),
    ("x  Supr", "x  Del"),
    ("Clic", "Click"),
    ("Doble clic", "Double Click"),
    ("Rueda", "Wheel"),
    ("Clic video", "Click Video"),
    ("Clic der.", "Right Click"),
    ("Clic barra", "Click Bar"),
];

/// The space bar's name, in `lang`.
pub fn space(lang: Lang) -> &'static str {
    lang.pick("Espacio", "Space")
}

/// A key's name in the interface language (the tables name them in Spanish).
pub fn key_name(keys: &str, lang: Lang) -> String {
    match KEY_NAMES.iter().find(|(es, _)| *es == keys) {
        Some((es, en)) => lang.pick(*es, *en).into(),
        None => keys.into(),
    }
}

/// `Ctrl+Y` commands plus the digits, in `lang`.
pub fn prefix_rows(lang: Lang) -> Vec<(String, &'static str)> {
    let mut commands: Vec<_> = PREFIX.iter().map(|(k, _, d)| (k.to_string(), d.get(lang))).collect();
    commands.insert(
        4,
        (
            "0-9".into(),
            lang.pick(
                "ir a la pestaña del menú (0 = volver a navegar)",
                "go to a tab in the menu (0 = back to browsing)",
            ),
        ),
    );
    commands
}

/// What macOS types for Option+, and Option+. (US and most layouts).
fn mac_option_tab(c: char) -> Option<Action> {
    match c {
        '≤' => Some(Action::PrevTab),
        '≥' => Some(Action::NextTab),
        _ => None,
    }
}

pub fn map(key: KeyEvent, ctx: KeyContext) -> Option<Action> {
    // Only presses: releases (Windows, or a terminal that reports them) and
    // repeats as their own kind never count twice.
    if key.kind != KeyEventKind::Press {
        return None;
    }
    let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
    if ctrl && key.code == KeyCode::Char('c') {
        return Some(Action::Quit);
    }
    if ctx.splash {
        return Some(Action::EndSplash);
    }
    if ctx.help {
        return Some(Action::ToggleHelp);
    }
    if ctx.confirm {
        return Some(Action::Confirm(matches!(key.code, KeyCode::Char('s' | 'S' | 'y' | 'Y'))));
    }
    if ctx.context_menu {
        // Like the yes/no question: any other key closes it.
        return Some(match key.code {
            KeyCode::Up | KeyCode::Char('k') => Action::Up,
            KeyCode::Down | KeyCode::Char('j') => Action::Down,
            KeyCode::Enter | KeyCode::Char(' ') => Action::Enter,
            _ => Action::Back,
        });
    }
    // Alt+, / Alt+.: previous / next tab, one hand, from anywhere (lists, the
    // search box, a video tab). Every terminal sends them as plain Esc + key,
    // so nothing has to be set up, and no terminal, nor Omarchy, binds them
    // by default. Ctrl+Alt (AltGr on some layouts) is left alone.
    if key.modifiers.contains(KeyModifiers::ALT) && !ctrl {
        match key.code {
            KeyCode::Char(',' | '<') => return Some(Action::PrevTab),
            KeyCode::Char('.' | '>') => return Some(Action::NextTab),
            _ => {}
        }
    }
    // macOS terminals type Option+, and Option+. as ≤ and ≥ unless told to
    // treat Option as Alt: those count too, so no setting is needed there.
    if cfg!(target_os = "macos")
        && !ctrl
        && let KeyCode::Char(c) = key.code
        && let Some(action) = mac_option_tab(c)
    {
        return Some(action);
    }
    if ctrl {
        return (key.code == KeyCode::Char('y')).then_some(Action::Prefix);
    }
    if ctx.prefix {
        return Some(prefixed(key.code));
    }
    if ctx.typing {
        return typing(key.code);
    }
    // The menu has the keys (over the browsing view or a video tab still
    // shown): moving, opening (→ too, but Quit), back, Tab, / and f.
    if ctx.menu {
        return match key.code {
            KeyCode::Right => Some(Action::MenuOpen),
            KeyCode::Char('k') | KeyCode::Up => Some(Action::Up),
            KeyCode::Char('j') | KeyCode::Down => Some(Action::Down),
            KeyCode::Enter => Some(Action::Enter),
            KeyCode::Esc | KeyCode::Char('h') | KeyCode::Left | KeyCode::Backspace => Some(Action::Back),
            KeyCode::Tab => Some(Action::ToggleFocus),
            KeyCode::Char('/') => Some(Action::FocusSearch),
            KeyCode::Char('f') => Some(Action::ToggleFullscreen),
            _ => None,
        };
    }
    match key.code {
        KeyCode::Char('w') => return Some(Action::AddLater),
        KeyCode::Char('l') => return Some(Action::SaveForLater),
        KeyCode::Char('s') => return Some(Action::ToggleSubscription),
        KeyCode::Char('g') => return Some(Action::Rate(Rating::Like)),
        KeyCode::Char('G') => return Some(Action::Rate(Rating::Dislike)),
        KeyCode::Char('d') => return Some(Action::Download),
        KeyCode::Char('a') => return Some(Action::Listen),
        KeyCode::Char('c') => return Some(Action::OpenChannel),
        _ => {}
    }
    // In a video tab ← goes to the menu, as from the first list; Shift+← →
    // seek (→ alone too).
    if ctx.in_tab {
        match key.code {
            KeyCode::Left if key.modifiers.contains(KeyModifiers::SHIFT) => return Some(Action::Seek(-SEEK_SECONDS)),
            KeyCode::Right => return Some(Action::Seek(SEEK_SECONDS)),
            KeyCode::Left => return Some(Action::FocusMenu),
            _ => {}
        }
    }
    match (ctx.in_tab, ctx.related) {
        (true, true) => related(key.code).or_else(|| player(key.code)),
        (true, false) => player(key.code),
        _ => lists(key.code),
    }
}

fn prefixed(code: KeyCode) -> Action {
    match code {
        KeyCode::Char(c @ '0'..='9') => Action::GoTab(c as usize - '0' as usize),
        KeyCode::Char(c) => {
            PREFIX.iter().find(|(key, ..)| *key == c).map_or(Action::CancelPrefix, |(_, action, _)| action.clone())
        }
        _ => Action::CancelPrefix,
    }
}

fn typing(code: KeyCode) -> Option<Action> {
    match code {
        KeyCode::Char(c) => Some(Action::Input(c)),
        KeyCode::Backspace => Some(Action::Backspace),
        KeyCode::Enter => Some(Action::Enter),
        KeyCode::Esc => Some(Action::Back),
        KeyCode::Tab => Some(Action::CycleScope),
        _ => None,
    }
}

fn player(code: KeyCode) -> Option<Action> {
    match code {
        KeyCode::Char(' ') => Some(Action::TogglePause),
        KeyCode::Up => Some(Action::Volume(VOLUME_STEP)),
        KeyCode::Down => Some(Action::Volume(-VOLUME_STEP)),
        KeyCode::Char('m') => Some(Action::ToggleMute),
        KeyCode::Char('j') => Some(Action::CycleSubtitles),
        KeyCode::Char('i') => Some(Action::CycleLanguage),
        KeyCode::Char('o') => Some(Action::CycleSubLang),
        KeyCode::Char('v') => Some(Action::CycleQuality),
        KeyCode::Char('b') => Some(Action::CycleCodec),
        KeyCode::Char('.') => Some(Action::NextTrack),
        KeyCode::Char('f') => Some(Action::ToggleFullscreen),
        KeyCode::PageDown => Some(Action::ScrollComments(3)),
        KeyCode::PageUp => Some(Action::ScrollComments(-3)),
        KeyCode::Enter => Some(Action::Enter),
        KeyCode::Char('[') => Some(Action::Chapter(-1)),
        KeyCode::Char(']') => Some(Action::Chapter(1)),
        KeyCode::Tab => Some(Action::ToggleFocus),
        KeyCode::Esc => Some(Action::Back),
        KeyCode::Char(',') => Some(Action::PrevTrack),
        _ => None,
    }
}

/// The related videos of a tab: move and open; the rest still drives the player.
fn related(code: KeyCode) -> Option<Action> {
    match code {
        KeyCode::Char('j') | KeyCode::Down => Some(Action::Down),
        KeyCode::Char('k') | KeyCode::Up => Some(Action::Up),
        KeyCode::Enter => Some(Action::Enter),
        KeyCode::Tab | KeyCode::Esc => Some(Action::ToggleFocus),
        _ => None,
    }
}

fn lists(code: KeyCode) -> Option<Action> {
    match code {
        KeyCode::Char(' ') => Some(Action::ToggleMark),
        KeyCode::Char('x') | KeyCode::Delete => Some(Action::Trash),
        KeyCode::Char('v') => Some(Action::ToggleWatched),
        // The video playing (in the tab shown last) fills the screen.
        KeyCode::Char('f') => Some(Action::ToggleFullscreen),
        KeyCode::Char('j') | KeyCode::Down => Some(Action::Down),
        KeyCode::Char('k') | KeyCode::Up => Some(Action::Up),
        KeyCode::Enter => Some(Action::Enter),
        KeyCode::Esc | KeyCode::Char('h') | KeyCode::Left | KeyCode::Backspace => Some(Action::Back),
        KeyCode::Char('/') => Some(Action::FocusSearch),
        KeyCode::Tab => Some(Action::ToggleFocus),
        _ => None,
    }
}

/// Sections of the help screen: title and `(keys, description)` rows.
pub fn help(lang: Lang) -> Vec<(&'static str, Vec<(String, &'static str)>)> {
    vec![
        (lang.pick("Comandos: Ctrl+Y y después una tecla", "Commands: Ctrl+Y, Then a Key"), prefix_rows(lang)),
        (lang.pick("Menú", "Menu"), {
            let mut menu = rows(IN_MENU, lang);
            // → opens too (not Quit); the bar keeps just Enter.
            menu[1] = ("Enter  →".into(), lang.pick("abrir (Salir: solo Enter)", "open (Quit: Enter only)"));
            menu
        }),
        (lang.pick("Listas", "Lists"), rows(IN_LISTS, lang)),
        (lang.pick("Sobre un video (lista o pestaña)", "On a Video (List or Tab)"), rows(ON_VIDEO, lang)),
        (lang.pick("Pestaña de video", "Video Tab"), rows(IN_TAB, lang)),
        (lang.pick("Cuadro de búsqueda", "Search Box"), rows(IN_SEARCH, lang)),
        ("Mouse", rows(WITH_MOUSE, lang)),
        (
            lang.pick("Siempre", "Always"),
            vec![
                ("Alt+,".into(), lang.pick("pestaña anterior del menú", "previous tab in the menu")),
                ("Alt+.".into(), lang.pick("pestaña siguiente del menú", "next tab in the menu")),
                ("Ctrl+C".into(), lang.pick("salir", "quit")),
            ],
        ),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::crossterm::event::KeyEventState;

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    fn char(c: char) -> KeyEvent {
        key(KeyCode::Char(c))
    }

    fn ctrl(c: char) -> KeyEvent {
        KeyEvent::new(KeyCode::Char(c), KeyModifiers::CONTROL)
    }

    const LISTS: KeyContext = KeyContext {
        prefix: false,
        help: false,
        confirm: false,
        typing: false,
        in_tab: false,
        related: false,
        context_menu: false,
        splash: false,
        menu: false,
    };
    const MENU: KeyContext = KeyContext { menu: true, ..LISTS };

    #[test]
    fn right_opens_the_menu_row_l_does_not() {
        assert_eq!(map(key(KeyCode::Right), MENU), Some(Action::MenuOpen));
        assert_eq!(map(char('l'), MENU), None, "l does nothing in the menu");
        assert_eq!(map(char(' '), MENU), None, "nor the player keys over a tab");
        assert_eq!(map(key(KeyCode::Tab), MENU), Some(Action::ToggleFocus), "Tab still switches panes");
        assert_eq!(map(key(KeyCode::Left), MENU), Some(Action::Back));
        // Outside the menu they keep their meaning.
        assert_eq!(map(char('l'), LISTS), Some(Action::SaveForLater));
        assert_eq!(map(key(KeyCode::Right), LISTS), None);
        assert_eq!(map(char('l'), KeyContext { typing: true, ..MENU }), Some(Action::Input('l')));
    }
    const TYPING: KeyContext = KeyContext { typing: true, ..LISTS };
    const TAB: KeyContext = KeyContext { in_tab: true, ..LISTS };
    const PREFIXED: KeyContext = KeyContext { prefix: true, ..LISTS };
    const HELP: KeyContext = KeyContext { help: true, ..LISTS };
    const CONFIRM: KeyContext = KeyContext { confirm: true, ..LISTS };

    #[test]
    fn ctrl_y_starts_prefix_everywhere_but_dialogs() {
        for ctx in [LISTS, TYPING, TAB] {
            assert_eq!(map(ctrl('y'), ctx), Some(Action::Prefix), "{ctx:?}");
        }
        assert_eq!(map(ctrl('y'), HELP), Some(Action::ToggleHelp));
    }

    #[test]
    fn prefix_commands_come_from_the_table() {
        for (c, action, _) in PREFIX {
            assert_eq!(map(char(*c), PREFIXED).as_ref(), Some(action), "{c}");
        }
        assert_eq!(map(char('0'), PREFIXED), Some(Action::GoTab(0)));
        assert_eq!(map(char('3'), PREFIXED), Some(Action::GoTab(3)));
        assert_eq!(map(char('?'), PREFIXED), Some(Action::ToggleHelp));
    }

    #[test]
    fn unknown_key_after_prefix_cancels_it() {
        assert_eq!(map(char('z'), PREFIXED), Some(Action::CancelPrefix));
        assert_eq!(map(key(KeyCode::Esc), PREFIXED), Some(Action::CancelPrefix));
    }

    #[test]
    fn prefix_wins_over_typing() {
        assert_eq!(map(char('b'), KeyContext { typing: true, ..PREFIXED }), Some(Action::ToggleSidebar));
    }

    #[test]
    fn any_key_closes_help() {
        assert_eq!(map(char('x'), HELP), Some(Action::ToggleHelp));
        assert_eq!(map(key(KeyCode::Esc), HELP), Some(Action::ToggleHelp));
    }

    #[test]
    fn confirm_only_accepts_yes() {
        assert_eq!(map(char('s'), CONFIRM), Some(Action::Confirm(true)));
        assert_eq!(map(char('y'), CONFIRM), Some(Action::Confirm(true)));
        assert_eq!(map(char('n'), CONFIRM), Some(Action::Confirm(false)));
        assert_eq!(map(key(KeyCode::Enter), CONFIRM), Some(Action::Confirm(false)));
    }

    #[test]
    fn typing_edits_the_query() {
        assert_eq!(map(char('l'), TYPING), Some(Action::Input('l')));
        assert_eq!(map(KeyEvent::new(KeyCode::Char('A'), KeyModifiers::SHIFT), TYPING), Some(Action::Input('A')));
        assert_eq!(map(key(KeyCode::Backspace), TYPING), Some(Action::Backspace));
        assert_eq!(map(key(KeyCode::Enter), TYPING), Some(Action::Enter));
        assert_eq!(map(key(KeyCode::Esc), TYPING), Some(Action::Back));
        assert_eq!(map(key(KeyCode::Tab), TYPING), Some(Action::CycleScope));
    }

    #[test]
    fn list_navigation() {
        assert_eq!(map(char('j'), LISTS), Some(Action::Down));
        assert_eq!(map(key(KeyCode::Down), LISTS), Some(Action::Down));
        assert_eq!(map(char('k'), LISTS), Some(Action::Up));
        assert_eq!(map(key(KeyCode::Enter), LISTS), Some(Action::Enter));
        assert_eq!(map(key(KeyCode::Esc), LISTS), Some(Action::Back));
        assert_eq!(map(char('h'), LISTS), Some(Action::Back));
        assert_eq!(map(char('/'), LISTS), Some(Action::FocusSearch));
        assert_eq!(map(key(KeyCode::Tab), LISTS), Some(Action::ToggleFocus));
    }

    #[test]
    fn video_keys_work_in_lists_and_tabs() {
        for ctx in [LISTS, TAB] {
            assert_eq!(map(char('w'), ctx), Some(Action::AddLater), "YouTube Watch Later");
            assert_eq!(map(char('l'), ctx), Some(Action::SaveForLater), "tubevibe Watch Later");
            assert_eq!(map(char('s'), ctx), Some(Action::ToggleSubscription));
            assert_eq!(map(char('g'), ctx), Some(Action::Rate(Rating::Like)));
            assert_eq!(map(char('G'), ctx), Some(Action::Rate(Rating::Dislike)));
            assert_eq!(map(char('d'), ctx), Some(Action::Download));
            assert_eq!(map(char('a'), ctx), Some(Action::Listen));
            assert_eq!(map(char('c'), ctx), Some(Action::OpenChannel));
        }
    }

    #[test]
    fn there_is_no_update_hotkey() {
        // Updating is About's Update Now button, never a stray key.
        assert_eq!(map(char('U'), LISTS), None);
        assert_ne!(map(char('U'), TAB), Some(Action::SelfUpdate));
    }

    #[test]
    fn player_controls_in_tab() {
        assert_eq!(map(char(' '), TAB), Some(Action::TogglePause));
        assert_eq!(map(key(KeyCode::Left), TAB), Some(Action::FocusMenu));
        assert_eq!(map(key(KeyCode::Left), KeyContext { related: true, ..TAB }), Some(Action::FocusMenu));
        let shift = |code| KeyEvent::new(code, KeyModifiers::SHIFT);
        assert_eq!(map(shift(KeyCode::Left), TAB), Some(Action::Seek(-SEEK_SECONDS)));
        assert_eq!(map(shift(KeyCode::Right), TAB), Some(Action::Seek(SEEK_SECONDS)));
        assert_eq!(map(key(KeyCode::Right), TAB), Some(Action::Seek(SEEK_SECONDS)));
        assert_eq!(map(key(KeyCode::Left), LISTS), Some(Action::Back), "lists unchanged");
        assert_eq!(map(key(KeyCode::Up), TAB), Some(Action::Volume(VOLUME_STEP)));
        assert_eq!(map(key(KeyCode::Down), TAB), Some(Action::Volume(-VOLUME_STEP)));
        assert_eq!(map(char('m'), TAB), Some(Action::ToggleMute));
        assert_eq!(map(char('j'), TAB), Some(Action::CycleSubtitles));
        assert_eq!(map(char('i'), TAB), Some(Action::CycleLanguage));
        assert_eq!(map(char('o'), TAB), Some(Action::CycleSubLang));
        assert_eq!(map(char('v'), TAB), Some(Action::CycleQuality));
        assert_eq!(map(char(' '), LISTS), Some(Action::ToggleMark));
        assert_eq!(map(char('x'), LISTS), Some(Action::Trash));
        assert_eq!(map(key(KeyCode::Delete), LISTS), Some(Action::Trash));
        assert_eq!(map(char('b'), TAB), Some(Action::CycleCodec));
        assert_eq!(map(char('.'), TAB), Some(Action::NextTrack));
        assert_eq!(map(char('f'), TAB), Some(Action::ToggleFullscreen));
        assert_eq!(map(key(KeyCode::Esc), TAB), Some(Action::Back));
        assert_eq!(map(char(','), TAB), Some(Action::PrevTrack));
        assert_eq!(map(char('['), TAB), Some(Action::Chapter(-1)));
        assert_eq!(map(char(']'), TAB), Some(Action::Chapter(1)));
        assert_eq!(map(key(KeyCode::Enter), TAB), Some(Action::Enter), "plays a closed video again");
    }

    #[test]
    fn plain_q_does_not_quit_but_ctrl_c_always_does() {
        assert_eq!(map(char('q'), LISTS), None);
        for ctx in [LISTS, TYPING, TAB, PREFIXED, HELP, CONFIRM] {
            assert_eq!(map(ctrl('c'), ctx), Some(Action::Quit), "{ctx:?}");
        }
    }

    #[test]
    fn key_releases_are_ignored() {
        // Windows reports both press and release.
        let release = KeyEvent { kind: KeyEventKind::Release, state: KeyEventState::NONE, ..ctrl('y') };
        assert_eq!(map(release, LISTS), None);
    }

    #[test]
    fn help_lists_every_prefix_command() {
        for lang in [Lang::Es, Lang::En] {
            let help = help(lang);
            let (_, commands) = &help[0];
            for (c, ..) in PREFIX {
                assert!(commands.iter().any(|(k, _)| *k == c.to_string()), "{c} missing from help");
            }
            assert!(commands.iter().any(|(k, _)| k == "0-9"));
        }
    }

    #[test]
    fn alt_comma_and_period_switch_tabs() {
        let alt = |c| KeyEvent::new(KeyCode::Char(c), KeyModifiers::ALT);
        for ctx in [LISTS, TYPING, TAB, KeyContext { related: true, ..TAB }, PREFIXED] {
            assert_eq!(map(alt(','), ctx), Some(Action::PrevTab), "{ctx:?}");
            assert_eq!(map(alt('.'), ctx), Some(Action::NextTab), "{ctx:?}");
            // With Shift held by mistake (US layout: < and >).
            assert_eq!(
                map(KeyEvent::new(KeyCode::Char('<'), KeyModifiers::ALT | KeyModifiers::SHIFT), ctx),
                Some(Action::PrevTab)
            );
            assert_eq!(
                map(KeyEvent::new(KeyCode::Char('>'), KeyModifiers::ALT | KeyModifiers::SHIFT), ctx),
                Some(Action::NextTab)
            );
        }
        assert_eq!((mac_option_tab('≤'), mac_option_tab('≥')), (Some(Action::PrevTab), Some(Action::NextTab)));
        assert_eq!(mac_option_tab('x'), None);
        // Plain , and . still change songs in a video tab, and type in the box.
        assert_eq!(map(char('.'), TAB), Some(Action::NextTrack));
        assert_eq!(map(char(','), TAB), Some(Action::PrevTrack));
        assert_eq!(map(char('.'), TYPING), Some(Action::Input('.')));
        // Ctrl+Alt (AltGr) is not a tab key.
        let altgr = KeyEvent::new(KeyCode::Char('.'), KeyModifiers::ALT | KeyModifiers::CONTROL);
        assert_ne!(map(altgr, LISTS), Some(Action::NextTab));
        // Ctrl+Tab is gone: Tab with Ctrl is just ignored, plain Tab as always.
        assert_eq!(map(KeyEvent::new(KeyCode::Tab, KeyModifiers::CONTROL), LISTS), None);
        assert_eq!(map(key(KeyCode::Tab), LISTS), Some(Action::ToggleFocus));
        assert_eq!(map(key(KeyCode::Tab), TAB), Some(Action::ToggleFocus));
        assert_eq!(map(key(KeyCode::Tab), TYPING), Some(Action::CycleScope));
        // No more Ctrl+Y n / p.
        assert_eq!(map(char('n'), PREFIXED), Some(Action::CancelPrefix));
        assert_eq!(map(char('p'), PREFIXED), Some(Action::CancelPrefix));
        // Repeats and releases count for nothing.
        for kind in [KeyEventKind::Repeat, KeyEventKind::Release] {
            assert_eq!(map(KeyEvent { kind, ..alt('.') }, LISTS), None);
        }
        // Both in the full help, in both languages.
        for lang in [Lang::Es, Lang::En] {
            let help = help(lang);
            let keys: Vec<_> = help.iter().flat_map(|(_, rows)| rows.iter().map(|(k, _)| k.as_str())).collect();
            assert!(keys.contains(&"Alt+,") && keys.contains(&"Alt+."), "{keys:?}");
            assert!(!keys.iter().any(|k| k.contains("Ctrl+Tab")));
            assert!(!help[0].1.iter().any(|(k, _)| k == "n" || k == "p"));
        }
    }

    #[test]
    fn help_is_translated() {
        assert_eq!(help(Lang::En)[0].1.iter().find(|(k, _)| k == "q").unwrap().1, "quit");
        assert_eq!(help(Lang::Es)[0].1.iter().find(|(k, _)| k == "q").unwrap().1, "salir");
        assert_eq!(rows(IN_TAB, Lang::En)[0].0, "Space");
        assert_eq!(rows(IN_TAB, Lang::Es)[0].0, "Espacio");
        assert!(rows(IN_LISTS, Lang::En).iter().any(|(k, _)| k == "x  Del"));
        assert!(rows(IN_LISTS, Lang::En).iter().any(|(_, d)| d.contains("Backspace")));
        let mouse: Vec<_> = rows(WITH_MOUSE, Lang::En).into_iter().map(|(k, _)| k).collect();
        assert_eq!(mouse, ["Click", "Double Click", "Wheel", "Click Video", "Right Click", "Click Bar", "Shift"]);
        assert_eq!(rows(WITH_MOUSE, Lang::Es)[1].0, "Doble clic");
        assert_eq!(map(char('l'), PREFIXED), Some(Action::ToggleUiLang));
    }
}
