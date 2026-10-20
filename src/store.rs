//! Local files: the "Ver después" list, my history and the cache of my channel ids.

use std::collections::HashSet;
use std::io;
use std::path::{Path, PathBuf};

use crate::domain::{Download, DownloadState, HistoryEntry, Video};

#[derive(Debug, Clone)]
pub struct Paths {
    /// `later.json`, newest first.
    pub later: PathBuf,
    /// One channel id per line.
    pub subs: PathBuf,
    /// Cookie exports.
    pub cache: PathBuf,
    /// Videos watched in tubevibe, one id per line, oldest first.
    pub watched: PathBuf,
    /// `history.json`: what I opened and where I left it, newest first.
    pub history: PathBuf,
    /// `downloads.json`: finished downloads, newest first.
    pub downloads: PathBuf,
    /// `tabs.json`: the video tabs open when the app last ran.
    pub tabs: PathBuf,
}

impl Paths {
    /// Platform data and cache dirs (`~/.local/share/tubevibe`, `~/.cache/tubevibe`).
    pub fn system() -> Option<Self> {
        let dirs = directories::ProjectDirs::from("", "", "tubevibe")?;
        Some(Self::new(dirs.data_dir(), dirs.cache_dir()))
    }

    pub fn new(data: &Path, cache: &Path) -> Self {
        Self {
            later: data.join("later.json"),
            subs: cache.join("subs.txt"),
            cache: cache.into(),
            watched: data.join("watched.txt"),
            history: data.join("history.json"),
            downloads: data.join("downloads.json"),
            tabs: data.join("tabs.json"),
        }
    }
}

pub fn load_later(path: &Path) -> io::Result<Vec<Video>> {
    match std::fs::read_to_string(path) {
        Ok(text) => serde_json::from_str(&text).map_err(io::Error::other),
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(Vec::new()),
        Err(e) => Err(e),
    }
}

pub fn save_later(path: &Path, videos: &[Video]) -> io::Result<()> {
    write_atomic(path, &serde_json::to_string_pretty(videos).map_err(io::Error::other)?)
}

pub fn load_subs(path: &Path) -> HashSet<String> {
    std::fs::read_to_string(path).map(|text| text.lines().map(str::to_owned).collect()).unwrap_or_default()
}

pub fn save_subs(path: &Path, subs: &HashSet<String>) -> io::Result<()> {
    let mut ids: Vec<&str> = subs.iter().map(String::as_str).collect();
    ids.sort_unstable();
    write_atomic(path, &(ids.join("\n") + "\n"))
}

/// My history; a damaged file is set aside (`history.json.bak`) rather than
/// overwritten on the next save.
pub fn load_history(path: &Path) -> Vec<HistoryEntry> {
    let Ok(text) = std::fs::read_to_string(path) else { return Vec::new() };
    serde_json::from_str(&text).unwrap_or_else(|_| {
        let _ = std::fs::rename(path, path.with_extension("json.bak"));
        Vec::new()
    })
}

pub fn save_history(path: &Path, history: &[HistoryEntry]) -> io::Result<()> {
    write_atomic(path, &serde_json::to_string(history).map_err(io::Error::other)?)
}

/// What `tabs.json` holds: the open tabs, in the menu's order.
#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SavedTabs {
    /// [`TABS_VERSION`]; a file from another version restores nothing.
    pub version: u32,
    /// The tab shown when it was saved (0: browsing), for the record.
    #[serde(default)]
    pub active: usize,
    #[serde(default)]
    pub tabs: Vec<SavedTab>,
}

/// One open tab: enough to list it in the menu and play it again.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SavedTab {
    pub title: String,
    /// Channel or folder.
    #[serde(default)]
    pub subtitle: String,
    /// The YouTube video; none for local files.
    #[serde(default)]
    pub video: Option<Video>,
    /// URLs or paths, played in order.
    pub queue: Vec<String>,
    #[serde(default)]
    pub audio_only: bool,
    /// The list it was opened from (`X` takes the video out of it). Unknown
    /// or damaged: none, the rest of the tab still comes back.
    #[serde(default, deserialize_with = "lenient", skip_serializing_if = "Option::is_none")]
    pub source: Option<TabSource>,
}

