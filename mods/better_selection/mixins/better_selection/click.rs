// EditBox.onClick: a double click selects the word under the pointer.
fn better_selection_click(
    field: &mut crate::ui::EditBox,
    atlas: &crate::hud::HudAtlas,
    x: f32,
    double: bool,
) -> std::ops::ControlFlow<()> {
    use crate::ui::better_selection as bs;
    if !double || field.multiline || !bs::on() {
        return std::ops::ControlFlow::Continue(());
    }
    let pos = field.position_at(atlas, x);
    let (start, end) = bs::word_around(&field.value, pos);
    field.set_cursor(start, false);
    field.set_cursor(end, true);
    std::ops::ControlFlow::Break(())
}
