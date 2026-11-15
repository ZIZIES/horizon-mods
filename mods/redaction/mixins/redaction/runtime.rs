// REDACTION for Horizon, after Polyfrost's Fabric mod (GPL-3.0-or-later): a
// handful of small visual tweaks, each off until it is turned on. Blackbar,
// a flat hotbar across the whole screen whose highlight slides between
// slots; snow around the inventory; a title screen button that rejoins the
// last server; a field of view of its own for the hand. This file is the
// mod's logic and its tests: the settings, the numbers and the state. The
// helpers draw with the game's own calls.
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};

pub const ID: &str = "redaction";
pub const VERSION: &str = "1.0.0";
pub const DESCRIPTION: &str = "Small visual tweaks: Blackbar (a flat full-width hotbar), inventory snow, a rejoin-last-server button and a custom hand FOV.";

const BAR_OPACITIES: [u32; 4] = [33, 50, 75, 100];
const HIGHLIGHTS: [(&str, u32); 3] = [("Solid", 255), ("Soft", 128), ("Faint", 64)];
const SNOW_COUNTS: [usize; 4] = [50, 100, 200, 300];
const HAND_FOV: std::ops::RangeInclusive<u32> = 30..=150;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Settings {
    pub blackbar: bool,
    /// Index in `BAR_OPACITIES`.
    pub bar_opacity: usize,
    /// Index in `HIGHLIGHTS`.
    pub highlight: usize,
    pub slot_numbers: bool,
    pub custom_hand_fov: bool,
    pub hand_fov: u32,
    pub last_server: bool,
    pub snow: bool,
    /// Index in `SNOW_COUNTS`.
    pub snow_count: usize,
}

impl Default for Settings {
    /// The mod's own: everything off, the bar a third opaque, a hand FOV of
    /// 125 and a hundred flakes for when they are turned on.
    fn default() -> Self {
        Self {
            blackbar: false,
            bar_opacity: 0,
            highlight: 0,
            slot_numbers: false,
            custom_hand_fov: false,
            hand_fov: 125,
            last_server: false,
            snow: false,
            snow_count: 1,
        }
    }
}

