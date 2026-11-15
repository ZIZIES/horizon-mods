// EditBox.keyPressed: plain Left and Right leave a selection at its edge.
fn better_selection_key(
    field: &mut crate::ui::EditBox,
    key: winit::keyboard::KeyCode,
    ctrl: bool,
    shift: bool,
) -> std::ops::ControlFlow<bool> {
    use crate::ui::better_selection as bs;
    use winit::keyboard::KeyCode as K;
    let dir = match key {
        K::ArrowLeft => -1,
        K::ArrowRight => 1,
        _ => return std::ops::ControlFlow::Continue(()),
    };
    if ctrl || shift || field.multiline || !bs::on() {
        return std::ops::ControlFlow::Continue(());
    }
    match bs::collapse(field.cursor, field.highlight, dir) {
        Some(edge) => {
            field.set_cursor(edge, false);
            std::ops::ControlFlow::Break(true)
        }
        None => std::ops::ControlFlow::Continue(()),
    }
}
