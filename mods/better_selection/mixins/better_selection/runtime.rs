// Better Selection for Horizon, after MDLC01's Fabric mod: text fields move,
// select and delete by word the way a text editor does. A word is a run of
// letters and digits or a run of punctuation, not everything up to the next
// space; Left and Right step out of a selection to its edge; a double click
// selects just the word under the pointer. This file is the mod's logic and
// its tests, and the module the bundle adds beside the game's text field.
use std::sync::atomic::{AtomicBool, Ordering};

pub const ID: &str = "better_selection";
pub const VERSION: &str = "1.0.0";
pub const DESCRIPTION: &str = "Makes text more pleasant to select: editor-like word movement, selection and deletion in text fields.";

/// Whether the player left the mod on in Options > Mods (the game says so
/// each frame; a text field cannot ask).
static ON: AtomicBool = AtomicBool::new(true);

pub fn set_on(on: bool) {
    ON.store(on, Ordering::Relaxed);
}

pub fn on() -> bool {
    ON.load(Ordering::Relaxed)
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Class {
    Space,
    Word,
    Punctuation,
}

fn class(c: char) -> Class {
    if c.is_whitespace() {
        Class::Space
    } else if c.is_alphanumeric() || c == '_' {
        Class::Word
    } else {
        Class::Punctuation
    }
}

/// Where Ctrl+Left (`dir` below 0) or Ctrl+Right takes a cursor at character
/// `from`: back over any spaces to the start of the word before them,
/// forward over any spaces to the end of the word after them.
pub fn word_position(value: &str, dir: isize, from: usize) -> usize {
    let chars: Vec<char> = value.chars().collect();
    let mut pos = from.min(chars.len());
    if dir < 0 {
        while pos > 0 && class(chars[pos - 1]) == Class::Space {
            pos -= 1;
        }
        if pos > 0 {
            let word = class(chars[pos - 1]);
            while pos > 0 && class(chars[pos - 1]) == word {
                pos -= 1;
            }
        }
    } else {
        while pos < chars.len() && class(chars[pos]) == Class::Space {
            pos += 1;
        }
        if pos < chars.len() {
            let word = class(chars[pos]);
            while pos < chars.len() && class(chars[pos]) == word {
                pos += 1;
            }
        }
    }
    pos
}

/// The word a double click at character `pos` selects: the run the pointer
/// is on (the character after it, or else the one before it).
pub fn word_around(value: &str, pos: usize) -> (usize, usize) {
    let chars: Vec<char> = value.chars().collect();
    let pos = pos.min(chars.len());
    let after = chars.get(pos).map(|c| class(*c));
    let before = pos.checked_sub(1).map(|p| class(chars[p]));
    let word = match (after, before) {
        (Some(a), _) if a != Class::Space => a,
        (_, Some(b)) if b != Class::Space => b,
        (Some(a), _) => a,
        (None, Some(b)) => b,
        (None, None) => return (0, 0),
    };
    let (mut start, mut end) = (pos, pos);
    while start > 0 && class(chars[start - 1]) == word {
        start -= 1;
    }
    while end < chars.len() && class(chars[end]) == word {
        end += 1;
    }
    (start, end)
}

/// Where plain Left (`dir` below 0) or Right puts the cursor of a field
/// with a selection: at the selection's edge on that side. `None` without
/// a selection (the key moves one character, as ever).
pub fn collapse(cursor: usize, highlight: usize, dir: isize) -> Option<usize> {
    if cursor == highlight {
        return None;
    }
    Some(if dir < 0 {
        cursor.min(highlight)
    } else {
        cursor.max(highlight)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn words_end_at_punctuation_not_only_spaces() {
        let s = "/give @p minecraft:diamond_sword 1";
        // Forward: each run in turn, spaces skipped first.
        let mut stops = vec![];
        let mut pos = 0;
        while pos < s.len() {
            pos = word_position(s, 1, pos);
            stops.push(pos);
        }
        assert_eq!(stops, [1, 5, 7, 8, 18, 19, 32, 34]);
        // Backward from the end: the same boundaries' starts.
        let mut stops = vec![];
        let mut pos = s.len();
        while pos > 0 {
            pos = word_position(s, -1, pos);
            stops.push(pos);
        }
        assert_eq!(stops, [33, 19, 18, 9, 7, 6, 1, 0]);
    }

    #[test]
    fn the_ends_and_positions_past_them_hold() {
        assert_eq!(word_position("", 1, 0), 0);
        assert_eq!(word_position("", -1, 3), 0);
        assert_eq!(word_position("hello", 1, 9), 5);
        assert_eq!(word_position("hello   ", 1, 5), 8);
        assert_eq!(word_position("   hello", -1, 3), 0);
        assert_eq!(word_position("héllo wörld", -1, 11), 6);
    }

    #[test]
    fn a_double_click_takes_the_run_under_the_pointer() {
        let s = "say hello, world";
        assert_eq!(word_around(s, 6), (4, 9));
        assert_eq!(word_around(s, 4), (4, 9), "at a word's start");
        assert_eq!(word_around(s, 9), (9, 10), "the comma is its own run");
        assert_eq!(word_around(s, 3), (0, 3), "after a word, before a space");
        assert_eq!(word_around(s, 16), (11, 16), "at the end");
        assert_eq!(word_around("a   b", 2), (1, 4), "inside spaces");
        assert_eq!(word_around("", 0), (0, 0));
    }

    #[test]
    fn arrows_leave_a_selection_at_its_edge() {
        assert_eq!(collapse(3, 3, -1), None);
        assert_eq!(collapse(2, 7, -1), Some(2));
        assert_eq!(collapse(7, 2, -1), Some(2));
        assert_eq!(collapse(2, 7, 1), Some(7));
        assert_eq!(collapse(7, 2, 1), Some(7));
    }
}
