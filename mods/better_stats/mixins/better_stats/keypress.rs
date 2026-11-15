// A key on the Statistics screen while the search box has the focus.
fn better_stats_key(
    game: &crate::game::Game<'_>,
    screen: &crate::game::native_hooks::NativeScreenView<'_>,
    key: &str,
    canceled: &mut bool,
) {
    use crate::game::better_stats as bs;
    if screen.id != "stats" || !game.native_on(bs::ID) {
        return;
    }
    bs::with(|s| {
        if s.focused {
            *canceled = true;
            s.key(key);
        }
    });
}
