// The mod's settings page.
fn tiertagger_page(game: &mut crate::game::Game<'_>, mod_id: &str, buttons: &mut Vec<String>) {
    use horizon_client::tab_list::tiertagger as tt;
    if mod_id != tt::ID {
        return;
    }
    let game = &*game;
    let settings = tt::settings()
        .unwrap_or_else(|| tt::Settings::read(|key| game.native_setting(tt::ID, key)));
    buttons.extend(settings.labels(&tt::modes()));
}
