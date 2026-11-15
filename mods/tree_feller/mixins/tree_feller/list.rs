// The mod's row in Options > Mods.
fn tree_feller_listed(mods: &mut Vec<crate::game::native_hooks::NativeMod>) {
    use crate::game::native_player_break::tree_feller as tf;
    mods.push(crate::game::native_hooks::NativeMod {
        id: tf::ID.into(),
        title: format!("Tree Feller {}", tf::VERSION),
        description: tf::DESCRIPTION.into(),
        keys: Vec::new(),
    });
}
