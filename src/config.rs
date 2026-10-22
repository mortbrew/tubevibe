//! User configuration, read from `config.toml`.
//!
//! Every field has a default, so a missing file or a partial file is fine.
//! Unknown keys and sections (a typo, or a setting from a newer tubevibe) and
//! values of the wrong type are skipped with a [`ConfigWarning`] shown in the
//! status bar: an older tubevibe still opens with a newer config. Only a file
//! that is not valid TOML stops the app.
//! Settings changed from the app are written back with [`set_value`], which
//! keeps the user's comments and formatting.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::i18n::{Lang, Text};

#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize, Serialize)]
#[serde(default)]
pub struct Config {
    pub session: SessionConfig,
    pub player: PlayerConfig,
    pub ui: UiConfig,
    pub search: SearchConfig,
    pub download: DownloadConfig,
    pub media: MediaConfig,
    pub effects: EffectsConfig,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize, Serialize)]
#[serde(default)]
pub struct MediaConfig {
    /// Folders with videos to play; empty means the system videos folder.
    pub video_dirs: Vec<String>,
    /// Folders with music; empty means the system music folder.
    pub music_dirs: Vec<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize, Serialize)]
#[serde(default)]
pub struct SessionConfig {
    /// Where the YouTube login comes from, in yt-dlp `--cookies-from-browser`
    /// syntax: `BROWSER[+KEYRING][:PROFILE]`, e.g. `brave+GNOMEKEYRING:Default`.
    /// Empty means no login.
    pub browser: String,
    /// Keyring used when picking a browser from the "Sesión" list, e.g.
    /// `GNOMEKEYRING`. Empty detects it (KWallet, gnome-keyring or another
    /// Secret Service), which matters on window managers where yt-dlp guesses wrong.
    pub keyring: String,
    /// An imported `cookies.txt`, for machines without a browser (SSH,
    /// servers). Takes precedence over `browser`.
    pub cookies_file: String,
    /// The app already asked which browser to take the login from.
    pub asked: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(default)]
pub struct PlayerConfig {
    pub mode: PlayerMode,
    /// Upper bound for the video height, e.g. 480, 720, 1080.
    pub max_height: u32,
    /// Preferred video codec prefix (`avc1` decodes in hardware almost
    /// everywhere). Empty means any codec.
    pub prefer_codec: String,
    /// Hardware decoding, in window mode (embedded video decodes in software
    /// to stay in sync).
    pub hwdec: bool,
    pub language: Language,
    /// Subtitle / dubbing language code.
    pub sub_lang: String,
    /// Keep listening when leaving the video tab (only the last video plays).
    pub background_audio: bool,
    /// Skip sponsors, self promotion and "subscribe!" reminders (SponsorBlock).
    pub sponsorblock: bool,
    /// Enter on a YouTube video starts playing it in its new tab; off, the
    /// tab opens with its details and a second Enter plays it.
    pub autoplay: bool,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum PlayerMode {
    /// Embedded when the terminal speaks the kitty graphics protocol.
    Auto,
    /// Video drawn inside the app pane.
    Embedded,
    /// Video in its own mpv window: hardware decoding, and the tab shows
    /// the video's details and what to watch next. The default.
    #[default]
    Window,
    /// An mpv window without borders laid over the app pane (X11): looks
    /// embedded, plays like a window.
    Overlay,
}

impl PlayerMode {
    pub fn next(self) -> Self {
        match self {
            Self::Auto => Self::Embedded,
            Self::Embedded => Self::Window,
            Self::Window => Self::Overlay,
            Self::Overlay => Self::Auto,
        }
    }

    /// Value as written in `config.toml`.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Auto => "auto",
            Self::Embedded => "embedded",
            Self::Window => "window",
            Self::Overlay => "overlay",
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Language {
    /// Original audio with subtitles in `sub_lang` (auto-translated if needed).
    #[default]
    Subtitles,
    /// Original audio, no subtitles.
    Original,
    /// YouTube's automatic dubbing in `sub_lang` when available.
    Dubbed,
}

impl Language {
    pub fn next(self) -> Self {
        match self {
            Self::Subtitles => Self::Original,
            Self::Original => Self::Dubbed,
            Self::Dubbed => Self::Subtitles,
        }
    }

    /// Value as written in `config.toml`.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Subtitles => "subtitles",
            Self::Original => "original",
            Self::Dubbed => "dubbed",
        }
    }
}

/// A size setting: its allowed values (`min..=max` by `step`) and default.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Size {
    pub key: &'static str,
    pub min: u16,
    pub max: u16,
    pub step: u16,
    pub default: u16,
}

impl Size {
    /// The next value Enter picks: one step up, from the largest back to the smallest.
    pub fn next(self, value: u16) -> u16 {
        let up = value.saturating_add(self.step);
        if value >= self.max || up > self.max { self.min } else { up.max(self.min) }
    }

    /// One step narrower (`dir < 0`) or wider, within the limits.
    pub fn step_by(self, value: u16, dir: i8) -> u16 {
        let v = if dir < 0 { value.saturating_sub(self.step) } else { value.saturating_add(self.step) };
        v.clamp(self.min, self.max)
    }

    pub fn contains(self, value: u16) -> bool {
        (self.min..=self.max).contains(&value)
    }
}

