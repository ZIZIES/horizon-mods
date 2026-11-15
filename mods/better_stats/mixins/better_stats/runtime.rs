// Statistics search for Horizon: the part of TheCSDev's Better Statistics
// Screen that fits the game's own Statistics screen, written from what that
// mod does. A search box in the screen's footer narrows the Items and Mobs
// tabs to the rows whose name or id holds what is typed. This file is the
// mod's logic and its tests: the box's text and focus, and the matching.
use std::cell::RefCell;

pub const ID: &str = "better_stats";
pub const VERSION: &str = "1.0.0";
pub const DESCRIPTION: &str = "A search box on the Statistics screen that filters the Items and Mobs tabs, after Better Statistics Screen.";

/// The most characters the box takes.
const MOST: usize = 32;

#[derive(Debug, Default)]
pub struct Search {
    pub text: String,
    pub focused: bool,
    /// The mod is inside one of the game's own row lists, which it asked for.
    pub inside: bool,
}

impl Search {
    pub fn insert(&mut self, c: char) {
        if !c.is_control() && self.text.chars().count() < MOST {
            self.text.push(c);
        }
    }
    /// A key by its vanilla name while the box is focused.
    pub fn key(&mut self, key: &str) {
        match key {
            "key.keyboard.backspace" => {
                self.text.pop();
            }
            "key.keyboard.delete" => self.text.clear(),
            "key.keyboard.enter" | "key.keyboard.keypad.enter" => self.focused = false,
            "key.keyboard.escape" => {
                // First the text, then the focus.
                if self.text.is_empty() {
                    self.focused = false;
                } else {
                    self.text.clear();
                }
            }
            _ => {}
        }
    }
}

thread_local! {
    static SEARCH: RefCell<Search> = RefCell::new(Search::default());
}

pub fn with<R>(f: impl FnOnce(&mut Search) -> R) -> R {
    SEARCH.with(|s| f(&mut s.borrow_mut()))
}

/// Whether a row with this registry `id` and shown `name` holds every word
/// of `query`, whatever the case; an empty query keeps every row.
pub fn matches(query: &str, id: &str, name: &str) -> bool {
    let (id, name) = (id.to_lowercase(), name.to_lowercase());
    query
        .to_lowercase()
        .split_whitespace()
        .all(|word| id.contains(word) || name.contains(word))
}

/// The box on a GUI `width` by `height`: x, y, width, height. Left of the
/// Done button (200 wide, centred), in the 33 pixel footer.
pub fn box_rect([width, height]: [f32; 2]) -> [f32; 4] {
    let room = (width / 2.0 - 100.0 - 12.0).clamp(40.0, 110.0);
    [6.0, height - 33.0 + 8.0, room, 16.0]
}

pub fn inside_rect([x, y, w, h]: [f32; 4], [mx, my]: [f32; 2]) -> bool {
    mx >= x && mx < x + w && my >= y && my < y + h
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rows_match_by_name_or_id_in_any_case() {
        assert!(matches("", "minecraft:stone", "Stone"));
        assert!(matches("STO", "minecraft:stone", "Stone"));
        assert!(matches("pick dia", "minecraft:diamond_pickaxe", "Diamond Pickaxe"));
        assert!(matches("minecraft:zom", "minecraft:zombie", "Zombie"));
        // Another language's name still answers to the id.
        assert!(matches("stone", "minecraft:stone", "Pedra"));
        assert!(matches("pedra", "minecraft:stone", "Pedra"));
        assert!(!matches("gold", "minecraft:stone", "Stone"));
        assert!(!matches("stone gold", "minecraft:stone", "Stone"));
    }

    #[test]
    fn the_box_types_erases_and_lets_go() {
        let mut s = Search { focused: true, ..Search::default() };
        for c in "zom\u{8}bie".chars() {
            s.insert(c);
        }
        assert_eq!(s.text, "zombie");
        s.key("key.keyboard.backspace");
        assert_eq!(s.text, "zombi");
        s.key("key.keyboard.escape");
        assert!(s.text.is_empty() && s.focused);
        s.key("key.keyboard.escape");
        assert!(!s.focused);
        for _ in 0..100 {
            s.insert('a');
        }
        assert_eq!(s.text.len(), 32);
    }

    #[test]
    fn the_box_sits_left_of_the_done_button() {
        for width in [320.0, 427.0, 640.0, 960.0] {
            let [x, y, w, h] = box_rect([width, 240.0]);
            assert!(x + w <= width / 2.0 - 100.0 || width < 330.0);
            assert!(y >= 240.0 - 33.0 && y + h <= 240.0);
        }
        assert!(inside_rect(box_rect([427.0, 240.0]), [10.0, 220.0]));
    }
}
