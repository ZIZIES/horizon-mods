// The mod's row in Options > Mods.
fn better_selection_listed(mods: &mut Vec<crate::game::native_hooks::NativeMod>) {
    use horizon_client::ui::better_selection as bs;
    mods.push(crate::game::native_hooks::NativeMod {
        id: bs::ID.into(),
        title: format!("Better Selection {}", bs::VERSION),
        description: bs::DESCRIPTION.into(),
        keys: Vec::new(),
    });
}
