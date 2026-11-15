// Camera.calculateHudFov: the hand's own field of view.
fn redaction_hand_fov() -> std::ops::ControlFlow<f32> {
    match horizon_client::hud_layout::redaction::settings() {
        Some(s) if s.custom_hand_fov => std::ops::ControlFlow::Break(s.hand_fov as f32),
        _ => std::ops::ControlFlow::Continue(()),
    }
}
