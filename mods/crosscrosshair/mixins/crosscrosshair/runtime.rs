// crosscrosshair for Horizon: a crosshair drawn by hand on a 15x15 grid, the
// size of vanilla's sprite, in the way of the Custom Crosshair Mod's draw
// mode. It starts as a small cross with a longer bottom arm. This file is the
// mod's logic and its tests: the shape, the settings, the editor's geometry
// and the state. The helpers draw with the game's own calls.
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};

pub const ID: &str = "crosscrosshair";
pub const VERSION: &str = "1.0.0";
pub const DESCRIPTION: &str =
    "A crosshair you draw yourself, pixel by pixel. Starts as a small cross.";

/// The grid's side: vanilla's crosshair sprite is 15x15.
pub const SIDE: i32 = 15;
/// The middle cell, which sits on the middle of the screen.
pub const MID: i32 = 7;
/// The buttons on the settings page, above the grid.
pub const BUTTONS: usize = 4;

/// Name, ARGB. `Inverted` is drawn white, then inverts what is under it, as
/// vanilla's crosshair does.
const COLOURS: [(&str, u32); 9] = [
    ("White", 0xFFFF_FFFF),
    ("Inverted", 0xFFFF_FFFF),
    ("Black", 0xFF00_0000),
    ("Red", 0xFFFF_5555),
    ("Green", 0xFF55_FF55),
    ("Blue", 0xFF55_55FF),
    ("Yellow", 0xFFFF_FF55),
    ("Aqua", 0xFF55_FFFF),
    ("Pink", 0xFFFF_55FF),
];
const INVERTED: usize = 1;

// ---- the shape ---------------------------------------------------------------

/// The drawn pixels: bit `x` of `rows[y]`, from the top left.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Shape {
    rows: [u16; SIDE as usize],
}

impl Shape {
    pub const fn empty() -> Self {
        Self {
            rows: [0; SIDE as usize],
        }
    }
    /// The mod's own: the middle, one pixel on each side of it, and one more
    /// under the bottom one.
    pub fn cross() -> Self {
        let mut shape = Self::empty();
        for (x, y) in [(0, -1), (-1, 0), (0, 0), (1, 0), (0, 1), (0, 2)] {
            shape.set(MID + x, MID + y, true);
        }
        shape
    }
    /// Whether the cell is drawn; false outside the grid.
    pub fn get(&self, x: i32, y: i32) -> bool {
        (0..SIDE).contains(&x) && (0..SIDE).contains(&y) && self.rows[y as usize] >> x & 1 == 1
    }
    pub fn set(&mut self, x: i32, y: i32, on: bool) {
        if (0..SIDE).contains(&x) && (0..SIDE).contains(&y) {
            let bit = 1u16 << x;
            if on {
                self.rows[y as usize] |= bit;
            } else {
                self.rows[y as usize] &= !bit;
            }
        }
    }
    /// The rows as hex, top first: `0000.0080.01c0...`.
    pub fn stored(&self) -> String {
        let rows: Vec<String> = self.rows.iter().map(|r| format!("{r:04x}")).collect();
        rows.join(".")
    }
    pub fn parse(text: &str) -> Option<Self> {
        let mut shape = Self::empty();
        let mut parts = text.trim().split('.');
        for row in &mut shape.rows {
            *row = u16::from_str_radix(parts.next()?, 16)
                .ok()
                .filter(|r| *r < 1 << SIDE)?;
        }
        parts.next().is_none().then_some(shape)
    }
    /// The pixels as horizontal runs `[x0, y, x1)`, measured from the middle
    /// cell: one quad a run.
    pub fn runs(&self) -> Vec<[i32; 3]> {
        runs(0, SIDE, |x, y| self.get(x, y))
    }
    /// The outline: the empty cells that touch a drawn one, corners too. It
    /// can stand one cell outside the grid.
    pub fn rim(&self) -> Vec<[i32; 3]> {
        runs(-1, SIDE + 1, |x, y| {
            !self.get(x, y)
                && (-1..=1).any(|dy| (-1..=1).any(|dx| self.get(x + dx, y + dy)))
        })
    }
}

fn runs(from: i32, to: i32, set: impl Fn(i32, i32) -> bool) -> Vec<[i32; 3]> {
    let mut out = Vec::new();
    for y in from..to {
        let mut x = from;
        while x < to {
            if !set(x, y) {
                x += 1;
                continue;
            }
            let start = x;
            while x < to && set(x, y) {
                x += 1;
            }
            out.push([start - MID, y - MID, x - MID]);
        }
    }
    out
}

// ---- settings ----------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Settings {
    pub shape: Shape,
    /// Index in `COLOURS`.
    pub colour: usize,
    pub outline: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            shape: Shape::cross(),
            colour: 0,
            outline: false,
        }
    }
}

