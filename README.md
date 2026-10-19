# tubevibe

> **Based on [tubetui](https://github.com/polblancoo/tubetui) by polblancoo.**
> tubevibe is a fork with its own additions (see
> [CHANGES-TUBEVIBE.md](CHANGES-TUBEVIBE.md)); the original's website is
> https://polblancoo.github.io/tubetui/.

YouTube — and your own videos and music — in the terminal, written in Rust
with [ratatui](https://ratatui.rs). Videos play through [mpv](https://mpv.io):
in their own window by default (its tab shows the video's details and what to
watch next), or drawn **inside the app** in [kitty](https://sw.kovidgoyal.net/kitty/).
Every video opens in its own tab.

> Interface in English and Spanish (`Ctrl+Y l` switches). Early but usable:
> feedback welcome.

![Search results with the preview panel and the selected video's thumbnail](docs/preview.png)

![A YouTube video playing inside tubevibe, in kitty](docs/video.png)

## What it does

**YouTube**
- **My Recommendations** (your home feed, in YouTube's own
  order, without the videos already in your YouTube Watch Later),
  **Subscriptions** (newest from your channels, live and upcoming first),
  **My Channels**
- **Watch Later** (under YOUTUBE): your real YouTube *Watch Later* playlist
  (needs a login; the old local-only list is the Watch Later under TUBEVIBE). `w` adds the
  selected video to it right away (if it is already there, it just says so);
  inside the list `Ctrl+Y` `x` removes the selected video (after asking)
- **My Playlists**: your YouTube playlists, plus *Liked videos* and *Watch
  Later*; `Enter` opens one, `x` deletes it from YouTube (after asking).
  Inside one, `Ctrl+Y` `x` takes the selected video out of it on YouTube
  (after asking; in *Liked videos* that takes the like back)
- **History**: everything you watched, newest first, with the day you
  watched it: in tubevibe and, logged in, on YouTube too (the browser, the
  phone), more pages coming as you scroll down (`x` removes one from the list)
- **Resume where you left off**: closing a video remembers the second; opening
  it again starts there, and lists show how far you got. Finished videos (or
  ones you barely started) start from the beginning
- **New videos notice**: every 15 minutes your subscriptions are checked;
  new videos are announced in the status bar, counted in the menu
  (*Subscriptions (3 new)*), marked NEW in the list and, with
  `notify-send`, as a desktop notification
- **Shorts hidden** from feeds, searches and channels (a setting turns them
  back on); playlists you open (Watch Later and Liked videos too) always show
  them
- **Your favorite channels first**: the channels you watch most (3 videos or
  more in the last 30 days, up to 10) are marked ⭐. Their videos from the last
  week go first in *Subscriptions* (after live ones), they top *My Channels*
  with how many of their videos you watched, and new video notices name them
  first
- **Search** videos in all of YouTube (about a second, more results load as
  you scroll), only in your channels, or inside a channel (`Tab` switches the
  scope); search channels (with their description, whole in the side panel)
- A **preview** of the selected video with its **thumbnail** (in kitty), its
  length and **what it weighs** to download at your quality
  (*💾 Download ≈ 84 MB (1080p)*), asked to yt-dlp once the selection rests
  on it, and YouTube's **AI summary** of it (*✦ AI Summary*) when it has one
- With the video in its own window, its tab shows the **video's details**
  (thumbnail, subscribers, views, likes, date) and **Up next**: `Tab` moves
  into it, `Enter` opens one in a new tab. Closing the video keeps the tab
  with its details and Up next; `Enter` plays it again where you left it
- **Look before playing** (tubevibe default, `autoplay = false`): `Enter` on a
  video opens its tab with the details and Up next but plays nothing; `Enter`
  (or `Space`) in the tab starts it, resuming where you left it. Sound only
  (`a`) and your own files still play at once. `autoplay = true` (Settings →
  *Enter on a video*) brings back playing straight away
- A **progress bar** under the video (played, downloaded, rest) and its
  volume, resolution and codec on top
- **Mouse**: click to select, double click to open, wheel to scroll (on the
  video: volume), click the video to pause, the progress bar to jump.
  **Right click** opens a menu with what can be done there: play, sound only,
  watch later, download, like, channel, subscribe, close the tab, delete…
- **Subscribe / unsubscribe** (`s`, unsubscribing asks first), **like /
  dislike** (`g` / `G`, again takes it back), **YouTube Watch Later** (`w` adds), **tubevibe Watch Later** (`l` adds),
  **download** with subtitles (`d`), open the **channel** (`c`)
- **Comments**: next to *Up next* when the video plays in its own window or
  as audio only (in terminals 100 columns wide or more); `PgUp` / `PgDn`
  scroll them
- **AI summary**: YouTube's *AI-generated video summary*, on the videos
  that have one, right under the details of the same tabs (and in the
  preview of lists). Video descriptions are not shown

**TUBEVIBE** (lists the app makes itself)
- **Según lo que vi / From What I Watched**: what YouTube puts *Up next* to the last 10 videos you
  watched, all together and taking turns, without the ones you already saw
- **Descubrir / Discover**: this week's most watched videos on your topics
  (e.g. `rust, nba, jazz`), outside your channels and your history
- **Descubrir canales / Discover Channels**: YouTube's home feed without your
  channels (picks from channels you don't follow yet)
- **Watch Later** (under TUBEVIBE): the app's own watch-later
  list, kept only on this computer (`~/.local/share/tubevibe/later.json`),
  apart from YouTube's Watch Later. `l` adds the selected video to it (if it
  is already there, it just says so); inside it `Ctrl+Y` `x` removes the
  selected video (after asking). Messages call it *tubevibe Watch Later*
  and the other one *YouTube Watch Later*.

**Playback**
- Embedded video in kitty, an mpv window, or an **overlay** (X11): an mpv
  window without borders laid exactly over the app, so it looks embedded
  but plays with the GPU; one tab per video, background audio when you go
  back to browsing
- Volume, mute, seek, **subtitles on/off**, switch **original / subtitles /
  auto-dubbed audio** and the **language** while watching (the video reloads at
  the same second)
- **Fullscreen** (`f`): the video fills the terminal, and the terminal window
  goes fullscreen too (Hyprland and X11). In window mode (the default) `f`
  makes mpv's own window fullscreen. `f` also works from the lists while a
  video is playing; when nothing is playing it says so instead
- **Audio only** (`a`): play a video as sound only, like music; inside its
  tab `a` switches between audio and video at the same second
- **Chapters**: `[` / `]` jump to the previous / next one; they are marked
  `┃` on the progress bar and the current one shows on top
- **[SponsorBlock](https://sponsor.ajay.app)**: sponsors, self promotion and
  "subscribe!" reminders are skipped by themselves (they show green on the
  progress bar). Intros and outros are left alone

**On your computer**
- **My Videos** and **My Music** list the files in the folders you choose
  (several per kind); music plays from the chosen song to the end of the list
- **Downloads**: what you download with `d`, with live progress, speed and
  time left, and its size. `x` cancels one (its half-downloaded pieces are
  deleted), moves a finished one's file to the system trash and takes it off
  the list (after asking), or takes a failed one off the list; `Space`
  marks several and `x` then does that to all of them with one question.
  `Enter` plays the file. A finished download whose file was moved or
  deleted drops off the list by itself the next time Downloads is opened.
  Quitting with downloads running asks first

**Account and settings**
- Log in with any installed browser (Brave, Chrome, Chromium, Vivaldi, Edge,
  Opera, Firefox — profiles are detected), or **import a `cookies.txt`** on
  machines without a browser (servers, SSH)
- A settings screen for everything; changes go to `config.toml`, keeping your
  comments. `$EDITOR` can open the file from the app
- The bottom bar always shows the keys that work right now; `Ctrl+Y ?` lists
  them all
- English or Spanish, following the system language until you pick one
- Short animations when pages and dialogs open, each one can be turned off
- An **About** page with links and how to support the project

## Omarchy colour sync

On [Omarchy](https://omarchy.org), tubevibe uses the colours of the current
theme (Gruvbox, Catppuccin, Rose Pine, a light theme…) instead of its
built-in Tokyo Night, and recolours itself within about 2 seconds when you run
`omarchy theme set` or pick another theme. It reads the theme's `colors.toml`
from `~/.local/state/omarchy/current/theme/` (Omarchy 4) or
`~/.config/omarchy/current/theme/` (earlier versions), or the theme's
`alacritty.toml` when an older theme has no `colors.toml`. Elsewhere, or when
the file can't be read, it stays Tokyo Night. Settings → *Omarchy Theme
Colors* (`[ui] omarchy_theme`) turns it off.

## Requirements

- [`mpv`](https://mpv.io) and [`yt-dlp`](https://github.com/yt-dlp/yt-dlp) in `PATH` (on Windows they are
  also found where winget, scoop or the mpv installer leave them)
- For embedded video: the [kitty](https://sw.kovidgoyal.net/kitty/) terminal
- Optional: `wmctrl` (X11) for window fullscreen (Hyprland needs nothing:
  `hyprctl` comes with it)

tubevibe checks this itself: when something needed is missing (or yt-dlp is
too old for YouTube) it opens a **System Check** that lists what is there,
what is not and the command that installs it with your package manager
(apt, pacman, dnf, zypper, Homebrew or winget). It is also in Settings →
*System Check*.

## Install

Every release on
[GitHub Releases](https://github.com/mortbrew/tubevibe/releases/latest) has:

| Machine | File |
|---|---|
| Linux x86_64 | `tubevibe-x86_64-linux.tar.gz` |
| Linux ARM64 | `tubevibe-aarch64-linux.tar.gz` |
| Debian / Ubuntu / Mint (x86_64) | `tubevibe_amd64.deb` (`sudo apt install ./tubevibe_amd64.deb`) |
| macOS (Apple Silicon) | `tubevibe-aarch64-macos.tar.gz` |
| Windows x86_64 | `tubevibe-x86_64-windows-setup.exe` (installer, can install mpv and yt-dlp with winget) or the portable `tubevibe-x86_64-windows.zip` |

On Linux x86_64, one line puts it in `~/.local/bin` (which should be in your
`PATH`; use `aarch64` instead of `x86_64` on ARM):

```sh
mkdir -p ~/.local/bin && curl -fsSL https://github.com/mortbrew/tubevibe/releases/latest/download/tubevibe-x86_64-linux.tar.gz | tar xz -C ~/.local/bin --strip-components=1 tubevibe/tubevibe
```

Or the installer for Linux / macOS, which picks the right file, falls back to
building from source where there is none, and checks mpv and yt-dlp (and,
to build, the C compiler and Rust 1.88+), printing the commands for whatever
is missing:

```sh
curl -sSf https://raw.githubusercontent.com/mortbrew/tubevibe/main/install.sh | sh
```

From source, anywhere Rust 1.88+ runs:

```sh
cargo install --locked --git https://github.com/mortbrew/tubevibe
```

or from a clone:

```sh
git clone https://github.com/mortbrew/tubevibe
cd tubevibe
cargo build --release
./target/release/tubevibe
```

Each time tubevibe opens it asks GitHub, in the background, whether a newer
release is out; when there is one, the **About** page shows the command that
updates it for how it was installed (Settings → *Look for new versions on
startup* turns this off).

**About → Update Now updates tubevibe in place** (from 0.8.2). Open
**About** in the menu (the menu entry shows ⬆ when a new version is out):
the **Update Now** button is selected (`Tab` moves between it and the menu)
and `Enter` presses it. It downloads the new release file for your system
(Linux x86_64 / aarch64, macOS Apple Silicon, Windows x86_64), checks that
the new program runs, swaps it in for the running one and asks you to
restart tubevibe. The button only works for a copy installed from a release
file (for example the one-liner into `~/.local/bin`, or the Windows
installer); otherwise it is greyed out and About says why: a copy built from
source (`target/release/`) says to run `git pull && cargo build --release`;
one installed with `cargo install`, the `.deb` (`/usr/bin`) or Homebrew says
to update it the same way; and a copy in a folder you can't write to says so.

(The original tubetui has its own packages on its
[website](https://polblancoo.github.io/tubetui/#descargas); those install
tubetui, not this fork.)

> `feature edition2024 is required` means your Rust is too old — distro
> packages often are (Ubuntu 22.04 ships 1.75). Install a current one with
> [rustup](https://rustup.rs):
>
> ```sh
> curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
> . "$HOME/.cargo/env"
> ```

## Keys

Global commands use a **`Ctrl+Y` prefix** (tmux style): press `Ctrl+Y`, then:

| Key | Action |
|---|---|
| `b` | show / hide the menu |
| `m` | go to the menu |
| `f` | fullscreen |
| `0-9` | the tab with that number in the menu (`0` = back to browsing) |
| `x` | close the tab · in either Watch Later or a playlist: remove the selected video (asks first) |
| `/` | search videos |
| `l` | English / español |
| `?` | every shortcut, explained |
| `q` | quit |

Without the prefix, one hand on the left of the keyboard:

| Key | Action |
|---|---|
| `Alt+,` | previous tab in the menu (from the first, round to the last; from browsing, the last) |
| `Alt+.` | next tab in the menu (from the last, back to the first; from browsing, the first) |

These work in any terminal as it comes: no setting to change (on macOS,
`Option+,` and `Option+.` work too, whether or not Option is set to act as
Alt). `Ctrl+Y`
`1`–`9` still jumps straight to a tab.

With the mouse: click selects (menu sections and tabs open), double click
plays / opens, the wheel scrolls lists and sets the volume over the video,
a click on the video pauses (double click: fullscreen), a click on the
progress bar jumps there. A right click opens the options of what is under
it (each with its key). Hold Shift to select text.

In the menu, `↑↓` walk the sections and the open tabs (under TABS) alike; `Enter` (or `→`, except on Quit, which only `Enter` triggers) opens either. `←` from a list or a video tab comes back to the menu with the cursor on the section or tab it came from.

In lists: `↑↓` / `jk`, `Enter`, `Esc` / `Backspace` to go back, `f` fullscreen for the video that is playing, `Tab` menu ↔ list,
`w` save to YouTube Watch Later, `l` save to tubevibe Watch Later, `s` subscribe, `g` / `G` like / dislike, `d` download, `c` channel, `a` play as
audio only. In **History** `x` removes the video; in YouTube's **Watch Later** and
**playlists** `Ctrl+Y` `x` removes it on YouTube (after asking), in tubevibe's **Watch Later** it removes it from that local list; in **Downloads** `x` cancels
the download, or moves a finished one's file to the system trash after asking
(only that file, never a folder or link; a file that is already gone just
leaves the list); `Space` marks several downloads for one `x`. In **My Videos**
and **My Music**: `Space` marks files, `x` / `Del` moves the marked ones (or
the selected one) to the system trash, after asking.
In a video tab that is not playing yet (or closed), `Enter` or `Space` plays it.
While watching: `Space` pause, `Shift+←` / `Shift+→` seek 5 s (`→` alone seeks forward too), `←` goes to the menu, `↑↓` volume, `m` mute, `j`
subtitles, `i` original / subtitles / dubbed, `o` language, `v` quality
(360p–1440p), `b` codec (H.264 / VP9 / AV1 / any), `f` fullscreen, `.` `,`
next / previous song, `[` `]` previous / next chapter, `a` audio only ⇄ video,
`PgUp` `PgDn` scroll the comments. Quality, codec and language reload the video at the
same second and are remembered. If embedded video looks behind the sound,
the CPU is dropping frames: `v` down to 480p, or try the overlay.

## Overlay mode (X11)

Embedded video goes through the CPU: every frame is decoded, scaled and sent
to kitty as an image (on a 2012 laptop: 134 % CPU, a third of the frames
dropped). `mode = "overlay"` (Settings → Video) instead opens a borderless
mpv window over the video pane: hardware decoding, every frame, about 10 %
CPU, and it still looks part of the app. It follows the terminal window when
it moves, hides when you switch tabs, open the help or focus another
program, and lets clicks and keys through to the app. It needs X11 and
`xdotool`, `xwininfo` and `xprop` (`wmctrl` keeps it off the taskbar);
without them, or on Wayland, it falls back to a separate window.

## Terminal multiplexers (tmux, zellij, herdr)

Inside a multiplexer, keys and images travel through it, which adds lag and
can break embedded video. When tubevibe detects one (`TMUX`, `ZELLIJ`,
`HERDR_ENV`), **automatic mode opens videos in an mpv window** and fullscreen
does not resize the outer terminal window. You can still force embedded video
(`mode = "embedded"`), but for the smoothest experience run tubevibe directly in
kitty.

## Configuration

`~/.config/tubevibe/config.toml` (or the platform equivalent; settings,
history and lists live in `~/.config/tubevibe`, `~/.local/share/tubevibe` and
`~/.cache/tubevibe`). The first time tubevibe runs, if those folders don't
exist yet and tubetui's do (`~/.config/tubetui` and so on), it copies them
over, so your settings, login, history and lists come along; the tubetui
folders are left untouched. Every key is
optional; everything can also be changed from the app (⚙ Settings).

```toml
[session]
browser = "brave+GNOMEKEYRING:Default"  # yt-dlp --cookies-from-browser syntax
cookies_file = ""                       # imported cookies.txt, wins over browser

[player]
mode = "window"          # window (default) | auto | embedded | overlay
max_height = 720
prefer_codec = "avc1"    # H.264 decodes in hardware almost everywhere
language = "subtitles"   # subtitles | original | dubbed
sub_lang = "en"          # subtitles and dubbing language
background_audio = true  # keep listening while browsing
sponsorblock = true      # skip sponsors, self promotion and "subscribe!" reminders
autoplay = false         # Enter opens the video's tab; Enter again plays (true: play at once)

[media]                  # empty: the system's own Videos and Music folders
video_dirs = ["~/Videos"]
music_dirs = ["~/Music", "/mnt/music"]

[download]
dir = "~/Downloads"      # empty: the system's Videos folder

[ui]
icons = "emoji"          # emoji | nerd | unicode
border_style = "plain"   # plain | rounded | thick | double | none
restore_tabs = true      # reopen the tabs that were open last time (kept in ~/.local/share/tubevibe/tabs.json)
omarchy_theme = true     # use the current Omarchy theme's colours (Tokyo Night when off or not on Omarchy)
lang = "en"              # en | es (unset: the system language)
thumbnails = true        # thumbnail in the preview panel (kitty)
mouse = true             # clicks and wheel; hold Shift to select text
splash = true            # the tubevibe logo for a moment when it opens (any key skips it)
check_updates = true     # on startup, tell me about new versions and how to install them here
hide_shorts = true       # leave Shorts out of every list
hide_later = true        # leave my YouTube Watch Later videos out of My Recommendations
notify_new = true        # check my subscriptions every 15 minutes and tell me about new videos
favorites_first = true   # my most watched channels first in Subscriptions and My Channels

[effects]                # animations
startup = true
pages = true
dialogs = true
```

## Performance

Idle it uses about 25 MB and no CPU. Videos in their own window play with
hardware decoding (8 % CPU on a 2012 laptop); embedded video is scaled on the
CPU, so 480p is the sweet spot there. See the [benchmark](docs/benchmark.md)
against ytfzf and youtube-tui (measured on tubetui, which tubevibe is based on).

## Troubleshooting

If the app closes unexpectedly (for example when launched from a window
manager shortcut, whose terminal closes with it), the error is kept in
`~/.cache/tubevibe/crash.log`.

To diagnose lag, run with `TUBEVIBE_DEBUG=/tmp/tubevibe.log tubevibe`
(the old `TUBETUI_DEBUG` works too): every key,
redraw and action is logged with its timing.

## Support

The **About** page lists donation addresses; they live in
[`donate.toml`](donate.toml).

> **These addresses belong to polblancoo, the original tubetui developer:
> donations go to him, not to the tubevibe fork maintainer (mortbrew).**

## Disclaimer

tubevibe talks to YouTube's unofficial internal API and uses your browser
session. That may go against YouTube's Terms of Service; use it at your own
risk. Cookies give access to your account: never share them.

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or
[MIT license](LICENSE-MIT), at your option.
