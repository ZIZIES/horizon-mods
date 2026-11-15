// Force Close Loading Screen for Horizon, after kennytv's Fabric mod: the
// "Loading terrain" screen shown after a login, a respawn or a change of
// dimension closes at once instead of waiting for the chunks around the
// player. This file is the mod's switch and its test, and the module the
// bundle adds beside the game's level load tracker.
use std::sync::atomic::{AtomicBool, Ordering};

pub const ID: &str = "force_close_loading_screen";
pub const VERSION: &str = "1.0.0";
pub const DESCRIPTION: &str = "Closes the terrain loading screen at once instead of waiting for the chunks you're in to be sent.";

/// Whether the player left the mod on in Options > Mods (the game says so
/// each frame; the tracker cannot ask).
static ON: AtomicBool = AtomicBool::new(true);

pub fn set_on(on: bool) {
    ON.store(on, Ordering::Relaxed);
}

pub fn on() -> bool {
    ON.load(Ordering::Relaxed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_switch_follows_the_game() {
        assert!(on());
        set_on(false);
        assert!(!on());
        set_on(true);
        assert!(on());
    }
}
