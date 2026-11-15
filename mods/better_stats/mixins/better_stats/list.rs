// The mod's row in Options > Mods.
fn better_stats_listed(mods: &mut Vec<crate::game::native_hooks::NativeMod>) {
    use crate::game::better_stats as bs;
    mods.push(crate::game::native_hooks::NativeMod {
        id: bs::ID.into(),
        title: format!("Statistics Search {}", bs::VERSION),
        description: bs::DESCRIPTION.into(),
        keys: Vec::new(),
    });
}
