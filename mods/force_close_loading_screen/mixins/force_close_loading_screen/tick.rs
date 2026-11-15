// LevelLoadTracker.tickClientLoad: ready now, whatever it was waiting for
// (the server's first chunks, or the player's own section).
fn force_close_tick(
    tracker: &mut crate::level_loading::LevelLoadTracker,
    now: u64,
) -> std::ops::ControlFlow<()> {
    if !crate::level_loading::force_close_loading_screen::on() {
        return std::ops::ControlFlow::Continue(());
    }
    if matches!(
        tracker.state,
        Some(ClientState::WaitingForServer { .. } | ClientState::WaitingForPlayerChunk { .. })
    ) {
        tracker.state = Some(ClientState::Ready { ready_at: now });
    }
    std::ops::ControlFlow::Break(())
}
