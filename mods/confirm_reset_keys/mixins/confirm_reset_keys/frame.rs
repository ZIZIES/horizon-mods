// Once a frame: tell the key bindings whether the mod is on, and disarm the
// button once the Key Binds page is left.
fn confirm_reset_keys_frame(game: &mut crate::game::Game<'_>) {
    use horizon_client::keybinds::confirm_reset_keys as confirm;
    use horizon_client::options_ui::Page;
    confirm::set_on(game.native_on(confirm::ID));
    let on_page =
        game.screen == crate::game::Screen::Options && game.ui.options.page() == Page::KeyBinds;
    if !on_page {
        confirm::disarm();
    }
}
