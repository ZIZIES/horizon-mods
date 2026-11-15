// The frame's name tags are collected with the name tag hook only when a
// hook-API mod is loaded; TierTagger answers that hook, so it asks for the
// collection again with the hook on (which comes back through here).
fn tiertagger_hooks(
    game: &crate::game::Game<'_>,
    camera: glam::DVec3,
    alpha: f64,
    hooks: bool,
    frustum: Option<&horizon_client::gfx::Frustum>,
) -> std::ops::ControlFlow<Vec<NameTag>> {
    use horizon_client::tab_list::tiertagger as tt;
    if hooks || tt::inside() || !tt::on() {
        return std::ops::ControlFlow::Continue(());
    }
    tt::set_inside(true);
    let tags = game.collect_name_tags(camera, alpha, true, frustum);
    tt::set_inside(false);
    std::ops::ControlFlow::Break(tags)
}
