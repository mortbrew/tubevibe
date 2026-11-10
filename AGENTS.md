# tubevibe

Linux terminal YouTube client (ratatui / crossterm), forked from polblancoo/tubetui. Releases are Linux-only. Work on `dev`.

## Build, test, lint

Same commands as CI (`.github/workflows/ci.yml`). Rust 1.90+ (`rust-version` in `Cargo.toml`). Put build output in the repo, not `/tmp`:

```sh
export CARGO_TARGET_DIR="$PWD/target"
cargo fmt --all --check
cargo build --locked
cargo build --release --locked
cargo test --locked
cargo clippy --all-targets --locked -- -D warnings
```

`audit` is `cargo audit` (`.github/workflows/audit.yml`). `codeql` scans Rust with no build. `msrv` builds with `--locked` on the `rust-version` from `Cargo.toml`. Tests use fixtures and a fake server, never live YouTube. Run the binary from this tree (`cargo run`, `./target/debug/tubevibe`). `dev` stays one version ahead of the latest release on purpose; do not hand-edit `Cargo.lock`.

## Layout (`src/`)

The library core is pure: `app::State::update` turns an `Action` into state plus `Effect`s. `main` runs the terminal and those effects.

| Module | Role |
| --- | --- |
| `app` | State, tabs, the pure update function |
| `domain` | Shared plain data |
| `config` | `config.toml` (defaults, tolerant load, write-back) |
| `keymap` | Keys to actions; Help is generated from the same tables |
| `ui` | Layout and drawing |
| `theme` | Tokyo Night, recolored from the current Omarchy theme |
| `i18n` | English and Spanish strings at the call site |
| `anim` | Short tachyonfx transitions |
| `mouse` | Pointer hits to actions |
| `youtube` | Feeds via yt-dlp; search and account calls via InnerTube |
| `session` | Browser cookies and the web-client signature |
| `player` | mpv over JSON IPC (kitty embed, window, overlay) |
| `sponsorblock` | Sponsor segments |
| `media` | Local video and music folders |
| `store` | History, cache, data paths |
| `thumbs` | Thumbnail download and draw |
| `desktop` | Optional `.desktop` launcher |
| `doctor` | Checks for mpv, yt-dlp, and how to install them |
| `update` | GitHub release check and self-update |

## Conventions

- UI labels are Title Case.
- Colours follow the Omarchy theme (Tokyo Night names in drawing code; `theme` swaps them).
- Shortcuts stay aligned with boxvibe: `O` opens Settings, `T` opens the current video in a background tab.
- Keep YouTube calls minimal. Reuse a reply already in hand; do not add a request for something the screen does not show.
- Never copy code from upstream tubetui v0.8.5 or later. That line is AGPL. tubevibe stays on the MIT / Apache-2.0 base it forked.
- Git: commit on `dev` only. `main` changes only through a pull request, and only one `dev` → `main` pull request is open at a time (a release). Feature work targets `dev`. See `docs/setup.md`.
