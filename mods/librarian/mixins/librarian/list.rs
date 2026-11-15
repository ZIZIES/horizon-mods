// Librarian's row in Options > Mods.
fn librarian_listed(mods: &mut Vec<crate::game::native_hooks::NativeMod>) {
    crate::game::librarian_glue::listed(mods);
}
