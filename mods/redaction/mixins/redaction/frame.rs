// Once a frame: the mod's switch, its settings the first time, and the
// rejoin the button asked for.
fn redaction_frame(game: &mut crate::game::Game<'_>) {
    use horizon_client::hud_layout::redaction as rd;
    rd::set_on(game.native_on(rd::ID));
    if !rd::loaded() {
        let game = &*game;
        rd::set_settings(rd::Settings::read(|key| game.native_setting(rd::ID, key)));
    }
    let rejoin = rd::take_rejoin();
    if rejoin {
        game.audio.play_button_click();
    }
    if rejoin
        && let Some(address) = game.native_setting(rd::ID, "last_server_address")
    {
        let name = game.native_setting(rd::ID, "last_server_name").unwrap_or_default();
        if let Err(error) = game.connect_server(&address, &name) {
            log::warn!("REDACTION: could not rejoin {address}: {error}");
        }
    }
}
