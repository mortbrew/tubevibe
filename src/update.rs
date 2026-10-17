//! Is there a newer release? Asks GitHub at most once a day and, when there
//! is one, says how to get it on this machine: the package that fits how
//! tubevibe was installed here, or building it from source when none does.

use std::path::Path;
use std::time::{Duration, SystemTime};

use serde_json::Value;

pub const REPO: &str = "https://github.com/mortbrew/tubevibe";
const LATEST: &str = "https://api.github.com/repos/mortbrew/tubevibe/releases/latest";
const DOWNLOAD: &str = "https://github.com/mortbrew/tubevibe/releases/download";
/// GitHub is asked again only after this long; meanwhile the answer is cached.
const RECHECK: Duration = Duration::from_secs(24 * 3600);
/// The cached "tag" when GitHub said there is no release yet.
const NO_RELEASE: &str = "none";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Update {
    /// `0.5.2`.
    pub version: String,
    pub how: Install,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Install {
    /// A command to paste in a terminal.
    Command(String),
    /// A file to download and open (the Windows installer).
    Link(String),
}

/// The newer release for this machine, if any. `cache` keeps the last
/// answer (`<unix time> <tag> <asset> <asset>…`) so GitHub is asked once a day.
/// No release published yet (GitHub's 404) is no news, not an error.
pub fn check(cache: &Path) -> Result<Option<Update>, String> {
    let cached = std::fs::read_to_string(cache).ok().and_then(|text| parse_cache(&text));
    let fresh = cached.as_ref().is_some_and(|(at, ..)| now().saturating_sub(*at) < RECHECK.as_secs());
    let (tag, assets) = match cached {
        Some((_, tag, assets)) if fresh => (tag, assets),
        _ => {
            let (tag, assets) = fetch_latest()?.unwrap_or_else(|| (NO_RELEASE.to_owned(), Vec::new()));
            let line = format!("{} {tag} {}", now(), assets.join(" "));
            if let Some(dir) = cache.parent() {
                let _ = std::fs::create_dir_all(dir);
            }
            let _ = std::fs::write(cache, line);
            (tag, assets)
        }
    };
    if tag == NO_RELEASE {
        return Ok(None);
    }
    let exe = std::env::current_exe().map(|p| p.display().to_string()).unwrap_or_default();
    Ok(update_for(env!("CARGO_PKG_VERSION"), &tag, &assets, &exe, std::env::consts::OS, std::env::consts::ARCH))
}

/// The latest release's tag and files; `None` when there is none yet.
fn fetch_latest() -> Result<Option<(String, Vec<String>)>, String> {
    let agent: ureq::Agent = ureq::Agent::config_builder().timeout_global(Some(Duration::from_secs(10))).build().into();
    let reply = match agent
        .get(LATEST)
        .header("User-Agent", concat!("tubevibe/", env!("CARGO_PKG_VERSION")))
        .header("Accept", "application/vnd.github+json")
        .call()
    {
        Ok(reply) => reply,
        Err(ureq::Error::StatusCode(404)) => return Ok(None),
        Err(e) => return Err(e.to_string()),
    };
    let text = reply.into_body().read_to_string().map_err(|e| e.to_string())?;
    let json: Value = serde_json::from_str(&text).map_err(|e| e.to_string())?;
    parse_latest(&json).map(Some).ok_or_else(|| "respuesta de GitHub sin versión".into())
}

/// Tag and asset names of GitHub's "latest release" answer.
pub fn parse_latest(json: &Value) -> Option<(String, Vec<String>)> {
    let tag = json["tag_name"].as_str()?.to_owned();
    let assets = json["assets"].as_array()?.iter().filter_map(|a| a["name"].as_str().map(str::to_owned)).collect();
    Some((tag, assets))
}

fn parse_cache(text: &str) -> Option<(u64, String, Vec<String>)> {
    let mut words = text.split_whitespace();
    let at = words.next()?.parse().ok()?;
    let tag = words.next()?.to_owned();
    Some((at, tag, words.map(str::to_owned).collect()))
}

fn now() -> u64 {
    SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).map_or(0, |d| d.as_secs())
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

/// The update for a machine (`os`, `arch`) running `current` from `exe`,
/// when `tag` is newer.
pub fn update_for(current: &str, tag: &str, assets: &[String], exe: &str, os: &str, arch: &str) -> Option<Update> {
    if version(tag)? <= version(current)? {
        return None;
    }
    let has = |name: &str| assets.iter().any(|a| a == name);
    let url = |name: &str| format!("{DOWNLOAD}/{tag}/{name}");
    let cargo = Install::Command(format!("cargo install --locked --git {REPO} --tag {tag}"));
    let from_cargo = exe.contains(".cargo/bin") || exe.contains(r".cargo\bin");
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
    Some(Update { version: tag.trim_start_matches('v').to_owned(), how })
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
        assert_eq!(update_for("0.8.1", NO_RELEASE, &[], "/usr/bin/tubevibe", "linux", "x86_64"), None);
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
    fn reads_github_and_the_cache() {
        let json = serde_json::json!({ "tag_name": "v0.6.0", "assets": [{ "name": "a.deb" }, { "name": "b.zip" }] });
        assert_eq!(parse_latest(&json), Some(("v0.6.0".into(), vec!["a.deb".into(), "b.zip".into()])));
        assert_eq!(parse_latest(&serde_json::json!({ "message": "Not Found" })), None);
        assert_eq!(
            parse_cache("1700000000 v0.6.0 a.deb b.zip"),
            Some((1_700_000_000, "v0.6.0".into(), vec!["a.deb".into(), "b.zip".into()]))
        );
        assert_eq!(parse_cache("1700000000 none "), Some((1_700_000_000, "none".into(), vec![])));
        assert_eq!(parse_cache(""), None);
    }
}
