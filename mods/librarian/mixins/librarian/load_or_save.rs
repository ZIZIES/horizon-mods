// The save and load keys, on the current page's file.
fn librarian_load_or_save(
    game: &mut crate::game::Game<'_>,
    index: usize,
    load: bool,
) -> std::ops::ControlFlow<()> {
    if !game.native_on(crate::game::librarian::ID) {
        return std::ops::ControlFlow::Continue(());
    }
    crate::game::librarian_glue::hotbar_load_or_save(game, index, load);
    std::ops::ControlFlow::Break(())
}
