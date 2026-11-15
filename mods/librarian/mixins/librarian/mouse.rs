// A click on a page button or on the title.
fn librarian_mouse(
    game: &crate::game::Game<'_>,
    screen: &crate::game::native_hooks::NativeScreenView<'_>,
    button: i32,
    canceled: &mut bool,
) {
    crate::game::librarian_glue::mouse_pressed(game, screen, button, canceled);
}