/// Columns; 26 is the layout of earlier versions.
pub const MENU_WIDTH: Size = Size { key: "menu_width", min: 20, max: 40, step: 2, default: 26 };
/// Columns; 46 is the layout of earlier versions.
pub const PREVIEW_WIDTH: Size = Size { key: "preview_width", min: 30, max: 70, step: 4, default: 46 };
/// Columns; 46 is what the earlier 45 % gave in a 130-column terminal. A
/// percent saved by the first try (30–60) reads as about as many columns.
pub const COMMENTS_WIDTH: Size = Size { key: "comments_width", min: 30, max: 78, step: 4, default: 46 };

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(default)]
pub struct UiConfig {
    pub sidebar: bool,
    /// Columns of the left menu ([`MENU_WIDTH`]).
    pub menu_width: u16,
    /// Columns of the preview panel right of video and channel lists ([`PREVIEW_WIDTH`]).
    pub preview_width: u16,
    /// Columns of the comments in a video tab ([`COMMENTS_WIDTH`]).
    pub comments_width: u16,
    pub icons: IconSet,
    /// Interface language; unset follows the system locale.
    pub lang: Option<Lang>,
    /// Video thumbnails in the preview panel and video tabs.
    pub thumbnails: ThumbMode,
    /// Thumbnails made blocky on purpose.
    pub thumb_pixelation: Pixelation,
    /// Clicks and the wheel (selecting text then needs Shift).
    pub mouse: bool,
    /// The tubevibe logo for a moment when the app opens.
    pub splash: bool,
    /// Ask GitHub, each time the app opens, whether there is a newer version.
    pub check_updates: bool,
    /// Follow the current Omarchy theme's colours (Tokyo Night otherwise).
    pub omarchy_theme: bool,
    /// Inicio, Descubrir and Suscripciones leave out the videos I watched
    /// (`v` shows them for a while).
    pub hide_watched: bool,
    /// My Recommendations leaves out the videos in my YouTube Watch Later.
    pub hide_later: bool,
    /// Feeds, searches and channels leave out YouTube Shorts.
    pub hide_shorts: bool,
    /// While the app is open, tell when my channels publish something new.
    pub notify_new: bool,
    /// The channels I watch most go first in Suscripciones and Mis canales.
    pub favorites_first: bool,
    /// How every pane's border is drawn.
    pub border_style: BorderStyle,
    /// The video tabs open when the app closed come back in the menu.
    pub restore_tabs: bool,
}

/// The line every pane is framed with (`[ui] border_style`).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum BorderStyle {
    /// `┌─┐`, the look tubevibe always had.
    #[default]
    Plain,
    /// `╭─╮`
    Rounded,
    /// `┏━┓`
    Thick,
    /// `╔═╗`
    Double,
    /// No lines; the panes keep their room and titles.
    None,
}

impl BorderStyle {
    pub const ALL: [Self; 5] = [Self::Plain, Self::Rounded, Self::Thick, Self::Double, Self::None];

    pub fn next(self) -> Self {
        let at = Self::ALL.iter().position(|b| *b == self).unwrap_or(0);
        Self::ALL[(at + 1) % Self::ALL.len()]
    }

    /// Value as written in `config.toml`.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Plain => "plain",
            Self::Rounded => "rounded",
            Self::Thick => "thick",
            Self::Double => "double",
            Self::None => "none",
        }
    }

    /// As the settings screen shows it.
    pub fn label(self, lang: Lang) -> &'static str {
        match self {
            Self::Plain => lang.pick("Simple", "Plain"),
            Self::Rounded => lang.pick("Redondeado", "Rounded"),
            Self::Thick => lang.pick("Grueso", "Thick"),
            Self::Double => lang.pick("Doble", "Double"),
            Self::None => lang.pick("Ninguno", "None"),
        }
    }
}

/// Short animations, each one can be turned off.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(default)]
pub struct EffectsConfig {
    /// The menu and the welcome screen appear when the app starts.
    pub startup: bool,
    /// A new list, section or tab sweeps in.
    pub pages: bool,
    /// Help and questions open with a flash.
    pub dialogs: bool,
}

impl Default for EffectsConfig {
    fn default() -> Self {
        Self { startup: true, pages: true, dialogs: true }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum IconSet {
    /// Colour emoji, like the original ytv script.
    #[default]
    Emoji,
    /// Needs a Nerd Font (kitty bundles the symbols).
    Nerd,
    /// Plain Unicode, works with any font.
    Unicode,
}

impl IconSet {
    pub fn next(self) -> Self {
        match self {
            Self::Emoji => Self::Nerd,
            Self::Nerd => Self::Unicode,
            Self::Unicode => Self::Emoji,
        }
    }

    /// As the settings screen shows it.
    pub fn label(self) -> &'static str {
        match self {
            Self::Emoji => "Emoji",
            Self::Nerd => "Nerd Font",
            Self::Unicode => "Unicode",
        }
    }

    /// Value as written in `config.toml`.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Emoji => "emoji",
            Self::Nerd => "nerd",
            Self::Unicode => "unicode",
        }
    }
}

/// Settings → Thumbnails: the terminal's own pictures (kitty, sixel,
/// iTerm2; half blocks elsewhere), half blocks always, or none.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ThumbMode {
    #[default]
    Picture,
    Blocks,
    Off,
}

