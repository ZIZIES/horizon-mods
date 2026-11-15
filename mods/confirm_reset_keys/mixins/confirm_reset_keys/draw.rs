// While Reset Keys waits for its second click, the button says so.
fn confirm_reset_keys_draw(
    game: &crate::game::Game<'_>,
    batch: &mut horizon_client::hud::HudBatch,
    screen: &crate::game::native_hooks::NativeScreenView<'_>,
) {
    use horizon_client::keybinds::confirm_reset_keys as confirm;
    use horizon_client::options_ui::Page;
    if screen.id != "options"
        || game.ui.options.page() != Page::KeyBinds
        || !confirm::armed(confirm::now_millis())
    {
        return;
    }
    // KeyBindsScreen: Reset Keys is 150 wide, left of the middle, in the footer.
    let (w, h) = game.gui_size();
    let (x, y) = (w / 2.0 - 154.0, h - 27.0);
    horizon_client::ui::blit_sprite(
        batch,
        &game.gui.atlas,
        "gui/sprites/widget/button_highlighted",
        x,
        y,
        150.0,
        20.0,
        u32::MAX,
    );
    let width = game.native_text_width(confirm::LABEL);
    game.native_text(
        batch,
        confirm::LABEL,
        [x + (150.0 - width) / 2.0, y + 6.0],
        0xFF_5555,
        true,
    );
}