impl Settings {
    pub fn read(get: impl Fn(&str) -> Option<String>) -> Self {
        let d = Self::default();
        let is = |key: &str, default: bool| get(key).map_or(default, |v| v.trim() == "true");
        let index = |key: &str, default: usize, len: usize| {
            get(key)
                .and_then(|v| v.trim().parse::<usize>().ok())
                .filter(|i| *i < len)
                .unwrap_or(default)
        };
        Self {
            blackbar: is("blackbar", d.blackbar),
            bar_opacity: index("bar_opacity", d.bar_opacity, BAR_OPACITIES.len()),
            highlight: index("highlight", d.highlight, HIGHLIGHTS.len()),
            slot_numbers: is("slot_numbers", d.slot_numbers),
            custom_hand_fov: is("custom_hand_fov", d.custom_hand_fov),
            hand_fov: get("hand_fov")
                .and_then(|v| v.trim().parse::<u32>().ok())
                .filter(|v| HAND_FOV.contains(v))
                .unwrap_or(d.hand_fov),
            last_server: is("last_server", d.last_server),
            snow: is("snow", d.snow),
            snow_count: index("snow_count", d.snow_count, SNOW_COUNTS.len()),
        }
    }
    pub fn stored(&self) -> [(&'static str, String); 9] {
        [
            ("blackbar", self.blackbar.to_string()),
            ("bar_opacity", self.bar_opacity.to_string()),
            ("highlight", self.highlight.to_string()),
            ("slot_numbers", self.slot_numbers.to_string()),
            ("custom_hand_fov", self.custom_hand_fov.to_string()),
            ("hand_fov", self.hand_fov.to_string()),
            ("last_server", self.last_server.to_string()),
            ("snow", self.snow.to_string()),
            ("snow_count", self.snow_count.to_string()),
        ]
    }
    pub fn labels(&self) -> Vec<String> {
        let on = |b: bool| if b { "ON" } else { "OFF" };
        vec![
            format!("Blackbar: {}", on(self.blackbar)),
            format!("Blackbar Opacity: {}%", BAR_OPACITIES[self.bar_opacity]),
            format!("Blackbar Highlight: {}", HIGHLIGHTS[self.highlight].0),
            format!("Blackbar Slot Numbers: {}", on(self.slot_numbers)),
            format!("Custom Hand FOV: {}", on(self.custom_hand_fov)),
            format!("Hand FOV: {}", self.hand_fov),
            format!("Last Server Joined Button: {}", on(self.last_server)),
            format!("Inventory Snow: {}", on(self.snow)),
            format!("Snow Particles: {}", SNOW_COUNTS[self.snow_count]),
        ]
    }
    pub fn click(&mut self, button: usize) {
        match button {
            0 => self.blackbar = !self.blackbar,
            1 => self.bar_opacity = (self.bar_opacity + 1) % BAR_OPACITIES.len(),
            2 => self.highlight = (self.highlight + 1) % HIGHLIGHTS.len(),
            3 => self.slot_numbers = !self.slot_numbers,
            4 => self.custom_hand_fov = !self.custom_hand_fov,
            5 => {
                self.hand_fov += 5;
                if self.hand_fov > *HAND_FOV.end() {
                    self.hand_fov = *HAND_FOV.start();
                }
            }
            6 => self.last_server = !self.last_server,
            7 => self.snow = !self.snow,
            8 => self.snow_count = (self.snow_count + 1) % SNOW_COUNTS.len(),
            _ => {}
        }
    }
    /// The bar's colour, ARGB: black at its opacity.
    pub fn bar_color(&self) -> u32 {
        (BAR_OPACITIES[self.bar_opacity] * 255 / 100) << 24
    }
    /// The selected slot's highlight, ARGB: white at its strength.
    pub fn highlight_color(&self) -> u32 {
        (HIGHLIGHTS[self.highlight].1 << 24) | 0xFF_FFFF
    }
    pub fn snow_flakes(&self) -> usize {
        SNOW_COUNTS[self.snow_count]
    }
}

// ---- state -----------------------------------------------------------------

static ON: AtomicBool = AtomicBool::new(true);
static SETTINGS: Mutex<Option<Settings>> = Mutex::new(None);
/// The player clicked the rejoin button: the game joins on its next frame.
static REJOIN: AtomicBool = AtomicBool::new(false);
/// The mod is inside the game's own hotbar layout, which it asked for.
static INSIDE: AtomicBool = AtomicBool::new(false);

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

pub fn ask_rejoin() {
    REJOIN.store(true, Ordering::Relaxed);
}

pub fn take_rejoin() -> bool {
    REJOIN.swap(false, Ordering::Relaxed)
}

pub fn inside() -> bool {
    INSIDE.load(Ordering::Relaxed)
}

pub fn set_inside(inside: bool) {
    INSIDE.store(inside, Ordering::Relaxed);
}

fn now_ms() -> u64 {
    static START: std::sync::OnceLock<std::time::Instant> = std::sync::OnceLock::new();
    START.get_or_init(std::time::Instant::now).elapsed().as_millis() as u64
}

// ---- Blackbar ----------------------------------------------------------------

/// The hotbar's left edge on a GUI `width` wide, each slot 20 apart.
pub fn slot_x(width: i32, slot: f32) -> i32 {
    width / 2 - 91 + (slot * 20.0).round() as i32
}

/// The highlight's place `dt_ms` later, on its way from `pos` to `target`
/// (in slots): most of the way in a tenth of a second, there when close.
pub fn ease(pos: f32, target: f32, dt_ms: u64) -> f32 {
    if dt_ms > 500 || (target - pos).abs() < 0.02 {
        return target;
    }
    pos + (target - pos) * (1.0 - (-(dt_ms as f32) / 45.0).exp())
}

/// Where the highlight is now, sliding to the `selected` slot.
pub fn slide(selected: i32) -> f32 {
    static AT: Mutex<Option<(f32, u64)>> = Mutex::new(None);
    let now = now_ms();
    let mut at = lock(&AT);
    let pos = match *at {
        Some((pos, then)) => ease(pos, selected as f32, now.saturating_sub(then)),
        None => selected as f32,
    };
    *at = Some((pos, now));
    pos
}

// ---- the rejoin button -------------------------------------------------------

/// The title screen's rejoin button: x, y, width, height.
pub fn rejoin_rect() -> [f32; 4] {
    [4.0, 4.0, 150.0, 20.0]
}

pub fn inside_rect([x, y, w, h]: [f32; 4], [mx, my]: [f32; 2]) -> bool {
    mx >= x && mx < x + w && my >= y && my < y + h
}

/// The button's label for a server called `name` at `address`, cut to fit
/// about `chars` characters.
pub fn rejoin_label(name: &str, address: &str, chars: usize) -> String {
    let server = if name.trim().is_empty() || name == "Minecraft Server" {
        address
    } else {
        name
    };
    let label = format!("Rejoin {server}");
    if label.chars().count() <= chars {
        return label;
    }
    let cut: String = label.chars().take(chars.saturating_sub(3)).collect();
    format!("{cut}...")
}

// ---- snow --------------------------------------------------------------------

/// A flake: position in 0..1 of the screen, fall and drift per second in
/// the same units, and its size in GUI pixels.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Flake {
    pub x: f32,
    pub y: f32,
    pub fall: f32,
    pub drift: f32,
    pub size: f32,
}

