// The mod's row in Options > Mods.
fn crosscrosshair_listed(mods: &mut Vec<crate::game::native_hooks::NativeMod>) {
    use horizon_client::hud_layout::crosscrosshair as cc;
    mods.push(crate::game::native_hooks::NativeMod {
        id: cc::ID.into(),
        title: format!("crosscrosshair {}", cc::VERSION),
        description: cc::DESCRIPTION.into(),
        keys: Vec::new(),
    });
}
