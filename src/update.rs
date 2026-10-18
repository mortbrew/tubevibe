//! Is there a newer release? Asks GitHub each time the app opens (in the
//! background) and, when there is one, says how to get it on this machine:
//! the package that fits how tubevibe was installed here, or building it from
//! source when none does. Where the release archive fits and the program's
//! folder is writable, About's Update Now button downloads it and puts the
//! new program in place of the running one ([`install`]).

use std::env::consts::{ARCH, OS};
use std::io::Read;
use std::path::{Path, PathBuf};
use std::time::Duration;

use serde_json::Value;

pub const REPO: &str = "https://github.com/mortbrew/tubevibe";
const LATEST: &str = "https://api.github.com/repos/mortbrew/tubevibe/releases/latest";
const DOWNLOAD: &str = "https://github.com/mortbrew/tubevibe/releases/download";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Update {
    /// `0.5.2`.
    pub version: String,
    pub how: Install,
    /// Whether Update Now can put the new version in place here, or why not.
    pub replace: Replace,
}

/// Can tubevibe update itself in place, and if not, why.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Replace {
    /// Download this archive and swap its program in for the running one.
    Ready(String),
    /// Running from a cargo `target` folder: build the new version instead.
    FromSource,
    /// Installed with `cargo install`: reinstall the same way.
    Cargo,
    /// Installed by a package manager (`/usr/bin`, Homebrew…): update the package.
    Package,
    /// The program's folder can't be written to.
    ReadOnly,
    /// No release file for this machine.
    NoAsset,
}

/// Where a self-update stands, shown in the status bar and "Acerca de".
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum Progress {
    #[default]
    Idle,
    /// Downloading: percent done.
    Downloading(u8),
    /// The new version is in place: it runs from the next start.
    Done(String),
    Failed(String),
}

/// The version to compare releases with: this build's, or
/// `TUBEVIBE_FAKE_VERSION` (to try the update flow against an existing
/// release, e.g. `TUBEVIBE_FAKE_VERSION=0.0.1 tubevibe`).
fn current_version() -> String {
    std::env::var("TUBEVIBE_FAKE_VERSION")
        .ok()
        .filter(|v| version(v).is_some())
        .unwrap_or_else(|| env!("CARGO_PKG_VERSION").to_owned())
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Install {
    /// A command to paste in a terminal.
    Command(String),
    /// A file to download and open (the Windows installer).
    Link(String),
}

/// What asking GitHub found out.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Checked {
    /// A newer release, and how to get it here.
    Newer(Update),
    /// GitHub's latest release is this version (or older): up to date.
    Latest,
    /// No answer worth telling: offline, no release yet, rate limited, or a
    /// release whose tag isn't a version.
    Unknown,
}

/// Is there a newer release for this machine? Anything that keeps GitHub
/// from answering (no release yet, offline, rate limits) is `Unknown`.
pub fn check() -> Checked {
    let Ok(Some((tag, assets))) = fetch_latest() else { return Checked::Unknown };
    let current = current_version();
    let exe = running_exe();
    match update_for(&current, &tag, &assets, &exe.display().to_string(), OS, ARCH) {
        Some(mut update) => {
            if matches!(update.replace, Replace::Ready(_)) && !exe.parent().is_some_and(writable) {
                update.replace = Replace::ReadOnly;
            }
            Checked::Newer(update)
        }
        None => latest_or_unknown(&current, &tag),
    }
}

/// No newer release: up to date when the tag really is a version.
fn latest_or_unknown(current: &str, tag: &str) -> Checked {
    match (version(tag), version(current)) {
        (Some(latest), Some(mine)) if latest <= mine => Checked::Latest,
        _ => Checked::Unknown,
    }
}

/// The program running, with links followed (a Homebrew link → its Cellar).
fn running_exe() -> PathBuf {
    let exe = std::env::current_exe().unwrap_or_default();
    std::fs::canonicalize(&exe).unwrap_or(exe)
}

/// A new file can be made (and so renamed over the program) in `dir`.
fn writable(dir: &Path) -> bool {
    let probe = dir.join(format!(".tubevibe-write-test-{}", std::process::id()));
    let ok = std::fs::File::create(&probe).is_ok();
    let _ = std::fs::remove_file(&probe);
    ok
}

