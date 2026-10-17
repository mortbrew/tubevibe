# tubevibe changes (fork of tubetui)

Changes made in this fork, [mortbrew/tubevibe](https://github.com/mortbrew/tubevibe),
on top of upstream [polblancoo/tubetui](https://github.com/polblancoo/tubetui) v0.7.7.

## Watch later is YouTube's own Watch later (`watch-later-online`)

Before, the **Ver después / Watch later** entry of the left menu was a list kept
only on this computer (`~/.local/share/tubetui/later.json`), unrelated to the
*Watch later* playlist of the YouTube account.

Now:

- **Menu → Watch later** opens the account's YouTube *Watch later* playlist
  (playlist id `WL`, fetched with yt-dlp and the login cookies, like every
  other playlist). Without a login it says
  *"Sign in to see your YouTube Watch later"* instead of showing a list.
- **`l`** (and the right-click item) on a video, in any list or video tab:
  - not in Watch later → it is added on YouTube right away
    (InnerTube `browse/edit_playlist`, `ACTION_ADD_VIDEO`);
  - already there → the usual yes/no question, and on *yes* it is removed
    (`ACTION_REMOVE_VIDEO_BY_VIDEO_ID`, `removedVideoId`).
  - Whether it is there comes from the Watch later list once it has been
    opened (and fit in one page). Otherwise YouTube is asked first through
    `playlist/get_add_to_playlist` (`containsSelectedVideos` of `WL`).
  - The open list is updated as soon as YouTube confirms; errors show in the
    status bar (❌ …). Without a login it says to sign in.
- **`x`** inside Watch later removes the selected video (after asking).
  Opening *Watch later (YouTube)* from **My playlists** behaves the same.
- The 🕒 marks in lists and the count next to the menu entry reflect the
  YouTube list (the count only when the whole list was loaded).
- Changing the login forgets the Watch later known so far.

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
  adding it (opening Watch later first avoids that question);
- YouTube may take a moment before a reloaded Watch later reflects an edit.

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

## Video descriptions (`descriptions`)

- **Video tab** (when the video plays in its own window, or as sound only,
  so the tab shows its details): the description goes between the details
  and *Up next*, up to 8 wrapped lines with its own line breaks kept, the
  last one saying *"… (more: e)"* when there is more. **`e`** shows it whole
  over *Up next* (comments stay beside it); **`PgUp` / `PgDn`** scroll it
  while it is open (otherwise they scroll the comments as before);
  **`e`**, **`Esc`** or **`Tab`** close it. Links are plain text.
  It comes from the watch page the tab already asks for
  (`videoSecondaryInfoRenderer.attributedDescription.content`, else the
  side panel's `expandableVideoDescriptionBodyRenderer` copy): no extra
  request.
- **Preview pane** of video lists: up to 5 lines (the last one `…` when cut)
  after the other details. It comes from the yt-dlp run that already asks
  the download size when the selection rests on a video
  (`--print "%(description)j"` added): no extra run. A video whose size was
  known without a description is asked once more; live and upcoming videos
  are not asked (as before), so they show none in the preview.
- Descriptions are kept in memory for the session (not saved), from either
  source.

### Uncertain

- The watch-page field was checked against one live anonymous reply
  (it carries the whole text with real URLs); the panel fallback and the
  yt-dlp `%(description)j` output (`null` / `"NA"` when missing) are
  handled but only tested with built replies.
- Very long descriptions scroll by source lines in the app's state; the
  view never scrolls past the last screenful.
