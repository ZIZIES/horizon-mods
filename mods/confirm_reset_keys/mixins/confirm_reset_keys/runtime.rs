// Confirm Reset Keys for Horizon: the Key Binds page's "Reset Keys" button
// resets every binding at one click, with no way back. With this mod the
// first click only arms it (the button says so for a few seconds); a second
// click in that time resets. This file is the mod's logic and its tests,
// and the module the bundle adds beside the game's key bindings.
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

pub const ID: &str = "confirm_reset_keys";
pub const VERSION: &str = "1.0.0";
pub const DESCRIPTION: &str =
    "Makes sure you don't accidentally reset your keys: Reset Keys needs a second click.";
/// What the armed button says.
pub const LABEL: &str = "Click again to confirm";
/// How long the first click stays armed, in milliseconds.
pub const WINDOW_MS: u64 = 5000;

/// Whether the player left the mod on in Options > Mods (the game says so
/// each frame; the key bindings cannot ask).
static ON: AtomicBool = AtomicBool::new(true);
/// Until when a second click resets; 0 when not armed.
static ARMED_UNTIL: AtomicU64 = AtomicU64::new(0);

pub fn now_millis() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_millis() as u64)
}

pub fn set_on(on: bool) {
    ON.store(on, Ordering::Relaxed);
    if !on {
        ARMED_UNTIL.store(0, Ordering::Relaxed);
    }
}

/// A click on Reset Keys at `now`: whether the keys reset. The first click
/// arms the button; a click while it is armed resets.
pub fn clicked(now: u64) -> bool {
    if !ON.load(Ordering::Relaxed) {
        return true;
    }
    if now <= ARMED_UNTIL.swap(0, Ordering::Relaxed) {
        return true;
    }
    ARMED_UNTIL.store(now + WINDOW_MS, Ordering::Relaxed);
    false
}

/// Whether the button is waiting for its second click at `now`.
pub fn armed(now: u64) -> bool {
    now <= ARMED_UNTIL.load(Ordering::Relaxed)
}

/// Leaving the page disarms it.
pub fn disarm() {
    ARMED_UNTIL.store(0, Ordering::Relaxed);
}

#[cfg(test)]
mod tests {
    use super::*;

    // One test: the state is the process's own.
    #[test]
    fn the_second_click_in_time_resets() {
        set_on(true);
        assert!(!clicked(1000));
        assert!(armed(1000 + WINDOW_MS));
        assert!(!armed(1001 + WINDOW_MS));
        assert!(clicked(1000 + WINDOW_MS));
        // Used up: the next click arms again.
        assert!(!armed(2000));
        assert!(!clicked(20_000));
        // Too late: armed again, not reset.
        assert!(!clicked(20_001 + WINDOW_MS));
        disarm();
        assert!(!armed(20_002 + WINDOW_MS));
        assert!(!clicked(40_000));
        // Turned off: every click resets, and nothing stays armed.
        set_on(false);
        assert!(!armed(40_001));
        assert!(clicked(50_000));
        set_on(true);
    }
}