/// The latest release's tag and files; `None` when GitHub has no answer for
/// us: no release yet (404), or rate limited (403, 429).
fn fetch_latest() -> Result<Option<(String, Vec<String>)>, String> {
    let agent: ureq::Agent = ureq::Agent::config_builder().timeout_global(Some(Duration::from_secs(10))).build().into();
    let reply = match agent
        .get(LATEST)
        .header("User-Agent", concat!("tubevibe/", env!("CARGO_PKG_VERSION")))
        .header("Accept", "application/vnd.github+json")
        .call()
    {
        Ok(reply) => reply,
        Err(ureq::Error::StatusCode(status)) if no_news(status) => return Ok(None),
        Err(e) => return Err(e.to_string()),
    };
    let text = reply.into_body().read_to_string().map_err(|e| e.to_string())?;
    let json: Value = serde_json::from_str(&text).map_err(|e| e.to_string())?;
    parse_latest(&json).map(Some).ok_or_else(|| "respuesta de GitHub sin versión".into())
}

/// GitHub statuses that just mean "nothing to say": no release published
/// yet, or too many requests from this address for now.
fn no_news(status: u16) -> bool {
    matches!(status, 404 | 403 | 429)
}

/// Tag and asset names of GitHub's "latest release" answer.
pub fn parse_latest(json: &Value) -> Option<(String, Vec<String>)> {
    let tag = json["tag_name"].as_str()?.to_owned();
    let assets = json["assets"].as_array()?.iter().filter_map(|a| a["name"].as_str().map(str::to_owned)).collect();
    Some((tag, assets))
}

/// `1.2.3` → `(1, 2, 3)`; a leading `v` and anything after `-` are ignored.
fn version(text: &str) -> Option<(u64, u64, u64)> {
    let core = text.trim_start_matches('v').split(['-', '+']).next()?;
    let mut parts = core.split('.').map(|p| p.parse().ok());
    Some((parts.next()??, parts.next().flatten().unwrap_or(0), parts.next().flatten().unwrap_or(0)))
}

/// The release file for a machine, as the release workflow names it.
pub fn tarball(os: &str, arch: &str) -> Option<&'static str> {
    match (os, arch) {
        ("linux", "x86_64") => Some("tubevibe-x86_64-linux.tar.gz"),
        ("linux", "aarch64") => Some("tubevibe-aarch64-linux.tar.gz"),
        ("macos", "aarch64") => Some("tubevibe-aarch64-macos.tar.gz"),
        _ => None,
    }
}

/// The release file Update Now installs on a machine: the archive with just the program.
pub fn asset(os: &str, arch: &str) -> Option<&'static str> {
    match (os, arch) {
        ("windows", "x86_64") => Some("tubevibe-x86_64-windows.zip"),
        _ => tarball(os, arch),
    }
}

/// Built with cargo in a checkout: `…/target/release/`, `…/target/debug/`
/// (or `…/target/<triple>/release/`).
pub fn is_dev_build(exe: &str) -> bool {
    let parts: Vec<&str> = exe.split(['/', '\\']).collect();
    parts.iter().position(|p| *p == "target").is_some_and(|t| {
        let between = parts.get(t + 1..parts.len().saturating_sub(1)).unwrap_or_default();
        between.iter().any(|p| matches!(*p, "release" | "debug"))
    })
}

fn from_cargo(exe: &str) -> bool {
    exe.contains(".cargo/bin") || exe.contains(r".cargo\bin")
}

/// Put there by a package manager, which should be the one to update it.
pub fn is_package(exe: &str) -> bool {
    const MANAGED: &[&str] = &[
        "/usr/bin/",
        "/usr/sbin/",
        "/bin/",
        "/usr/lib/",
        "/usr/share/",
        "/nix/store/",
        "/snap/",
        "/opt/homebrew/",
        "/usr/local/Cellar/",
        "/home/linuxbrew/",
    ];
    MANAGED.iter().any(|dir| exe.starts_with(dir))
}

