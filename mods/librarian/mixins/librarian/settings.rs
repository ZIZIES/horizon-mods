// Librarian's settings page.
fn librarian_page(game: &mut crate::game::Game<'_>, mod_id: &str, buttons: &mut Vec<String>) {
    crate::game::librarian_glue::settings_page(game, mod_id, buttons);
}
