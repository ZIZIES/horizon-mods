// A key while the rename box is open.
fn librarian_key_pressed(
    game: &crate::game::Game<'_>,
    screen: &crate::game::native_hooks::NativeScreenView<'_>,
    key: &str,
    canceled: &mut bool,
) {
    crate::game::librarian_glue::key_pressed(game, screen, key, canceled);
}
