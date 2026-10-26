//! Video thumbnails, for the preview panel and video tabs. One background
//! thread downloads YouTube's small JPEGs (320×180, about 15 KB) and keeps
//! them on disk (`~/.cache/tubevibe/thumbs`, capped); ratatui-image draws the
//! one shown with the terminal's own graphics (kitty, sixel or iTerm2, asked
//! once at startup) or with Unicode half blocks anywhere else (Alacritty,
//! tmux, Zellij), encoding it only when the video, size or a setting changes.

use std::cell::{Cell, RefCell};
use std::collections::{HashMap, HashSet, VecDeque};
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::time::{Duration, Instant};

use image::DynamicImage;
use image::imageops::FilterType;
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui_image::picker::{Picker, ProtocolType};
use ratatui_image::protocol::{StatefulProtocol, StatefulProtocolType};
use ratatui_image::{Resize, StatefulImage};

use crate::config::{Pixelation, ThumbMode};

/// Decoded thumbnails kept in memory (about 170 KB each).
const CACHE: usize = 64;
/// The disk cache is trimmed (oldest first) to this many bytes.
pub const DISK_CAP: u64 = 30 * 1024 * 1024;
/// A new thumbnail waits this long after the selection last moved, so
/// scrolling a list does not send a picture per row.
pub const SETTLE: Duration = Duration::from_millis(120);
/// Smaller than this many cells, no thumbnail (it would be a smudge).
const MIN_CELLS: (u16, u16) = (8, 3);

/// Inside tmux or Zellij, graphics are unreliable: half blocks.
pub fn in_multiplexer() -> bool {
    ["TMUX", "ZELLIJ"].iter().any(|v| std::env::var_os(v).is_some_and(|s| !s.is_empty()))
}

/// Asks the terminal what it can draw (call once, after entering the
/// alternate screen, before reading keys). `None`: half blocks only.
pub fn detect() -> Option<Picker> {
    if in_multiplexer() {
        return None;
    }
    Picker::from_query_stdio().ok().filter(|p| p.protocol_type() != ProtocolType::Halfblocks)
}

struct Shown {
    id: String,
    mode: ThumbMode,
    pixelation: Pixelation,
    big: bool,
    protocol: StatefulProtocol,
}

pub struct Thumbnails {
    /// The terminal's graphics, when it has any (Picture).
    graphics: Option<Picker>,
    /// Each with whether it is the big one (video tabs).
    cache: HashMap<String, (DynamicImage, bool)>,
    order: VecDeque<String>,
    requested: HashSet<String>,
    jobs: Option<mpsc::Sender<(String, bool)>>,
    done: Option<mpsc::Receiver<(String, bool, Option<DynamicImage>)>>,
    shown: RefCell<Option<Shown>>,
    /// The thumbnail asked for, and since when, while it settles.
    pending: RefCell<Option<(String, Instant)>>,
    /// A picture (not blocks) was on screen and went: clear the screen once.
    pub needs_clear: Cell<bool>,
}

impl Default for Thumbnails {
    fn default() -> Self {
        Self::with_disk(cache_dir())
    }
}

impl Thumbnails {
    fn empty() -> Self {
        Self {
            graphics: None,
            cache: HashMap::new(),
            order: VecDeque::new(),
            requested: HashSet::new(),
            jobs: None,
            done: None,
            shown: RefCell::new(None),
            pending: RefCell::new(None),
            needs_clear: Cell::new(false),
        }
    }

    fn with_disk(dir: Option<PathBuf>) -> Self {
        let (jobs, queue) = mpsc::channel::<(String, bool)>();
        let (finished, done) = mpsc::channel();
        std::thread::spawn(move || {
            if let Some(dir) = &dir {
                trim_cache(dir, DISK_CAP);
            }
            for (id, big) in queue {
                if finished.send((id.clone(), big, fetch(&id, big, dir.as_deref()))).is_err() {
                    return;
                }
            }
        });
        Self { jobs: Some(jobs), done: Some(done), ..Self::empty() }
    }

    pub fn set_graphics(&mut self, picker: Option<Picker>) {
        self.graphics = picker;
        self.forget();
    }

