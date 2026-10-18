//! Is there a newer release? Asks GitHub each time the app opens (in the
//! background) and, when there is one, says how to get it on this machine:
//! the package that fits how tubevibe was installed here, or building it from
//! source when none does.

use std::path::Path;
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
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Install {
    /// A command to paste in a terminal.
    Command(String),
    /// A file to download and open (the Windows installer).
    Link(String),
}

/// The newer release for this machine, if any. Anything that keeps GitHub
/// from answering (no release yet, offline, rate limits) is no news.
pub fn check() -> Option<Update> {
    let (tag, assets) = fetch_latest().ok()??;
    let exe = std::env::current_exe().map(|p| p.display().to_string()).unwrap_or_default();
    update_for(env!("CARGO_PKG_VERSION"), &tag, &assets, &exe, std::env::consts::OS, std::env::consts::ARCH)
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
    fn reads_githubs_answer() {
        let json = serde_json::json!({ "tag_name": "v0.6.0", "assets": [{ "name": "a.deb" }, { "name": "b.zip" }] });
        assert_eq!(parse_latest(&json), Some(("v0.6.0".into(), vec!["a.deb".into(), "b.zip".into()])));
        assert_eq!(parse_latest(&serde_json::json!({ "message": "Not Found" })), None);
        assert!(no_news(404) && no_news(403) && no_news(429), "no release, rate limits");
        assert!(!no_news(500) && !no_news(200));
    }
}
