// The mod's settings page: the grid the crosshair is drawn on, under the
// buttons. A held button paints the cells the pointer passes over.
fn crosscrosshair_draw(
    game: &crate::game::Game<'_>,
    batch: &mut horizon_client::hud::HudBatch,
    screen: &crate::game::native_hooks::NativeScreenView<'_>,
) {
    use horizon_client::hud_layout::crosscrosshair as cc;
    use horizon_client::options_ui::Page;
    let mine = screen.id == "options"
        && matches!(game.ui.options.page(), Page::NativeMod(i)
            if game.settings.mods.installed.get(i).is_some_and(|m| m.id == cc::ID));
    let (width, height) = game.gui_size();
    let Some(grid) = cc::grid([width, height]).filter(|_| mine) else {
        cc::stop_paint();
        return;
    };
    let over = grid.cell_at(screen.mouse);
    if let Some((x, y)) = over
        && let Some(pixels) = cc::paint(x, y)
    {
        game.native_set_setting(cc::ID, "pixels", &pixels);
    }
    let Some(settings) = cc::settings() else {
        return;
    };
    let side = grid.side();
    game.native_fill(batch, [grid.x - 1.0, grid.y - 1.0, side + 2.0, side + 2.0], 0xFF00_0000);
    for y in 0..cc::SIDE {
        for x in 0..cc::SIDE {
            let colour = if settings.shape.get(x, y) {
                settings.argb()
            } else if (x, y) == (cc::MID, cc::MID) {
                // The cell on the middle of the screen.
                0xFF5A_3A3A
            } else if (x + y) % 2 == 0 {
                0xFF3A_3A3A
            } else {
                0xFF30_3030
            };
            game.native_fill(batch, grid.rect(x, y), colour);
        }
    }
    if let Some((x, y)) = over {
        game.native_fill(batch, grid.rect(x, y), 0x60FF_FFFF);
    }
    // Beside the grid: the crosshair at its real size, and what the buttons do.
    let (px, py) = (grid.x - 31.0, grid.y);
    game.native_fill(batch, [px - 1.0, py - 1.0, 23.0, 23.0], 0xFF00_0000);
    game.native_fill(batch, [px, py, 21.0, 21.0], 0xFF6A_8A5A);
    let rect = |[x0, y, x1]: [i32; 3]| [px + 10.0 + x0 as f32, py + 10.0 + y as f32, (x1 - x0) as f32, 1.0];
    if settings.outline {
        for run in settings.shape.rim() {
            game.native_fill(batch, rect(run), 0xFF00_0000);
        }
    }
    let start = batch.vertices.len();
    for run in settings.shape.runs() {
        game.native_fill(batch, rect(run), settings.argb());
    }
    if settings.inverted() {
        batch.invert_since(start);
    }
    let tx = grid.x + side + 8.0;
    game.native_text(batch, "Left: draw", [tx, grid.y], 0xA0_A0A0, true);
    game.native_text(batch, "Right: erase", [tx, grid.y + 11.0], 0xA0_A0A0, true);
}
