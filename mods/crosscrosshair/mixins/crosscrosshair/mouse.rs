// A press on the grid: the left button draws, the right one erases, and the
// drag that follows does the same (draw.rs).
fn crosscrosshair_mouse(
    game: &crate::game::Game<'_>,
    screen: &crate::game::native_hooks::NativeScreenView<'_>,
    button: i32,
    canceled: &mut bool,
) {
    use horizon_client::hud_layout::crosscrosshair as cc;
    use horizon_client::options_ui::Page;
    if screen.id != "options" || *canceled || !(0..=1).contains(&button) || cc::settings().is_none() {
        return;
    }
    let mine = matches!(game.ui.options.page(), Page::NativeMod(i)
        if game.settings.mods.installed.get(i).is_some_and(|m| m.id == cc::ID));
    let (width, height) = game.gui_size();
    let Some((x, y)) = cc::grid([width, height])
        .filter(|_| mine)
        .and_then(|grid| grid.cell_at(screen.mouse))
    else {
        return;
    };
    cc::start_paint(button == 0);
    if let Some(pixels) = cc::paint(x, y) {
        game.native_set_setting(cc::ID, "pixels", &pixels);
    }
    *canceled = true;
}
