// PlayerTabOverlay.getNameForDisplay: the tier before the name the game
// gives (asked for below, which comes back through here).
fn tiertagger_tab(
    p: &crate::tab_list::TabPlayer,
    board: &Scoreboard,
) -> std::ops::ControlFlow<Text> {
    use crate::tab_list::{tiertagger as tt, tiertagger_net as net};
    use std::ops::ControlFlow;
    if tt::inside() || !tt::on() || !tt::settings().is_some_and(|s| s.player_list) {
        return ControlFlow::Continue(());
    }
    let Some((label, color)) = net::tier(p.uuid) else {
        return ControlFlow::Continue(());
    };
    tt::set_inside(true);
    let name = name_for_display(p, board);
    tt::set_inside(false);
    ControlFlow::Break(
        Text::colored(label, color)
            .append(Text::colored(tt::SEPARATOR, tt::GRAY))
            .append(name),
    )
}
