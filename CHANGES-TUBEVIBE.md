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
- **`Ctrl+y` `x` removes from YouTube lists**: in Watch Later or any
  YouTube playlist opened from *My Playlists* (mine, saved), it asks
  *"Remove <video> from <playlist>?"* and, on yes, takes the selected video
  out on YouTube; the list updates when YouTube says it is done, errors go
  to the status bar. In **Liked videos** removing is taking the like back
  (the same request as `g` on a liked video). In a video tab, or anywhere
  else, `Ctrl+y` `x` still closes the tab. The right-click menu of those
  lists offers *Remove from this list*.
- **Plain `x` no longer removes** videos from Watch Later or playlists (it
  says *"Ctrl+y x removes it from this list"*). Elsewhere it is unchanged:
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

- **What most likely happened** (not yet confirmed on a live account): `Ctrl+y` `x` in *Saved Shorts* answered
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
  - Inside it `Ctrl+y` `x` (or right-click → *Remove from this list*) takes
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
  key hints, help screen (`Ctrl+y ?`), right-click menu and README say so.
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
- **Menu**: one blank row between the title and the YOUTUBE header, like the
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

## Unreleased

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
  no question: when Downloads is opened, and every 2 s while it is shown
  (only the listed finished paths are looked at). Running downloads are
  never dropped. (At start, missing files were already left out.)
- My Videos / My Music already rescan their folders each time they are
  opened (now covered by a test).

