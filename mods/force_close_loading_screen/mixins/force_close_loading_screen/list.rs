// The mod's row in Options > Mods.
fn force_close_listed(mods: &mut Vec<crate::game::native_hooks::NativeMod>) {
    use horizon_client::level_loading::force_close_loading_screen as force_close;
    mods.push(crate::game::native_hooks::NativeMod {
        id: force_close::ID.into(),
        title: format!("Force Close Loading Screen {}", force_close::VERSION),
        description: force_close::DESCRIPTION.into(),
        keys: Vec::new(),
    });
}