impl Settings {
    pub fn read(get: impl Fn(&str) -> Option<String>) -> Self {
        let d = Self::default();
        Self {
            shape: get("pixels")
                .and_then(|v| Shape::parse(&v))
                .unwrap_or(d.shape),
            colour: get("colour")
                .and_then(|v| v.trim().parse::<usize>().ok())
                .filter(|i| *i < COLOURS.len())
                .unwrap_or(d.colour),
            outline: get("outline").map_or(d.outline, |v| v.trim() == "true"),
        }
    }
    pub fn stored(&self) -> [(&'static str, String); 3] {
        [
            ("pixels", self.shape.stored()),
            ("colour", self.colour.to_string()),
            ("outline", self.outline.to_string()),
        ]
    }
    /// The settings page's buttons, `BUTTONS` of them.
    pub fn labels(&self) -> Vec<String> {
        vec![
            format!("Colour: {}", COLOURS[self.colour].0),
            format!("Outline: {}", if self.outline { "ON" } else { "OFF" }),
            "Reset to the cross".into(),
            "Clear".into(),
        ]
    }
    pub fn click(&mut self, button: usize) {
        match button {
            0 => self.colour = (self.colour + 1) % COLOURS.len(),
            1 => self.outline = !self.outline,
            2 => self.shape = Shape::cross(),
            3 => self.shape = Shape::empty(),
            _ => {}
        }
    }
    pub fn argb(&self) -> u32 {
        COLOURS[self.colour].1
    }
    /// Whether the pixels invert what is under them instead of covering it.
    pub fn inverted(&self) -> bool {
        self.colour == INVERTED
    }
}

// ---- the editor ----------------------------------------------------------------

/// The editor's grid on the settings page: its top left corner and the side
/// of one cell, in GUI pixels.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Grid {
    pub x: f32,
    pub y: f32,
    pub cell: f32,
}

/// The grid, under the page's buttons and over its footer on a GUI of `gui`
/// (width, height); `None` when the window is too short for one.
pub fn grid([width, height]: [f32; 2]) -> Option<Grid> {
    // The page's list starts at 33 + 2: a 16 high title, then 25 a button.
    let top = 33.0 + 2.0 + 16.0 + 25.0 * BUTTONS as f32 + 4.0;
    // Its footer starts 33 from the bottom.
    let room = height - 33.0 - 4.0 - top;
    let cell = (room / SIDE as f32).floor().min(8.0);
    (cell >= 2.0).then(|| Grid {
        x: (width / 2.0 - cell * SIDE as f32 / 2.0).floor(),
        y: top,
        cell,
    })
}

impl Grid {
    pub fn side(&self) -> f32 {
        self.cell * SIDE as f32
    }
    /// The cell under `mouse`.
    pub fn cell_at(&self, [mx, my]: [f32; 2]) -> Option<(i32, i32)> {
        let (x, y) = (
            ((mx - self.x) / self.cell).floor() as i32,
            ((my - self.y) / self.cell).floor() as i32,
        );
        ((0..SIDE).contains(&x) && (0..SIDE).contains(&y)).then_some((x, y))
    }
    /// Cell `(x, y)` as `[x, y, w, h]`.
    pub fn rect(&self, x: i32, y: i32) -> [f32; 4] {
        [
            self.x + x as f32 * self.cell,
            self.y + y as f32 * self.cell,
            self.cell,
            self.cell,
        ]
    }
}

// ---- state -----------------------------------------------------------------

static ON: AtomicBool = AtomicBool::new(true);
static SETTINGS: Mutex<Option<Settings>> = Mutex::new(None);
/// A mouse button is held on the grid: 1 draws, 2 erases.
static PAINT: AtomicU8 = AtomicU8::new(0);

fn lock<T>(mutex: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(|e| e.into_inner())
}

pub fn set_on(on: bool) {
    ON.store(on, Ordering::Relaxed);
}

/// The settings, when the mod is on and the game has read them.
pub fn settings() -> Option<Settings> {
    if !ON.load(Ordering::Relaxed) {
        return None;
    }
    *lock(&SETTINGS)
}

pub fn loaded() -> bool {
    lock(&SETTINGS).is_some()
}

pub fn set_settings(settings: Settings) {
    *lock(&SETTINGS) = Some(settings);
}

/// A press on the grid: the drag that follows draws, or erases.
pub fn start_paint(draw: bool) {
    PAINT.store(if draw { 1 } else { 2 }, Ordering::Relaxed);
}

pub fn stop_paint() {
    PAINT.store(0, Ordering::Relaxed);
}