/// The snow: its flakes, and the generator that makes them.
#[derive(Debug, Default)]
pub struct Snow {
    pub flakes: Vec<Flake>,
    seed: u64,
}

impl Snow {
    fn random(&mut self) -> f32 {
        // A 64-bit LCG (Knuth's): snow needs no better.
        self.seed = self
            .seed
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        ((self.seed >> 40) as f32) / ((1u64 << 24) as f32)
    }
    fn flake(&mut self, top: bool) -> Flake {
        Flake {
            x: self.random(),
            y: if top { -0.02 } else { self.random() },
            fall: 0.04 + self.random() * 0.10,
            drift: (self.random() - 0.5) * 0.04,
            size: 1.0 + (self.random() * 2.0).floor(),
        }
    }
    /// `count` flakes, `dt` seconds on: they fall and drift, and one off
    /// the bottom comes back in at the top.
    pub fn step(&mut self, count: usize, dt: f32) {
        self.flakes.truncate(count);
        while self.flakes.len() < count {
            let flake = self.flake(false);
            self.flakes.push(flake);
        }
        let dt = dt.clamp(0.0, 0.1);
        for i in 0..self.flakes.len() {
            let mut f = self.flakes[i];
            f.y += f.fall * dt;
            f.x = (f.x + f.drift * dt).rem_euclid(1.0);
            if f.y > 1.0 {
                f = self.flake(true);
            }
            self.flakes[i] = f;
        }
    }
}

