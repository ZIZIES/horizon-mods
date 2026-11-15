// TierTagger for Horizon, after uku's Fabric mod (MPL-2.0): a player's PvP
// tier from MCTiers or SubTiers before their name, on their name tag and in
// the player list ("HT3 | name"). This file is the mod's logic and its
// tests: which tier to show, its text and colour, the settings and the
// cache. net.rs fetches, and the helpers put the text in the game.
use std::collections::HashMap;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};

pub const ID: &str = "tiertagger";
pub const VERSION: &str = "1.0.0";
pub const DESCRIPTION: &str = "Displays players' PvP tiers from MCTiers or SubTiers in their name tag and the player list.";

/// The tier lists: name and API root.
pub const TIERLISTS: [(&str, &str); 2] = [
    ("MCTiers", "https://mctiers.net/api"),
    ("SubTiers", "https://subtiers.net/api"),
];

/// The mod's tier colours, and the one of a retired tier.
const TIER_COLORS: [(&str, u32); 10] = [
    ("HT1", 0xE8BA3A),
    ("LT1", 0xD5B355),
    ("HT2", 0xC4D3E7),
    ("LT2", 0xA0A7B2),
    ("HT3", 0xF89F5A),
    ("LT3", 0xC67B42),
    ("HT4", 0x81749A),
    ("LT4", 0x655B79),
    ("HT5", 0x8F82A8),
    ("LT5", 0x655B79),
];
const RETIRED_COLOR: u32 = 0xA2D6FF;
const OTHER_COLOR: u32 = 0xD3D3D3;
pub const GRAY: u32 = 0xAAAAAA;
/// What stands between the tier and the name.
pub const SEPARATOR: &str = " | ";

/// One game mode's ranking of a player (`/v2/profile/<uuid>/rankings`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Ranking {
    pub tier: i32,
    /// 0 high, 1 low.
    pub pos: i32,
    pub peak: Option<(i32, i32)>,
    pub retired: bool,
}

impl Ranking {
    /// Lower is better.
    fn comparable(&self) -> i32 {
        self.tier * 2 + self.pos
    }
    /// "HT3", "LT5", or a retired player's peak, "RHT2".
    pub fn label(&self) -> String {
        let name = |tier: i32, pos: i32| format!("{}T{tier}", if pos == 0 { "H" } else { "L" });
        match self.peak {
            Some((tier, pos)) if self.retired => format!("R{}", name(tier, pos)),
            _ => name(self.tier, self.pos),
        }
    }
}

/// A tier label's colour.
pub fn color(label: &str) -> u32 {
    if label.starts_with('R') {
        return RETIRED_COLOR;
    }
    TIER_COLORS
        .iter()
        .find(|(tier, _)| *tier == label)
        .map_or(OTHER_COLOR, |(_, color)| *color)
}

/// Show Highest Tier.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Highest {
    /// Only tiers in the selected game mode.
    Never,
    /// The highest tier of a player not ranked in the selected game mode.
    NotFound,
    /// Always the highest tier.
    Always,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Settings {
    /// The selected game mode's id; empty for none.
    pub mode: String,
    pub show_retired: bool,
    pub highest: Highest,
    pub player_list: bool,
    /// Index in `TIERLISTS`.
    pub tierlist: usize,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            mode: String::new(),
            show_retired: true,
            highest: Highest::NotFound,
            player_list: true,
            tierlist: 0,
        }
    }
}

/// The settings page's buttons, in order.
pub const BUTTON_MODE: usize = 0;
pub const BUTTON_CLEAR: usize = 5;

