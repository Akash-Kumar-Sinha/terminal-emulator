use winit::event::MouseScrollDelta;

// INFO: Converts raw mouse scroll delta values into normalized terminal text line counts.
pub fn scroll_delta_to_lines(delta: MouseScrollDelta, cell_h: f32) -> f32 {
    let effective_h = if cell_h <= 0.0 { 16.0 } else { cell_h };
    match delta {
        MouseScrollDelta::LineDelta(_, y) => y,
        MouseScrollDelta::PixelDelta(p) => (p.y as f32) / effective_h,
    }
}

// INFO: Encodes mouse click/release into standard SGR 1006 ANSI escape sequence.
pub fn encode_sgr_mouse(col: usize, row: usize, button_code: u8, pressed: bool) -> Vec<u8> {
    let final_char = if pressed { 'M' } else { 'm' };
    format!(
        "\x1b[<{};{};{}{}",
        button_code,
        col + 1,
        row + 1,
        final_char
    )
    .into_bytes()
}
