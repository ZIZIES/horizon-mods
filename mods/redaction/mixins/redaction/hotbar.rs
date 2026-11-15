// Hud.extractItemHotbar: Blackbar. The game lays the hotbar out (asked for
// below, which comes back through here); its texture becomes a flat bar the
// width of the screen and its selection a highlight that slides.
fn redaction_hotbar(w: i32, h: i32, i: &HotbarInputs) -> std::ops::ControlFlow<Vec<GuiDraw>> {
    use crate::hud_layout::redaction as rd;
    use std::ops::ControlFlow;
    let Some(settings) = rd::settings().filter(|s| s.blackbar && !rd::inside()) else {
        return ControlFlow::Continue(());
    };
    rd::set_inside(true);
    let draws = hotbar(w, h, i);
    rd::set_inside(false);
    let mut out = Vec::with_capacity(draws.len() + 9);
    for draw in draws {
        match &draw {
            GuiDraw::Sprite { sprite, .. } if sprite == "hud/hotbar" => {
                out.push(fill(0, h - 22, w, h, settings.bar_color()));
            }
            GuiDraw::Sprite { sprite, .. } if sprite == "hud/hotbar_selection" => {
                let x = rd::slot_x(w, rd::slide(i.selected));
                out.push(fill(x, h - 22, x + 22, h, settings.highlight_color()));
            }
            // The off hand's item sits on the bar: no frame of its own.
            GuiDraw::Sprite { sprite, .. } if sprite.starts_with("hud/hotbar_offhand") => {}
            _ => out.push(draw),
        }
    }
    if settings.slot_numbers {
        for slot in 0..9 {
            out.push(GuiDraw::Text {
                text: Text::literal((slot + 1).to_string()),
                x: rd::slot_x(w, slot as f32) + 3,
                y: h - 20,
                color: 0xFFFF_FFFF,
                shadow: true,
            });
        }
    }
    ControlFlow::Break(out)
}