    /// Downloads these thumbnails, in order, unless they are here or on the way.
    pub fn want<'a>(&mut self, ids: impl IntoIterator<Item = &'a str>) {
        for id in ids {
            if !id.is_empty() && !self.cache.contains_key(id) && self.requested.insert(id.to_owned()) {
                if let Some(jobs) = &self.jobs {
                    let _ = jobs.send((id.to_owned(), false));
                }
            }
        }
    }

    /// Downloads the big thumbnail of `id` (for a big area), unless here or on the way.
    pub fn want_big(&mut self, id: &str) {
        let key = format!("{id}#hd");
        if !id.is_empty() && !self.cache.get(id).is_some_and(|c| c.1) && self.requested.insert(key) {
            if let Some(jobs) = &self.jobs {
                let _ = jobs.send((id.to_owned(), true));
            }
        }
    }

    /// Cell height over width: the font's when the terminal said, else 2.
    pub fn cell_aspect(&self) -> f32 {
        self.graphics
            .as_ref()
            .map(Picker::font_size)
            .filter(|f| f.width > 0 && f.height > 0)
            .map_or(2.0, |f| f32::from(f.height) / f32::from(f.width))
    }

    /// A thumbnail is waiting for the selection to settle: draw again soon.
    pub fn settling(&self) -> bool {
        self.pending.borrow().is_some()
    }

    /// Takes the finished downloads; whether any arrived.
    pub fn receive(&mut self) -> bool {
        let mut arrived = Vec::new();
        if let Some(done) = &self.done {
            while let Ok(one) = done.try_recv() {
                arrived.push(one);
            }
        }
        let any = arrived.iter().any(|(_, _, t)| t.is_some());
        for (id, big, thumb) in arrived {
            self.arrived_as(id, big, thumb);
        }
        any
    }

    #[cfg(test)]
    fn arrived(&mut self, id: String, thumb: Option<DynamicImage>) {
        self.arrived_as(id, false, thumb);
    }

    fn arrived_as(&mut self, id: String, big: bool, thumb: Option<DynamicImage>) {
        self.requested.remove(&if big { format!("{id}#hd") } else { id.clone() });
        let Some(thumb) = thumb else { return };
        if self.cache.get(&id).is_some_and(|c| c.1 && !big) {
            return;
        }
        if !self.cache.contains_key(&id) {
            self.order.push_back(id.clone());
        }
        self.cache.insert(id, (thumb, big));
        while self.order.len() > CACHE {
            if let Some(old) = self.order.pop_front() {
                self.cache.remove(&old);
            }
        }
    }

    /// Draws the thumbnail of `target` (video id, cells) from the top-left
    /// corner, scaled up or down to fill them keeping its shape; nothing when
    /// it isn't here yet, is off, or the cells are too few. A different
    /// video waits [`SETTLE`] after the selection last moved.
    pub fn draw(&self, frame: &mut Frame, target: Option<(String, Rect)>, mode: ThumbMode, pixelation: Pixelation) {
        self.draw_at(frame, target, mode, pixelation, Instant::now());
    }

    fn draw_at(&self, frame: &mut Frame, target: Option<(String, Rect)>, mode: ThumbMode, pixelation: Pixelation, now: Instant) {
        let target = target.filter(|(id, area)| {
            mode != ThumbMode::Off
                && self.cache.contains_key(id)
                && area.width >= MIN_CELLS.0
                && area.height >= MIN_CELLS.1
        });
        let Some((id, area)) = target else {
            self.pending.borrow_mut().take();
            self.hide();
            return;
        };
        let big = self.cache[&id].1;
        let mut shown = self.shown.borrow_mut();
        let same_video = shown.as_ref().is_some_and(|s| s.id == id);
        let current = shown.as_ref().is_some_and(|s| s.id == id && s.mode == mode && s.pixelation == pixelation && s.big == big);
        if !current {
            if !same_video {
                let mut pending = self.pending.borrow_mut();
                let since = match pending.as_ref() {
                    Some((p, t)) if *p == id => *t,
                    _ => {
                        *pending = Some((id.clone(), now));
                        now
                    }
                };
                if now.duration_since(since) < SETTLE {
                    // Still scrolling: the old picture goes, the new one waits.
                    drop(pending);
                    drop(shown);
                    self.hide();
                    return;
                }
                pending.take();
            }
            if shown.take().is_some_and(|s| !is_blocks(&s.protocol)) {
                self.needs_clear.set(true);
            }
            let picker = match (mode, &self.graphics) {
                (ThumbMode::Picture, Some(p)) => p.clone(),
                _ => Picker::halfblocks(),
            };
            let protocol = picker.new_resize_protocol(pixelate(&self.cache[&id].0, pixelation));
            *shown = Some(Shown { id, mode, pixelation, big, protocol });
        }
        let Some(s) = shown.as_mut() else { return };
        let fit = s.protocol.size_for(Resize::Scale(None), area.as_size());
        // Centred in the area, whichever protocol draws it (a picture a cell
        // or two short of the area does not hug its top-left corner).
        let (w, h) = (fit.width.min(area.width), fit.height.min(area.height));
        let at = Rect::new(area.x + (area.width - w) / 2, area.y + (area.height - h) / 2, w, h);
        frame.render_stateful_widget(StatefulImage::default().resize(Resize::Scale(None)), at, &mut s.protocol);
    }

    /// Nothing to show: a picture left on screen goes.
    fn hide(&self) {
        if self.shown.borrow_mut().take().is_some_and(|s| !is_blocks(&s.protocol)) {
            self.needs_clear.set(true);
        }
    }

    /// Something removed every image from the terminal (or a setting
    /// changed): encode and send it again next time.
    pub fn forget(&mut self) {
        self.shown.get_mut().take();
    }

    /// Whether `id` is in memory.
    pub fn has(&self, id: &str) -> bool {
        self.cache.contains_key(id)
    }

    #[cfg(test)]
    fn shown_id(&self) -> Option<String> {
        self.shown.borrow().as_ref().map(|s| s.id.clone())
    }
}