/// A list a tab's video can be taken out of.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum TabSource {
    /// My YouTube Watch Later.
    YouTubeLater,
    /// The local tubevibe Watch Later.
    Saved,
    /// One of my YouTube playlists (or Liked videos).
    Playlist { id: String, title: String },
}

fn lenient<'de, D: serde::Deserializer<'de>>(d: D) -> Result<Option<TabSource>, D::Error> {
    let value = <serde_json::Value as serde::Deserialize>::deserialize(d)?;
    Ok(serde_json::from_value(value).ok())
}

pub const TABS_VERSION: u32 = 1;

/// The tabs open last time. Missing: none. Another version: none, quietly.
/// Damaged: none, the file set aside as `tabs.json.bak`, and why.
pub fn load_tabs(path: &Path) -> Result<SavedTabs, String> {
    let text = match std::fs::read_to_string(path) {
        Ok(text) => text,
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(SavedTabs::default()),
        Err(e) => return Err(format!("{}: {e}", path.display())),
    };
    let version = serde_json::from_str::<serde_json::Value>(&text).ok().and_then(|v| v["version"].as_u64());
    if version.is_some_and(|v| v != u64::from(TABS_VERSION)) {
        return Ok(SavedTabs::default());
    }
    serde_json::from_str::<SavedTabs>(&text).map_err(|e| {
        let _ = std::fs::rename(path, path.with_extension("json.bak"));
        format!("{}: {e}", path.display())
    })
}

pub fn save_tabs(path: &Path, tabs: &SavedTabs) -> io::Result<()> {
    write_atomic(path, &serde_json::to_string_pretty(tabs).map_err(io::Error::other)?)
}

/// Finished downloads whose file is still there.
pub fn load_downloads(path: &Path) -> Vec<Download> {
    let text = std::fs::read_to_string(path).unwrap_or_default();
    let downloads: Vec<Download> = serde_json::from_str(&text).unwrap_or_default();
    downloads.into_iter().filter(|d| matches!(&d.state, DownloadState::Done(file) if file.exists())).collect()
}

/// Keeps the finished ones only: the rest means nothing after a restart.
pub fn save_downloads(path: &Path, downloads: &[Download]) -> io::Result<()> {
    let done: Vec<&Download> = downloads.iter().filter(|d| matches!(d.state, DownloadState::Done(_))).collect();
    write_atomic(path, &serde_json::to_string(&done).map_err(io::Error::other)?)
}

/// Ids kept in `watched.txt`; older ones are dropped beyond this.
const WATCHED_MAX: usize = 5000;

pub fn load_watched(path: &Path) -> HashSet<String> {
    load_subs(path)
}

/// Adds `id` to `watched.txt`, keeping the newest [`WATCHED_MAX`].
pub fn add_watched(path: &Path, id: &str) -> io::Result<()> {
    let text = std::fs::read_to_string(path).unwrap_or_default();
    let mut ids: Vec<&str> = text.lines().filter(|line| !line.is_empty() && *line != id).collect();
    ids.push(id);
    let keep = &ids[ids.len().saturating_sub(WATCHED_MAX)..];
    write_atomic(path, &(keep.join("\n") + "\n"))
}

/// Reads the "Ver después" list of the old `ytv` script
/// (`id, canal, título, duración, url del canal, fecha, id del canal` per
/// line, oldest first) and returns it newest first.
pub fn parse_ytv_later(text: &str) -> Vec<Video> {
    let clean = |s: &str| strip_ansi(s).trim().to_owned();
    let mut videos: Vec<Video> = text
        .lines()
        .filter_map(|line| {
            let fields: Vec<String> = line.split('\t').map(clean).collect();
            let field = |i: usize| fields.get(i).filter(|f| !f.is_empty() && *f != "-").cloned();
            Some(Video {
                id: field(0)?,
                title: field(2)?,
                channel: field(1).unwrap_or_default(),
                channel_id: field(6),
                channel_url: field(4),
                duration: field(3).as_deref().and_then(parse_duration),
                published: None,
                live: Default::default(),
                short: false,
            })
        })
        .collect();
    videos.reverse();
    videos
}

