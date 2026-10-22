# tubevibe changes (fork of tubetui)

Changes made in this fork, [mortbrew/tubevibe](https://github.com/mortbrew/tubevibe),
on top of upstream [polblancoo/tubetui](https://github.com/polblancoo/tubetui) v0.7.7.

## Watch Later is YouTube's own Watch Later (`watch-later-online`)

Before, the **Ver después / Watch Later** entry of the left menu was a list kept
only on this computer (`~/.local/share/tubetui/later.json`), unrelated to the
*Watch Later* playlist of the YouTube account.

Now:

- **Menu → Watch Later** opens the account's YouTube *Watch Later* playlist
  (playlist id `WL`, fetched with yt-dlp and the login cookies, like every
  other playlist). Without a login it says
  *"Sign in to see your YouTube Watch Later"* instead of showing a list.
- **`l`** (and the right-click item) on a video, in any list or video tab:
  - not in Watch Later → it is added on YouTube right away
    (InnerTube `browse/edit_playlist`, `ACTION_ADD_VIDEO`);
  - already there → the usual yes/no question, and on *yes* it is removed
    (`ACTION_REMOVE_VIDEO_BY_VIDEO_ID`, `removedVideoId`).
  - Whether it is there comes from the Watch Later list once it has been
    opened (and fit in one page). Otherwise YouTube is asked first through
    `playlist/get_add_to_playlist` (`containsSelectedVideos` of `WL`).
  - The open list is updated as soon as YouTube confirms; errors show in the
    status bar (❌ …). Without a login it says to sign in.
- **`x`** inside Watch Later removes the selected video (after asking).
  Opening *Watch Later (YouTube)* from **My Playlists** behaves the same.
- The 🕒 marks in lists and the count next to the menu entry reflect the
  YouTube list (the count only when the whole list was loaded).
- Changing the login forgets the Watch Later known so far.

`later.json` is no longer read or written; an existing file is left on disk
untouched (the one-time import of the old `ytv` list is gone too). The
functions that read it (`store::load_later`) are still there.

### Untested against a live account

The unit tests cover the app logic and the request and reply shapes, but
**none of this was tried against a real YouTube account**. In particular:

- the `browse/edit_playlist` payloads (`ACTION_ADD_VIDEO` with
  `addedVideoId`, `ACTION_REMOVE_VIDEO_BY_VIDEO_ID` with `removedVideoId`)
  follow what youtube.com sends, but YouTube may change them;
- success is assumed unless the reply carries a `status` other than
  `STATUS_SUCCEEDED` (HTTP errors are reported as errors);
- the `playlist/get_add_to_playlist` reply is read by looking for a
  `playlistAddToOptionRenderer` with `playlistId: "WL"`; if YouTube answers in
  another shape, `l` on a video not yet known shows an error instead of
  adding it (opening Watch Later first avoids that question);
- YouTube may take a moment before a reloaded Watch Later reflects an edit.

## Enter opens the tab, Enter again plays (`no-autoplay`)

Upstream starts mpv as soon as Enter is pressed on a video. Now a new setting,
`[player] autoplay` (Settings → *Enter on a video*), decides; **in tubevibe it
defaults to `false`**:

- **Enter (or a double click) on a YouTube video** in a list, or on a video of
  a tab's *Up next*, opens its tab without playing. The details, Up next,
  comments and SponsorBlock segments load as usual; the tab says
  *"▶ Enter plays it"* (in embedded / overlay mode, in the middle of the
  empty video pane; with the video in its own window, in the details).
- **Enter or space in that tab** (or a click on the empty video pane) starts
  it, resuming where it was left (the status bar already says where).
- The video counts as watched (hidden from feeds, first in *History*) only
  once it really plays.
- **Sound only (`a`)** and **your own files and music** still play at once.
- The hint bar says *Enter: play* on a tab that never played, and keeps
  *play it again* for a closed one.

`autoplay = true` brings back upstream's behavior exactly; the existing tests
run with it on, new ones cover it off.

## AI video summaries (`descriptions`)

YouTube shows an *"AI-generated video summary"* under some videos (not
most: about a third of those tried, more often recent English ones). It now
shows in tubevibe too. The video's own description is **not** shown (an
earlier version of this branch did; it was taken out whole).

- **Video tab** (when the video plays in its own window, or as sound only,
  so the tab shows its details): *"✦ AI summary · AI can make mistakes"*
  and the whole summary, wrapped, one blank line after the last detail
  (the status line, e.g. *"▶ Enter plays it"*). The details card grows to
  hold it and *Up next* moves down; without a summary nothing changes. A
  very long one is cut with `…` so *Up next* keeps a few rows.
- **Preview pane** of video lists: the same, one blank line after the other
  details, using
  all the room left in the pane (`…` if it does not fit).
- **Where it comes from**: the watch page (`/youtubei/v1/next`) the tab
  already asks for, so no extra request there:
  `engagementPanels[].engagementPanelSectionListRenderer.content`
  `.structuredDescriptionContentRenderer.items[].expandableMetadataRenderer`
  `.expandedContent.videoSummaryContentViewModel.paragraphs[]`
  `.videoSummaryParagraphViewModel.text.content` (paragraphs joined by a
  blank line), else that box's `header.collapsedTitle`. For the preview the
  same request is made for the selected video once the selection rests on
  it for 800 ms, one at a time, signed in when logged in (like the tab);
  live and upcoming videos are not asked. Answers (also "none" and
  failures) are kept for the session, not saved.
- No new keys (`e` from the description version is gone).
- `ratatui`'s `unstable-rendered-line-info` feature is turned on to measure
  how many rows the wrapped details take.

### Uncertain

- Checked live only anonymously from a US address (`hl=en`, `gl=US` and
  others: the summary came every time, in the video's language; only the
  labels follow `hl`). Not tried signed in, nor from outside the US.
- The field names are YouTube's undocumented internals and can change;
  the parser is tested against a trimmed real reply
  (`tests/fixtures/next_summary.json`).
- Each preview summary costs one watch-page request (~400 KB) per video
  looked at.

## "No AI summary" note (`summary-unavailable`)

- When YouTube was asked and the video has **no AI summary**, the video tab
  and the preview pane now say so, in the same place and style as a summary
  (one blank line above, blue bold heading, muted note):
  *"✦ AI summary · not available for this video"*
  (*"✦ Resumen IA · no disponible para este video"*). It is one line;
  *Up next* sits just below it and clicks on its rows still line up.
- A lookup that **failed** (network, YouTube error) shows the same note, to
  keep it simple; it is not asked again during the session.
- Nothing is shown **while it is being asked** (before the tab's watch page
  arrives, or before the preview's 800 ms rest and request finish), nor for
  **live and upcoming videos**, which are never asked.

## Playlist fixes (`playlist-fixes`)

- **Language of signed-in answers**: requests made with your login used to
  ask YouTube for Spanish (`hl=es`) whatever the app's language, so the
  playlists YouTube names itself came back as *"Shorts guardados"* and the
  like. They now follow the app's language (Settings → language), and a
  change applies to the next request. Calls that read dates (History, a
  video's watch page) still ask for English, as before; the other
  signed-in replies are parsed by structure, or in both languages
  (subscriber counts), so English is safe. `gl` (region) is still not sent:
  YouTube takes it from the account / address.
- **Shorts in playlists**: *Hide Shorts* no longer applies to a playlist you
  open (yours, saved ones, Watch Later, Liked videos): a list of only Shorts
  showed empty. Its status count (*"Mine · 12"*) and the Watch Later count
  in the menu now match what is shown. Feeds, Home, Subscriptions,
  searches, channels and the new-video notices still leave Shorts out.

## Playlist removal, `l` only adds, real subscriptions (`playlist-delete`)

- **`l` only adds** the selected video to YouTube Watch Later. If it is
  there already it says *"🕒 Already in Watch Later"*: no prompt, nothing
  removed. The right-click item reads *Save to Watch Later* / *Already in
  Watch Later*.
- **`Ctrl+Y` `x` removes from YouTube lists**: in Watch Later or any
  YouTube playlist opened from *My Playlists* (mine, saved), it asks
  *"Remove <video> from <playlist>?"* and, on yes, takes the selected video
  out on YouTube; the list updates when YouTube says it is done, errors go
  to the status bar. In **Liked videos** removing is taking the like back
  (the same request as `g` on a liked video). In a video tab, or anywhere
  else, `Ctrl+Y` `x` still closes the tab. The right-click menu of those
  lists offers *Remove from this list*.
- **Plain `x` no longer removes** videos from Watch Later or playlists (it
  says *"Ctrl+Y x removes it from this list"*). Elsewhere it is unchanged:
  History (remove), Downloads (cancel / forget), My Videos / My Music
  (trash), My Playlists (delete the whole playlist).
- **How a playlist removal is sent**: Watch Later as before
  (`ACTION_REMOVE_VIDEO_BY_VIDEO_ID`). Other playlists: the entry's
  `setVideoId` is read from the playlist (signed-in `browse` `VL<id>`, page
  by page until the video shows, at most 50 pages; from
  `playlistVideoRenderer.setVideoId` or any `setVideoId` inside the video's
  lockup), kept per playlist for the session, and sent as
  `ACTION_REMOVE_VIDEO`; with none found, `ACTION_REMOVE_VIDEO_BY_VIDEO_ID`
  is tried.
- **My Channels / subscription checkmarks**: the sidebar (`guide`) used to
  add every channel link in it, so YouTube's own channels in *Explore*
  (Music, Live, Gaming, News, Sports…) and *Your channel* counted as
  subscriptions, in *My Channels* and as ✓ marks (the subscribed set comes
  from the same list). Only the `guideSubscriptionsSectionRenderer`
  section is read now (its *Show more* entries included).

### Uncertain

- None of this was tried against a live account. The anonymous playlist
  page carries no `setVideoId` (only the owner's view has remove commands),
  so where YouTube puts it for the owner is a guess covering both known
  shapes; the by-video-id fallback may or may not work for normal
  playlists. Saved playlists of other people cannot be edited: YouTube
  should answer with an error, shown in the status bar.
- The guide fixture (`tests/fixtures/guide.json`) is a real anonymous
  reply with the signed-in parts (the subscriptions section, *Your
  channel*) added by hand in the shape YouTube is known to send.
- The subscribed set saved from an earlier run (with the topic channels)
  is replaced the next time the channels are read.

### Fix: removing from Saved Shorts failed with "http status 400"

- **What most likely happened** (not yet confirmed on a live account): `Ctrl+Y` `x` in *Saved Shorts* answered
  *"Could not remove from Saved Shorts <video>: YouTube: http status 400"*.
  The app could not find the entry's `setVideoId` on that playlist's page
  (it only looked in two shapes of row, and Shorts come in others), so it
  fell back to removing by video id, which YouTube only takes for Watch
  Later: 400 is YouTube refusing the request.
- **Now** the app sends exactly what YouTube's own *Remove from …* menu on
  the playlist page sends (the entry's `playlistEditEndpoint`, with its
  playlist id, `setVideoId` and any extra `params`), found in classic rows,
  newer lockups and Shorts rows. Removing by video id is only the last
  resort.
- **Errors say why**: when YouTube refuses, the status line shows its reason
  (e.g. *"http status: 400 · Request contains an invalid argument."*), and
  reads *"Could not remove <video> from <playlist>: …"*.
- **Debug log**: with `TUBEVIBE_DEBUG=/path/to/log` (or the old `TUBETUI_DEBUG`), each removal logs how it
  was sent and the request, every failed YouTube request logs what was sent
  and YouTube's full answer, and a playlist page with no remove commands is
  logged (so the shape of Saved Shorts can be seen if it still fails).

## My Recommendations shows the whole YouTube home feed

The original app kept only videos from channels you subscribe to, so with few
subscriptions the list came up empty. It now shows YouTube's home feed as is,
the same recommendations you see on youtube.com or the phone app.

## My Recommendations hides your Watch Later, and Title Case menu names (`recs-titlecase`)

- **My Recommendations no longer shows videos that are already in your
  YouTube Watch Later.** When you open it, the app quietly reads your Watch
  Later in the background; when that arrives, those videos drop out of the
  list. Saving a video with `l` while you're looking at My Recommendations
  takes it out of the list right away; removing one from Watch Later brings
  it back. The list title says how many were hidden (e.g. "🕒 3 in Watch
  Later hidden"). Only My Recommendations does this; the other feeds are
  unchanged.
- **New setting**: Settings → *Hide Watch Later videos in My
  Recommendations* (on by default). It is saved in `config.toml` as
  `hide_later` under `[ui]`; turning it off shows everything again at once.
- **The English left menu is in Title Case**: My Recommendations, From What
  I Watched, Discover, Subscriptions, Watch Later, History, My Playlists, My
  Channels, Discover Channels, Search Videos, Search Channels, My Videos, My
  Music, Downloads, Settings, About, Quit. The same names are capitalized in
  the help screen (`?`), key hints, search box titles and these docs. The
  Spanish menu is unchanged.
- The *Hide watched* setting now reads "Hide watched videos in feeds (v
  shows them)" (it always applied to every feed, not only Home).
- Limit: if your Watch Later is longer than one page (60 videos by default,
  `[search] results`), only the first page is used for hiding.

## TUBEVIBE menu section and the local Watch Later list (`tubevibe-section`)

- **New left-menu header "TUBEVIBE"**, between YOUTUBE and ON THIS
  COMPUTER, for lists the app makes itself rather than YouTube pages.
- **From What I Watched** moved there.
- **The local Watch Later** (first called *Saved for Later*) is back there: the app's original local watch-later
  list (`later.json` in the data folder, e.g.
  `~/.local/share/tubevibe/later.json`; tubetui's copy is brought over on the first run), which the Watch Later change hid
  when the menu entry became YouTube's Watch Later. Whatever was in that
  file shows again. It is separate from YouTube's Watch Later.
  - Inside it `Ctrl+Y` `x` (or right-click → *Remove from this list*) takes
    the selected video out after asking; plain `x` only says how.
  - `l` adds to it (see below; `w` is now YouTube's Watch Later).
  - The menu name is one constant, `SAVED_FOR_LATER` in `src/app.rs`;
    messages use `LOCAL_LATER` / `YOUTUBE_LATER` next to it.
  - If `later.json` can't be read it is left alone and never written over.

### Toby's choices: one TUBEVIBE section, two Watch Laters, `w` and `l`

- **TUBEVIBE now holds**: From What I Watched, Discover, Discover Channels,
  Watch Later (all made by the app, not YouTube pages). YOUTUBE keeps My
  Recommendations, Subscriptions, Watch Later, History, My Playlists, My
  Channels, Search Videos, Search Channels.
- **The local list is called "Watch Later"** in the menu (Spanish *Ver
  después*), under TUBEVIBE; the header tells it from YouTube's. Messages
  and questions always say which one: *tubevibe Watch Later* (local) or
  *YouTube Watch Later* (online), e.g. "🔖 Saved to tubevibe Watch Later:
  …", "🕒 Saved to YouTube Watch Later: …", "Remove … from tubevibe Watch
  Later?". The video preview shows "In YouTube Watch Later" and/or "In
  tubevibe Watch Later".
- **Keys**: `w` adds to YouTube's Watch Later (what `l` did, "Already in
  YouTube Watch Later" when it is there); `l` adds to tubevibe's local
  Watch Later ("Already in tubevibe Watch Later" when it is there). The
  key hints, help screen (`Ctrl+Y ?`), right-click menu and README say so.
- The setting now reads *Hide YouTube Watch Later videos in My
  Recommendations* (only YouTube's list is hidden there).


## The app is called tubevibe, and Settings line up (`rebrand`)

tubevibe is based on [tubetui](https://github.com/polblancoo/tubetui) by
polblancoo; this fork now carries its own name.

- **The command is `tubevibe`** (Cargo package and binary):
  `cargo build --release` makes `./target/release/tubevibe`. The menu title,
  the startup logo, the welcome and About screens and the terminal window
  title say *tubevibe* ("tube" and "vibe" in two colors). About says
  *Based on tubetui by polblancoo* with the original's link.
- **Folders**: `~/.config/tubevibe`, `~/.local/share/tubevibe` and
  `~/.cache/tubevibe` (or the platform equivalents). On the first run, when
  a tubevibe folder doesn't exist and the tubetui one does, the tubetui one is
  **copied** over (settings, login copies of the cookies, history, watched,
  downloads, the local Watch Later). The tubetui folders are never changed or
  removed; once a tubevibe folder exists nothing is copied again. If
  `session.cookies_file` in the settings pointed inside the old cache folder,
  it is pointed at the copy.
- **Debug log**: `TUBEVIBE_DEBUG=/path/to/log`; `TUBETUI_DEBUG` still works.
- **Packaging files** (`.desktop`, Windows installer script, release workflow,
  Debian metadata) use the new name. The update check (`src/update.rs`)
  still asked the original project's releases at this point (see below).
- **Settings line up**: values start in one column whatever the icon. Some
  emoji (🖼, 🗣, 🖱, 🎞, 🏷, 👁, ⏯, ⬆) are drawn one or two cells wide
  depending on the terminal, which pushed their row's value over; they were
  replaced by emoji every terminal draws two cells wide (📷, 🔤, 🐭, 🎦, 📌, 👀,
  👆, 🆙). The label column is as wide as the longest label plus a gap, so
  *Hide YouTube Watch Later videos in My Recommendations* no longer runs into
  its value. The same kind of symbol was replaced where it starts a column
  elsewhere: the progress mark in lists (▸), the downloads' progress and
  cancelled marks (⇩, ✕) and the system check's warnings (!).

## Releases from this fork, version 0.8.0 (`releases`)

- **Version 0.8.0.**
- **Update check** asks this fork's releases
  (`api.github.com/repos/mortbrew/tubevibe/releases/latest`, User-Agent
  `tubevibe/<version>`) once a day and says *"⬆ tubevibe X available"*. No
  release published yet (GitHub answers 404) is quietly no news, and is
  remembered for the day like any answer. The About page's command fits how
  it was installed:
  - the `.deb` (`/usr/bin/tubevibe`): `curl -LO …/tubevibe_amd64.deb && sudo apt install ./tubevibe_amd64.deb`;
  - an archive (Linux x86_64 / ARM64, macOS Apple Silicon):
    `curl -fsSL …/tubevibe-<arch>-<os>.tar.gz | tar xz -C <folder it runs from> --strip-components=1 tubevibe/tubevibe`;
  - Windows: the installer's link;
  - built in a clone (`…/target/release/tubevibe`):
    `cd <clone> && git pull && git checkout vX && cargo build --release`;
  - `cargo install` or anything else:
    `cargo install --locked --git https://github.com/mortbrew/tubevibe --tag vX`
    (`cargo install tubevibe` is not suggested: it is not on crates.io).
- **Release workflow** (`.github/workflows/release.yml`), on every `v*` tag (or
  run by hand for an existing tag): creates the GitHub release and adds
  `tubevibe-x86_64-linux.tar.gz`, `tubevibe-aarch64-linux.tar.gz` (new),
  `tubevibe_amd64.deb`, `tubevibe-x86_64-windows.zip`,
  `tubevibe-x86_64-windows-setup.exe` and `tubevibe-aarch64-macos.tar.gz`.
  It needs no secrets. The Windows installer has its own id, so it doesn't
  replace an installed tubetui.
- **install.sh** installs the latest release's program into `~/.local/bin`
  (`TUBEVIBE_BIN_DIR` to change it), or builds it with
  `cargo install --git https://github.com/mortbrew/tubevibe` where there is no
  release file for the machine (or with `TUBEVIBE_FROM_SOURCE=1`).

## 0.8.1: red ▶ logo, a blank row above YOUTUBE, whose donations (`v0.8.1`)

- **Fix: Spanish after Edit config.toml**: when the file has no `lang` line
  (the app follows the system language), reloading it after editing
  switched the whole app to Spanish. It now keeps the current language.
- **Logo**: the menu title, the welcome screen and About show a plain ▶
  (U+25B6, no emoji variation selector) in the same red as "tube", instead of
  the orange ▶️ emoji. It is one cell wide everywhere, with one space after
  it (`▶ tubevibe`). The Nerd Font logo is red too; the plain-Unicode set
  already used ▶.
- **Menu** (removed again in 0.8.6): one blank row between the title and the YOUTUBE header, like the
  ones between the other groups. Clicks still land on the right entry (the
  click map is built from the same rows); the whole menu needs 27 rows.
- **Donations**: About, README, `donate.toml` and the old website page (`docs/index.html`) say the addresses belong to
  polblancoo, the original tubetui developer, and that donations go to him,
  not to the tubevibe fork maintainer (mortbrew). The addresses are unchanged.
- **Update check on every start**: GitHub is asked each time the app opens
  (in the background, as before) instead of once a day, so a new release shows
  up right away. Nothing is cached any more: the old `latest-release.txt` in
  the cache folder is deleted. No release yet (404), rate limits (403, 429)
  and network errors stay silent. The setting reads *Look for new versions on
  startup* (*Buscar versiones nuevas al abrir*).

## 0.8.2: tubevibe updates itself from About → Update Now (`self-update`)

- **About → Update Now**: once the startup check has found a new version,
  the status bar says *⬆ tubevibe X available · see About* and About shows
  an **Update Now** button (*Actualizar Ahora*). Opening About selects it
  (green, between ▶ ◀); `Tab` moves between it and the menu, and `Enter`
  presses it. While it can't be pressed it is greyed out. After pressing:
  *⬇ Downloading… N%* (also in the status bar), then
  *✅ tubevibe X Installed · Restart tubevibe to Use It*, or
  *❌ Could Not Update: …* with the reason (the button can be pressed again).
  There is no global hotkey for it.
- How: the release file for this machine (`tubevibe-x86_64-linux.tar.gz`,
  `tubevibe-aarch64-linux.tar.gz`, `tubevibe-aarch64-macos.tar.gz`,
  `tubevibe-x86_64-windows.zip`) is downloaded in a background thread
  (`Effect::SelfUpdate` → `Action::UpdateProgress` / `Action::SelfUpdated`),
  the program is taken out of it (`tubevibe/tubevibe`, or `tubevibe.exe`),
  written next to the running one, run once with `--version` to check it
  starts here and is the expected version, and then swapped in with the
  `self-replace` crate (a rename on Linux and macOS, so a failure leaves the
  old program; on Windows the running `.exe` is moved aside and cleaned up
  when it exits). New dependencies: `self-replace`, `tar`, `flate2` (already
  used by ureq), and `zip` with only deflate. Downloads use ureq's rustls, so
  there is no OpenSSL in any release build.
- **When the program can't be replaced** the button is greyed out and About
  (and Enter on it) says why:
  - run from a cargo `target/release/` or `target/debug/` folder:
    *Built from source: run git pull && cargo build --release*;
  - installed with `cargo install` (`~/.cargo/bin`): reinstall with cargo;
  - installed by a package manager (`/usr/bin`, `/usr/lib`, `/nix/store`,
    `/snap`, Homebrew): update the package (About keeps showing the command);
  - the program's folder can't be written to, or the release has no file
    for this system.
- **`TUBEVIBE_FAKE_VERSION`** (e.g. `TUBEVIBE_FAKE_VERSION=0.0.1 tubevibe`)
  makes the update check compare releases against that version instead of the
  real one, to try the whole flow against the current release. It changes
  nothing else.
- The bottom bar shows *Enter update now* on the button and *Tab select
  Update Now* from the menu; the README describes the button.
- **Up to date**: when the startup check confirms GitHub's latest release is
  this version (or older), About shows *✅ You're on the Latest Version*
  (*Tenés la última versión*) in green. Not while the check runs, and not when
  it could not ask GitHub (offline, rate limited, no release): `update::check`
  now answers `Newer`, `Latest` or `Unknown`.
- **Icons**: About's update lines follow the icon set (new `Icons` entries
  `up_to_date`, `update`, `downloading`, `failed`): emoji ✅ ⬆ ⬇ ❌, Nerd Font
  check-circle / arrow-circle-up / arrow-circle-down / times-circle
  (U+F058, U+F0AA, U+F0AB, U+F057), plain Unicode ✔ ↑ ↓ ✘. The ⬆ after
  *About* in the menu does too.
- **The command by hand** (*Update with: …* and the Shift + drag hint) only
  shows when the button can't update this copy (built from source, package,
  cargo, read-only folder, no file for this system), or right after the
  button failed. When the button can do it, About doesn't show the command.

## 0.8.3: Omarchy colour sync (`omarchy-colour-sync`)

- **The Omarchy theme's colours**: on Omarchy, the interface takes the
  colours of the current theme and follows `omarchy theme set` live (the
  theme file is looked at every 2 seconds: a `stat`, read again only when its
  path, inode, size or time changed). Not on Omarchy, or with a file that
  can't be read: the built-in Tokyo Night, as before.
- **Where Omarchy keeps the theme**: `~/.local/state/omarchy/current/theme/`
  (Omarchy 4, a folder swapped on each switch; `$XDG_STATE_HOME` is honoured),
  else `~/.config/omarchy/current/theme/` (Omarchy 3: a copy; Omarchy 2: a
  symlink to the theme). In it, `colors.toml` is read: Omarchy 4's named
  palette (`foreground`, `dark_foreground`, `selection`, `muted`, `accent`,
  `red`…`magenta`, `bright_*`, `orange`) or Omarchy 3's `color0`–`color15`
  with `foreground`/`background`/`selection_background`. Themes without it
  (Omarchy 2) are read from `alacritty.toml` (`[colors.primary]`,
  `[colors.normal]`, `[colors.bright]`).
- **Roles**: text ← `bright_foreground`/`foreground`; secondary text ←
  `dark_foreground` (alacritty `dim_foreground`); selected row ← `selection`;
  dialog lines ← `muted`; blue, cyan, green, yellow, red, orange ← the same
  names; magenta ← `bright_magenta`, purple ← `magenta`, teal ←
  `bright_cyan`; animations fade from `background` and flash `accent`.
  Missing or unreadable roles are mixed from foreground and background
  (0.8.6: no more mixing; real theme colours instead, see that entry)
  (secondary text 45% of the way, the selected row 14%), and so are colours
  that would be too faint (secondary text under 2:1 contrast, or a selection
  that is louder than a row background, like Omarchy 3 Gruvbox's orange).
  YouTube red stays red.
- **How**: the screen is still drawn with the Tokyo Night colours (now
  `theme::TOKYO_NIGHT`), and `theme::recolor` swaps each of them for the
  theme's in the finished frame, before the animations. `State::palette`
  holds the colours; `Action::Palette` sets them.
- **Setting**: *Omarchy Theme Colors* (*Colores del tema de Omarchy*) in
  Settings, `[ui] omarchy_theme = true` in config.toml (saved there when
  changed in the app). Off: Tokyo Night right away.
- Tests read real theme files from Omarchy (Gruvbox in its Omarchy 2, 3 and
  4 forms, Tokyo Night, Catppuccin Latte in 2 and 4, Rose Pine), in
  `tests/fixtures/omarchy/`.

## 0.8.4: `x` on Downloads moves the file to the trash (`downloads-trash`)

Before, `x` on a finished download only took it off the list and the file
stayed on disk. Now it works like `x` on My Videos / My Music:

- **Finished download** → *"Move to the trash <file>?"* (*"¿Mandar a la
  papelera <archivo>?"*); on *yes* that file goes to the system trash
  (`trash` crate, recoverable from there) and the entry leaves the list and
  `downloads.json`. Status: *"🗑 <file> moved to the trash (recover it from
  the system trash)"*; the icon follows the icon set (🗑, Nerd Font
  U+F1F8, ✕).
- **Running download** → `x` still cancels it, as before.
- **Failed / cancelled download** (no file) → `x` still just takes it off the
  list, without asking.
- **File already gone** → the entry leaves the list, saying so.
- **Trash error** → shown (❌ …) and the entry stays.
- Only the exact recorded path goes to the trash, and only a regular file:
  never a folder or a symlink, no other files next to it
  (`media::trash_file`). Nothing is touched if the same video is being
  downloaded again.
- Hint bar on a finished download: *"x move to trash (asks first)"*; the
  right-click menu says *Move to Trash (Asks First)*. The help screen's
  `x  Del` row already reads *"to the trash (asks y)"*.
- `d` is still Download everywhere; My Videos / My Music are unchanged.

### An older tubevibe opens a newer config.toml (`downloads-trash`)

0.8.2 refused to start with a config.toml written by 0.8.3
(*unknown field `omarchy_theme`*), and since the app never opened it
could not update itself either. From now on:

- **Unknown keys and sections are skipped, never fatal.** All
  `deny_unknown_fields` are gone (config structs and `donate.toml`'s
  wallets). Each `[section]` is read key by key (`config::section`, with
  `serde_ignored`), so a key this version doesn't know (a typo, or a
  setting from a newer tubevibe) is left out and the rest still apply.
- **A wrong-type value for a known key** (`sidebar = 3`,
  `mode = "fullscreen"`, `ui = 5`) falls back to that key's default
  instead of stopping the app.
- **Typos are still caught**: a status-bar line on start, e.g.
  *"⚠ Ignored unknown setting ui.future_glow in config.toml (made by a newer
  tubevibe?)"* / *"⚠ Se ignoró el ajuste desconocido ui.future_glow de
  config.toml (¿de un tubevibe más nuevo?)"*, or *"⚠ Bad value for
  player.max_height in config.toml: using the default"*, with *(+N more)*
  when there are several. Each one also goes to the debug log
  (`TUBEVIBE_DEBUG`).
- **Settings → Edit config.toml** reloads the same way:
  *"Settings reloaded · ⚠ Ignored unknown setting …"*.
- Only a file that is not valid TOML still stops the app, naming the file.
- Settings changed from the app (`config::set_value`, via `toml_edit`)
  keep unknown keys and sections and their comments as they were.
- Warnings are listed in file order (`toml` `preserve_order` feature).

## 0.8.5: Downloads multi-select and missing files drop off (2026-10-18)

### Mark several downloads with `space` (`downloads-multi-select`)

- **Downloads**: `space` marks the selected download (the same red ✗ as
  My Videos / My Music) and moves down; the list title shows
  *"✗ 2 marked"*. Marks of entries that leave the list go with them.
- **`x` with marks** acts on all the marked ones:
  - finished ones → **one** question, *"Move 3 files to the trash?"*
    (*"¿Mandar 3 archivos a la papelera?"*), then each file goes to the
    system trash with the same rules as one (regular files only, the exact
    recorded path, left alone if its video is downloading again);
  - running ones → cancelled; failed / cancelled ones → off the list. When
    there are also finished ones these happen on the same *yes* (the hint
    says *"yes (the other marked ones get cancelled or taken off)"*);
    with none, right away without asking;
  - one summary line, e.g. *"🗑 2 moved to the trash · 1 already gone
    (taken off the list) · 1 cancelled · ❌ 1 could not go to the trash:
    e.mp4: Permission denied"* (the icon follows the icon set).
  - Marks are cleared after acting (kept on *no*). Like My Videos / My
    Music, marks are not cleared just by leaving the screen.
- **No marks**: `x` works on the selected download exactly as before.
- **Hints**: Downloads shows *"space mark several"* (and, with marks,
  *"space mark / unmark · x the marked ones: trash (asks first)"*); My
  Videos / My Music already showed it. The bar drops whole hints from the
  end when narrow, never cuts one. Right-click menu: *Mark / Unmark*. Help:
  *"mark several (videos, music, downloads)"*.

### Downloads follows the disk

- Finished downloads whose file was moved or deleted while tubevibe was
  open now drop off the list **and** `downloads.json` by themselves, with
  no question: each time Downloads is opened, and right after a trash done
  from it, like My Videos / My Music rescan when opened (no timer; only the
  listed finished paths are looked at). Running downloads are never
  dropped. (At start, missing files were already left out.)
- My Videos / My Music already rescan their folders each time they are
  opened (now covered by a test).


## 0.8.6: tabs in the menu, tab keys, fullscreen fix, width settings, theme accent (2026-10-19)

### Settings, About and Quit at the bottom of the menu (`menu-layout-borders`)

- **Settings, About and Quit** sit pinned to the bottom of the left menu,
  under a short rule in the dim colour that stops two columns short of
  each side. YOUTUBE, TUBEVIBE, ON THIS COMPUTER and the open tabs stay at
  the top; the space between them grows with the terminal.
- `↑`/`↓` still go through every section in order and wrap around (`↑` on
  My Recommendations goes to Quit, `↓` on Quit back to the top); clicks
  land on the right rows.
- A terminal too short for the gap puts them right below the rest (a blank
  row, then the rule). Shorter still, the menu scrolls so the chosen
  section always shows.

### The menu lights up when it has the keys (`menu-focus-border`)

- The left menu's border stayed dim whether or not it had the keys, while
  the list (and *Up next* in a video tab) turn blue when focused, so it
  wasn't obvious the menu was the active pane. Its frame was drawn with the
  dim colour, hard-coded. Now the menu uses the same focused style as the
  other panes: blue (the theme's blue under Omarchy, e.g. Gruvbox) while it
  has the keys on the browsing view, dim otherwise.
- With **Border Style → None** there is no line to light, so the focused
  pane's title is shown on the highlight instead (the menu's *tubevibe*, the
  list's title).
- The short rule above Settings, About and Quit stays dim either way.

### Border Style setting (`menu-layout-borders`)

- **Settings → Border Style** (*Estilo de los bordes*): Plain (the look it
  always had, the default) → Rounded → Thick → Double → None, applied to
  every pane, dialog and the help. Saved as `[ui] border_style = "plain" |
  "rounded" | "thick" | "double" | "none"` in config.toml.
- **None** draws blanks where the lines were: the panes keep their room and
  titles, so nothing moves.
- An unknown value (`border_style = "wavy"`) warns in the status bar and
  uses Plain; the rest of the config still loads.
- Its own icon in each set: 🔲 / nf-md-border_outside `f00ce` / ▢.

### AI summary heading follows the icon set (`menu-layout-borders`)

- The *AI summary* heading in the preview and the video tab uses the icon
  set's own summary icon: ✦ in Emoji and Unicode, nf-md-creation `f0674`
  in Nerd.

### Tabs live in the menu only (`menu-layout-borders`)

- **No more tab bar.** The row across the top (*Home* and the video tabs)
  is gone: open tabs show only under TABS in the left menu, and the main
  pane gets that row back. *Home* was the browsing view's own tab (upstream
  *Inicio*); the menu already is that view.
- `Ctrl+Y 1`–`9` still went to the tabs as numbered in the menu and
  `Ctrl+Y 0` back to browsing (both removed later in 0.8.6, see *Unlimited
  tabs*). Help and hints say "in the menu" instead of
  mentioning the tab bar. Clicks land on the menu rows (nothing left to
  click at the top).

### Alt+, and Alt+. switch tabs (`browser-tab-keys`)

- **`Alt+.`** goes to the next tab in the menu and **`Alt+,`** to the
  previous one (the `>` and `<` keys), from any pane: lists, the search
  box, a video tab. One hand, next to each other. They wrap around: from
  the last tab to the first and back. From the browsing view, `Alt+.` goes
  to the first tab and `Alt+,` to the last. With no tabs open they do
  nothing.
- Every terminal sends `Alt+key` as `Esc` and the key without any setting,
  and none of the usual ones (kitty, Ghostty, Alacritty, foot, WezTerm,
  GNOME Terminal, Konsole) nor Omarchy's Hyprland binds use `Alt+,` or
  `Alt+.` by default, so nothing has to be changed in the terminal. On
  macOS, where Option+, and Option+. type `≤` and `≥` unless the terminal
  treats Option as Alt, those count as the tab keys too.
- The first try, `Ctrl+Tab` / `Ctrl+Shift+Tab`, is gone: it needed the kitty
  keyboard protocol and kitty and Ghostty keep `Ctrl+Tab` for their own
  tabs. tubevibe no longer turns that protocol on, and the README no longer
  asks you to edit `kitty.conf` or the Ghostty config.
- **`Ctrl+Y n` and `Ctrl+Y p` are gone** (help, README). `Ctrl+Y 0`–`9`
  stayed then (removed later, see *Unlimited tabs*). The new keys are in the full help (`Ctrl+Y ?`, under *Always*) and
  the README, not in the shortcut bar.

### Fullscreen works, from the video tab and from the lists (`browser-tab-keys`)

- **What went wrong:** in window mode (the default) `f` in a video tab
  blanked the terminal to "make room" for a picture that lives in mpv's own
  window, and when the tab had no running player (a restored tab, or a video
  that had ended) the terminal stayed blank with nothing to show. In the
  lists `f` did nothing at all.
- **Window mode:** `f` now asks mpv to put its own window in fullscreen and
  leaves the terminal as it is. On Hyprland, if mpv's window did not follow
  (a window rule, an older mpv), tubevibe tells Hyprland directly through
  `hyprctl`: the current Lua dispatcher (`hl.dsp.window.fullscreen`) first,
  the classic `fullscreen` dispatcher on older versions. mpv's own `f` and
  `Esc` are followed (tubevibe watches mpv's `fullscreen` property), and
  `f`, `Esc` or moving to another tab in tubevibe takes mpv out again.
- **Embedded and overlay:** the video fills the terminal as before, and the
  terminal window itself goes fullscreen on Hyprland (found through the
  process tree, then `hyprctl`) as well as on X11 (`wmctrl`).
- **From the lists:** `f` goes fullscreen for the video that is playing (in
  embedded mode it switches to that video's tab first).
- **Never a blank screen:** with nothing to show, `f` says why in the status
  bar instead: *Nothing is playing · open a video, then f*, *The video isn't
  playing · Enter plays it, then f*, or *Audio Only: no picture · a brings
  the video back*.

### Consistency pass (`browser-tab-keys`)

- **Key names** are written one way: `Space`, `Del`, `Backspace`, `Enter`,
  `Esc` in English, `Espacio`, `Supr`, `Retroceso` in Spanish. No more
  *espacio* in the English interface.
- **One language at a time:** error messages from YouTube, the updater,
  downloads and the config file now follow the interface language (they
  were Spanish-only or English-only); HTTP errors read *HTTP 400 · reason*.
  The help's mouse names follow the language (*Click*, *Double Click*,
  *Wheel*, *Click Video*, *Right Click*, *Click Bar*).
- **Settings in Title Case:** every setting name in English (*Downloads
  Folder*, *Tell Me About New Videos from My Channels*…), values styled the
  same way (*On* / *Off*, *Emoji* / *Nerd Font* / *Unicode*, *Original +
  subtitles*…), Spanish ones capitalised (*Sí* / *No*, *Español (es)*).
- **Pane, dialog and help titles in Title Case:** *AI Summary*, *Up Next*,
  *No Related Videos*, *Comments Are Off*, *Keyboard Shortcuts · Any Key
  Closes*, *System Check*…
- **One name per thing:** *System Check* everywhere (Settings, the page,
  messages and the README; it was also *Check the system*), *Audio Only*
  likewise.
- **`Ctrl+Y`** is written that way everywhere: the app, the help, the
  README, the website and this file.
- **Folders follow the system:** with no folders set, My Videos, My Music
  and downloads use the system's own folders (XDG `Videos` / `Music`, in
  the system's language) instead of a hard-coded `~/Música`, and their
  labels follow the interface language.
- **Help:** lists `Enter` in a video tab, `j` `k` and `Esc` in Up Next,
  `Backspace` as back, `f` in the lists and the new tab keys.
- **Icons:** the marks in Downloads (marked, downloading, done, failed,
  stopped) and in the System Check (fine, warning, missing, optional) come
  from the chosen icon set instead of fixed characters, each two cells wide
  so the columns line up.
- **docs/:** the old Spanish tubetui website and benchmark page are now
  English tubevibe pages; the benchmark says its numbers were measured on
  tubetui and both credit polblancoo.
- **README config example** uses English and standard paths (`~/Videos`,
  `~/Music`, `~/Downloads`, `sub_lang = "en"`) and section names in English.

### The menu walks the open tabs (`browser-tab-keys`)

- `↑` / `↓` in the left menu used to jump from Downloads straight to
  Settings, skipping TABS. Now they walk every row in the order it shows:
  the sections, then each open tab, then Settings, About and Quit (still
  wrapping around). On a tab the cursor lights it like a focused section,
  and `Enter` opens it. Closing the tab under the cursor puts the cursor
  back on the sections.
- Coming back to the menu from a video tab (`←` or `Ctrl+Y m`) leaves the
  cursor on that tab. `Alt+,` / `Alt+.` are unchanged (`Ctrl+Y 1`–`9`
  were later removed).

### → opens the menu's row; ← comes back to where you were (`browser-tab-keys`)

- In the left menu `→` now does what `Enter` does on the selected row:
  open the section, open the tab, or open Settings / About. **Quit is the
  exception**: only `Enter` quits, `→` on Quit does nothing. Before, `→`
  did nothing in the menu. `l` still does nothing there (outside the menu
  it is *save to tubevibe Watch Later*, as always). `Tab` still switches
  panes. The help's Menu box reads *Enter → · open (Quit: Enter only)*;
  the bar keeps just `Enter`.
- `←` (or `Esc`) from the first list, and `←` from a video tab, put the
  menu's cursor back on what that content came from: the section or the
  tab. Fixed where it did not:
  - **Search results** (`/` or `Ctrl+Y /` from any list, searching all of
    YouTube or my channels; a channel search) replace the list, but the
    cursor stayed on the section you searched from: it now goes to
    *Search Videos* / *Search Channels*. A search inside a channel opens
    on top of that channel, so `←` goes back to it first, as before.
  - The **System Check** that opens by itself at startup, and the first-run
    **browser picker**, left the cursor on My Recommendations: it is now on
    Settings, where both live.
  - Every section, Settings and About open through the same place, which
    now sets the cursor itself, so a section opened any other way (a
    click, a key) comes back right too.

### Esc on a video tab goes to the menu; only the focused pane is lit (`browser-tab-keys`)

- `Esc` in Up Next still goes back to the video's info. `Esc` there now
  goes to the left menu with the cursor on the tab, like `Esc` from a list
  and like `←`. (In fullscreen the first `Esc` still just leaves it.)
  Before, `Esc` on the video's info did nothing you could see, and when the
  list behind the tab was one level deep (a channel, a playlist) it quietly
  took that list away, so you were back a level after leaving the tab.
- While Up Next has the keys, the video pane's border (or, with Border
  Style → None, its title) is dim, so only Up Next is lit; back on the
  video's info it lights up again. It uses the same focused / dim colours
  as the menu and lists, so it follows the Omarchy theme. The comments
  pane never takes the keys (`PgUp` / `PgDn` scroll it from the tab) and
  stays dim.

### The video stays on screen while the menu has the keys (`browser-tab-keys`)

- `Esc`, `←` or `Ctrl+Y m` on a video tab used to switch the main area
  back to the list behind it. Now the tab stays shown (and keeps playing,
  embedded, overlaid or in its window) while the menu takes the keys: the
  menu's border is lit, its cursor is on the tab, and the video pane and
  Up Next go dim.
- `↑` / `↓` only move the cursor; the main area changes when `Enter` (or
  `→`, not on Quit) opens a row: a section (the browsing view), another
  tab, or the same tab, which simply gets the keys back. `Tab` also goes
  back to the tab shown.
- While the menu has the keys, the player keys (`Space`, `←`, volume…)
  and the video letters (`l`, `w`, `a`, `d`…) do nothing; `/` searches and
  `f` goes fullscreen (the video then takes the keys). Fullscreen ends when
  the menu is called, since it would hide it. A click on Up Next gives it
  the keys back.
- `Alt+,` / `Alt+.` and restored tabs are unchanged; going
  to a tab with them gives it the keys.

### t opens a video in a background tab (`browser-tab-keys`)

- **`t`** on a YouTube video opens it in a new tab in the background. That
  works in the browsing lists, search results, channels, History, Watch
  Later and playlists, and in Up Next in a video tab.
- The tab joins TABS in the menu and is saved to tabs.json. The keys and
  the screen stay where they are, and nothing plays.
- The tab behaves like a restored one: nothing is fetched until it's shown
  (Enter plays it).
- The status says *Opened in a New Tab: <title>* (*Abierto en una pestaña
  nueva*). If the video is already in a tab there's no second one, and the
  status says *Already Open in a Tab: <title>* (*Ya está en una pestaña*).
- The tab list scrolls and its chevrons update as usual: a tab added out
  of view shows the down chevron.
- **`t` wasn't used anywhere before**, so there's no conflict.
  - In the menu, Settings, About, System Check, My Videos, My Music and
    Downloads (local files, not YouTube videos) it does nothing.
  - In a search box it's typed. In a video tab, on the video itself, the
    video is already in a tab, so the status says so.
- It's in the help (*On a Video*) and the README, not in the bottom bar.
- Tests: `t_opens_the_video_in_a_background_tab` and
  `a_background_tab_joins_the_scrolled_tab_list`.

### The menu's cursor row in the theme's accent; no blended colours (`browser-tab-keys`)

- **One colour for the menu's cursor row**, both the `▌` and the text:
  sections, tabs, and Settings / About / Quit.
  - Before this release each section had its own colour and a tab used
    magenta. The first try in 0.8.6 used orange.
  - It's now the theme's **`accent`**, from colors.toml with Omarchy
    colours on (Gruvbox `#7daea3`). Built-in Tokyo Night uses its accent,
    `#7aa2f7`.
  - Tokyo Night's accent is the same colour as its blue, so the interface
    draws the accent with a stand-in one step off and the recolouring
    turns it into the real accent, built-in theme included.
  - The pink "shown" mark is unchanged.
- **No more blended palette colours.** Each role now falls back only to
  colours the theme file has:
  - **orange:** `orange`, else `yellow` (was red and yellow mixed).
  - **secondary text:** `dark_foreground` / alacritty `dim_foreground`,
    else `muted`, else bright or normal black, the first readable on the
    background; else the text colour. Before, it was mixed 45% from text to
    background.
  - **selected row:** `selection` / `selection_background`, else
    `lighter_background`, else bright or normal black, the first close to
    the background yet readable under the text. Failing that, the first of
    those that differs from the background, else the background. Before,
    it was mixed 14% from background to text.
  - **dialog lines:** `muted`, else bright or normal black, else secondary
    text. Before, it was mixed 25% from background to text.
  - **accent:** `accent`, else `blue`, as before.
- **What changes in the fixture themes:**
  - Omarchy 3 and 2 Gruvbox get `#3c3836` (their black) for the selected
    row, and their text colour for secondary text, since they have no
    readable dim colour. Their orange is now their yellow.
  - Themes with a full Omarchy 4 colors.toml are unchanged.
- **Accent on the selected row (contrast):**

  | Theme | Contrast |
  |---|---|
  | Tokyo Night (built-in) | 5.3 or more |
  | Gruvbox v3 / v2 | 4.67 |
  | Gruvbox | 3.55 |
  | Catppuccin Latte | 3.18 |
  | Rosé Pine | 2.47 |
  | Catppuccin Latte from alacritty.toml | 2.27 (no accent: blue on its bright black) |

  The two light ones are weak. The row is bold and marked with `▌`.
- Tests: `every_menu_cursor_row_is_in_the_theme_accent` (render, Tokyo
  Night and Gruvbox, every row), `every_colour_is_one_the_theme_file_has`
  (all seven fixtures), and `the_accent_reads_on_the_selected_row`.

### Menu, Preview and Comments widths in Settings (`browser-tab-keys`)

Three new rows in Settings, right after Border Style. Enter steps each one up
and wraps from the largest back to the smallest, like the other cycling
settings. They're saved to `[ui]` in config.toml and apply at once.

| Setting (es) | Key | Unit | Range, step | Default |
|---|---|---|---|---|
| Menu Width (Ancho del menú) | `menu_width` | columns | 20–40, 2 | 26 |
| Preview Width (Ancho de la vista previa) | `preview_width` | columns | 30–70, 4 | 46 |
| Comments Width (Ancho de los comentarios) | `comments_width` | columns | 30–78, 4 | 46 |

- **The defaults are the old layout**, so nothing changes for anyone who
  doesn't touch them. Comments Width was a percent at first (30–60, default
  45). It's now in columns, and 46 is what 45 % gave in a 130-column
  terminal. A saved percent is read as that many columns, which is about
  the same width at that size: no warning, and Enter steps on from it.
- **Never vanish, never overlap:**
  - The menu is never wider than half the screen.
  - The preview shows only while the list beside it keeps 50 columns. That's
    the old 96-column rule at the default 46, so a wider preview needs a
    wider screen (70 needs 120 columns of content).
  - The comments still need a video tab 100 columns wide inside, and Up Next
    keeps at least 40 columns beside them.
- **Loading is tolerant:** a value out of range or of the wrong type is
  reported like other bad settings ("Bad value for ui.menu_width in
  config.toml: using the default") and falls back to its
  default. It never stops startup, and older builds skip the new keys.
- Tests: `sizes_load_tolerantly`,
  `menu_width_min_and_max_and_never_more_than_half`,
  `preview_width_min_and_max_keep_the_list_its_columns`, and
  `comments_width_min_and_max_and_the_100_column_threshold`. The settings
  cycling test covers the new rows.

### Video tab: comments full height, Up next right under the summary (`browser-tab-keys`)

- **Comments take the right side of the video tab from top to bottom**
  (when it's 100 columns wide or more, as before). The left column has the
  thumbnail and details on top, the AI summary under them across the
  column's whole width, and **Up next** one blank row below the summary's
  last line.
- **The whole summary shows, with no more rows than it needs.** The layout
  is recomputed on every draw, so it follows the summary's arrival, a
  resize, and new detail lines (the size, the "Enter plays it" note). A very
  long summary or a short terminal caps it so Up next keeps 7 rows (its
  border plus 5 videos). Past that the summary is cut with "…", as before.
  Clicks on Up next still land on the rows as drawn.
- The thumbnail is now two fifths of the left column, at most 36 columns
  (22 at 130 columns), so the details beside it keep room.
- **Embedded / overlay video:** unchanged. The picture covers the whole
  inside of the video tab; the details, summary, Up next and comments
  layout only shows when the video plays in its own window or hasn't
  started.
- **Menu:** the blank row between the logo and YOUTUBE (added in 0.8.1) is
  gone, and YOUTUBE sits right under the logo. The other groups keep their
  blank row.
- Tests: `up_next_keeps_seven_rows_under_a_long_summary_at_any_height`,
  `a_short_summary_takes_only_its_rows_and_narrow_tabs_have_no_comments`,
  `youtube_is_right_under_the_logo`; the summary render test now checks the
  full-height comments.

### x in the menu closes a tab; chevrons above and below (`browser-tab-keys`)

- **`x` on a tab row in the menu closes that tab**, the same as `Ctrl+Y x`
  inside it: its player stops and tabs.json is saved, with no confirmation
  (`Ctrl+Y x` has none either). The cursor goes to the next tab, else the
  previous one, else back to the sections. If the closed tab was shown, the
  main area shows what `Ctrl+Y x` would: the tab that took its place, or the
  browsing view when none is left. `x` on a section row does nothing. It's
  in the help's Menu box, and the bar shows *x Close Tab* only while the
  cursor is on a tab.
- **The short rule above TABS is gone**; the one above Settings stays.
- **Scroll chevrons:** the one next to the TABS title now only means "more
  above" (an up chevron). A row under the tab list holds a down chevron,
  in the same column, while tabs are hidden below. Toby's screenshot showed
  *TABS ⌄*: before, the title showed the down chevron, the up one, or both.
  Both chevrons come from the chosen icon set. With nothing hidden there
  are no chevrons and no reserved row. The down chevron sits on the blank
  row just above the rule over Settings, with no blank row between them.
  The tabs keep every row above it, so as many tabs show whether or not it's
  there. This holds down to the shortest menu that still has a tab row.
- Tests: `x_in_the_menu_closes_the_tab_under_the_cursor`,
  `few_tabs_have_no_chevrons_and_no_rule_above`, and the 35-tab render test,
  which now checks both chevrons and their column.

### Unlimited tabs; the menu's tab list scrolls (`browser-tab-keys`)

- **No tab limit.** The `max_tabs` setting is gone, and so is its "At most N
  tabs" message. An old `max_tabs` line in config.toml is ignored without a
  warning (it was a real key, not a typo). Restored tabs (tabs.json) all
  come back.
- **`Ctrl+Y 0`–`9` are removed**, from the keys, the help, the shortcut bar,
  the README and the website. `Ctrl+Y 1`–`9` went to the tab with that
  number; `Ctrl+Y 0` went back to the browsing view. Use `Alt+,` / `Alt+.`,
  the menu (`↑ ↓`, `Enter`) or a click. The tab numbers in the menu ("1 ",
  "2 ") only existed for those keys, so they are gone too.
- **The menu's tab list scrolls on its own.** The sections at the top and the
  rule with Settings / About / Quit at the bottom stay put; TABS takes the
  rows in between. With the menu's keys it keeps the cursor's tab in view,
  otherwise the tab shown. The TABS title shows ▲/▼-style marks (from the
  icon set) when tabs are hidden above or below. The mouse wheel over the
  tabs scrolls them; the list follows the cursor or the tab shown again as
  soon as that changes.
- A short rule above TABS was added here, then removed (see the entry above).
- **Short terminals:** the tab list shrinks down to one row. With no room
  even for that, the whole menu becomes one list scrolled to the cursor (or
  the tab shown), as a short menu did before.
- Tests: `thirty_five_tabs_scroll_between_a_fixed_top_and_bottom`,
  `a_short_menu_keeps_one_tab_row_then_falls_back_to_one_scrolled_list`,
  `tab_rows_have_no_number`, `tabs_are_unlimited`,
  `restoring_tabs_keeps_them_all_respects_the_setting_and_bad_files` (35
  tabs), `the_wheel_over_the_menu_tabs_scrolls_them`,
  `an_old_max_tabs_line_is_ignored_quietly`.

### One "shown" mark in the menu (`browser-tab-keys`)

The menu row for whatever the main area is showing now always has the same
pink "shown" mark (the theme's magenta, remapped under Omarchy) when the
cursor isn't on it. That applies to a section and to a video tab alike. The
cursor row keeps its own focus highlight, and exactly one row is marked.

- Search results mark Search Videos or Search Channels. The startup System
  Check and the browser picker mark Settings.
- The mark moves only when another list actually replaces the one shown.
  Opening a search box, or a section that only shows a hint (for example My
  Playlists while logged out), leaves the mark where it was. In that case ←
  lands on the marked row.
- Before this, a shown tab had a highlighted background and the selected
  section kept its own section colour.
- Tests: `one_row_is_marked_shown` (render),
  `the_shown_row_is_what_the_main_area_shows`, and
  `search_results_and_the_startup_check_mark_their_section_shown`.

### My Recommendations is kept for a minute (`browser-tab-keys`)

- Opening **My Recommendations** asked YouTube for the whole home feed
  every time, even a few seconds after the last one. Now a feed that loaded
  is kept for 60 seconds: opening it again within that minute shows the
  same list right away, with the cursor and the pages already loaded as
  you left them, and nothing is fetched (not the feed, nor the history or
  Watch Later reads that go with it). Videos watched meanwhile are still
  hidden. After 60 seconds it is read again as before.
- A load that failed is not kept: the next open tries again at once. A new
  login (or *Read the Login Again*) and switching the interface language
  drop the copy too.
- There is no refresh key; the minute is counted from the last load that
  worked. Only My Recommendations changed: Subscriptions, Discover
  Channels (same feed code), Watch Later, My Playlists, My Channels and
  History still read on every open.

### Video tab: ← goes to the menu, Shift+← → seek (`browser-tab-keys`)

- `←` in a video tab (and in its Up Next list) now goes to the left menu,
  as `←` does from the first list everywhere else. It used to seek back
  5 s; seeking is now `Shift+←` / `Shift+→` (5 s), and `→` alone still
  seeks forward. A click on the progress bar still jumps anywhere.

### Shorter video tab bar (`browser-tab-keys`)

- The bottom bar of a video tab no longer repeats the help's *Enter: play ·
  in Up Next: the selected video* row, `Space` comes before `Tab`, and
  `v` / `b` read *Quality Change* / *Codec Change* (*Cambiar calidad* /
  *Cambiar códec*) instead of listing every step. The bar's last item, on
  every screen, is now `Ctrl+Y ?` *Help* (*Ayuda*).

### Open tabs come back after a restart (`menu-layout-borders`)

- The open tabs are kept in **`~/.local/share/tubevibe/tabs.json`** (the
  data folder, not config.toml): each tab's title, subtitle, video (id,
  title, channel, duration…), queue (URLs or file paths) and audio-only
  flag, in order, plus which tab was in front. It is rewritten whenever a
  tab opens or closes, so quitting any way (`q`, `Ctrl+C`, a closed
  terminal, a crash) leaves it current.
- On startup they come back under TABS in the menu, all of them (once up to
  `max_tabs`, a limit since removed), with
  the browsing view in front. Nothing plays and nothing is fetched: a tab
  loads its details, comments and SponsorBlock segments the first time it's
  shown, and `Enter` plays it (from where the history says it was left).
- A missing file, or one from another version, restores nothing. A file
  that can't be read warns in the status bar (*"Could not restore the tabs:
  …"*), is set aside as `tabs.json.bak`, and startup carries on.
- **Settings → Restore Tabs on Startup** (*Recuperar las pestañas al
  abrir*), on by default, saved as `[ui] restore_tabs = true | false`. Off,
  tubevibe starts with no tabs (and stops keeping the old ones). Icon: 📑 /
  nf-fa-window_restore `f2d2` / ❐.

### Watch Later fails with a clear message, errors in English (`watch-later-null-fix`)

- **Watch Later that YouTube won't hand over.** When yt-dlp can't read a
  whole list with `--dump-single-json --ignore-errors`, it prints `null` on
  stdout (exit 1) and the reason on stderr. tubevibe took the `null` as a
  list and showed *"respuesta inesperada: invalid type: null, expected
  struct FlatList"*. For Watch Later (`WL`) and Liked videos (`LL`) the
  reason is *"YouTube said: The playlist does not exist."*: they only exist
  for a login YouTube takes. Now:
  - a `null` list with a failed run reports yt-dlp's own `ERROR:` line;
  - for `WL`/`LL` it says *"YouTube returned no data for your Watch Later:
    check that you're signed in (⚙ Settings → Log in with a browser)"*, and
    counts as a stale login, so the browser's cookies are read again once
    and the list retried before giving up;
  - a `null` from a run that succeeded is an empty list, not an error.
- **The failed section's own title.** A section opened from the menu that
  fails to load now shows an empty list under its own title, not the
  previous section's (Watch Later used to sit under *"My Recommendations ·
  0"*). Reloading the list on screen still keeps it.
- **Errors follow the language.** yt-dlp/YouTube errors were Spanish
  whatever the language (*"no se pudo ejecutar yt-dlp"*, *"yt-dlp falló"*,
  *"respuesta inesperada"*, *"sin sesión…"*, *"sin detalle"*, *"yt-dlp sin
  salida"*). Also channel-search follower counts (*"1,2 M suscriptores"* →
  *"1.2 M subscribers"*, views/likes numbers too), the startup *"no se
  encontró la carpeta personal"* and the updater's *"respuesta de GitHub sin
  versión"*.

### My Playlists icons follow the icon set (`watch-later-null-fix`)

- **Liked Videos and Watch Later (YouTube)** had a hard-coded 👍 / 🕒 inside
  their names, after the playlists icon, whatever the icon set (Nerd and
  Unicode showed emoji too), and their names sat one icon to the right of
  the other playlists'. Now each row starts with one icon from the chosen
  set: liked for Liked Videos, Watch Later for Watch Later, the playlists
  icon for the rest, so every name starts in the same column. The names
  carry no emoji (the pane title too), in Title Case: *"Liked Videos"*,
  *"Watch Later (YouTube)"* (Spanish: *"Videos que me gustan"*, *"Ver más
  tarde (YouTube)"*).
- Same fix where other hard-coded emoji ignored the icon set: the 👍 like
  counts in the video tab and on comments, the 💾 download-size line in
  the preview (now the Downloads icon), and status messages that have an
  icon of their own (Watch Later saving/checking 🕒, Liked/Disliked
  👍/👎, the favourite ⭐ in new-video notices, and the update
  ⬆/⬇/✅/❌ lines).

### Every status icon follows the icon set (`watch-later-null-fix`)

- New icons in all three sets (Emoji keeps the old emoji; Nerd uses Nerd
  Fonts v3 Font Awesome / Material Design glyphs; Unicode uses text symbols
  that never turn into colour emoji):

  | Purpose | Emoji | Nerd | Unicode |
  |---|---|---|---|
  | Subscribed, new videos | 🔔 | nf-fa-bell `f0f3` | ⚑ |
  | Unsubscribed | 🔕 | nf-fa-bell_slash `f1f6` | ⚐ |
  | Resumes at | ⏯ | nf-md-play_pause `f040e` | ▸ |
  | SponsorBlock skip | ⏩ | nf-fa-forward `f04e` | » |
  | Video closed, download cancelled | ⏹ | nf-fa-stop `f04d` | ■ |
  | Sound only | 🎧 | nf-fa-headphones `f025` | ♪ |
  | Saved to tubevibe Watch Later | 🔖 | nf-fa-bookmark `f02e` | ♦ |
  | Warnings (config.toml, missing tools) | ⚠ | nf-fa-warning `f071` | △ |
  | Thumbnail loading | ⏳ | nf-fa-hourglass_half `f252` | ◌ |
  | Donation line on About | ☕ | nf-fa-coffee `f0f4` | ♥ |

- Every status message now takes its icon from the set: the ones above,
  plus 🗑 (removed / deleted / trash), ❌ (failures), ✅ (download done) and
  ⬇ (downloading, and the Downloads count in the menu), which already had
  icons in each set. The ⚙ in "not logged in" messages is gone (they say
  *Settings*).
- Tests: every set has every icon; Nerd and Unicode have no emoji and Nerd
  stays in the v3 Font Awesome / Material Design ranges; a status line
  follows the chosen set.


## 0.8.7: dynamic layouts, Title Case labels, fitted list rows, Watch Later moved (2026-10-20)

### Title Case bottom bar, Up Next titles cut with … (`titlecase-and-upnext-ellipsis`)

- Every English label in the bottom key bar is now Title Case, like
  *Quality Change* and *Help* ("Pause", "Close the Tab", "YouTube Watch
  Later", "Mark / Unmark"…). Short joining words (to, the, of…) stay
  lowercase, *tubevibe* stays lowercase, key names are untouched. Spanish
  is unchanged.
- Up Next: a long title is cut with a single `…` so the video length always
  shows whole at the right edge, lined up row to row. Measured in screen
  cells, so wide characters and emoji don't push it out. Very narrow panes
  cut the title first, then the date and channel; the length goes last.
- The same fitting now applies to every list opened from the menu: My
  Recommendations, Watch Later, Subscriptions, History, playlists and their
  videos, channels' videos, Search Videos, From What I Watched, Discover,
  tubevibe Watch Later (all video lists), My Playlists (the count stays at
  the right edge), My Videos / My Music (file names) and Downloads (the
  path or error gives way first, then the title). Before, long titles were
  cut at the pane edge and pushed the length out of view.
- The channel column (and the My Videos / My Music folder column) ends in
  `…` when cut, instead of stopping mid-word.
- Right-to-left titles (Arabic, Hebrew): invisible direction marks are
  dropped and widths are measured the way the screen draws them, so those
  rows keep the length column lined up.
- Menu: **Watch Later** now sits right above **Subscriptions** (My
  Recommendations, Watch Later, Subscriptions, History, …). Icons, keys and
  the shown mark follow the new order.
- Narrow windows: the **preview** goes under the list (across it) when the
  list would keep fewer than 50 columns beside a Preview Width panel, so
  it is no longer cut off; the list keeps at least 10 rows and the preview
  takes up to 16 (with fewer than 6 to spare it is left out, never
  squeezed in). Its thumbnail stays 16:9 and takes at most half its rows.
- Likewise the **comments** in a video tab go under the details and *Up
  next*, across the tab, when those would keep fewer than 50 columns
  beside Comments Width (two fifths of the rows; left out when the tab is
  too short for them and *Up next*). Wide windows keep the side-by-side
  layout and the width settings; stacked panes take the full width.


## Unreleased (0.8.9)

### Thumbnails in any terminal

- Thumbnails are drawn with ratatui-image instead of the kitty-only code: real
  pictures in kitty, sixel and iTerm2 terminals, coloured half blocks
  elsewhere (Alacritty, and always inside tmux/Zellij). In lists the thumbnail
  sits at the top of the preview pane; on a video tab, left of the details.
  It is skipped when the space is too small.
- Settings: **Video Thumbnails** is now Picture / Blocks / Off (the old
  `thumbnails = true/false` still reads), and **Thumbnail Pixelation**
  (Off/Low/Medium/High/Max, `[ui] thumb_pixelation`).
- Images are fetched once (the same mqdefault URL) and cached in memory and in
  `~/.cache/tubevibe/thumbs` (capped at 30 MB).
- Only embedded video still needs kitty; the note no longer shows on Thumbnails.

### Up Next is at most 21 rows tall

- The Up Next pane on a video tab is capped at 21 videos plus its border
  (it never lists more) and sits at the bottom of the left column; spare
  rows go above it, under the details and AI summary. On short windows it keeps scrolling so the
  selected video stays visible. Comments keep their full height.

### → from the menu opens a tab on its video

- Pressing → on a video tab in the menu now focuses the video pane
  instead of Up Next (the tab used to keep its last Up Next focus).

### Linux-only releases (`linux-only-releases`)

- The release workflow builds only `tubevibe-x86_64-linux.tar.gz`,
  `tubevibe-aarch64-linux.tar.gz` and `tubevibe_amd64.deb`; the macOS and
  Windows jobs (and the Windows installer step) are gone. macOS and Windows
  users build from source with cargo; README, the docs page and `install.sh`
  say so (on macOS `install.sh` builds from source).
- Self-update on a machine with no prebuilt file (macOS, Windows): About says
  "No Prebuilt Update for This Platform: Build From Source" and shows the
  `cargo install` command, instead of a bare "No Download for This System".
- Test: `no_prebuilt_file_on_macos_or_windows_says_build_from_source`.

### Needs Kitty Terminal note (`settings-kitty-note`)

- **Settings → Player Mode and Video Thumbnails** show **Needs Kitty Terminal**
  (in the theme's red) when they need kitty and the app isn't running in kitty:
  Player Mode set to Embedded, or Thumbnails turned on. Embedded video and
  thumbnails are drawn with kitty's graphics protocol, which terminals like
  Alacritty don't have.

### Balanced dialog padding (`dialog-padding`)

- **Confirm dialogs** have a blank line below the hint as well as above the
  question, so the text sits in the middle of the box.

### Key-cap bottom bar and shorter labels (`keycap-bar`)

- **Key-cap bottom bar**: each key is a solid cap (the theme's background on its
  text colour, exact theme colours), followed by its label in the text colour.
  Capital keys stay bold. `? Help` always fits.
- **Shorter labels** in the bottom bar and Help, e.g. *Close Tab*, *Close, Remove
  From List*, *Change Resolution*, *Change Codec*, *Replay*, *Seek ±5s*,
  *Open Channel*, *Trash Marked*, *Resume*, *Search In*. No more "(Asks First)":
  the confirm dialog already asks.

### Version on the menu border

- The left menu's top border shows the version (`v0.8.9`, muted colour)
  right-aligned, on the same line as the ▶ tubevibe logo. On a menu too narrow
  for both (under 23 columns) the version is left out and the logo stays.
- Version bumped to 0.8.9.

### `{` and `}` resize the menu (from boxvibe)

- `{` makes the left menu 2 columns narrower and `}` 2 columns wider, live,
  between 20 and 40 columns (and never more than half the screen, as before).
  Saved right away as `[ui] menu_width` in config.toml, the same value as
  Settings → Menu Width. The status bar says the new width. Works anywhere
  but in the search box; Help lists it on the `M { }` row.

## 0.8.8: single-key shortcuts (no more Ctrl+Y), Restart Now, t/T tabs, wrapped dialogs and Help (2026-10-20)

### Help rows never run out of their box (`key-remap`)

- Help descriptions now wrap by cell width, continuing indented under the
  description, so no row is cut at any width. Menu: `x` closes the tab under
  the cursor, `X` closes it and removes it from its list (two rows); `Tab`
  and `/` share one. A few long descriptions were shortened, so Help still
  fits 40 lines at 130 columns with nothing wrapped.
- In a window too small for it (e.g. 100x40), Help scrolls: ↑ ↓ / j k,
  PgUp PgDn (and Space); the title says so; any other key closes it.
- The confirm box's area (used to keep a video from drawing over it) now
  matches its wrapped size.
- Test: `no_help_row_is_clipped` (both languages, 130x40, 118x40, 100x40,
  80x30).

### R opens My Recommendations; X on a tab in the menu (`key-remap`)

- **`R`** opens My Recommendations from anywhere, like choosing it in the
  menu (it reloads only if older than 60 s). Not while typing. In Help it
  shares the `R  S  A` row.
- Fix: **`X`** (or Del) on a tab under the menu cursor did nothing. Now it
  works like `X` in that tab: it asks, naming the list the video came from,
  then closes the tab and takes the video out of that list. A tab with no
  such list just closes, like `x`.
- Tests: `capital_keys_open_sections_from_anywhere_but_not_while_typing`
  (now with `R`), `shift_x_on_a_tab_in_the_menu_works_like_in_the_tab`.

### Confirm dialogs wrap instead of cutting the question (`key-remap`)

- Every yes/no box word-wraps its question by cell width, keeping the
  coloured title and list name, and grows to fit (40–72 columns, never past
  the screen less a margin; height capped too). Titles are no longer cut at
  20–24 characters; only when the question would not fit the screen is the
  title shortened with `…`, so the list name and the `?` always show. The
  `y = yes · any other key = no` line stays on its own. No panic at tiny sizes.
- Tests: `confirm_wraps_long_messages_at_80_columns`,
  `confirm_shortens_only_the_title_on_a_narrow_screen`.

### S, A, Y, V open screens from anywhere (`key-remap`)

- **`S`** Settings, **`A`** About, **`Y`** YouTube Watch Later, **`V`**
  tubevibe Watch Later: from any list, video tab, *Up next* or Downloads,
  like choosing that menu item. Not while typing (they type the letter). No
  clash: these capitals were free everywhere (`s`, `a`, `v` stay as they are).
- Help: they share two rows of Commands (`S  A`, `Y  V`), and `M  L`,
  `Q  Ctrl+C` now share rows too, so Help still fits a 40-line terminal. Not
  in the bottom bar (no room).
- Test: `capital_keys_open_sections_from_anywhere_but_not_while_typing`.

### t goes to the new tab, T stays; uppercase keys in bold (`key-remap`)

- **`t`** on a video (any list or *Up next*) now opens it in a new tab and
  shows it; **`T`** opens it in a background tab without moving, as `t` did
  before. If the video is already open, `t` goes to its tab. Both are in the
  video's context menu too, and in Help.
- Shortcuts that are an **uppercase letter** (`Q`, `M`, `L`, `W`, `X`, `T`,
  `G`…) are drawn **bold** in the bottom bar, Help and context menus, in the
  same theme colour; lowercase keys are drawn regular, so Shift keys stand out.
- Tests: `t_opens_a_tab_and_goes_to_it_shift_t_stays`,
  `uppercase_keys_are_bold_lowercase_are_not`.
- The help key is shown as `?` everywhere (bottom bar `? Help`, Help,
  splash) instead of `Shift+/`; the key itself is unchanged.

### One key per command: no more Ctrl+Y (`key-remap`)

- The `Ctrl+Y` prefix is gone (`Ctrl+Y` alone does nothing). Every
  command is now a single key: lowercase for everyday ones, Shift for the
  bigger ones.
  - `?`: help. `M`: show / hide the menu. `L`: English /
    español. `Q`: quit, after a yes/no question (it still says when
    downloads are running). Plain `q` does nothing; `Ctrl+C` still quits
    at once, and so does Enter on Quit in the menu. The old "go to the
    menu" command is gone (`←` / `Esc` / `Tab` do that).
  - `w`: YouTube Watch Later; `W`: tubevibe Watch Later. Plain `l` is free.
  - `x` only ever **closes**: the tab under the cursor in the menu, or the
    video tab you are in (Up Next too). It never deletes, trashes or
    removes anything.
  - `X` takes things out, **always after asking**: files to the trash in
    My Videos / My Music / Downloads (all the marked ones with Space), a
    video out of History, YouTube Watch Later, tubevibe Watch Later or a
    playlist, a running download cancelled, a failed one taken off the
    list, a playlist deleted in My Playlists. `Del` does the same in lists.
  - `X` in a video tab: close it **and** take its video out of the list it
    was opened from; the question names that list (YouTube Watch Later,
    tubevibe Watch Later or the playlist). A tab opened from anywhere else
    has nothing to take it out of, so `X` just closes it, like `x`. The
    list is remembered in `tabs.json` (an unknown or damaged value is
    ignored; the tab still comes back).
- **Restart Now**: once Update Now has installed the new version, the About
  button becomes *Restart Now*. It saves the tabs, stops playback, gives the
  terminal back and starts the new tubevibe with the same arguments (on
  Linux and macOS in place of the old one; on Windows a new process, then
  this one exits). Builds from source never restart this way.
- Bottom bar: only action keys now; moving around (arrows, `j k`, `Tab`,
  `Esc`, "go to the menu", "back") is in the help. The last hint is
  `? Help`.
- Help, README, the website and every hint name the new keys; the help's
  first box lists the commands that work anywhere, with `Alt+,` / `Alt+.`
  and `Ctrl+C`.