/// Drawn in the cells themselves (half blocks, kitty's placeholders): the
/// next frame replaces it, no clearing needed.
fn is_blocks(p: &StatefulProtocol) -> bool {
    matches!(p.protocol_type(), StatefulProtocolType::Halfblocks(_) | StatefulProtocolType::Kitty(_))
}

/// The thumbnail shrunk to `pixelation`'s grid and blown back up, nearest
/// neighbour both ways, so every protocol shows the same big pixels.
pub fn pixelate(image: &DynamicImage, pixelation: Pixelation) -> DynamicImage {
    let Some(grid) = pixelation.grid() else { return image.clone() };
    let (w, h) = (image.width().max(1), image.height().max(1));
    let (gw, gh) = if w >= h { (grid, (grid * h / w).max(1)) } else { ((grid * w / h).max(1), grid) };
    if gw >= w && gh >= h {
        return image.clone();
    }
    image.resize_exact(gw, gh, FilterType::Nearest).resize_exact(w, h, FilterType::Nearest)
}

/// From the disk cache, else from YouTube (and then kept on disk). `big`:
/// the largest YouTube has (maxres, then sd, hq), else the small one.
fn fetch(id: &str, big: bool, dir: Option<&Path>) -> Option<DynamicImage> {
    let name = if big { format!("{}-hd.jpg", file_name(id)) } else { format!("{}.jpg", file_name(id)) };
    let path = dir.map(|d| d.join(name));
    if let Some(bytes) = path.as_ref().and_then(|p| std::fs::read(p).ok())
        && let Some(image) = decode(&bytes)
    {
        return Some(crop_letterbox(image));
    }
    let sizes: &[&str] = if big { &["maxresdefault", "sddefault", "hqdefault", "mqdefault"] } else { &["mqdefault"] };
    let (bytes, image) = sizes.iter().find_map(|size| {
        let bytes = download(id, size).ok()?;
        let image = decode(&bytes)?;
        Some((bytes, image))
    })?;
    if let (Some(dir), Some(path)) = (dir, path) {
        let _ = std::fs::create_dir_all(dir).and_then(|()| std::fs::write(path, &bytes));
    }
    Some(crop_letterbox(image))
}

