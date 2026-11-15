// The mod's settings page.
fn redaction_page(game: &mut crate::game::Game<'_>, mod_id: &str, buttons: &mut Vec<String>) {
    use horizon_client::hud_layout::redaction as rd;
    if mod_id != rd::ID {
        return;
    }
    let game = &*game;
    buttons.extend(rd::Settings::read(|key| game.native_setting(rd::ID, key)).labels());
}
