// The mod's row in Options > Mods.
fn redaction_listed(mods: &mut Vec<crate::game::native_hooks::NativeMod>) {
    use horizon_client::hud_layout::redaction as rd;
    mods.push(crate::game::native_hooks::NativeMod {
        id: rd::ID.into(),
        title: format!("REDACTION {}", rd::VERSION),
        description: rd::DESCRIPTION.into(),
        keys: Vec::new(),
    });
}
