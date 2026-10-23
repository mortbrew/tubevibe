//! Settings → TUBEVIBE → App menu entry: a `tubevibe.desktop` file so app
//! launchers (walker on Omarchy, GNOME, KDE, ...) list tubevibe. It is a file
//! on disk, not a config key: the setting reads the disk every time.

use std::ffi::OsString;
use std::io;
use std::path::{Path, PathBuf};

/// The line that marks a file as tubevibe's own; only such a file is deleted.
pub const MARKER: &str = "X-Tubevibe-Generated=true";

/// `$XDG_DATA_HOME/applications`, else `~/.local/share/applications`.
/// Tests get a temp folder instead, so they never touch the real home.
pub fn applications_dir() -> Option<PathBuf> {
    if cfg!(test) {
        return Some(std::env::temp_dir().join(format!("tubevibe-test-apps-{}", std::process::id())));
    }
    applications_dir_from(std::env::var_os("XDG_DATA_HOME"), std::env::var_os("HOME"))
}

/// `applications_dir` from given `XDG_DATA_HOME` and `HOME` values; a
/// relative `XDG_DATA_HOME` is ignored, as the spec says.
pub fn applications_dir_from(xdg_data_home: Option<OsString>, home: Option<OsString>) -> Option<PathBuf> {
    let data = xdg_data_home
        .map(PathBuf::from)
        .filter(|p| p.is_absolute())
        .or_else(|| home.filter(|h| !h.is_empty()).map(|h| PathBuf::from(h).join(".local/share")))?;
    Some(data.join("applications"))
}

pub fn path_in(dir: &Path) -> PathBuf {
    dir.join("tubevibe.desktop")
}

pub fn installed(dir: &Path) -> bool {
    path_in(dir).is_file()
}

/// `exe` as one Exec argument, quoted and escaped as the desktop entry spec
/// asks: inside double quotes `"` `` ` `` `$` `\` get a backslash, then the
/// value's own escaping doubles every backslash; `%` is written `%%`.
pub fn exec_arg(exe: &Path) -> String {
    let s = exe.to_string_lossy();
    let reserved = |c: char| " \t\n\"'\\><~|&;$*?#()`".contains(c);
    let s = if s.contains(reserved) {
        let mut q = String::from("\"");
        for c in s.chars() {
            if matches!(c, '"' | '`' | '$' | '\\') {
                q.push('\\');
            }
            q.push(c);
        }
        q.push('"');
        q.replace('\\', "\\\\")
    } else {
        s.into_owned()
    };
    s.replace('%', "%%")
}

pub fn contents(exe: &Path) -> String {
    format!(
        "[Desktop Entry]\n\
         Type=Application\n\
         Name=tubevibe\n\
         Comment=YouTube in the terminal\n\
         Exec={}\n\
         Icon=multimedia-video-player\n\
         Terminal=true\n\
         Categories=AudioVideo;Video;Player;\n\
         Keywords=youtube;video;\n\
         {MARKER}\n",
        exec_arg(exe)
    )
}

/// Writes the entry (replacing one tubevibe wrote) for `exe`.
pub fn install(dir: &Path, exe: &Path) -> io::Result<PathBuf> {
    let path = path_in(dir);
    if path.exists() && !generated(&path) {
        return Err(io::Error::other("a tubevibe.desktop tubevibe didn't write is already there"));
    }
    std::fs::create_dir_all(dir)?;
    std::fs::write(&path, contents(exe))?;
    refresh(dir);
    Ok(path)
}

/// Deletes the entry, only if tubevibe wrote it.
pub fn remove(dir: &Path) -> io::Result<PathBuf> {
    let path = path_in(dir);
    if !generated(&path) {
        return Err(io::Error::other("tubevibe didn't write it, so it is left alone"));
    }
    std::fs::remove_file(&path)?;
    refresh(dir);
    Ok(path)
}

fn generated(path: &Path) -> bool {
    std::fs::read_to_string(path).is_ok_and(|s| s.lines().any(|l| l.trim() == MARKER))
}

/// Lets launchers that use the cache see the change; fine if it's missing.
fn refresh(dir: &Path) {
    let _ = std::process::Command::new("update-desktop-database")
        .arg(dir)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status();
}

/// The running binary, symlinks resolved.
pub fn current_exe() -> io::Result<PathBuf> {
    std::env::current_exe()?.canonicalize()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("tubevibe-desktop-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        dir
    }

    #[test]
    fn installs_and_removes_its_own_entry() {
        let dir = temp("own").join("applications");
        assert!(!installed(&dir));
        let path = install(&dir, Path::new("/usr/bin/tubevibe")).unwrap();
        assert!(installed(&dir));
        let text = std::fs::read_to_string(&path).unwrap();
        for line in [
            "Type=Application",
            "Name=tubevibe",
            "Exec=/usr/bin/tubevibe",
            "Terminal=true",
            "Categories=AudioVideo;Video;Player;",
            "Keywords=youtube;video;",
            MARKER,
        ] {
            assert!(text.lines().any(|l| l == line), "{line} in\n{text}");
        }
        remove(&dir).unwrap();
        assert!(!installed(&dir));
        std::fs::remove_dir_all(dir.parent().unwrap()).unwrap();
    }

    #[test]
    fn never_deletes_or_overwrites_a_file_it_didnt_write() {
        let dir = temp("theirs");
        std::fs::create_dir_all(&dir).unwrap();
        let mine = "[Desktop Entry]\nType=Application\nName=tubevibe (mine)\nExec=tubevibe\n";
        std::fs::write(path_in(&dir), mine).unwrap();
        assert!(remove(&dir).is_err());
        assert!(install(&dir, Path::new("/x/tubevibe")).is_err());
        assert_eq!(std::fs::read_to_string(path_in(&dir)).unwrap(), mine);
        assert!(remove(&temp("none")).is_err(), "nothing there");
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn the_folder_follows_xdg_data_home_else_home() {
        let os = |s: &str| Some(OsString::from(s));
        assert_eq!(applications_dir_from(os("/x/data"), os("/home/t")), Some(PathBuf::from("/x/data/applications")));
        assert_eq!(
            applications_dir_from(None, os("/home/t")),
            Some(PathBuf::from("/home/t/.local/share/applications"))
        );
        assert_eq!(
            applications_dir_from(os("relative"), os("/home/t")),
            Some(PathBuf::from("/home/t/.local/share/applications"))
        );
        assert_eq!(applications_dir_from(None, None), None);
        assert!(applications_dir().unwrap().starts_with(std::env::temp_dir()), "tests never use the real home");
    }

    #[test]
    fn contents_carry_the_marker_and_the_exe() {
        let text = contents(Path::new("/usr/bin/tubevibe"));
        assert!(text.starts_with("[Desktop Entry]\n"));
        assert!(text.contains("\nExec=/usr/bin/tubevibe\n"));
        assert!(text.contains("\nComment="));
        assert!(text.ends_with(&format!("{MARKER}\n")));
    }

    #[test]
    fn exec_paths_are_quoted_per_the_spec() {
        assert_eq!(exec_arg(Path::new("/home/t/.cargo/bin/tubevibe")), "/home/t/.cargo/bin/tubevibe");
        assert_eq!(exec_arg(Path::new("/home/t/my apps/tubevibe")), "\"/home/t/my apps/tubevibe\"");
        assert_eq!(exec_arg(Path::new("/a b/$x\"y")), r#""/a b/\\$x\\"y""#);
        assert_eq!(exec_arg(Path::new("/opt/100%/tubevibe")), "/opt/100%%/tubevibe");
    }
}
