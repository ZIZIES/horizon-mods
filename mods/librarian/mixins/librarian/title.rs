// The hotbar tab's title: the page's name or number.
fn librarian_title(
    game: &crate::game::Game<'_>,
    tab: &horizon_client::creative::Tab,
) -> std::ops::ControlFlow<String> {
    crate::game::librarian_glue::tab_title(game, tab)
}
