# Status

`Cargo.toml` on `dev` is **0.9.1** (ahead of the v0.9.0 release on purpose). Latest shipped tag is **v0.9.0** (2026-10-26). Since that tag, `dev` also defaults subtitles and dubbing to the app language until a language is chosen (`CHANGES-TUBEVIBE.md`, Unreleased).

## Done

- **v0.9.0** released: cookie export trimmed to YouTube and Google, cookie files owner-only (`0600`) and the cache folder `0700`, TLS crates bumped, boxvibe-style Settings, two-column Help, `O` opens Settings.
- Subtitles and dubbing follow the app language until the user picks one (unreleased, on `dev`).

## Next

Ideas for **0.9.1**:

- Resolution rework. What the `v` key should cover is still undecided. Reword Quality to Resolution.
- Remove leftover packaging and Windows files.
- A shorter changelog and a shorter About.
- boxvibe-style colour tint and retro pixelation for thumbnails.
- Later: move to vibekit's shared flat keymap.

## Decisions

- Screenshots use Nerd Font icons.
- No copying of upstream tubetui code (v0.8.5+ is AGPL; this repo stays MIT / Apache-2.0).
