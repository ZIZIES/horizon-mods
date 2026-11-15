// Once a frame: the mod's switch, and its settings the first time.
fn crosscrosshair_frame(game: &mut crate::game::Game<'_>) {
    use horizon_client::hud_layout::crosscrosshair as cc;
    cc::set_on(game.native_on(cc::ID));
    if !cc::loaded() {
        let game = &*game;
        cc::set_settings(cc::Settings::read(|key| game.native_setting(cc::ID, key)));
    }
}
