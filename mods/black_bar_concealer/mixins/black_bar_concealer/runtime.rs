// Black Bar Concealer for Horizon, after Gerrygames' Fabric mod: a skin drawn
// for the slim (3 pixel) arms but worn on the wide model shows black bars
// down its arms, where the wide arms read pixels the skin never painted.
// Such a skin is given the slim model instead, or, with Prefer Wide Model,
// its arm pixels are stretched to the wide layout. This file is the mod's
// logic and its tests, and the module the bundle adds beside the game's
// skin code.
use std::cell::{Cell, RefCell};
use std::collections::{HashMap, HashSet};
use std::sync::Mutex;

pub const ID: &str = "black_bar_concealer";
pub const VERSION: &str = "1.0.0";
pub const DESCRIPTION: &str = "Conceals the black bars of slim (Alex) skins used on the wide (Steve) model.";

const SIZE: usize = 64;
const BYTES: usize = SIZE * SIZE * 4;

/// Each arm's corner in the skin: the two arms, then their sleeves.
const RIGHT_ARM: (usize, usize) = (40, 16);
const LEFT_ARM: (usize, usize) = (32, 48);
const RIGHT_SLEEVE: (usize, usize) = (40, 32);
const LEFT_SLEEVE: (usize, usize) = (48, 48);

fn at(x: usize, y: usize) -> usize {
    (y * SIZE + x) * 4
}

/// The pixels of a 16x16 arm block only the wide model reads: the last two
/// columns of the top and bottom faces' rows and of the side faces' rows.
fn wide_only(x: usize, y: usize) -> bool {
    if y < 4 { (10..12).contains(&x) } else { x >= 14 }
}

/// A 64x64 skin as it was drawn (before the game makes its base layer
/// opaque) is a slim one: both arms leave the wide model's own pixels clear,
/// and have something painted where the slim arms read.
pub fn slim_drawn(rgba: &[u8]) -> bool {
    if rgba.len() != BYTES {
        return false;
    }
    [RIGHT_ARM, LEFT_ARM].into_iter().all(|(bx, by)| {
        let clear = (0..16).all(|y| {
            (0..16).all(|x| !wide_only(x, y) || rgba[at(bx + x, by + y) + 3] == 0)
        });
        let painted = (4..16).any(|y| (4..7).any(|x| rgba[at(bx + x, by + y) + 3] != 0));
        clear && painted
    })
}

/// A skin's colours without its alpha, which is all the game changes when
/// it processes a 64x64 skin: the same before and after.
pub fn fingerprint(rgba: &[u8]) -> u64 {
    let mut hash = 0xcbf2_9ce4_8422_2325u64;
    for pixel in rgba.chunks_exact(4) {
        for byte in &pixel[..3] {
            hash = (hash ^ u64::from(*byte)).wrapping_mul(0x0000_0100_0000_01b3);
        }
    }
    hash
}

/// The slim-drawn skins seen by the downloader's threads, by fingerprint.
static NOTED: Mutex<Option<HashSet<u64>>> = Mutex::new(None);
const MOST_NOTED: usize = 8192;

/// The downloader is about to process this decoded skin.
pub fn note_raw(width: u32, height: u32, rgba: &[u8]) {
    if width != 64 || height != 64 || !slim_drawn(rgba) {
        return;
    }
    let mut noted = NOTED.lock().unwrap_or_else(|e| e.into_inner());
    let noted = noted.get_or_insert_with(HashSet::new);
    if noted.len() >= MOST_NOTED {
        noted.clear();
    }
    noted.insert(fingerprint(rgba));
}

/// A processed skin was slim-drawn when it came.
pub fn noted(processed: &[u8]) -> bool {
    let print = fingerprint(processed);
    NOTED
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .as_ref()
        .is_some_and(|noted| noted.contains(&print))
}

/// The slim layout's column for a wide face's column: the middle one twice.
const STRETCH: [usize; 4] = [0, 1, 1, 2];

/// Where the wide layout's pixel of an arm block comes from in the slim one.
fn slim_source(x: usize, y: usize) -> usize {
    if y < 4 {
        match x {
            4..8 => 4 + STRETCH[x - 4],
            8..12 => 7 + STRETCH[x - 8],
            _ => x,
        }
    } else {
        match x {
            4..8 => 4 + STRETCH[x - 4],
            8..12 => 7 + (x - 8),
            12..16 => 11 + STRETCH[x - 12],
            _ => x,
        }
    }
}

