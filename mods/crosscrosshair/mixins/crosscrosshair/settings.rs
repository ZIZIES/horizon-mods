// The mod's settings page: its buttons. The grid under them is draw.rs.
fn crosscrosshair_page(game: &mut crate::game::Game<'_>, mod_id: &str, buttons: &mut Vec<String>) {
    use horizon_client::hud_layout::crosscrosshair as cc;
    if mod_id != cc::ID {
        return;
    }
    let game = &*game;
    let settings = cc::settings()
        .unwrap_or_else(|| cc::Settings::read(|key| game.native_setting(cc::ID, key)));
    buttons.extend(settings.labels());
}