impl Settings {
    pub fn read(get: impl Fn(&str) -> Option<String>) -> Self {
        let default = Self::default();
        let is = |key: &str, default: bool| get(key).map_or(default, |v| v.trim() == "true");
        Self {
            mode: get("mode").map_or(default.mode, |v| v.trim().to_owned()),
            show_retired: is("show_retired", default.show_retired),
            highest: match get("highest").as_deref().map(str::trim) {
                Some("never") => Highest::Never,
                Some("always") => Highest::Always,
                _ => Highest::NotFound,
            },
            player_list: is("player_list", default.player_list),
            tierlist: get("tierlist")
                .and_then(|v| TIERLISTS.iter().position(|(name, _)| *name == v.trim()))
                .unwrap_or(0),
        }
    }
    pub fn stored(&self) -> [(&'static str, String); 5] {
        [
            ("mode", self.mode.clone()),
            ("show_retired", self.show_retired.to_string()),
            (
                "highest",
                match self.highest {
                    Highest::Never => "never",
                    Highest::NotFound => "not_found",
                    Highest::Always => "always",
                }
                .to_owned(),
            ),
            ("player_list", self.player_list.to_string()),
            ("tierlist", TIERLISTS[self.tierlist].0.to_owned()),
        ]
    }
    pub fn api(&self) -> &'static str {
        TIERLISTS[self.tierlist].1
    }
    /// The page's buttons; `modes` are the tier list's game modes (id, title).
    pub fn labels(&self, modes: &[(String, String)]) -> Vec<String> {
        let on = |b: bool| if b { "ON" } else { "OFF" };
        let mode = modes
            .iter()
            .find(|(id, _)| *id == self.mode)
            .map_or(if self.mode.is_empty() { "None" } else { &self.mode }, |(_, title)| title);
        vec![
            format!("Gamemode: {mode}"),
            format!("Show Retired: {}", on(self.show_retired)),
            format!(
                "Show Highest Tier: {}",
                match self.highest {
                    Highest::Never => "Never",
                    Highest::NotFound => "When Not Found",
                    Highest::Always => "Always",
                }
            ),
            format!("Show Tiers in Player List: {}", on(self.player_list)),
            format!("Tierlist: {}", TIERLISTS[self.tierlist].0),
            "Clear Tiers Cache".to_owned(),
        ]
    }
    /// A click on button `button`. True when the tier list changed (the game
    /// modes and every cached tier are another list's).
    pub fn click(&mut self, button: usize, modes: &[(String, String)]) -> bool {
        match button {
            BUTTON_MODE => {
                // None, then each mode in the list's order.
                let at = modes.iter().position(|(id, _)| *id == self.mode);
                self.mode = match at {
                    None => modes.first().map(|(id, _)| id.clone()).unwrap_or_default(),
                    Some(at) => modes.get(at + 1).map(|(id, _)| id.clone()).unwrap_or_default(),
                };
            }
            1 => self.show_retired = !self.show_retired,
            2 => {
                self.highest = match self.highest {
                    Highest::Never => Highest::NotFound,
                    Highest::NotFound => Highest::Always,
                    Highest::Always => Highest::Never,
                }
            }
            3 => self.player_list = !self.player_list,
            4 => {
                self.tierlist = (self.tierlist + 1) % TIERLISTS.len();
                self.mode.clear();
                return true;
            }
            _ => {}
        }
        false
    }
}

/// The ranking shown for a player with these `rankings` (mode id, ranking).
pub fn pick(rankings: &[(String, Ranking)], settings: &Settings) -> Option<Ranking> {
    let selected = rankings
        .iter()
        .find(|(mode, _)| *mode == settings.mode && !settings.mode.is_empty())
        .map(|(_, r)| *r);
    let highest = rankings.iter().map(|(_, r)| *r).min_by_key(Ranking::comparable);
    let shown = match (selected, settings.highest) {
        (None, Highest::Never) => None,
        (None, _) | (Some(_), Highest::Always) => highest,
        (Some(selected), _) => Some(selected),
    }?;
    (settings.show_retired || !shown.retired).then_some(shown)
}

/// A UUID a tier list can know: version 4, an account's (offline players'
/// are version 3).
pub fn ranked_uuid(uuid: u128) -> bool {
    (uuid >> 76) & 0xF == 4
}

