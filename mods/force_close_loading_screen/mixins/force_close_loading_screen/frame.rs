// Once a frame: tell the tracker whether the mod is on.
fn force_close_frame(game: &mut crate::game::Game<'_>) {
    use horizon_client::level_loading::force_close_loading_screen as force_close;
    force_close::set_on(game.native_on(force_close::ID));
}