/// sd and hq thumbnails are 4:3 with black bars around a 16:9 picture: cut
/// to the middle 16:9.
pub fn crop_letterbox(image: DynamicImage) -> DynamicImage {
    let (w, h) = (image.width(), image.height());
    let wanted = w * 9 / 16;
    if h > wanted + 2 { image.crop_imm(0, (h - wanted) / 2, w, wanted) } else { image }
}

fn download(id: &str, size: &str) -> Result<Vec<u8>, String> {
    let url = format!("https://i.ytimg.com/vi/{id}/{size}.jpg");
    let agent: ureq::Agent = ureq::Agent::config_builder().timeout_global(Some(Duration::from_secs(10))).build().into();
    let mut jpeg = Vec::new();
    // A missing size is a 404: an error, and the next size is tried.
    let reply = agent.get(&url).call().map_err(|e| e.to_string())?;
    reply.into_body().into_reader().take(4 << 20).read_to_end(&mut jpeg).map_err(|e| e.to_string())?;
    Ok(jpeg)
}

fn decode(jpeg: &[u8]) -> Option<DynamicImage> {
    image::load_from_memory_with_format(jpeg, image::ImageFormat::Jpeg).ok()
}

/// Video ids are safe file names already; anything else is dropped.
fn file_name(id: &str) -> String {
    id.chars().filter(|c| c.is_ascii_alphanumeric() || *c == '-' || *c == '_').collect()
}

/// `~/.cache/tubevibe/thumbs` (XDG).
fn cache_dir() -> Option<PathBuf> {
    directories::ProjectDirs::from("", "", "tubevibe").map(|d| d.cache_dir().join("thumbs"))
}

