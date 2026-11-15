// After a screen is drawn: the rejoin button on the title screen, and the
// snow around the inventory.
fn redaction_draw(
    game: &crate::game::Game<'_>,
    batch: &mut horizon_client::hud::HudBatch,
    screen: &crate::game::native_hooks::NativeScreenView<'_>,
) {
    use horizon_client::hud_layout::redaction as rd;
    let Some(settings) = rd::settings() else {
        return;
    };
    if screen.id == "title" && settings.last_server {
        let Some(address) = game.native_setting(rd::ID, "last_server_address") else {
            return;
        };
        let name = game.native_setting(rd::ID, "last_server_name").unwrap_or_default();
        let [x, y, w, h] = rd::rejoin_rect();
        let sprite = if rd::inside_rect(rd::rejoin_rect(), screen.mouse) {
            "gui/sprites/widget/button_highlighted"
        } else {
            "gui/sprites/widget/button"
        };
        horizon_client::ui::blit_sprite(batch, &game.gui.atlas, sprite, x, y, w, h, u32::MAX);
        let label = rd::rejoin_label(&name, &address, 24);
        let width = game.native_text_width(&label);
        game.native_text(batch, &label, [x + (w - width) / 2.0, y + 6.0], 0xFF_FFFF, true);
    } else if settings.snow && matches!(screen.id, "inventory" | "creative") {
        let (width, height) = game.gui_size();
        for [x, y, size] in rd::snow_frame(settings.snow_flakes(), [width, height], screen.rect) {
            game.native_fill(batch, [x, y, size, size], 0xD0FF_FFFF);
        }
    }
}
