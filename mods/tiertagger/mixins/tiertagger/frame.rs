// Once a frame: the mod's switch, and the first time its settings and the
// tier list's game modes.
fn tiertagger_frame(game: &mut crate::game::Game<'_>) {
    use horizon_client::tab_list::{tiertagger as tt, tiertagger_net as net};
    let on = game.native_on(tt::ID);
    tt::set_on(on);
    if !on {
        return;
    }
    if tt::settings().is_none() {
        let game = &*game;
        tt::set_settings(tt::Settings::read(|key| game.native_setting(tt::ID, key)));
    }
    net::ask_modes();
}
