// LevelLoadTracker.isLevelReady: no closing delay once it is ready.
fn force_close_ready(
    tracker: &crate::level_loading::LevelLoadTracker,
) -> std::ops::ControlFlow<bool> {
    if crate::level_loading::force_close_loading_screen::on()
        && matches!(tracker.state, Some(ClientState::Ready { .. }))
    {
        return std::ops::ControlFlow::Break(true);
    }
    std::ops::ControlFlow::Continue(())
}
