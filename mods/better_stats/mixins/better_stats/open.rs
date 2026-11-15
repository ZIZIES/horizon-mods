// The Statistics screen opens: an empty search box.
fn better_stats_open() {
    crate::game::better_stats::with(|s| *s = Default::default());
}
