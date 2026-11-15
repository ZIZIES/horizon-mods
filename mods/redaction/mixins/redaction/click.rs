// A click on the mod's settings page: change the setting and keep it.
fn redaction_click(game: &mut crate::game::Game<'_>, mod_id: &str, button: usize) {
    use horizon_client::hud_layout::redaction as rd;
    if mod_id != rd::ID {
        return;
    }
    let mut settings = {
        let game = &*game;
        rd::Settings::read(|key| game.native_setting(rd::ID, key))
    };
    settings.click(button);
    for (key, value) in settings.stored() {
        game.native_set_setting(rd::ID, key, &value);
    }
    rd::set_settings(settings);
}
