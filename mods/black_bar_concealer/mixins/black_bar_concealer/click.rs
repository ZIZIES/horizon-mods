// A click on the mod's settings page: change the setting and keep it.
fn black_bar_concealer_click(game: &mut crate::game::Game<'_>, mod_id: &str, button: usize) {
    use horizon_client::skins::black_bar_concealer as bnv;
    if mod_id != bnv::ID {
        return;
    }
    let mut settings = bnv::settings(|key| game.native_setting(bnv::ID, key));
    settings.click(button);
    bnv::set_settings(settings);
    for (key, value) in settings.stored() {
        game.native_set_setting(bnv::ID, key, &value);
    }
}