/// Whether Update Now may replace `exe` with the release's archive for (`os`, `arch`).
/// (Whether its folder is writable is checked apart, on the real machine.)
pub fn replace_for(exe: &str, os: &str, arch: &str, assets: &[String], tag: &str) -> Replace {
    if is_dev_build(exe) {
        Replace::FromSource
    } else if from_cargo(exe) {
        Replace::Cargo
    } else if is_package(exe) {
        Replace::Package
    } else {
        match asset(os, arch).filter(|name| assets.iter().any(|a| a == name)) {
            Some(name) => Replace::Ready(format!("{DOWNLOAD}/{tag}/{name}")),
            None => Replace::NoAsset,
        }
    }
}

/// The update for a machine (`os`, `arch`) running `current` from `exe`,
/// when `tag` is newer.
pub fn update_for(current: &str, tag: &str, assets: &[String], exe: &str, os: &str, arch: &str) -> Option<Update> {
    if version(tag)? <= version(current)? {
        return None;
    }
    let has = |name: &str| assets.iter().any(|a| a == name);
    let url = |name: &str| format!("{DOWNLOAD}/{tag}/{name}");
    let cargo = Install::Command(format!("cargo install --locked --git {REPO} --tag {tag}"));
    let from_cargo = from_cargo(exe);
    // Built here with `cargo build --release`: build the new version the same way.
    let checkout =
        exe.strip_suffix("/target/release/tubevibe").or_else(|| exe.strip_suffix(r"\target\release\tubevibe.exe"));
    let how = match (os, arch) {
        _ if from_cargo => cargo,
        _ if checkout.is_some() => Install::Command(format!(
            "cd {} && git pull && git checkout {tag} && cargo build --release",
            checkout.unwrap_or_default()
        )),
        ("linux", "x86_64") if exe.starts_with("/usr/bin/") && has("tubevibe_amd64.deb") => {
            Install::Command(format!("curl -LO {} && sudo apt install ./tubevibe_amd64.deb", url("tubevibe_amd64.deb")))
        }
        ("windows", "x86_64") if has("tubevibe-x86_64-windows-setup.exe") => {
            Install::Link(url("tubevibe-x86_64-windows-setup.exe"))
        }
        _ => match tarball(os, arch).filter(|name| has(name)) {
            // Only the program out of the archive, straight over the one running.
            Some(name) => {
                let dir = Path::new(exe).parent().map_or_else(|| "~/.local/bin".into(), |d| d.display().to_string());
                Install::Command(format!(
                    "curl -fsSL {} | tar xz -C {dir} --strip-components=1 tubevibe/tubevibe",
                    url(name)
                ))
            }
            None => cargo,
        },
    };
    let replace = replace_for(exe, os, arch, assets, tag);
    Some(Update { version: tag.trim_start_matches('v').to_owned(), how, replace })
}

/// Largest download accepted (the archives are a few MB).
const MAX_DOWNLOAD: u64 = 200 * 1024 * 1024;

/// Downloads the archive at `url`, takes the program out of it, checks that it
/// runs and says it is `version`, and swaps it in for the running program
/// (a rename, so a failure halfway leaves the old one). `progress` hears the
/// percent downloaded. Returns where the program was put.
pub fn install(url: &str, version: &str, mut progress: impl FnMut(u8)) -> Result<PathBuf, String> {
    let exe = running_exe();
    let shown = exe.display().to_string();
    if is_dev_build(&shown) {
        return Err("built from source: git pull && cargo build --release".into());
    }
    let dir = exe.parent().ok_or("no folder for the program")?;
    let archive = download(url, &mut progress)?;
    let program = if url.ends_with(".zip") { from_zip(&archive)? } else { from_tarball(&archive)? };
    let staged = dir.join(format!(".tubevibe-update-{}{}", std::process::id(), std::env::consts::EXE_SUFFIX));
    let result = stage(&staged, &program)
        .and_then(|()| check_runs(&staged, version))
        .and_then(|()| self_replace::self_replace(&staged).map_err(|e| format!("replacing {shown}: {e}")));
    let _ = std::fs::remove_file(&staged);
    result.map(|()| exe)
}

