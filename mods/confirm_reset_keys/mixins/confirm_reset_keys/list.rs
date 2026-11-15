// The mod's row in Options > Mods.
fn confirm_reset_keys_listed(mods: &mut Vec<crate::game::native_hooks::NativeMod>) {
    use horizon_client::keybinds::confirm_reset_keys as confirm;
    mods.push(crate::game::native_hooks::NativeMod {
        id: confirm::ID.into(),
        title: format!("Confirm Reset Keys {}", confirm::VERSION),
        description: confirm::DESCRIPTION.into(),
        keys: Vec::new(),
    });
}
