// Once a frame: tell the text fields whether the mod is on.
fn better_selection_frame(game: &mut crate::game::Game<'_>) {
    use horizon_client::ui::better_selection as bs;
    bs::set_on(game.native_on(bs::ID));
}
