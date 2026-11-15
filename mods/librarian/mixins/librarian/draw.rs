// The page buttons and rename box over the creative screen's hotbar tab.
fn librarian_draw(
    game: &crate::game::Game<'_>,
    batch: &mut horizon_client::hud::HudBatch,
    screen: &crate::game::native_hooks::NativeScreenView<'_>,
) {
    crate::game::librarian_glue::draw(game, batch, screen);
}