/// Deletes the oldest files in `dir` until it holds at most `cap` bytes.
pub fn trim_cache(dir: &Path, cap: u64) {
    let Ok(entries) = std::fs::read_dir(dir) else { return };
    let mut files: Vec<_> = entries
        .filter_map(Result::ok)
        .filter_map(|e| {
            let m = e.metadata().ok()?;
            m.is_file().then(|| (m.modified().ok(), m.len(), e.path()))
        })
        .collect();
    let mut total: u64 = files.iter().map(|f| f.1).sum();
    files.sort_by_key(|f| f.0);
    for (_, len, path) in files {
        if total <= cap {
            break;
        }
        if std::fs::remove_file(&path).is_ok() {
            total -= len;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{GenericImageView, Rgb, RgbImage};
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;

    fn stripes() -> DynamicImage {
        DynamicImage::ImageRgb8(RgbImage::from_fn(320, 180, |x, _| {
            if (x / 20) % 2 == 0 { Rgb([250, 40, 40]) } else { Rgb([40, 40, 250]) }
        }))
    }

    #[test]
    fn pixelation_makes_big_pixels_at_the_same_size() {
        let t = stripes();
        assert_eq!(pixelate(&t, Pixelation::Off).dimensions(), (320, 180));
        let blocky = pixelate(&t, Pixelation::Max);
        assert_eq!(blocky.dimensions(), (320, 180));
        // 8 across: each 40-pixel column is one colour.
        let c = blocky.get_pixel(41, 10);
        assert!((40..80).all(|x| blocky.get_pixel(x, 100) == c));
    }

    fn screen(thumbs: &Thumbnails, target: Option<(String, Rect)>, mode: ThumbMode) -> String {
        let mut terminal = Terminal::new(TestBackend::new(40, 12)).unwrap();
        // Once to ask, once after it settled.
        let t = Instant::now();
        terminal.draw(|f| thumbs.draw_at(f, target.clone(), mode, Pixelation::Off, t)).unwrap();
        terminal.draw(|f| thumbs.draw_at(f, target.clone(), mode, Pixelation::Off, t + SETTLE)).unwrap();
        // A painted cell shows as '▀' (half blocks may draw a flat cell as a
        // coloured space, so count colours, not glyphs).
        let buf = terminal.backend().buffer();
        buf.content.iter().map(|c| {
            use ratatui::style::Color;
            if c.fg != Color::Reset || c.bg != Color::Reset { "▀" } else { " " }
        }).collect()
    }

    #[test]
    fn half_blocks_draw_without_terminal_graphics_and_only_when_it_fits() {
        let mut thumbs = Thumbnails::empty();
        let area = Rect::new(2, 1, 32, 9);
        assert!(!screen(&thumbs, Some(("a".into(), area)), ThumbMode::Picture).contains('▀'), "not here yet");
        thumbs.arrived("a".into(), Some(stripes()));
        assert!(thumbs.has("a"));
        let drawn = screen(&thumbs, Some(("a".into(), area)), ThumbMode::Picture);
        assert!(drawn.contains('▀'), "half blocks, as in Alacritty");
        assert!(!screen(&thumbs, Some(("a".into(), area)), ThumbMode::Off).contains('▀'), "off");
        let tiny = Rect::new(0, 0, 6, 2);
        assert!(!screen(&thumbs, Some(("a".into(), tiny)), ThumbMode::Blocks).contains('▀'), "too small: skipped");
    }

    #[test]
    fn failures_are_not_kept_and_the_cache_is_capped() {
        let mut thumbs = Thumbnails::empty();
        thumbs.requested.insert("bad".into());
        thumbs.arrived("bad".into(), None);
        assert!(!thumbs.has("bad") && thumbs.requested.is_empty());
        for i in 0..CACHE + 5 {
            thumbs.arrived(format!("v{i}"), Some(DynamicImage::new_rgb8(2, 2)));
        }
        assert_eq!(thumbs.cache.len(), CACHE);
        assert!(!thumbs.has("v0") && thumbs.has(&format!("v{}", CACHE + 4)));
    }

    #[test]
    fn the_disk_cache_is_trimmed_oldest_first() {
        let dir = std::env::temp_dir().join(format!("tubevibe-thumbs-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        for (i, name) in ["a", "b", "c"].iter().enumerate() {
            let p = dir.join(name);
            std::fs::write(&p, vec![0u8; 100]).unwrap();
            let t = std::time::SystemTime::UNIX_EPOCH + Duration::from_secs(1000 * (i as u64 + 1));
            std::fs::File::options().write(true).open(&p).unwrap().set_modified(t).unwrap();
        }
        trim_cache(&dir, 250);
        assert!(!dir.join("a").exists() && dir.join("b").exists() && dir.join("c").exists());
        std::fs::remove_dir_all(&dir).unwrap();
        assert_eq!(file_name("ab-_C1/../x"), "ab-_C1x");
    }

    fn painted(terminal: &Terminal<TestBackend>) -> Vec<(u16, u16)> {
        let buf = terminal.backend().buffer();
        let mut cells = Vec::new();
        for y in 0..buf.area.height {
            for x in 0..buf.area.width {
                let c = &buf[(x, y)];
                if c.fg != ratatui::style::Color::Reset || c.bg != ratatui::style::Color::Reset {
                    cells.push((x, y));
                }
            }
        }
        cells
    }

    #[test]
    fn the_picture_scales_up_to_fill_its_cells() {
        let mut thumbs = Thumbnails::empty();
        thumbs.arrived("a".into(), Some(stripes()));
        // 320x180 pixels in 64x18 cells (half blocks: 1x2 px a cell would be
        // 64x36 px): bigger than the picture, it grows to fill them.
        let mut terminal = Terminal::new(TestBackend::new(80, 24)).unwrap();
        let area = Rect::new(3, 2, 64, 18);
        let t = Instant::now();
        for now in [t, t + SETTLE] {
            terminal.draw(|f| thumbs.draw_at(f, Some(("a".into(), area)), ThumbMode::Blocks, Pixelation::Off, now)).unwrap();
        }
        let cells = painted(&terminal);
        let (x0, y0) = (cells.iter().map(|c| c.0).min().unwrap(), cells.iter().map(|c| c.1).min().unwrap());
        let (x1, y1) = (cells.iter().map(|c| c.0).max().unwrap(), cells.iter().map(|c| c.1).max().unwrap());
        assert_eq!((x0, y0), (3, 2), "the area is exactly its size: no slack to centre");
        assert!(x1 >= 3 + 60 || y1 >= 2 + 16, "fills the area: {x1},{y1}");
        assert!(x1 < 3 + 64 && y1 < 2 + 18, "inside it");
    }

    #[test]
    fn a_picture_smaller_than_its_area_is_centred_in_it() {
        let mut thumbs = Thumbnails::empty();
        thumbs.arrived("a".into(), Some(stripes()));
        // 60x30 cells for a 16:9 picture: full width, about 17 rows; the spare
        // rows split above and below. The same in a roomy area to the right.
        let mut terminal = Terminal::new(TestBackend::new(100, 40)).unwrap();
        let t = Instant::now();
        for area in [Rect::new(2, 1, 60, 30), Rect::new(2, 1, 90, 20)] {
            for now in [t, t + SETTLE] {
                terminal.draw(|f| thumbs.draw_at(f, Some(("a".into(), area)), ThumbMode::Blocks, Pixelation::Off, now)).unwrap();
            }
            let cells = painted(&terminal);
            let (x0, y0) = (cells.iter().map(|c| c.0).min().unwrap(), cells.iter().map(|c| c.1).min().unwrap());
            let (x1, y1) = (cells.iter().map(|c| c.0).max().unwrap(), cells.iter().map(|c| c.1).max().unwrap());
            let (left, right) = (x0 - area.x, area.right() - 1 - x1);
            let (above, below) = (y0 - area.y, area.bottom() - 1 - y1);
            assert!(left.abs_diff(right) <= 1 && above.abs_diff(below) <= 1, "{area:?}: {left}/{right} {above}/{below}");
            assert!(above > 0 || left > 0, "{area:?}: there was room to centre in");
        }
    }

    #[test]
    fn a_new_thumbnail_waits_for_the_selection_to_settle() {
        let mut thumbs = Thumbnails::empty();
        thumbs.arrived("a".into(), Some(stripes()));
        thumbs.arrived("b".into(), Some(stripes()));
        let mut terminal = Terminal::new(TestBackend::new(40, 12)).unwrap();
        let area = Rect::new(0, 0, 32, 9);
        let t = Instant::now();
        let mut draw = |id: &str, now: Instant| {
            terminal.draw(|f| thumbs.draw_at(f, Some((id.into(), area)), ThumbMode::Blocks, Pixelation::Off, now)).unwrap();
        };
        draw("a", t);
        draw("a", t + SETTLE);
        assert_eq!(thumbs.shown_id().as_deref(), Some("a"));
        // Scrolling past b to a and back, quickly: nothing is encoded.
        draw("b", t + SETTLE * 2);
        assert!(thumbs.settling() && thumbs.shown_id().is_none());
        draw("b", t + SETTLE * 2 + SETTLE / 2);
        assert!(thumbs.shown_id().is_none(), "still settling");
        draw("b", t + SETTLE * 3);
        assert_eq!(thumbs.shown_id().as_deref(), Some("b"));
        assert!(!thumbs.settling());
        assert!(!thumbs.needs_clear.get(), "blocks never ask for a clear");
    }

    #[test]
    fn big_thumbnails_replace_small_ones_and_letterboxes_are_cut() {
        let mut thumbs = Thumbnails::empty();
        thumbs.arrived_as("a".into(), true, Some(DynamicImage::new_rgb8(1280, 720)));
        thumbs.arrived_as("a".into(), false, Some(DynamicImage::new_rgb8(320, 180)));
        assert_eq!(thumbs.cache["a"].0.width(), 1280, "the small one never replaces the big one");
        thumbs.requested.insert("a#hd".into());
        thumbs.want_big("a");
        let cut = crop_letterbox(DynamicImage::new_rgb8(640, 480));
        assert_eq!((cut.width(), cut.height()), (640, 360));
        let kept = crop_letterbox(DynamicImage::new_rgb8(1280, 720));
        assert_eq!((kept.width(), kept.height()), (1280, 720));
    }
}
