// The hotbar tab's rows, from the current page's file.
fn librarian_refresh(game: &mut crate::game::Game<'_>) -> std::ops::ControlFlow<()> {
    if !game.native_on(crate::game::librarian::ID) {
        return std::ops::ControlFlow::Continue(());
    }
    crate::game::librarian_glue::refresh_saved_hotbars(game);
    std::ops::ControlFlow::Break(())
}
