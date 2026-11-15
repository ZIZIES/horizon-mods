// A click on Librarian's settings page.
fn librarian_click(game: &mut crate::game::Game<'_>, mod_id: &str, button: usize) {
    crate::game::librarian_glue::settings_click(game, mod_id, button);
}
