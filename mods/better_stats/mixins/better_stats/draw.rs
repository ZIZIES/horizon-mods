// After the Statistics screen is drawn: the search box in its footer.
fn better_stats_draw(
    game: &crate::game::Game<'_>,
    batch: &mut horizon_client::hud::HudBatch,
    screen: &crate::game::native_hooks::NativeScreenView<'_>,
) {
    use crate::game::better_stats as bs;
    if screen.id != "stats" || !game.native_on(bs::ID) {
        return;
    }
    let (width, height) = game.gui_size();
    let [x, y, w, h] = bs::box_rect([width, height]);
    let (text, focused) = bs::with(|s| (s.text.clone(), s.focused));
    let border = if focused { 0xFFFF_FFFF } else { 0xFFA0_A0A0 };
    game.native_fill(batch, [x - 1.0, y - 1.0, w + 2.0, h + 2.0], border);
    game.native_fill(batch, [x, y, w, h], 0xFF00_0000);
    // The end of a text too long for the box.
    let mut shown = text.as_str();
    while game.native_text_width(shown) > w - 10.0 {
        let Some((at, _)) = shown.char_indices().nth(1) else {
            break;
        };
        shown = &shown[at..];
    }
    if text.is_empty() && !focused {
        game.native_text(batch, "Search...", [x + 4.0, y + 4.0], 0x70_7070, false);
    } else {
        game.native_text(batch, shown, [x + 4.0, y + 4.0], 0xE0_E0E0, false);
    }
    if focused && horizon_client::hud_overlays::now_ms() / 300 % 2 == 0 {
        let at = x + 4.0 + game.native_text_width(shown);
        game.native_fill(batch, [at, y + 3.0, 1.0, 10.0], 0xFFD0_D0D0);
    }
}
