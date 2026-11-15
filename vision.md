# Vision
## Purpose
YouTube in the terminal: real account feeds and Watch Later, thumbnails, AI summaries, and Omarchy theme sync, as a keyboard-first Rust fork of tubetui.
## In scope
YouTube and local Watch Later, browser or `cookies.txt` login, account lists, in-app video tabs with Up Next and comments, mpv playback, SponsorBlock, mid-play resolution/codec switches, self-update, English/Spanish, System Check, and CI on `dev` and `main`.
## Out of scope
Copying upstream tubetui code from v0.8.5+ (AGPL; this repo stays MIT/Apache-2.0); tests that call live YouTube; changing About donation addresses away from polblancoo.
## Design principles and non-negotiables
Share vibekit theming and style. Capital `Q` is the only quit key; lowercase `q` never quits. Shared keys are `Q`, Esc and `?`. Cookie export stays trimmed to YouTube and Google, with owner-only cookie files. Never share cookies. Tests use fixtures and a fake server only. The unofficial YouTube API and browser session may conflict with YouTube's ToS. The repo is public and auditable. Git is `dev`-only, one `dev`→`main` PR at a time, and no push or merge to `main` without Toby's OK.
## Bots may decide alone
Bugfixes and in-scope work already specified; leftover packaging/Windows-file cleanup; relevant tests and CI; docs and changelog shortening that do not change behavior.
## Captain decision hold (needs Toby)
Behavior changes, new dependencies, and anything security- or data-related, including cookie/login handling. What the `v` key covers and the Quality-to-Resolution reword, boxvibe-style thumbnail tint/pixelation, moving to vibekit's shared flat keymap, and any push or merge to `main`.
