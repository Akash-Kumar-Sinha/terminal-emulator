use winit::event::MouseScrollDelta;

// INFO: Converts raw mouse scroll delta values into normalized terminal text line counts.
pub fn scroll_delta_to_lines(delta: MouseScrollDelta, cell_h: f32) -> f32 {
    let effective_h = if cell_h <= 0.0 { 16.0 } else { cell_h };
    match delta {
        MouseScrollDelta::LineDelta(_, y) => y,
        MouseScrollDelta::PixelDelta(p) => (p.y as f32) / effective_h,
    }
}
