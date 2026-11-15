// A click on the Statistics screen: the search box takes the focus, or
// gives it up.
fn better_stats_mouse(
    game: &crate::game::Game<'_>,
    screen: &crate::game::native_hooks::NativeScreenView<'_>,
    button: i32,
    canceled: &mut bool,
) {
    use crate::game::better_stats as bs;
    if screen.id != "stats" || !game.native_on(bs::ID) {
        return;
    }
    let (width, height) = game.gui_size();
    let over = bs::inside_rect(bs::box_rect([width, height]), screen.mouse);
    bs::with(|s| {
        s.focused = over;
        // A right click on the box empties it.
        if over && button == 1 {
            s.text.clear();
        }
    });
    if over {
        *canceled = true;
    }
}
