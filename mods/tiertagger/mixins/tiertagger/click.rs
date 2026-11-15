// A click on the mod's settings page: change the setting and keep it.
fn tiertagger_click(game: &mut crate::game::Game<'_>, mod_id: &str, button: usize) {
    use horizon_client::tab_list::{tiertagger as tt, tiertagger_net as net};
    if mod_id != tt::ID {
        return;
    }
    let mut settings = {
        let game = &*game;
        tt::settings().unwrap_or_else(|| tt::Settings::read(|key| game.native_setting(tt::ID, key)))
    };
    let other_list = settings.click(button, &tt::modes());
    if other_list || button == tt::BUTTON_CLEAR {
        tt::clear();
        net::forget_modes();
    }
    for (key, value) in settings.stored() {
        game.native_set_setting(tt::ID, key, &value);
    }
    tt::set_settings(settings);
    net::ask_modes();
}
