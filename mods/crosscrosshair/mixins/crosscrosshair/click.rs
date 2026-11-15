// A click on the mod's settings page: change the setting and keep it.
fn crosscrosshair_click(game: &mut crate::game::Game<'_>, mod_id: &str, button: usize) {
    use horizon_client::hud_layout::crosscrosshair as cc;
    if mod_id != cc::ID {
        return;
    }
    let mut settings = {
        let game = &*game;
        cc::Settings::read(|key| game.native_setting(cc::ID, key))
    };
    settings.click(button);
    for (key, value) in settings.stored() {
        game.native_set_setting(cc::ID, key, &value);
    }
    cc::set_settings(settings);
}
