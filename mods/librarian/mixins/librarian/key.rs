// The next, previous and backup page keys.
fn librarian_key(game: &mut crate::game::Game<'_>, key: &str, pressed: bool) {
    crate::game::librarian_glue::key(game, key, pressed);
}