/// `"1:02:03"` / `"23:33"` → seconds.
fn parse_duration(text: &str) -> Option<u32> {
    text.split(':').try_fold(0u32, |total, part| Some(total * 60 + part.parse::<u32>().ok()?))
}

fn strip_ansi(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut chars = text.chars();
    while let Some(c) = chars.next() {
        if c == '\u{1b}' {
            // Skip `ESC [ ... letter`.
            chars.by_ref().take_while(|c| !c.is_ascii_alphabetic()).for_each(drop);
        } else {
            out.push(c);
        }
    }
    out
}

/// Write to a sibling temp file and rename, so a crash never leaves half a file.
fn write_atomic(path: &Path, content: &str) -> io::Result<()> {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    let tmp = path.with_extension("tmp");
    std::fs::write(&tmp, content)?;
    std::fs::rename(tmp, path)
}

/// The original app's name: its folders are copied over on the first run.
pub const OLD_NAME: &str = "tubetui";
pub const NAME: &str = "tubevibe";

/// `(old, new)` pairs of the platform config, data and cache folders
/// (`~/.config/tubetui` → `~/.config/tubevibe`, and so on), each once.
pub fn renamed_dirs() -> Vec<(PathBuf, PathBuf)> {
    let (Some(old), Some(new)) =
        (directories::ProjectDirs::from("", "", OLD_NAME), directories::ProjectDirs::from("", "", NAME))
    else {
        return Vec::new();
    };
    let mut pairs: Vec<(PathBuf, PathBuf)> = Vec::new();
    for (o, n) in
        [(old.config_dir(), new.config_dir()), (old.data_dir(), new.data_dir()), (old.cache_dir(), new.cache_dir())]
    {
        if !pairs.iter().any(|(_, seen)| seen == n) {
            pairs.push((o.to_path_buf(), n.to_path_buf()));
        }
    }
    pairs
}

/// Copies (never moves) each old folder to its new place when the new one
/// does not exist yet and the old one does. Returns the folders copied.
/// A copy that fails halfway is removed, so the next run tries again.
pub fn copy_old_dirs(pairs: &[(PathBuf, PathBuf)]) -> io::Result<Vec<(PathBuf, PathBuf)>> {
    let mut copied = Vec::new();
    for (old, new) in pairs {
        if new.exists() || !old.is_dir() {
            continue;
        }
        if let Err(e) = copy_dir(old, new) {
            let _ = std::fs::remove_dir_all(new);
            return Err(e);
        }
        copied.push((old.clone(), new.clone()));
    }
    Ok(copied)
}

/// `from` and everything in it into `to` (files keep their permissions,
/// e.g. the private cookie copies). Links and sockets are left out.
fn copy_dir(from: &Path, to: &Path) -> io::Result<()> {
    std::fs::create_dir_all(to)?;
    for entry in std::fs::read_dir(from)? {
        let entry = entry?;
        let kind = entry.file_type()?;
        let target = to.join(entry.file_name());
        if kind.is_dir() {
            copy_dir(&entry.path(), &target)?;
        } else if kind.is_file() {
            std::fs::copy(entry.path(), &target)?;
        }
    }
    Ok(())
}

