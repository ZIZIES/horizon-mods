// A click on the title screen's rejoin button: the game joins next frame.
fn redaction_mouse(
    game: &crate::game::Game<'_>,
    screen: &crate::game::native_hooks::NativeScreenView<'_>,
    button: i32,
    canceled: &mut bool,
) {
    use horizon_client::hud_layout::redaction as rd;
    if screen.id != "title" || button != 0 || *canceled {
        return;
    }
    if rd::settings().is_some_and(|s| s.last_server)
        && rd::inside_rect(rd::rejoin_rect(), screen.mouse)
        && game.native_setting(rd::ID, "last_server_address").is_some()
    {
        rd::ask_rejoin();
        *canceled = true;
    }
}
