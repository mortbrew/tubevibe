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
use std::time::Duration;

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
    protocol: StatefulProtocol,
}

pub struct Thumbnails {
    /// The terminal's graphics, when it has any (Picture).
    graphics: Option<Picker>,
    cache: HashMap<String, DynamicImage>,
    order: VecDeque<String>,
    requested: HashSet<String>,
    jobs: Option<mpsc::Sender<String>>,
    done: Option<mpsc::Receiver<(String, Option<DynamicImage>)>>,
    shown: RefCell<Option<Shown>>,
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
            needs_clear: Cell::new(false),
        }
    }

    fn with_disk(dir: Option<PathBuf>) -> Self {
        let (jobs, queue) = mpsc::channel::<String>();
        let (finished, done) = mpsc::channel();
        std::thread::spawn(move || {
            if let Some(dir) = &dir {
                trim_cache(dir, DISK_CAP);
            }
            for id in queue {
                if finished.send((id.clone(), fetch(&id, dir.as_deref()))).is_err() {
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
                    let _ = jobs.send(id.to_owned());
                }
            }
        }
    }

    /// Takes the finished downloads; whether any arrived.
    pub fn receive(&mut self) -> bool {
        let mut arrived = Vec::new();
        if let Some(done) = &self.done {
            while let Ok(one) = done.try_recv() {
                arrived.push(one);
            }
        }
        let any = arrived.iter().any(|(_, t)| t.is_some());
        for (id, thumb) in arrived {
            self.arrived(id, thumb);
        }
        any
    }

    fn arrived(&mut self, id: String, thumb: Option<DynamicImage>) {
        self.requested.remove(&id);
        let Some(thumb) = thumb else { return };
        self.order.push_back(id.clone());
        self.cache.insert(id, thumb);
        while self.order.len() > CACHE {
            if let Some(old) = self.order.pop_front() {
                self.cache.remove(&old);
            }
        }
    }

    /// Draws the thumbnail of `target` (video id, cells), centred and keeping
    /// its shape; nothing when it isn't here yet, is off, or the cells are too few.
    pub fn draw(&self, frame: &mut Frame, target: Option<(String, Rect)>, mode: ThumbMode, pixelation: Pixelation) {
        let target = target.filter(|(id, area)| {
            mode != ThumbMode::Off
                && self.cache.contains_key(id)
                && area.width >= MIN_CELLS.0
                && area.height >= MIN_CELLS.1
        });
        let Some((id, area)) = target else {
            self.hide();
            return;
        };
        let mut shown = self.shown.borrow_mut();
        let current = shown.as_ref().is_some_and(|s| s.id == id && s.mode == mode && s.pixelation == pixelation);
        if !current {
            if shown.take().is_some_and(|s| !is_blocks(&s.protocol)) {
                self.needs_clear.set(true);
            }
            let picker = match (mode, &self.graphics) {
                (ThumbMode::Picture, Some(p)) => p.clone(),
                _ => Picker::halfblocks(),
            };
            let protocol = picker.new_resize_protocol(pixelate(&self.cache[&id], pixelation));
            *shown = Some(Shown { id, mode, pixelation, protocol });
        }
        let Some(s) = shown.as_mut() else { return };
        let fit = s.protocol.size_for(Resize::Fit(None), area.as_size());
        let (w, h) = (fit.width.min(area.width), fit.height.min(area.height));
        let at = Rect::new(area.x + (area.width - w) / 2, area.y + (area.height - h) / 2, w, h);
        frame.render_stateful_widget(StatefulImage::default(), at, &mut s.protocol);
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

/// From the disk cache, else from YouTube (and then kept on disk).
fn fetch(id: &str, dir: Option<&Path>) -> Option<DynamicImage> {
    let path = dir.map(|d| d.join(format!("{}.jpg", file_name(id))));
    if let Some(bytes) = path.as_ref().and_then(|p| std::fs::read(p).ok())
        && let Some(image) = decode(&bytes)
    {
        return Some(image);
    }
    let bytes = download(id).ok()?;
    let image = decode(&bytes)?;
    if let (Some(dir), Some(path)) = (dir, path) {
        let _ = std::fs::create_dir_all(dir).and_then(|()| std::fs::write(path, &bytes));
    }
    Some(image)
}

fn download(id: &str) -> Result<Vec<u8>, String> {
    let url = format!("https://i.ytimg.com/vi/{id}/mqdefault.jpg");
    let agent: ureq::Agent = ureq::Agent::config_builder().timeout_global(Some(Duration::from_secs(10))).build().into();
    let mut jpeg = Vec::new();
    let reply = agent.get(&url).call().map_err(|e| e.to_string())?;
    reply.into_body().into_reader().take(1 << 20).read_to_end(&mut jpeg).map_err(|e| e.to_string())?;
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
        terminal.draw(|f| thumbs.draw(f, target.clone(), mode, Pixelation::Off)).unwrap();
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
}
