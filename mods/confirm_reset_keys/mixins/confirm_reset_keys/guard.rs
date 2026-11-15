// Reset Keys: the first click arms, the second resets.
fn confirm_reset_keys_guard() -> std::ops::ControlFlow<()> {
    use crate::keybinds::confirm_reset_keys as confirm;
    if confirm::clicked(confirm::now_millis()) {
        std::ops::ControlFlow::Continue(())
    } else {
        std::ops::ControlFlow::Break(())
    }
}
