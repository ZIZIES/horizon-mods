// ConnectScreen.startConnecting: remember the server for the rejoin button.
fn redaction_connect(game: &crate::game::Game<'_>, address: &str, name: &str, lan: bool, realm: bool) {
    use horizon_client::hud_layout::redaction as rd;
    if lan || realm || address.is_empty() {
        return;
    }
    game.native_set_setting(rd::ID, "last_server_address", address);
    game.native_set_setting(rd::ID, "last_server_name", name);
}