/// Also reads the old `thumbnails = true / false`.
impl<'de> Deserialize<'de> for ThumbMode {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum Raw {
            Bool(bool),
            Text(String),
        }
        match Raw::deserialize(d)? {
            Raw::Bool(true) => Ok(Self::Picture),
            Raw::Bool(false) => Ok(Self::Off),
            Raw::Text(t) => match t.trim().to_ascii_lowercase().as_str() {
                "picture" | "on" | "true" => Ok(Self::Picture),
                "blocks" => Ok(Self::Blocks),
                "off" | "false" => Ok(Self::Off),
                other => Err(serde::de::Error::custom(format!("unknown thumbnails `{other}`: picture, blocks or off"))),
            },
        }
    }
}

impl ThumbMode {
    pub const ALL: [Self; 3] = [Self::Picture, Self::Blocks, Self::Off];

    pub fn next(self) -> Self {
        let at = Self::ALL.iter().position(|a| *a == self).unwrap_or(0);
        Self::ALL[(at + 1) % Self::ALL.len()]
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Picture => "picture",
            Self::Blocks => "blocks",
            Self::Off => "off",
        }
    }

    pub fn label(self, lang: Lang) -> &'static str {
        match self {
            Self::Picture => lang.pick("Imagen", "Picture"),
            Self::Blocks => lang.pick("Bloques", "Blocks"),
            Self::Off => lang.pick("No", "Off"),
        }
    }
}

/// Settings → Thumbnail Pixelation: the thumbnail shrunk to a small grid and
/// blown up again (nearest neighbour), so it looks like pixel art.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Pixelation {
    #[default]
    Off,
    Low,
    Medium,
    High,
    Max,
}

impl Pixelation {
    pub const ALL: [Self; 5] = [Self::Off, Self::Low, Self::Medium, Self::High, Self::Max];

    pub fn next(self) -> Self {
        let at = Self::ALL.iter().position(|a| *a == self).unwrap_or(0);
        Self::ALL[(at + 1) % Self::ALL.len()]
    }

    /// Pixels across the shrunk thumbnail (`None`: full resolution).
    pub fn grid(self) -> Option<u32> {
        match self {
            Self::Off => None,
            Self::Low => Some(64),
            Self::Medium => Some(32),
            Self::High => Some(16),
            Self::Max => Some(8),
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Off => "off",
            Self::Low => "low",
            Self::Medium => "medium",
            Self::High => "high",
            Self::Max => "max",
        }
    }

    pub fn label(self, lang: Lang) -> &'static str {
        match self {
            Self::Off => lang.pick("No", "Off"),
            Self::Low => lang.pick("Baja", "Low"),
            Self::Medium => lang.pick("Media", "Medium"),
            Self::High => lang.pick("Alta", "High"),
            Self::Max => lang.pick("Máxima", "Max"),
        }
    }
}

/// Subtitle / dubbing languages offered by the settings screen (any code works in the file).
pub const LANGUAGES: [(&str, Text); 9] = [
    ("es", Text::new("Español", "Spanish")),
    ("en", Text::new("Inglés", "English")),
    ("pt", Text::new("Portugués", "Portuguese")),
    ("fr", Text::new("Francés", "French")),
    ("it", Text::new("Italiano", "Italian")),
    ("de", Text::new("Alemán", "German")),
    ("ja", Text::new("Japonés", "Japanese")),
    ("ko", Text::new("Coreano", "Korean")),
    ("ru", Text::new("Ruso", "Russian")),
];

pub fn next_language(code: &str) -> &'static str {
    let index = LANGUAGES.iter().position(|(c, _)| *c == code).map_or(0, |i| (i + 1) % LANGUAGES.len());
    LANGUAGES[index].0
}

pub fn language_name(code: &str, lang: Lang) -> String {
    LANGUAGES
        .iter()
        .find(|(c, _)| *c == code)
        .map_or_else(|| code.to_owned(), |(c, name)| format!("{} ({c})", name.get(lang)))
}

/// A donation address, shown on the About page.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Wallet {
    pub network: String,
    pub address: String,
}

/// The wallets in `donate.toml`, compiled into the app; empty ones are skipped.
pub fn donations() -> Vec<Wallet> {
    parse_donations(include_str!("../donate.toml"))
}

fn parse_donations(text: &str) -> Vec<Wallet> {
    #[derive(Deserialize)]
    struct File {
        #[serde(default)]
        wallet: Vec<Wallet>,
    }
    let wallets = toml::from_str::<File>(text).map(|f| f.wallet).unwrap_or_default();
    wallets.into_iter().filter(|w| !w.address.trim().is_empty()).collect()
}

/// Heights offered by the settings screen and the `v` key.
pub const QUALITIES: [u32; 5] = [360, 480, 720, 1080, 1440];

pub fn next_quality(current: u32) -> u32 {
    QUALITIES.iter().copied().find(|q| *q > current).unwrap_or(QUALITIES[0])
}

/// Video codecs offered by the `b` key: the `vcodec` prefix yt-dlp reports
/// (VP9 shows as `vp9` or `vp09`) and its name; empty means any.
pub const CODECS: [(&str, &str); 4] = [("avc1", "H.264"), ("vp", "VP9"), ("av01", "AV1"), ("", "")];

pub fn next_codec(current: &str) -> &'static str {
    let at = CODECS.iter().position(|(c, _)| *c == current);
    CODECS[at.map_or(0, |i| (i + 1) % CODECS.len())].0
}

