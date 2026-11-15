// After the crosshair layer: the drawn crosshair, its middle cell where the
// middle of vanilla's 15x15 sprite is. The outline first, in black, then the
// pixels, which invert what is under them when the colour is Inverted.
fn crosscrosshair_layer(
    game: &crate::game::Game<'_>,
    layer: &str,
    batch: &mut horizon_client::hud::HudBatch,
) {
    use horizon_client::hud_layout::crosscrosshair as cc;
    if layer != "minecraft:crosshair" {
        return;
    }
    let Some(settings) = cc::settings() else {
        return;
    };
    let (cx, cy) = (
        (batch.width as i32 - 15) / 2 + cc::MID,
        (batch.height as i32 - 15) / 2 + cc::MID,
    );
    let rect = |[x0, y, x1]: [i32; 3]| [(cx + x0) as f32, (cy + y) as f32, (x1 - x0) as f32, 1.0];
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
}
