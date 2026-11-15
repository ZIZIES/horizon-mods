// Once a frame: what the screen's clicks and keys asked for.
fn librarian_frame(game: &mut crate::game::Game<'_>) {
    crate::game::librarian_glue::frame(game);
}
