// Hud.extractCrosshair: vanilla's sprite gives way to the drawn crosshair
// (layer.rs draws it). The sprite's place stays the first draw, as an empty
// fill, since the game counts on it; the attack indicator after it is kept.
fn crosscrosshair_sprite(
    w: i32,
    h: i32,
    attack: Option<f32>,
    full: bool,
) -> std::ops::ControlFlow<Vec<GuiDraw>> {
    use std::ops::ControlFlow;
    if crate::hud_layout::crosscrosshair::settings().is_none() {
        return ControlFlow::Continue(());
    }
    let mut draws: Vec<GuiDraw> = crosshair_draws(w, h, attack, full)
        .into_iter()
        .map(crosshair_blend)
        .collect();
    if let Some(sprite) = draws.first_mut() {
        *sprite = fill(0, 0, 0, 0, 0);
    }
    ControlFlow::Break(draws)
}
