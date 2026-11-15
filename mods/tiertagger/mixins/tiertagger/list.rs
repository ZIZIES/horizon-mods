// The mod's row in Options > Mods.
fn tiertagger_listed(mods: &mut Vec<crate::game::native_hooks::NativeMod>) {
    use horizon_client::tab_list::tiertagger as tt;
    mods.push(crate::game::native_hooks::NativeMod {
        id: tt::ID.into(),
        title: format!("TierTagger {}", tt::VERSION),
        description: tt::DESCRIPTION.into(),
        keys: Vec::new(),
    });
}