pub fn codec_name(codec: &str, lang: Lang) -> &str {
    match CODECS.iter().find(|(c, _)| *c == codec) {
        Some((_, "")) => lang.pick("cualquiera", "any"),
        Some((_, name)) => name,
        None => codec,
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(default)]
pub struct SearchConfig {
    /// Videos fetched per list.
    pub results: u32,
    /// What "Descubrir" looks for, e.g. `["rust", "nba", "jazz"]`.
    pub topics: Vec<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize, Serialize)]
#[serde(default)]
pub struct DownloadConfig {
    /// Empty means the system videos folder (`~/Videos`).
    pub dir: String,
}

impl Default for PlayerConfig {
    fn default() -> Self {
        Self {
            mode: PlayerMode::Window,
            max_height: 720,
            prefer_codec: "avc1".into(),
            hwdec: true,
            language: Language::Subtitles,
            sub_lang: "es".into(),
            background_audio: true,
            sponsorblock: true,
            // tubevibe: open the tab first, Enter plays (upstream plays at once).
            autoplay: false,
        }
    }
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            sidebar: true,
            menu_width: MENU_WIDTH.default,
            preview_width: PREVIEW_WIDTH.default,
            comments_width: COMMENTS_WIDTH.default,
            icons: IconSet::Emoji,
            lang: None,
            thumbnails: ThumbMode::Picture,
            thumb_pixelation: Pixelation::Off,
            mouse: true,
            splash: true,
            check_updates: true,
            omarchy_theme: true,
            hide_watched: true,
            hide_later: true,
            hide_shorts: true,
            notify_new: true,
            favorites_first: true,
            border_style: BorderStyle::Plain,
            restore_tabs: true,
        }
    }
}

impl Default for SearchConfig {
    fn default() -> Self {
        Self { results: 60, topics: Vec::new() }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("cannot read {path}: {source}")]
    Read { path: PathBuf, source: std::io::Error },
    #[error("invalid config {path}: {source}")]
    Parse { path: PathBuf, source: toml::de::Error },
    #[error("cannot update {path}: {reason}")]
    Write { path: PathBuf, reason: String },
}

impl ConfigError {
    /// What the status bar says, in `lang` (the details come from the system
    /// or the TOML reader, in English).
    pub fn message(&self, lang: Lang) -> String {
        match self {
            Self::Read { path, source } => {
                format!("{} {}: {source}", lang.pick("No se pudo leer", "Couldn't read"), path.display())
            }
            Self::Parse { path, source } => {
                format!("{} {}: {source}", lang.pick("Configuración inválida en", "Invalid config in"), path.display())
            }
            Self::Write { path, reason } => {
                format!("{} {}: {reason}", lang.pick("No se pudo guardar", "Couldn't save"), path.display())
            }
        }
    }
}

/// Something in `config.toml` that was skipped instead of stopping the app.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfigWarning {
    /// `section.key` (or a whole section) this version does not know.
    Unknown(String),
    /// `section.key` with a value of the wrong type: its default is used.
    BadValue { key: String, error: String },
}

impl ConfigWarning {
    /// One line for the status bar: the first warning, and how many more.
    pub fn summary(warnings: &[Self], lang: Lang, icon: &str) -> Option<String> {
        let first = warnings.first()?;
        let text = match first {
            Self::Unknown(key) => format!(
                "{icon} {}",
                lang.pick(
                    &format!("Se ignoró el ajuste desconocido {key} de config.toml (¿de un tubevibe más nuevo?)"),
                    &format!("Ignored unknown setting {key} in config.toml (made by a newer tubevibe?)"),
                )
            ),
            Self::BadValue { key, .. } => format!(
                "{icon} {}",
                lang.pick(
                    &format!("Valor inválido en {key} de config.toml: se usa el predeterminado"),
                    &format!("Bad value for {key} in config.toml: using the default"),
                )
            ),
        };
        Some(match warnings.len() - 1 {
            0 => text,
            more => format!("{text} (+{more} {})", lang.pick("más", "more")),
        })
    }
}

impl std::fmt::Display for ConfigWarning {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unknown(key) => write!(f, "unknown setting {key} ignored"),
            Self::BadValue { key, error } => write!(f, "bad value for {key} ({error}): default used"),
        }
    }
}

/// Real keys of older versions, now gone (`ui.max_tabs`: tabs are unlimited).
const RETIRED: &[&str] = &["ui.max_tabs"];

/// One `[section]`: each key is tried on its own, so an unknown or broken
/// one is skipped (and reported) while the rest still apply.
fn section<T: serde::de::DeserializeOwned + Default>(
    name: &str,
    value: toml::Value,
    warnings: &mut Vec<ConfigWarning>,
) -> T {
    let toml::Value::Table(table) = value else {
        warnings.push(ConfigWarning::BadValue { key: name.into(), error: "not a [section]".into() });
        return T::default();
    };
    let mut good = toml::Table::new();
    for (key, value) in table {
        let path = format!("{name}.{key}");
        // Settings older versions had and this one dropped: ignored without a word.
        if RETIRED.contains(&path.as_str()) {
            continue;
        }
        let one = toml::Table::from_iter([(key.clone(), value.clone())]);
        let mut unknown = false;
        let tried: Result<T, toml::de::Error> = serde_ignored::deserialize(toml::Value::Table(one), |_| unknown = true);
        match tried {
            Ok(_) if unknown => warnings.push(ConfigWarning::Unknown(path)),
            Ok(_) => {
                good.insert(key, value);
            }
            Err(error) => {
                warnings.push(ConfigWarning::BadValue { key: path, error: error.message().trim().to_owned() });
            }
        }
    }
    T::deserialize(toml::Value::Table(good)).unwrap_or_default()
}

