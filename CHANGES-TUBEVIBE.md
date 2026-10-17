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
