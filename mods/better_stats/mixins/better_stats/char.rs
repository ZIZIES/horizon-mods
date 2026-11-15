// A character typed on the Statistics screen: into the focused search box.
fn better_stats_char(
    game: &crate::game::Game<'_>,
    screen: &crate::game::native_hooks::NativeScreenView<'_>,
    c: char,
    canceled: &mut bool,
) {
    use crate::game::better_stats as bs;
    if screen.id != "stats" || !game.native_on(bs::ID) {
        return;
    }
    bs::with(|s| {
        if s.focused {
            *canceled = true;
            s.insert(c);
        }
    });
}