fn download(url: &str, progress: &mut impl FnMut(u8)) -> Result<Vec<u8>, String> {
    let agent: ureq::Agent =
        ureq::Agent::config_builder().timeout_global(Some(Duration::from_secs(300))).build().into();
    let reply = agent
        .get(url)
        .header("User-Agent", concat!("tubevibe/", env!("CARGO_PKG_VERSION")))
        .call()
        .map_err(|e| format!("download: {e}"))?;
    let total = reply.body().content_length().filter(|n| *n > 0);
    let mut reader = reply.into_body().into_with_config().limit(MAX_DOWNLOAD).reader();
    let mut data = Vec::with_capacity(total.unwrap_or(0) as usize);
    let mut chunk = [0u8; 64 * 1024];
    let mut shown = None;
    loop {
        let n = reader.read(&mut chunk).map_err(|e| format!("download: {e}"))?;
        if n == 0 {
            break;
        }
        data.extend_from_slice(&chunk[..n]);
        if let Some(total) = total {
            let percent = (data.len() as u64 * 100 / total).min(100) as u8;
            if shown != Some(percent) {
                shown = Some(percent);
                progress(percent);
            }
        }
    }
    if data.is_empty() {
        return Err("download: empty file".into());
    }
    Ok(data)
}

/// The program out of a release tarball (`tubevibe/tubevibe`).
pub fn from_tarball(gz: &[u8]) -> Result<Vec<u8>, String> {
    let mut archive = tar::Archive::new(flate2::read::GzDecoder::new(gz));
    for entry in archive.entries().map_err(|e| format!("archive: {e}"))? {
        let mut entry = entry.map_err(|e| format!("archive: {e}"))?;
        let path = entry.path().map_err(|e| format!("archive: {e}"))?.into_owned();
        if entry.header().entry_type().is_file()
            && path.strip_prefix("./").unwrap_or(&path) == Path::new("tubevibe/tubevibe")
        {
            let mut program = Vec::new();
            entry.read_to_end(&mut program).map_err(|e| format!("archive: {e}"))?;
            return Ok(program);
        }
    }
    Err("archive: no tubevibe/tubevibe inside".into())
}

/// The program out of the Windows zip (`tubevibe.exe`).
pub fn from_zip(data: &[u8]) -> Result<Vec<u8>, String> {
    let mut archive = zip::ZipArchive::new(std::io::Cursor::new(data)).map_err(|e| format!("archive: {e}"))?;
    for i in 0..archive.len() {
        let mut file = archive.by_index(i).map_err(|e| format!("archive: {e}"))?;
        let name = file.name().map(|n| n.into_owned()).unwrap_or_default();
        if file.is_file() && name.rsplit(['/', '\\']).next() == Some("tubevibe.exe") {
            let mut program = Vec::new();
            file.read_to_end(&mut program).map_err(|e| format!("archive: {e}"))?;
            return Ok(program);
        }
    }
    Err("archive: no tubevibe.exe inside".into())
}

fn stage(path: &Path, program: &[u8]) -> Result<(), String> {
    std::fs::write(path, program).map_err(|e| format!("writing {}: {e}", path.display()))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o755))
            .map_err(|e| format!("making {} runnable: {e}", path.display()))?;
    }
    Ok(())
}

