// The Mobs tab's rows: those the search box keeps.
fn better_stats_mobs(game: &crate::game::Game<'_>) -> std::ops::ControlFlow<Vec<String>> {
    use crate::game::better_stats as bs;
    let query = bs::with(|s| (!s.inside).then(|| s.text.clone())).unwrap_or_default();
    if query.trim().is_empty() || !game.native_on(bs::ID) {
        return std::ops::ControlFlow::Continue(());
    }
    bs::with(|s| s.inside = true);
    let mut rows = game.stat_mobs();
    bs::with(|s| s.inside = false);
    rows.retain(|mob| bs::matches(&query, mob, &game.mob_row(mob).0.plain()));
    std::ops::ControlFlow::Break(rows)
}