/// The frame's flakes on a `width` by `height` GUI, as x, y, size: stepped
/// by the time since the last call, without those over `panel` (x, y,
/// width, height), so the snow reads as behind it.
pub fn snow_frame(count: usize, [width, height]: [f32; 2], panel: [f32; 4]) -> Vec<[f32; 3]> {
    static SNOW: Mutex<(Option<Snow>, u64)> = Mutex::new((None, 0));
    let now = now_ms();
    let mut state = lock(&SNOW);
    let dt = now.saturating_sub(state.1) as f32 / 1000.0;
    state.1 = now;
    let snow = state.0.get_or_insert_with(Snow::default);
    snow.step(count, dt);
    snow.flakes
        .iter()
        .map(|f| [f.x * width, f.y * height, f.size])
        .filter(|[x, y, size]| {
            let [px, py, pw, ph] = panel;
            *x + *size <= px || *x >= px + pw || *y + *size <= py || *y >= py + ph
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn everything_is_off_until_turned_on() {
        let s = Settings::read(|_| None);
        assert_eq!(s, Settings::default());
        assert!(!(s.blackbar || s.custom_hand_fov || s.last_server || s.snow || s.slot_numbers));
        assert_eq!(s.bar_color(), 0x5400_0000);
        assert_eq!(s.highlight_color(), 0xFFFF_FFFF);
        assert_eq!((s.hand_fov, s.snow_flakes()), (125, 100));
    }

    #[test]
    fn settings_click_through_and_store() {
        let mut s = Settings::default();
        for button in 0..9 {
            s.click(button);
        }
        assert_eq!(s.labels().len(), 9);
        assert_eq!(s.labels()[1], "Blackbar Opacity: 50%");
        assert_eq!(s.labels()[5], "Hand FOV: 130");
        let stored = s.stored();
        let back = Settings::read(|key| {
            stored.iter().find(|(k, _)| *k == key).map(|(_, v)| v.clone())
        });
        assert_eq!(back, s);
        // The hand FOV wraps, and nonsense on disk is the default.
        s.hand_fov = 150;
        s.click(5);
        assert_eq!(s.hand_fov, 30);
        let odd = Settings::read(|key| (key == "hand_fov" || key == "highlight").then(|| "999".into()));
        assert_eq!((odd.hand_fov, odd.highlight), (125, 0));
    }

    #[test]
    fn the_highlight_slides_and_arrives() {
        let mut pos = 0.0;
        let mut last = 0.0;
        for _ in 0..60 {
            pos = ease(pos, 8.0, 16);
            assert!(pos >= last && pos <= 8.0);
            last = pos;
        }
        assert_eq!(pos, 8.0);
        // Back the other way, and at once after a long pause.
        assert!(ease(8.0, 0.0, 16) < 8.0);
        assert_eq!(ease(8.0, 0.0, 2000), 0.0);
        // The slots sit where the game's own selection does.
        assert_eq!(slot_x(427, 0.0), 122);
        assert_eq!(slot_x(427, 8.0), 282);
    }

    #[test]
    fn the_rejoin_button_names_the_server() {
        assert_eq!(rejoin_label("Hypixel", "mc.hypixel.net", 24), "Rejoin Hypixel");
        assert_eq!(rejoin_label("", "play.example.org", 24), "Rejoin play.example.org");
        assert_eq!(rejoin_label("Minecraft Server", "10.0.0.2", 24), "Rejoin 10.0.0.2");
        assert_eq!(rejoin_label("A very long server name here", "x", 16), "Rejoin A very...");
        assert!(inside_rect(rejoin_rect(), [5.0, 5.0]));
        assert!(!inside_rect(rejoin_rect(), [154.0, 5.0]));
    }

    #[test]
    fn snow_falls_stays_on_screen_and_keeps_its_count() {
        let mut snow = Snow::default();
        snow.step(100, 0.0);
        assert_eq!(snow.flakes.len(), 100);
        let before = snow.flakes.clone();
        snow.step(100, 0.05);
        assert!(snow.flakes.iter().zip(&before).all(|(now, then)| now.y > then.y));
        for _ in 0..2000 {
            snow.step(100, 0.1);
            assert!(snow.flakes.iter().all(|f| (0.0..1.0).contains(&f.x) && f.y <= 1.0));
        }
        snow.step(50, 0.1);
        assert_eq!(snow.flakes.len(), 50);
        // None is drawn over the panel.
        let panel = [100.0, 50.0, 176.0, 166.0];
        for [x, y, size] in snow_frame(300, [427.0, 240.0], panel) {
            assert!(x + size <= 100.0 || x >= 276.0 || y + size <= 50.0 || y >= 216.0);
        }
    }
}