impl Config {
    /// Reads a config, skipping (without reporting) what it can't use.
    pub fn from_toml(text: &str) -> Result<Self, toml::de::Error> {
        Self::parse(text).map(|(config, _)| config)
    }

    /// Reads a config and lists what was skipped; only invalid TOML fails.
    pub fn parse(text: &str) -> Result<(Self, Vec<ConfigWarning>), toml::de::Error> {
        let table: toml::Table = toml::from_str(text)?;
        let mut warnings = Vec::new();
        let mut c = Self::default();
        for (name, value) in table {
            let w = &mut warnings;
            match name.as_str() {
                "session" => c.session = section(&name, value, w),
                "player" => c.player = section(&name, value, w),
                "ui" => c.ui = section(&name, value, w),
                "search" => c.search = section(&name, value, w),
                "download" => c.download = section(&name, value, w),
                "media" => c.media = section(&name, value, w),
                "effects" => c.effects = section(&name, value, w),
                _ => w.push(ConfigWarning::Unknown(name)),
            }
        }
        // Sizes out of range: said, and back to the default.
        for (size, value) in [
            (MENU_WIDTH, &mut c.ui.menu_width),
            (PREVIEW_WIDTH, &mut c.ui.preview_width),
            (COMMENTS_WIDTH, &mut c.ui.comments_width),
        ] {
            if !size.contains(*value) {
                warnings.push(ConfigWarning::BadValue {
                    key: format!("ui.{}", size.key),
                    error: format!("{} is not between {} and {}", value, size.min, size.max),
                });
                *value = size.default;
            }
        }
        Ok((c, warnings))
    }

    /// Loads `path`; a missing file yields the defaults.
    pub fn load(path: &Path) -> Result<Self, ConfigError> {
        Self::load_with_warnings(path).map(|(config, _)| config)
    }

    /// Loads `path` and lists the settings that were skipped.
    pub fn load_with_warnings(path: &Path) -> Result<(Self, Vec<ConfigWarning>), ConfigError> {
        let text = match std::fs::read_to_string(path) {
            Ok(text) => text,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok((Self::default(), Vec::new())),
            Err(source) => return Err(ConfigError::Read { path: path.into(), source }),
        };
        Self::parse(&text).map_err(|source| ConfigError::Parse { path: path.into(), source })
    }

    /// `~/.config/tubevibe/config.toml` on Linux, the platform equivalent elsewhere.
    pub fn default_path() -> Option<PathBuf> {
        directories::ProjectDirs::from("", "", "tubevibe").map(|dirs| dirs.config_dir().join("config.toml"))
    }
}

/// The debug log file: `TUBEVIBE_DEBUG`, or the old `TUBETUI_DEBUG`.
pub fn debug_path() -> Option<String> {
    ["TUBEVIBE_DEBUG", "TUBETUI_DEBUG"].iter().find_map(|k| std::env::var(k).ok().filter(|v| !v.is_empty()))
}

/// Sets `[table] key = value` in the file at `path`, creating file and
/// table when missing and leaving everything else untouched.
pub fn set_value(path: &Path, table: &str, key: &str, value: impl Into<toml_edit::Value>) -> Result<(), ConfigError> {
    let error = |reason: String| ConfigError::Write { path: path.into(), reason };
    let text = match std::fs::read_to_string(path) {
        Ok(text) => text,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => String::new(),
        Err(e) => return Err(error(e.to_string())),
    };
    let mut doc: toml_edit::DocumentMut = text.parse().map_err(|e: toml_edit::TomlError| error(e.to_string()))?;
    let section = doc
        .entry(table)
        .or_insert_with(toml_edit::table)
        .as_table_mut()
        .ok_or_else(|| error(format!("`{table}` is not a table")))?;
    section[key] = toml_edit::Item::Value(value.into());
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).map_err(|e| error(e.to_string()))?;
    }
    std::fs::write(path, doc.to_string()).map_err(|e| error(e.to_string()))
}