/// The held button passes over cell `(x, y)`: the pixels to store, when that
/// changed them.
pub fn paint(x: i32, y: i32) -> Option<String> {
    let draw = match PAINT.load(Ordering::Relaxed) {
        1 => true,
        2 => false,
        _ => return None,
    };
    let mut settings = lock(&SETTINGS);
    let settings = settings.as_mut()?;
    if settings.shape.get(x, y) == draw {
        return None;
    }
    settings.shape.set(x, y, draw);
    Some(settings.shape.stored())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_cross_is_a_plus_with_a_longer_bottom_arm() {
        let cross = Shape::cross();
        assert_eq!(
            cross.runs(),
            vec![[0, -1, 1], [-1, 0, 2], [0, 1, 1], [0, 2, 1]]
        );
        assert!(cross.get(MID, MID + 2) && !cross.get(MID, MID - 2));
        assert!(!cross.get(-1, 0) && !cross.get(SIDE, 0));
    }

    #[test]
    fn pixels_survive_being_stored() {
        let mut shape = Shape::cross();
        shape.set(0, 0, true);
        shape.set(14, 14, true);
        assert_eq!(Shape::parse(&shape.stored()), Some(shape));
        assert_eq!(Shape::parse(&Shape::empty().stored()), Some(Shape::empty()));
    }

    #[test]
    fn broken_pixels_are_not_read() {
        assert_eq!(Shape::parse(""), None);
        assert_eq!(Shape::parse("0000.0000"), None);
        let long = format!("{}.0000", Shape::cross().stored());
        assert_eq!(Shape::parse(&long), None);
        let wide = Shape::cross().stored().replacen("0000", "8000", 1);
        assert_eq!(Shape::parse(&wide), None);
        let d = Settings::read(|key| (key == "pixels").then(|| "nope".to_string()));
        assert_eq!(d, Settings::default());
    }

    #[test]
    fn the_rim_surrounds_the_pixels_and_leaves_the_grid() {
        let mut dot = Shape::empty();
        dot.set(MID, MID, true);
        assert_eq!(dot.rim(), vec![[-1, -1, 2], [-1, 0, 0], [1, 0, 2], [-1, 1, 2]]);
        let mut corner = Shape::empty();
        corner.set(0, 0, true);
        assert_eq!(
            corner.rim(),
            vec![[-8, -8, -5], [-8, -7, -7], [-6, -7, -5], [-8, -6, -5]]
        );
        assert!(Shape::empty().rim().is_empty());
    }

    #[test]
    fn settings_are_read_back_as_stored() {
        let mut s = Settings::default();
        s.click(0);
        s.click(1);
        s.shape.set(3, 4, true);
        let stored = s.stored();
        let read = Settings::read(|key| {
            stored
                .iter()
                .find(|(k, _)| *k == key)
                .map(|(_, v)| v.clone())
        });
        assert_eq!(read, s);
        assert!(read.inverted() && read.outline);
        assert_eq!(s.labels().len(), BUTTONS);
    }

    #[test]
    fn the_buttons_cycle_reset_and_clear() {
        let mut s = Settings::default();
        for _ in 0..COLOURS.len() {
            s.click(0);
        }
        assert_eq!(s.colour, 0);
        s.click(3);
        assert_eq!(s.shape, Shape::empty());
        s.click(2);
        assert_eq!(s.shape, Shape::cross());
    }

    #[test]
    fn the_grid_fits_between_the_buttons_and_the_footer() {
        let g = grid([480.0, 270.0]).unwrap();
        assert_eq!((g.y, g.cell), (155.0, 5.0));
        assert_eq!(g.x, 240.0 - 37.5 - 0.5);
        assert!(g.y + g.side() <= 270.0 - 33.0);
        assert_eq!(grid([1920.0, 1080.0]).unwrap().cell, 8.0);
        assert_eq!(grid([320.0, 200.0]), None);
        assert_eq!(g.cell_at([g.x, g.y]), Some((0, 0)));
        assert_eq!(g.cell_at([g.x + 74.9, g.y + 74.9]), Some((14, 14)));
        assert_eq!(g.cell_at([g.x - 0.1, g.y]), None);
        assert_eq!(g.cell_at([g.x + 75.0, g.y]), None);
        assert_eq!(g.rect(1, 2), [g.x + 5.0, g.y + 10.0, 5.0, 5.0]);
    }

    #[test]
    fn a_held_button_paints_and_says_what_to_store() {
        set_settings(Settings::default());
        assert_eq!(paint(0, 0), None);
        start_paint(true);
        let stored = paint(0, 0).unwrap();
        assert!(Shape::parse(&stored).unwrap().get(0, 0));
        assert_eq!(paint(0, 0), None);
        start_paint(false);
        assert_eq!(paint(0, 0), Some(Shape::cross().stored()));
        stop_paint();
        assert_eq!(paint(MID, MID), None);
        assert_eq!(settings().unwrap().shape, Shape::cross());
    }
}