/// A path inside an old folder, moved to the matching new folder (the
/// imported cookies file named in `config.toml`, say).
pub fn moved_path(path: &Path, pairs: &[(PathBuf, PathBuf)]) -> Option<PathBuf> {
    pairs.iter().find_map(|(old, new)| path.strip_prefix(old).ok().map(|rest| new.join(rest)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::temp_dir;
    use crate::domain::video;

    #[test]
    fn watched_ids_are_remembered_once() {
        let path = temp_dir("watched").join("data/watched.txt");
        assert!(load_watched(&path).is_empty());
        add_watched(&path, "a").unwrap();
        add_watched(&path, "b").unwrap();
        add_watched(&path, "a").unwrap();
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "b\na\n", "no repeats, newest last");
        assert_eq!(load_watched(&path), HashSet::from(["a".to_string(), "b".to_string()]));
    }

    #[test]
    fn history_round_trips_and_a_damaged_file_is_kept_aside() {
        let path = temp_dir("history").join("data/history.json");
        assert!(load_history(&path).is_empty());
        let entry = HistoryEntry { video: video("a"), at: 5, position: Some(70) };
        save_history(&path, std::slice::from_ref(&entry)).unwrap();
        assert_eq!(load_history(&path), vec![entry]);
        std::fs::write(&path, "[oops").unwrap();
        assert!(load_history(&path).is_empty());
        assert_eq!(std::fs::read_to_string(path.with_extension("json.bak")).unwrap(), "[oops");
    }

    #[test]
    fn downloads_keep_the_finished_ones_still_on_disk() {
        let dir = temp_dir("downloads");
        let path = dir.join("data/downloads.json");
        let file = dir.join("a.mp4");
        std::fs::write(&file, "x").unwrap();
        let download = |id, state| Download { id, video: video("a"), state };
        let list = [
            download(1, DownloadState::Done(file.clone())),
            download(2, DownloadState::Done(dir.join("gone.mp4"))),
            download(3, DownloadState::Failed("x".into())),
            download(4, DownloadState::Running { permille: 5, speed: String::new(), eta: String::new() }),
        ];
        save_downloads(&path, &list).unwrap();
        assert_eq!(load_downloads(&path), [download(1, DownloadState::Done(file))]);
    }

    #[test]
    fn later_round_trips_and_missing_is_empty() {
        let path = temp_dir("later").join("data/later.json");
        assert!(load_later(&path).unwrap().is_empty());
        save_later(&path, &[video("a"), video("b")]).unwrap();
        assert_eq!(load_later(&path).unwrap(), vec![video("a"), video("b")]);
    }

    #[test]
    fn corrupt_later_is_an_error_not_an_empty_list() {
        // Silently treating it as empty would wipe the list on the next save.
        let path = temp_dir("corrupt").join("later.json");
        std::fs::write(&path, "{oops").unwrap();
        assert!(load_later(&path).is_err());
    }

    #[test]
    fn imports_ytv_later_list_newest_first() {
        let text = "old1\t\u{1b}[35mCanal A   \u{1b}[0m\tViejo\t\u{1b}[90m1:02:03\u{1b}[0m\thttps://www.youtube.com/@a\t06/10/26 3d\tUCa\n\
                    new2\t-                   \tNuevo\t23:33\t\t\t\n\
                    broken line\n";
        let videos = parse_ytv_later(text);
        assert_eq!(videos.len(), 2);
        assert_eq!(videos[0].id, "new2");
        assert_eq!(videos[0].channel, "");
        assert_eq!(videos[0].channel_id, None);
        assert_eq!(videos[0].duration, Some(23 * 60 + 33));
        assert_eq!(videos[1].channel, "Canal A");
        assert_eq!(videos[1].channel_id.as_deref(), Some("UCa"));
        assert_eq!(videos[1].channel_url.as_deref(), Some("https://www.youtube.com/@a"));
        assert_eq!(videos[1].duration, Some(3723));
    }

    #[test]
    fn tabs_round_trip_and_tolerate_bad_files() {
        let path = temp_dir("tabs").join("data/tabs.json");
        assert_eq!(load_tabs(&path), Ok(SavedTabs::default()), "no file: no tabs");
        let tabs = SavedTabs {
            version: TABS_VERSION,
            active: 2,
            tabs: vec![
                SavedTab {
                    title: "title a".into(),
                    subtitle: "chan a".into(),
                    video: Some(video("a")),
                    queue: vec![video("a").url()],
                    audio_only: false,
                    source: Some(TabSource::Playlist { id: "PLx".into(), title: "Mine".into() }),
                },
                SavedTab {
                    title: "song.mp3".into(),
                    subtitle: "~/Music".into(),
                    video: None,
                    queue: vec!["/m/song.mp3".into(), "/m/next.mp3".into()],
                    audio_only: true,
                    source: None,
                },
            ],
        };
        save_tabs(&path, &tabs).unwrap();
        assert_eq!(load_tabs(&path), Ok(tabs));
        // Another version's file: nothing, and it stays as it is.
        std::fs::write(&path, r#"{"version": 9, "tabs": "who knows"}"#).unwrap();
        assert_eq!(load_tabs(&path), Ok(SavedTabs::default()));
        assert!(path.exists());
        // Damaged: nothing, a reason, and the file set aside.
        std::fs::write(&path, "{ not json").unwrap();
        let error = load_tabs(&path).unwrap_err();
        assert!(error.contains("tabs.json"), "{error}");
        assert!(!path.exists() && path.with_extension("json.bak").exists());
        // Unknown fields from a newer tubevibe are fine.
        std::fs::write(&path, r#"{"version": 1, "glow": true, "tabs": [{"title": "t", "queue": ["u"], "x": 1}]}"#)
            .unwrap();
        assert_eq!(load_tabs(&path).unwrap().tabs.len(), 1);
    }

    #[test]
    fn subs_round_trip() {
        let path = temp_dir("subs").join("subs.txt");
        assert!(load_subs(&path).is_empty());
        let subs = HashSet::from(["UCb".to_string(), "UCa".to_string()]);
        save_subs(&path, &subs).unwrap();
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "UCa\nUCb\n");
        assert_eq!(load_subs(&path), subs);
    }

    #[test]
    fn old_folders_are_copied_once_and_kept() {
        let root = temp_dir("rename");
        let (old, new) = (root.join("tubetui"), root.join("tubevibe"));
        std::fs::create_dir_all(old.join("data")).unwrap();
        std::fs::write(old.join("config.toml"), "[ui]\nlang = \"en\"\n").unwrap();
        std::fs::write(old.join("data/later.json"), "[]").unwrap();
        let pairs = vec![(old.clone(), new.clone()), (root.join("missing"), root.join("other"))];
        assert_eq!(copy_old_dirs(&pairs).unwrap(), vec![(old.clone(), new.clone())]);
        assert_eq!(std::fs::read_to_string(new.join("data/later.json")).unwrap(), "[]");
        assert!(old.join("config.toml").exists(), "copied, not moved");
        assert!(!root.join("other").exists(), "nothing to copy: nothing made");
        // The new folder exists now: never copied over again.
        std::fs::write(new.join("config.toml"), "changed").unwrap();
        assert!(copy_old_dirs(&pairs).unwrap().is_empty());
        assert_eq!(std::fs::read_to_string(new.join("config.toml")).unwrap(), "changed");
        assert_eq!(moved_path(&old.join("cookies-imported.txt"), &pairs), Some(new.join("cookies-imported.txt")));
        assert_eq!(moved_path(Path::new("/elsewhere/c.txt"), &pairs), None);
    }

    #[test]
    fn a_tab_source_it_does_not_know_is_just_dropped() {
        let path = temp_dir("tabs-source").join("tabs.json");
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        let text = r#"{"version":1,"tabs":[{"title":"t","queue":["u"],"source":{"kind":"martian"}},
            {"title":"u","queue":["v"],"source":{"kind":"saved"}}]}"#;
        std::fs::write(&path, text).unwrap();
        let tabs = load_tabs(&path).unwrap().tabs;
        assert_eq!((tabs[0].source.clone(), tabs[1].source.clone()), (None, Some(TabSource::Saved)));
    }
}
