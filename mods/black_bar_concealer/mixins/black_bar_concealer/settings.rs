// The mod's settings page.
fn black_bar_concealer_page(
    game: &mut crate::game::Game<'_>,
    mod_id: &str,
    buttons: &mut Vec<String>,
) {
    use horizon_client::skins::black_bar_concealer as bnv;
    if mod_id != bnv::ID {
        return;
    }
    let game = &*game;
    buttons.extend(bnv::settings(|key| game.native_setting(bnv::ID, key)).labels());
}