/// Generate Missing Pixels: a processed slim-drawn skin's arms and sleeves
/// laid out for the wide model, each 3 pixel face stretched to 4.
pub fn widen(img: &mut [u8]) {
    if img.len() != BYTES {
        return;
    }
    for ((bx, by), opaque) in [
        (RIGHT_ARM, true),
        (LEFT_ARM, true),
        (RIGHT_SLEEVE, false),
        (LEFT_SLEEVE, false),
    ] {
        for y in 0..16 {
            let start = at(bx, by + y);
            let mut row = [0u8; 64];
            row.copy_from_slice(&img[start..start + 64]);
            for x in 0..16 {
                let from = slim_source(x, y) * 4;
                let to = start + x * 4;
                img[to..to + 4].copy_from_slice(&row[from..from + 4]);
                // The faces the model shows; the corners stay as they were.
                let shown = if y < 4 { (4..12).contains(&x) } else { true };
                if opaque && shown {
                    img[to + 3] = 255;
                }
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Settings {
    /// Keep the wide arms and generate their missing pixels, instead of
    /// giving the skin the slim model.
    pub prefer_wide: bool,
}

impl Settings {
    pub fn read(get: impl Fn(&str) -> Option<String>) -> Self {
        Self {
            prefer_wide: get("prefer_wide").is_some_and(|v| v.trim() == "true"),
        }
    }
    pub fn labels(&self) -> Vec<String> {
        vec![format!(
            "Prefer Wide Model: {}",
            if self.prefer_wide { "ON" } else { "OFF" }
        )]
    }
    pub fn click(&mut self, button: usize) {
        if button == 0 {
            self.prefer_wide = !self.prefer_wide;
        }
    }
    pub fn stored(&self) -> [(&'static str, String); 1] {
        [("prefer_wide", self.prefer_wide.to_string())]
    }
}

/// What the mod does with a wide player's skin.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fix {
    /// Not a slim-drawn skin, or one already widened: as the game has it.
    Leave,
    /// Wear it on the slim model.
    Slim,
    /// Stretch its arms to the wide layout, once.
    Widen,
}

/// The fix for a skin: `slim_drawn` is whether it was noted, `widened`
/// whether its pixels were already stretched (then the slim model would be
/// the wrong one, whatever the setting says now).
pub fn fix(settings: Settings, slim_drawn: bool, widened: bool) -> Fix {
    match (slim_drawn, widened, settings.prefer_wide) {
        (false, _, _) | (true, true, _) => Fix::Leave,
        (true, false, true) => Fix::Widen,
        (true, false, false) => Fix::Slim,
    }
}

thread_local! {
    static SETTINGS: Cell<Option<Settings>> = const { Cell::new(None) };
    /// Each skin location looked at: whether it was slim-drawn.
    static KNOWN: RefCell<HashMap<String, bool>> = RefCell::new(HashMap::new());
    /// The locations whose pixels were widened.
    static WIDENED: RefCell<HashSet<String>> = RefCell::new(HashSet::new());
    /// The mod is inside the game's own skin pass, which it asked for.
    static INSIDE: Cell<bool> = const { Cell::new(false) };
}

pub fn inside() -> bool {
    INSIDE.with(Cell::get)
}

pub fn set_inside(inside: bool) {
    INSIDE.with(|i| i.set(inside));
}

/// The settings; `get` reads a stored one the first time.
pub fn settings(get: impl Fn(&str) -> Option<String>) -> Settings {
    SETTINGS.with(|kept| {
        kept.get().unwrap_or_else(|| {
            let read = Settings::read(get);
            kept.set(Some(read));
            read
        })
    })
}

pub fn set_settings(settings: Settings) {
    SETTINGS.with(|kept| kept.set(Some(settings)));
}

/// Whether the skin at `location` was slim-drawn; `image` gives its
/// processed pixels the first time it is asked (`None`: not there yet, ask
/// again).
pub fn known<'a>(location: &str, image: impl FnOnce() -> Option<&'a [u8]>) -> bool {
    if let Some(known) = KNOWN.with(|k| k.borrow().get(location).copied()) {
        return known;
    }
    let Some(image) = image() else {
        return false;
    };
    let slim = noted(image);
    KNOWN.with(|k| {
        let mut k = k.borrow_mut();
        if k.len() >= MOST_NOTED {
            k.clear();
        }
        k.insert(location.to_owned(), slim);
    });
    slim
}

pub fn widened(location: &str) -> bool {
    WIDENED.with(|w| w.borrow().contains(location))
}

pub fn set_widened(location: &str) {
    WIDENED.with(|w| w.borrow_mut().insert(location.to_owned()));
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A skin with every pixel painted `colour`, opaque.
    fn filled(colour: [u8; 3]) -> Vec<u8> {
        let mut img = vec![0u8; BYTES];
        for p in img.chunks_exact_mut(4) {
            p.copy_from_slice(&[colour[0], colour[1], colour[2], 255]);
        }
        img
    }

    /// A slim-drawn skin: each arm block's column `x` is painted (x, y, 9)
    /// where the slim layout has pixels, and clear elsewhere.
    fn slim_skin() -> Vec<u8> {
        let mut img = filled([50, 60, 70]);
        for (bx, by) in [RIGHT_ARM, LEFT_ARM, RIGHT_SLEEVE, LEFT_SLEEVE] {
            for y in 0..16 {
                for x in 0..16 {
                    let used = if y < 4 { (4..10).contains(&x) } else { x < 14 };
                    let p = at(bx + x, by + y);
                    let pixel = if used { [x as u8, y as u8, 9, 255] } else { [0, 0, 0, 0] };
                    img[p..p + 4].copy_from_slice(&pixel);
                }
            }
        }
        img
    }

    #[test]
    fn only_slim_drawn_skins_are_told_apart() {
        assert!(slim_drawn(&slim_skin()));
        assert!(!slim_drawn(&filled([1, 2, 3])));
        assert!(!slim_drawn(&vec![0u8; BYTES]), "a blank image is no skin");
        assert!(!slim_drawn(&[0u8; 64 * 32 * 4]), "legacy skins have wide arms");
        // One painted pixel in a wide-only column: a wide skin.
        let mut wide = slim_skin();
        wide[at(RIGHT_ARM.0 + 15, RIGHT_ARM.1 + 8) + 3] = 255;
        assert!(!slim_drawn(&wide));
    }

    #[test]
    fn a_skin_is_known_again_once_its_alpha_is_gone() {
        let raw = slim_skin();
        let mut processed = raw.clone();
        for p in processed.chunks_exact_mut(4) {
            p[3] = 255;
        }
        assert_eq!(fingerprint(&raw), fingerprint(&processed));
        assert_ne!(fingerprint(&raw), fingerprint(&filled([1, 2, 3])));
        assert!(!noted(&processed));
        note_raw(64, 64, &raw);
        assert!(noted(&processed));
        note_raw(64, 64, &filled([4, 5, 6]));
        assert!(!noted(&filled([4, 5, 6])));
    }

    #[test]
    fn widening_stretches_each_face_and_leaves_no_bar() {
        let mut img = slim_skin();
        widen(&mut img);
        for (bx, by) in [RIGHT_ARM, LEFT_ARM] {
            // Side rows: outer 0..4, front 4..8, inner 8..12, back 12..16.
            let row: Vec<u8> = (0..16).map(|x| img[at(bx + x, by + 8)]).collect();
            assert_eq!(row, [0, 1, 2, 3, 4, 5, 5, 6, 7, 8, 9, 10, 11, 12, 12, 13]);
            // Top 4..8 and bottom 8..12.
            let top: Vec<u8> = (4..12).map(|x| img[at(bx + x, by + 1)]).collect();
            assert_eq!(top, [4, 5, 5, 6, 7, 8, 8, 9]);
            for y in 0..16 {
                for x in 0..16 {
                    if y >= 4 || (4..12).contains(&x) {
                        assert_eq!(img[at(bx + x, by + y) + 3], 255, "bar at {x},{y}");
                    }
                }
            }
        }
        // The rest of the skin is untouched.
        assert_eq!(img[at(8, 8)..at(8, 8) + 4], [50, 60, 70, 255]);
    }

    #[test]
    fn the_fix_follows_the_setting_but_never_slims_a_widened_skin() {
        let (slim, wide) = (Settings { prefer_wide: false }, Settings { prefer_wide: true });
        assert_eq!(fix(slim, false, false), Fix::Leave);
        assert_eq!(fix(slim, true, false), Fix::Slim);
        assert_eq!(fix(wide, true, false), Fix::Widen);
        assert_eq!(fix(wide, true, true), Fix::Leave);
        assert_eq!(fix(slim, true, true), Fix::Leave);
    }

    #[test]
    fn settings_default_click_and_store() {
        let mut s = Settings::read(|_| None);
        assert!(!s.prefer_wide);
        s.click(0);
        let stored = s.stored();
        assert!(Settings::read(|_| Some(stored[0].1.clone())).prefer_wide);
    }
}
