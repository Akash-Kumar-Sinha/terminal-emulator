use crate::sink::IconSink;

// INFO: Minimal native outline icon representing the window Maximize control.
pub struct WindowMaximizeIcon;

impl WindowMaximizeIcon {
    pub const SIZE: f32 = 11.0;
    pub const DEFAULT_STROKE: f32 = 1.2;

    pub fn draw<S: IconSink>(sink: &mut S, cx: f32, cy: f32, stroke: f32, color: [f32; 4]) {
        let half = Self::SIZE * 0.5;
        let left = cx - half;
        let right = cx + half;
        let top = cy - half;
        let bottom = cy + half;

        // Top
        sink.push_quad(left, top, right, top + stroke, color);
        // Bottom
        sink.push_quad(left, bottom - stroke, right, bottom, color);
        // Left
        sink.push_quad(left, top + stroke, left + stroke, bottom - stroke, color);
        // Right
        sink.push_quad(right - stroke, top + stroke, right, bottom - stroke, color);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sink::MockIconSink;

    #[test]
    fn test_window_maximize_icon_draw() {
        let mut sink = MockIconSink::default();
        WindowMaximizeIcon::draw(&mut sink, 100.0, 50.0, 1.2, [1.0, 1.0, 1.0, 1.0]);
        assert_eq!(sink.quads.len(), 4);
    }
}
