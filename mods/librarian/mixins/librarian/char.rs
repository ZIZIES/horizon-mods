// A character typed while the rename box is open.
fn librarian_char(
    game: &crate::game::Game<'_>,
    screen: &crate::game::native_hooks::NativeScreenView<'_>,
    c: char,
    canceled: &mut bool,
) {
    crate::game::librarian_glue::char_typed(game, screen, c, canceled);
}