#[cfg(test)]
pub(crate) fn temp_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("tubevibe-test-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

#[cfg(test)]
mod tests {
    #[test]
    fn errors_and_icon_names_follow_the_language() {
        let error = ConfigError::Write { path: "/c/config.toml".into(), reason: "denied".into() };
        assert_eq!(error.message(Lang::En), "Couldn't save /c/config.toml: denied");
        assert_eq!(error.message(Lang::Es), "No se pudo guardar /c/config.toml: denied");
        assert_eq!(
            [IconSet::Emoji.label(), IconSet::Nerd.label(), IconSet::Unicode.label()],
            ["Emoji", "Nerd Font", "Unicode"]
        );
    }

    use super::*;

    #[test]
    fn defaults() {
        let c = Config::default();
        assert_eq!(c.session.browser, "");
        assert_eq!(c.session.cookies_file, "");
        assert_eq!(c.player.mode, PlayerMode::Window);
        assert_eq!(c.player.max_height, 720);
        assert_eq!(c.player.prefer_codec, "avc1");
        assert!(c.player.hwdec);
        assert_eq!(c.player.language, Language::Subtitles);
        assert_eq!(c.player.sub_lang, "es");
        assert!(c.ui.sidebar);
        assert_eq!(c.ui.icons, IconSet::Emoji);
        assert_eq!(c.ui.lang, None, "follows the system until chosen");
        assert_eq!(c.effects, EffectsConfig { startup: true, pages: true, dialogs: true });
        assert_eq!(c.search.results, 60);
        assert_eq!(c.download.dir, "");
        assert!(c.ui.hide_later, "Watch Later is left out of My Recommendations");
        let off = Config::from_toml("[ui]\nhide_later = false\n").unwrap();
        assert!(!off.ui.hide_later);
    }

    #[test]
    fn donations_skip_empty_wallets() {
        let text = "[[wallet]]\nnetwork = \"USDT (TRC20)\"\naddress = \"TXyz\"\n[[wallet]]\nnetwork = \"BTC\"\naddress = \"\"\n";
        assert_eq!(parse_donations(text), vec![Wallet { network: "USDT (TRC20)".into(), address: "TXyz".into() }]);
        assert_eq!(parse_donations(""), vec![]);
        donations(); // the shipped file parses
        assert!(toml::from_str::<toml::Table>(include_str!("../donate.toml")).is_ok());
    }

    #[test]
    fn empty_file_is_default() {
        assert_eq!(Config::from_toml("").unwrap(), Config::default());
    }

    #[test]
    fn partial_file_keeps_other_defaults() {
        let c = Config::from_toml("[player]\nmode = \"window\"\nmax_height = 480\n").unwrap();
        assert_eq!(c.player.mode, PlayerMode::Window);
        assert_eq!(c.player.max_height, 480);
        assert_eq!(c.player.prefer_codec, "avc1");
        assert_eq!(c.ui, UiConfig::default());
    }

    #[test]
    fn enums_use_lowercase_names() {
        let c = Config::from_toml("[ui]\nicons = \"nerd\"\nlang = \"en\"\n[player]\nlanguage = \"dubbed\"\n").unwrap();
        assert_eq!(c.ui.icons, IconSet::Nerd);
        assert_eq!(c.ui.lang, Some(Lang::En));
        assert_eq!(c.player.language, Language::Dubbed);
        assert_eq!(Language::Dubbed.as_str(), "dubbed");
    }

    #[test]
    fn cycles_wrap_around() {
        assert_eq!(PlayerMode::Auto.next().next().next().next(), PlayerMode::Auto);
        assert_eq!(PlayerMode::Window.next().as_str(), "overlay");
        assert_eq!(PlayerMode::Embedded.as_str(), "embedded");
        assert_eq!(IconSet::Emoji.next(), IconSet::Nerd);
        assert_eq!(IconSet::Unicode.next(), IconSet::Emoji);
        assert_eq!(IconSet::Nerd.as_str(), "nerd");
        assert_eq!(next_quality(480), 720);
        assert_eq!(next_quality(1440), 360);
        assert_eq!(next_codec("avc1"), "vp");
        assert_eq!(next_codec(""), "avc1");
        assert_eq!(next_codec("hev1"), "avc1");
        assert_eq!(codec_name("", Lang::En), "any");
        assert_eq!(codec_name("av01", Lang::Es), "AV1");
        assert_eq!(next_quality(600), 720);
    }

    #[test]
    fn languages_cycle_and_have_names() {
        assert_eq!(next_language("es"), "en");
        assert_eq!(next_language("ru"), "es");
        assert_eq!(next_language("xx"), "es");
        assert_eq!(language_name("pt", Lang::Es), "Portugués (pt)");
        assert_eq!(language_name("pt", Lang::En), "Portuguese (pt)");
        assert_eq!(language_name("eu", Lang::En), "eu");
    }

    #[test]
    fn media_folders_are_lists() {
        let c = Config::from_toml("[media]\nvideo_dirs = [\"~/Videos\", \"/mnt/pelis\"]\n").unwrap();
        assert_eq!(c.media.video_dirs, ["~/Videos", "/mnt/pelis"]);
        assert!(c.media.music_dirs.is_empty());
        let path = temp_dir("lists").join("config.toml");
        let list: toml_edit::Array = ["~/Música", "/mnt/discos"].into_iter().collect();
        set_value(&path, "media", "music_dirs", list).unwrap();
        assert_eq!(Config::load(&path).unwrap().media.music_dirs, ["~/Música", "/mnt/discos"]);
    }

    #[test]
    fn set_value_writes_numbers_as_numbers() {
        let path = temp_dir("number").join("config.toml");
        set_value(&path, "player", "max_height", 1080_i64).unwrap();
        assert_eq!(Config::load(&path).unwrap().player.max_height, 1080);
    }

    #[test]
    fn language_cycles() {
        assert_eq!(Language::Subtitles.next(), Language::Original);
        assert_eq!(Language::Original.next(), Language::Dubbed);
        assert_eq!(Language::Dubbed.next(), Language::Subtitles);
    }

    #[test]
    fn typos_are_skipped_and_reported() {
        let (c, w) = Config::parse("[player]\nmax_heigth = 480\nhwdec = false\n").unwrap();
        assert_eq!(w, [ConfigWarning::Unknown("player.max_heigth".into())]);
        assert_eq!(c.player.max_height, 720, "the typo is not applied");
        assert!(!c.player.hwdec, "the rest still is");
        let (c, w) = Config::parse("[player]\nmode = \"fullscreen\"\nmax_height = 480\n").unwrap();
        assert!(matches!(&w[..], [ConfigWarning::BadValue { key, .. }] if key == "player.mode"), "{w:?}");
        assert_eq!((c.player.mode, c.player.max_height), (PlayerMode::Window, 480));
    }

    #[test]
    fn border_style_reads_writes_and_tolerates_bad_values() {
        assert_eq!(Config::default().ui.border_style, BorderStyle::Plain, "the look it always had");
        for style in BorderStyle::ALL {
            let (c, w) = Config::parse(&format!("[ui]\nborder_style = \"{}\"\n", style.as_str())).unwrap();
            assert_eq!((c.ui.border_style, w.len()), (style, 0));
        }
        let (c, w) = Config::parse("[ui]\nborder_style = \"wavy\"\nicons = \"nerd\"\n").unwrap();
        assert_eq!(c.ui.border_style, BorderStyle::Plain, "the default");
        assert_eq!(c.ui.icons, IconSet::Nerd, "the rest of [ui] still loads");
        assert!(matches!(&w[..], [ConfigWarning::BadValue { key, .. }] if key == "ui.border_style"), "{w:?}");
        assert_eq!(BorderStyle::None.next(), BorderStyle::Plain);
        assert_eq!(BorderStyle::Thick.label(Lang::En), "Thick");
    }

    #[test]
    fn a_newer_config_still_opens() {
        // What an older tubevibe sees in a config written by a newer one:
        // a key it never heard of, a whole new section, and a new top-level key.
        let text = "[ui]\nicons = \"nerd\"\nfuture_glow = true\n\n[future]\nlevel = 3\n\n[player]\nmax_height = 1080\n";
        let (c, w) = Config::parse(&format!("version = 2\n{text}")).unwrap();
        assert_eq!(c.ui.icons, IconSet::Nerd);
        assert_eq!(c.player.max_height, 1080);
        assert_eq!(
            w,
            [
                ConfigWarning::Unknown("version".into()),
                ConfigWarning::Unknown("ui.future_glow".into()),
                ConfigWarning::Unknown("future".into()),
            ]
        );
        assert_eq!(Config::from_toml(text).unwrap().ui.icons, IconSet::Nerd);
        let path = temp_dir("newer").join("config.toml");
        std::fs::write(&path, text).unwrap();
        let (c, w) = Config::load_with_warnings(&path).unwrap();
        assert_eq!((c.ui.icons, w.len()), (IconSet::Nerd, 2));
    }

    #[test]
    fn wrong_types_fall_back_to_their_default() {
        let (c, w) = Config::parse("[ui]\nsidebar = 3\nlang = \"klingon\"\nmouse = false\nplayer = 1\n").unwrap();
        assert_eq!((c.ui.sidebar, c.ui.lang, c.ui.mouse), (true, None, false));
        let keys: Vec<_> = w
            .iter()
            .map(|w| match w {
                ConfigWarning::BadValue { key, .. } | ConfigWarning::Unknown(key) => key.as_str(),
            })
            .collect();
        assert_eq!(keys, ["ui.sidebar", "ui.lang", "ui.player"]);
        // A section that is not a table.
        let (c, w) = Config::parse("ui = 5\n").unwrap();
        assert_eq!(c, Config::default());
        assert!(matches!(&w[..], [ConfigWarning::BadValue { key, .. }] if key == "ui"));
    }

    #[test]
    fn warnings_read_as_one_status_line() {
        let unknown = ConfigWarning::Unknown("ui.foo".into());
        let bad = ConfigWarning::BadValue { key: "player.max_height".into(), error: "invalid type".into() };
        assert_eq!(ConfigWarning::summary(&[], Lang::En, "⚠"), None);
        assert_eq!(
            ConfigWarning::summary(std::slice::from_ref(&unknown), Lang::En, "⚠").unwrap(),
            "⚠ Ignored unknown setting ui.foo in config.toml (made by a newer tubevibe?)"
        );
        assert_eq!(
            ConfigWarning::summary(std::slice::from_ref(&unknown), Lang::Es, "⚠").unwrap(),
            "⚠ Se ignoró el ajuste desconocido ui.foo de config.toml (¿de un tubevibe más nuevo?)"
        );
        assert_eq!(
            ConfigWarning::summary(&[bad, unknown], Lang::En, "⚠").unwrap(),
            "⚠ Bad value for player.max_height in config.toml: using the default (+1 more)"
        );
    }

    #[test]
    fn set_value_keeps_unknown_settings() {
        let path = temp_dir("unknown").join("config.toml");
        std::fs::write(&path, "[ui]\nfuture_glow = true\n\n[future]\nlevel = 3 # new\n").unwrap();
        set_value(&path, "ui", "icons", "nerd").unwrap();
        let text = std::fs::read_to_string(&path).unwrap();
        assert!(text.contains("future_glow = true") && text.contains("[future]\nlevel = 3 # new"), "{text}");
        assert_eq!(Config::from_toml(&text).unwrap().ui.icons, IconSet::Nerd);
    }

    #[test]
    fn donations_ignore_unknown_wallet_fields() {
        let wallets = parse_donations("[[wallet]]\nnetwork = \"BTC\"\naddress = \"bc1q\"\nlabel = \"new\"\n");
        assert_eq!(wallets, [Wallet { network: "BTC".into(), address: "bc1q".into() }]);
    }

    #[test]
    fn serializes_back_to_equal_config() {
        let c = Config::default();
        let text = toml::to_string(&c).unwrap();
        assert_eq!(Config::from_toml(&text).unwrap(), c);
    }

    #[test]
    fn missing_file_loads_defaults() {
        let c = Config::load(Path::new("/definitely/not/here/config.toml")).unwrap();
        assert_eq!(c, Config::default());
    }

    #[test]
    fn invalid_file_reports_path() {
        let path = temp_dir("invalid").join("config.toml");
        std::fs::write(&path, "[ui\nsidebar = true\n").unwrap();
        let err = Config::load(&path).unwrap_err().to_string();
        assert!(err.contains("config.toml"), "{err}");
    }

    #[test]
    fn default_path_ends_in_config_toml() {
        if let Some(p) = Config::default_path() {
            assert!(p.ends_with("config.toml") && p.to_string_lossy().contains("tubevibe"), "{p:?}");
        }
    }

    #[test]
    fn set_value_creates_missing_file() {
        let path = temp_dir("create").join("sub/config.toml");
        set_value(&path, "session", "browser", "brave").unwrap();
        assert_eq!(Config::load(&path).unwrap().session.browser, "brave");
    }

    #[test]
    fn set_value_keeps_comments_and_other_keys() {
        let path = temp_dir("keep").join("config.toml");
        std::fs::write(&path, "# mi config\n[player]\nmax_height = 480 # bajo consumo\n").unwrap();
        set_value(&path, "player", "language", "dubbed").unwrap();
        let text = std::fs::read_to_string(&path).unwrap();
        assert!(text.contains("# mi config") && text.contains("# bajo consumo"), "{text}");
        let c = Config::from_toml(&text).unwrap();
        assert_eq!(c.player.max_height, 480);
        assert_eq!(c.player.language, Language::Dubbed);
    }

    #[test]
    fn an_old_max_tabs_line_is_ignored_quietly() {
        let (config, warnings) = Config::parse("[ui]\nmax_tabs = 3\nsidebar = false\n").unwrap();
        assert!(warnings.is_empty(), "{warnings:?}");
        assert!(!config.ui.sidebar, "the rest still applies");
    }

    #[test]
    fn sizes_load_tolerantly() {
        let (c, w) = Config::parse("[ui]\nmenu_width = 30\npreview_width = 70\ncomments_width = 78\n").unwrap();
        assert!(w.is_empty(), "{w:?}");
        assert_eq!((c.ui.menu_width, c.ui.preview_width, c.ui.comments_width), (30, 70, 78));
        // The first try's percent: read as columns, no warning.
        let (c, w) = Config::parse("[ui]\ncomments_width = 45\n").unwrap();
        assert!(w.is_empty() && c.ui.comments_width == 45, "{w:?}");
        let (c, w) =
            Config::parse("[ui]\nmenu_width = 5\npreview_width = \"wide\"\ncomments_width = 200\nmouse = false\n")
                .unwrap();
        assert_eq!(w.len(), 3, "{w:?}");
        assert_eq!((c.ui.menu_width, c.ui.preview_width, c.ui.comments_width), (26, 46, 46));
        assert!(!c.ui.mouse, "the rest still applies");
        let d = UiConfig::default();
        assert_eq!((d.menu_width, d.preview_width, d.comments_width), (26, 46, 46), "the old layout");
        assert_eq!(MENU_WIDTH.next(40), 20);
        assert_eq!(MENU_WIDTH.next(26), 28);
        assert_eq!(COMMENTS_WIDTH.next(78), 30);
        assert_eq!(COMMENTS_WIDTH.next(45), 49, "an old percent steps on");
    }

    #[test]
    fn thumbnails_read_old_bools_and_new_modes() {
        let ui = |t: &str| Config::parse(&format!("[ui]\n{t}\n")).unwrap();
        assert_eq!(Config::default().ui.thumbnails, ThumbMode::Picture);
        assert_eq!(ui("thumbnails = true").0.ui.thumbnails, ThumbMode::Picture);
        assert_eq!(ui("thumbnails = false").0.ui.thumbnails, ThumbMode::Off);
        assert_eq!(ui("thumbnails = \"Blocks\"").0.ui.thumbnails, ThumbMode::Blocks);
        let (c, w) = ui("thumbnails = \"hologram\"\nthumb_pixelation = \"high\"");
        assert_eq!((c.ui.thumbnails, c.ui.thumb_pixelation, w.len()), (ThumbMode::Picture, Pixelation::High, 1));
        assert!(ThumbMode::ALL.iter().all(|m| ui(&format!("thumbnails = \"{}\"", m.as_str())).0.ui.thumbnails == *m));
        assert!(Pixelation::ALL.iter().all(|m| ui(&format!("thumb_pixelation = \"{}\"", m.as_str())).0.ui.thumb_pixelation == *m));
        assert_eq!(ThumbMode::Off.next(), ThumbMode::Picture);
        assert_eq!(Pixelation::ALL.map(Pixelation::grid), [None, Some(64), Some(32), Some(16), Some(8)]);
    }
}