/// The new program starts here and says it is `version` (not a file for
/// another machine, or a broken download).
fn check_runs(program: &Path, version: &str) -> Result<(), String> {
    let out = std::process::Command::new(program)
        .arg("--version")
        .stdin(std::process::Stdio::null())
        .output()
        .map_err(|e| format!("the new version does not start here: {e}"))?;
    let said = String::from_utf8_lossy(&out.stdout);
    if out.status.success() && said.split_whitespace().any(|w| w == version) {
        Ok(())
    } else {
        Err(format!("the new version says {:?}, not {version}", said.trim()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assets() -> Vec<String> {
        [
            "tubevibe_amd64.deb",
            "tubevibe-x86_64-linux.tar.gz",
            "tubevibe-aarch64-linux.tar.gz",
            "tubevibe-x86_64-windows-setup.exe",
            "tubevibe-x86_64-windows.zip",
            "tubevibe-aarch64-macos.tar.gz",
        ]
        .map(String::from)
        .to_vec()
    }

    fn how(exe: &str, os: &str, arch: &str) -> Install {
        update_for("0.8.1", "v0.9.0", &assets(), exe, os, arch).unwrap().how
    }

    fn cmd(text: &str) -> Install {
        Install::Command(text.into())
    }

    const CARGO: &str = "cargo install --locked --git https://github.com/mortbrew/tubevibe --tag v0.9.0";

    #[test]
    fn only_newer_versions_count() {
        assert_eq!(update_for("0.8.1", "v0.8.1", &assets(), "/usr/bin/tubevibe", "linux", "x86_64"), None);
        assert_eq!(update_for("0.8.1", "v0.7.7", &assets(), "/usr/bin/tubevibe", "linux", "x86_64"), None);
        assert_eq!(update_for("0.8.1", "garbage", &assets(), "/usr/bin/tubevibe", "linux", "x86_64"), None);
        let update = update_for("0.8.1", "v0.10.0", &assets(), "/usr/bin/tubevibe", "linux", "x86_64").unwrap();
        assert_eq!(update.version, "0.10.0", "numbers, not text: 10 > 8");
        assert!(version("1.2.3-beta") == Some((1, 2, 3)) && version("v2") == Some((2, 0, 0)));
    }

    #[test]
    fn the_package_matches_how_it_was_installed() {
        assert_eq!(
            how("/usr/bin/tubevibe", "linux", "x86_64"),
            cmd("curl -LO https://github.com/mortbrew/tubevibe/releases/download/v0.9.0/tubevibe_amd64.deb \
                 && sudo apt install ./tubevibe_amd64.deb")
        );
        assert_eq!(
            how("/home/u/.local/bin/tubevibe", "linux", "x86_64"),
            cmd(
                "curl -fsSL https://github.com/mortbrew/tubevibe/releases/download/v0.9.0/tubevibe-x86_64-linux.tar.gz \
                 | tar xz -C /home/u/.local/bin --strip-components=1 tubevibe/tubevibe"
            )
        );
        let Install::Command(arm) = how("/home/u/.local/bin/tubevibe", "linux", "aarch64") else { panic!() };
        assert!(arm.contains("/v0.9.0/tubevibe-aarch64-linux.tar.gz | tar xz -C /home/u/.local/bin"), "{arm}");
        let Install::Command(mac) = how("/usr/local/bin/tubevibe", "macos", "aarch64") else { panic!() };
        assert!(mac.contains("tubevibe-aarch64-macos.tar.gz | tar xz -C /usr/local/bin"), "{mac}");
        assert_eq!(how("/home/u/.cargo/bin/tubevibe", "linux", "x86_64"), cmd(CARGO));
        assert_eq!(
            how(r"C:\Users\u\AppData\Local\Programs\tubevibe\tubevibe.exe", "windows", "x86_64"),
            Install::Link(
                "https://github.com/mortbrew/tubevibe/releases/download/v0.9.0/tubevibe-x86_64-windows-setup.exe"
                    .into()
            )
        );
        assert_eq!(how(r"C:\Users\u\.cargo\bin\tubevibe.exe", "windows", "x86_64"), cmd(CARGO));
        // Built from a checkout: build the new tag there.
        assert_eq!(
            how("/home/u/Projects/tubevibe/target/release/tubevibe", "linux", "x86_64"),
            cmd("cd /home/u/Projects/tubevibe && git pull && git checkout v0.9.0 && cargo build --release")
        );
        // No package for these machines: build it.
        assert_eq!(how("/usr/local/bin/tubevibe", "macos", "x86_64"), cmd(CARGO));
        assert_eq!(how("/usr/bin/tubevibe", "freebsd", "x86_64"), cmd(CARGO));
        // A release without the .deb: the tarball; without any file: build it.
        let only_tar = vec!["tubevibe-x86_64-linux.tar.gz".to_string()];
        let update = update_for("0.8.1", "v0.9.0", &only_tar, "/usr/bin/tubevibe", "linux", "x86_64").unwrap();
        assert!(matches!(update.how, Install::Command(c) if c.contains("tar xz -C /usr/bin")));
        let update = update_for("0.8.1", "v0.9.0", &[], "/home/u/.local/bin/tubevibe", "linux", "x86_64").unwrap();
        assert_eq!(update.how, cmd(CARGO));
    }

    #[test]
    fn asset_names_match_the_release_workflow() {
        let workflow = include_str!("../.github/workflows/release.yml");
        let names = [tarball("linux", "x86_64"), tarball("linux", "aarch64"), tarball("macos", "aarch64")];
        // The Linux job builds one archive per `arch` of its matrix.
        let linux = "dist/tubevibe-${{ matrix.arch }}-linux.tar.gz";
        let built = |name: &str| {
            workflow.contains(&format!("dist/{name}"))
                || name
                    .strip_prefix("tubevibe-")
                    .and_then(|n| n.strip_suffix("-linux.tar.gz"))
                    .is_some_and(|arch| workflow.contains(linux) && workflow.contains(&format!("- arch: {arch}\n")))
        };
        for name in names.into_iter().flatten().chain(["tubevibe_amd64.deb", "tubevibe-x86_64-windows-setup.exe"]) {
            assert!(built(name), "{name} is not built by release.yml");
        }
        assert!(!built("tubevibe-riscv64-linux.tar.gz"));
        let script = include_str!("../install.sh");
        for name in names.into_iter().flatten() {
            assert!(script.contains(name), "{name} is not in install.sh");
        }
    }

    #[test]
    fn up_to_date_only_on_a_real_version() {
        assert_eq!(latest_or_unknown("0.8.2", "v0.8.2"), Checked::Latest);
        assert_eq!(latest_or_unknown("0.8.2", "v0.8.1"), Checked::Latest, "a dev build ahead of the release");
        assert_eq!(latest_or_unknown("0.8.2", "nightly"), Checked::Unknown);
    }

    #[test]
    fn reads_githubs_answer() {
        let json = serde_json::json!({ "tag_name": "v0.6.0", "assets": [{ "name": "a.deb" }, { "name": "b.zip" }] });
        assert_eq!(parse_latest(&json), Some(("v0.6.0".into(), vec!["a.deb".into(), "b.zip".into()])));
        assert_eq!(parse_latest(&serde_json::json!({ "message": "Not Found" })), None);
        assert!(no_news(404) && no_news(403) && no_news(429), "no release, rate limits");
        assert!(!no_news(500) && !no_news(200));
    }

    #[test]
    fn the_self_update_file_fits_the_machine() {
        assert_eq!(asset("linux", "x86_64"), Some("tubevibe-x86_64-linux.tar.gz"));
        assert_eq!(asset("linux", "aarch64"), Some("tubevibe-aarch64-linux.tar.gz"));
        assert_eq!(asset("macos", "aarch64"), Some("tubevibe-aarch64-macos.tar.gz"));
        assert_eq!(asset("windows", "x86_64"), Some("tubevibe-x86_64-windows.zip"));
        assert_eq!(asset("macos", "x86_64"), None, "no Intel Mac build");
        assert_eq!(asset("windows", "aarch64"), None);
        assert_eq!(asset("freebsd", "x86_64"), None);
        let workflow = include_str!("../.github/workflows/release.yml");
        assert!(workflow.contains("dist/tubevibe-x86_64-windows.zip"), "release.yml builds the zip");
    }

    #[test]
    fn builds_from_source_are_recognized() {
        assert!(is_dev_build("/home/u/Projects/tubevibe/target/release/tubevibe"));
        assert!(is_dev_build("/home/u/Projects/tubevibe/target/debug/tubevibe"));
        assert!(is_dev_build("/home/u/tubevibe/target/x86_64-unknown-linux-gnu/release/tubevibe"));
        assert!(
            !is_dev_build("/home/u/target")
                && !is_dev_build("/home/u/target/tubevibe")
                && !is_dev_build("/home/u/release/target/tubevibe")
        );
        assert!(is_dev_build(r"C:\src\tubevibe\target\release\tubevibe.exe"));
        assert!(!is_dev_build("/home/u/.local/bin/tubevibe"));
        assert!(!is_dev_build("/home/u/targets/tubevibe"));
        assert!(is_package("/usr/bin/tubevibe") && is_package("/opt/homebrew/Cellar/tubevibe/0.8.2/bin/tubevibe"));
        assert!(!is_package("/home/u/.local/bin/tubevibe") && !is_package("/usr/local/bin/tubevibe"));
    }

    #[test]
    fn self_update_only_where_it_is_safe() {
        let ready = |name: &str| Replace::Ready(format!("{DOWNLOAD}/v0.9.0/{name}"));
        let replace = |exe: &str, os: &str, arch: &str| replace_for(exe, os, arch, &assets(), "v0.9.0");
        assert_eq!(replace("/home/u/.local/bin/tubevibe", "linux", "x86_64"), ready("tubevibe-x86_64-linux.tar.gz"));
        assert_eq!(replace("/home/u/.local/bin/tubevibe", "linux", "aarch64"), ready("tubevibe-aarch64-linux.tar.gz"));
        assert_eq!(replace("/usr/local/bin/tubevibe", "macos", "aarch64"), ready("tubevibe-aarch64-macos.tar.gz"));
        assert_eq!(
            replace(r"C:\Users\u\AppData\Local\Programs\tubevibe\tubevibe.exe", "windows", "x86_64"),
            ready("tubevibe-x86_64-windows.zip")
        );
        assert_eq!(
            replace("/home/u/Projects/tubevibe/target/release/tubevibe", "linux", "x86_64"),
            Replace::FromSource
        );
        assert_eq!(replace("/home/u/Projects/tubevibe/target/debug/tubevibe", "linux", "x86_64"), Replace::FromSource);
        assert_eq!(replace("/home/u/.cargo/bin/tubevibe", "linux", "x86_64"), Replace::Cargo);
        assert_eq!(replace("/usr/bin/tubevibe", "linux", "x86_64"), Replace::Package);
        assert_eq!(replace("/usr/local/bin/tubevibe", "macos", "x86_64"), Replace::NoAsset);
        let none = replace_for("/home/u/.local/bin/tubevibe", "linux", "x86_64", &[], "v0.9.0");
        assert_eq!(none, Replace::NoAsset, "the release lacks the file");
        let update = update_for("0.8.1", "v0.9.0", &assets(), "/home/u/.local/bin/tubevibe", "linux", "x86_64");
        assert_eq!(update.unwrap().replace, ready("tubevibe-x86_64-linux.tar.gz"));
    }

    #[test]
    fn the_program_comes_out_of_the_archives() {
        // A tarball laid out like the release one: tubevibe/ with README.md and tubevibe.
        let mut tar = tar::Builder::new(flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::fast()));
        let mut add = |path: &str, data: &[u8], kind: tar::EntryType| {
            let mut header = tar::Header::new_gnu();
            header.set_entry_type(kind);
            header.set_size(data.len() as u64);
            header.set_mode(0o755);
            header.set_cksum();
            tar.append_data(&mut header, path, data).unwrap();
        };
        add("tubevibe/", b"", tar::EntryType::Directory);
        add("tubevibe/README.md", b"readme", tar::EntryType::Regular);
        add("tubevibe/tubevibe", b"\x7fELF program", tar::EntryType::Regular);
        let gz = tar.into_inner().unwrap().finish().unwrap();
        assert_eq!(from_tarball(&gz).unwrap(), b"\x7fELF program");
        assert!(from_tarball(b"not a tarball").is_err());

        let mut zip = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
        let options = zip::write::SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);
        for (name, data) in [("README.md", &b"readme"[..]), ("tubevibe.exe", b"MZ program")] {
            zip.start_file(name, options).unwrap();
            std::io::Write::write_all(&mut zip, data).unwrap();
        }
        let data = zip.finish().unwrap().into_inner();
        assert_eq!(from_zip(&data).unwrap(), b"MZ program");
        assert!(from_zip(&gz).is_err());
    }
}