/// `01234567-89ab-cdef-0123-456789abcdef`.
pub fn hyphenated(uuid: u128) -> String {
    let hex = format!("{uuid:032x}");
    format!(
        "{}-{}-{}-{}-{}",
        &hex[..8],
        &hex[8..12],
        &hex[12..16],
        &hex[16..20],
        &hex[20..]
    )
}

// ---- state: the switch, the settings, the game modes and the tiers ---------

static ON: AtomicBool = AtomicBool::new(true);
static SETTINGS: Mutex<Option<Settings>> = Mutex::new(None);
static MODES: Mutex<Vec<(String, String)>> = Mutex::new(Vec::new());
/// Each player asked about: `None` while the answer is on its way, then
/// their rankings (empty for a player the list does not know).
static TIERS: Mutex<Option<HashMap<u128, Option<Vec<(String, Ranking)>>>>> = Mutex::new(None);
/// The most players kept before the cache starts over.
const MOST_TIERS: usize = 4096;

thread_local! {
    /// The mod is inside the game's own function, which it asked for.
    static INSIDE: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

pub fn inside() -> bool {
    INSIDE.with(std::cell::Cell::get)
}

pub fn set_inside(inside: bool) {
    INSIDE.with(|i| i.set(inside));
}

fn lock<T>(mutex: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(|e| e.into_inner())
}

pub fn set_on(on: bool) {
    ON.store(on, Ordering::Relaxed);
}

pub fn on() -> bool {
    ON.load(Ordering::Relaxed)
}

/// The settings, once the game has read them.
pub fn settings() -> Option<Settings> {
    lock(&SETTINGS).clone()
}

pub fn set_settings(settings: Settings) {
    *lock(&SETTINGS) = Some(settings);
}

pub fn modes() -> Vec<(String, String)> {
    lock(&MODES).clone()
}

pub fn set_modes(modes: Vec<(String, String)>) {
    *lock(&MODES) = modes;
}

/// What is known of a player's tier.
#[derive(Debug, PartialEq, Eq)]
pub enum Known {
    /// Not asked yet: ask (the player is now marked as asked).
    Ask,
    /// Asked, no answer yet; or not a player a list can know.
    Nothing,
    /// The tier's label and colour.
    Tier(String, u32),
}

/// The tier to show for `uuid`.
pub fn tier(uuid: u128) -> Known {
    let Some(settings) = settings() else {
        return Known::Nothing;
    };
    if !on() || !ranked_uuid(uuid) {
        return Known::Nothing;
    }
    let mut tiers = lock(&TIERS);
    let tiers = tiers.get_or_insert_with(HashMap::new);
    match tiers.get(&uuid) {
        None => {
            if tiers.len() >= MOST_TIERS {
                tiers.clear();
            }
            tiers.insert(uuid, None);
            Known::Ask
        }
        Some(None) => Known::Nothing,
        Some(Some(rankings)) => match pick(rankings, &settings) {
            Some(ranking) => {
                let label = ranking.label();
                let color = color(&label);
                Known::Tier(label, color)
            }
            None => Known::Nothing,
        },
    }
}

/// The answer for `uuid` came.
pub fn store(uuid: u128, rankings: Vec<(String, Ranking)>) {
    if let Some(tiers) = lock(&TIERS).as_mut() {
        // Only a player still waited for: the cache may have been cleared.
        if let Some(slot) = tiers.get_mut(&uuid) {
            *slot = Some(rankings);
        }
    }
}

pub fn clear() {
    *lock(&TIERS) = None;
}

// ---- name tags: legacy text ------------------------------------------------

/// The sixteen legacy colours, `§0` to `§f`.
const LEGACY: [u32; 16] = [
    0x000000, 0x0000AA, 0x00AA00, 0x00AAAA, 0xAA0000, 0xAA00AA, 0xFFAA00, 0xAAAAAA, 0x555555,
    0x5555FF, 0x55FF55, 0x55FFFF, 0xFF5555, 0xFF55FF, 0xFFFF55, 0xFFFFFF,
];

/// The legacy colour code nearest `rgb`.
pub fn legacy_code(rgb: u32) -> char {
    let parts = |c: u32| [(c >> 16) & 0xFF, (c >> 8) & 0xFF, c & 0xFF].map(|p| p as i32);
    let want = parts(rgb);
    let (index, _) = LEGACY
        .iter()
        .enumerate()
        .min_by_key(|(_, c)| {
            let have = parts(**c);
            (0..3).map(|i| (have[i] - want[i]).pow(2)).sum::<i32>()
        })
        .unwrap_or((15, &0xFFFFFF));
    char::from_digit(index as u32, 16).unwrap_or('f')
}

/// The legacy colour a tier's colour is written as on a name tag.
pub fn legacy_color(rgb: u32) -> u32 {
    LEGACY[legacy_code(rgb).to_digit(16).unwrap_or(15) as usize]
}

/// The tier before a name, in legacy codes: the tier in its (nearest
/// legacy) colour, a grey separator, then back to the name's own style.
pub fn legacy_prefix(label: &str, color: u32) -> String {
    format!("§{}{label}§7{SEPARATOR}§r", legacy_code(color))
}

/// One character's style: colour, then bold, italic, underlined,
/// strikethrough and obfuscated.
pub type Look = (Option<u32>, [bool; 5]);

/// Styled characters as one legacy string that reads back the same (colours
/// to their nearest legacy ones).
pub fn flatten(chars: impl Iterator<Item = (char, Look)>) -> String {
    let mut out = String::new();
    let mut last: Option<Look> = None;
    for (c, look) in chars {
        if last != Some(look) {
            match look.0 {
                Some(rgb) => {
                    out.push('§');
                    out.push(legacy_code(rgb));
                }
                None => out.push_str("§r"),
            }
            for (on, code) in look.1.iter().zip(['l', 'o', 'n', 'm', 'k']) {
                if *on {
                    out.push('§');
                    out.push(code);
                }
            }
            last = Some(look);
        }
        out.push(c);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ranking(tier: i32, pos: i32) -> Ranking {
        Ranking {
            tier,
            pos,
            peak: None,
            retired: false,
        }
    }

    fn with(mode: &str, highest: Highest) -> Settings {
        Settings {
            mode: mode.into(),
            highest,
            ..Settings::default()
        }
    }

    #[test]
    fn labels_and_their_colours() {
        assert_eq!(ranking(1, 0).label(), "HT1");
        assert_eq!(ranking(5, 1).label(), "LT5");
        let retired = Ranking {
            peak: Some((2, 0)),
            retired: true,
            ..ranking(3, 1)
        };
        assert_eq!(retired.label(), "RHT2");
        // Retired without a peak: its tier, as it is.
        assert_eq!(Ranking { retired: true, ..ranking(3, 1) }.label(), "LT3");
        assert_eq!(color("HT1"), 0xE8BA3A);
        assert_eq!(color("RHT2"), 0xA2D6FF);
        assert_eq!(color("HT9"), 0xD3D3D3);
    }

    #[test]
    fn the_tier_shown_follows_the_mode_and_the_highest_setting() {
        let rankings = vec![
            ("sword".to_owned(), ranking(3, 1)),
            ("vanilla".to_owned(), ranking(2, 0)),
            ("axe".to_owned(), ranking(2, 1)),
        ];
        let best = Some(ranking(2, 0));
        assert_eq!(pick(&rankings, &with("sword", Highest::Never)), Some(ranking(3, 1)));
        assert_eq!(pick(&rankings, &with("sword", Highest::NotFound)), Some(ranking(3, 1)));
        assert_eq!(pick(&rankings, &with("sword", Highest::Always)), best);
        assert_eq!(pick(&rankings, &with("uhc", Highest::Never)), None);
        assert_eq!(pick(&rankings, &with("uhc", Highest::NotFound)), best);
        assert_eq!(pick(&rankings, &with("", Highest::NotFound)), best);
        assert_eq!(pick(&[], &with("", Highest::Always)), None);
    }

    #[test]
    fn retired_tiers_hide_when_asked() {
        let rankings = vec![(
            "pot".to_owned(),
            Ranking {
                peak: Some((1, 1)),
                retired: true,
                ..ranking(2, 0)
            },
        )];
        let mut settings = with("pot", Highest::NotFound);
        assert_eq!(pick(&rankings, &settings).map(|r| r.label()).as_deref(), Some("RLT1"));
        settings.show_retired = false;
        assert_eq!(pick(&rankings, &settings), None);
    }

    #[test]
    fn uuids_read_as_the_api_wants_them() {
        let uuid = 0x069a79f4_44e9_4726_a5be_fca90e38aaf5u128;
        assert_eq!(hyphenated(uuid), "069a79f4-44e9-4726-a5be-fca90e38aaf5");
        assert!(ranked_uuid(uuid));
        assert!(!ranked_uuid(0x069a79f4_44e9_3726_a5be_fca90e38aaf5));
        assert_eq!(hyphenated(1).len(), 36);
    }

    #[test]
    fn settings_default_click_and_store() {
        let modes = vec![
            ("vanilla".to_owned(), "Vanilla".to_owned()),
            ("sword".to_owned(), "Sword".to_owned()),
        ];
        let mut s = Settings::read(|_| None);
        assert_eq!(s, Settings::default());
        assert_eq!(s.labels(&modes)[0], "Gamemode: None");
        // None, each mode, None again.
        s.click(BUTTON_MODE, &modes);
        assert_eq!(s.labels(&modes)[0], "Gamemode: Vanilla");
        s.click(BUTTON_MODE, &modes);
        assert_eq!(s.mode, "sword");
        s.click(1, &modes);
        s.click(2, &modes);
        s.click(3, &modes);
        let stored = s.stored();
        let back = Settings::read(|key| {
            stored.iter().find(|(k, _)| *k == key).map(|(_, v)| v.clone())
        });
        assert_eq!(back, s);
        assert_eq!((back.show_retired, back.highest, back.player_list), (false, Highest::Always, false));
        s.click(BUTTON_MODE, &modes);
        assert_eq!(s.mode, "");
        // Another tier list: its own modes.
        s.mode = "sword".into();
        assert!(s.click(4, &modes));
        assert_eq!((s.api(), s.mode.as_str()), ("https://subtiers.net/api", ""));
        assert_eq!(s.labels(&modes).len(), BUTTON_CLEAR + 1);
    }

    #[test]
    fn the_cache_asks_once_and_answers_when_told() {
        let uuid = 0x11111111_2222_4333_8444_555555555555u128;
        set_settings(Settings::default());
        clear();
        assert_eq!(tier(uuid), Known::Ask);
        assert_eq!(tier(uuid), Known::Nothing);
        store(uuid, vec![("vanilla".to_owned(), ranking(3, 0))]);
        assert_eq!(tier(uuid), Known::Tier("HT3".into(), 0xF89F5A));
        // An offline player's UUID is never asked about.
        assert_eq!(tier(0x11111111_2222_3333_8444_555555555555), Known::Nothing);
        // A late answer after a clear is dropped.
        clear();
        store(uuid, vec![("vanilla".to_owned(), ranking(1, 0))]);
        assert_eq!(tier(uuid), Known::Ask);
    }

    #[test]
    fn legacy_text_reads_back_the_same() {
        assert_eq!(legacy_code(0xFFAA00), '6');
        assert_eq!(legacy_code(0xE8BA3A), '6');
        assert_eq!(legacy_code(0xA2D6FF), 'b');
        assert_eq!(legacy_code(0x000000), '0');
        assert_eq!(legacy_prefix("HT1", 0xE8BA3A), "§6HT1§7 | §r");
        let plain = (None, [false; 5]);
        let red_bold = (Some(0xFF5555), [true, false, false, false, false]);
        let text = [('[', red_bold), ('A', red_bold), (']', red_bold), (' ', plain), ('x', plain)];
        assert_eq!(flatten(text.into_iter()), "§c§l[A]§r x");
        assert_eq!(flatten(std::iter::empty()), "");
    }
}
