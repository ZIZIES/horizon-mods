// The mod's row in Options > Mods.
fn black_bar_concealer_listed(mods: &mut Vec<crate::game::native_hooks::NativeMod>) {
    use horizon_client::skins::black_bar_concealer as bnv;
    mods.push(crate::game::native_hooks::NativeMod {
        id: bnv::ID.into(),
        title: format!("Black Bar Concealer {}", bnv::VERSION),
        description: bnv::DESCRIPTION.into(),
        keys: Vec::new(),
    });
}
