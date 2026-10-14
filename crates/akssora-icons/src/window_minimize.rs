use crate::sink::IconSink;

// INFO: Minimal native outline icon representing the window Minimize control.
pub struct WindowMinimizeIcon;

impl WindowMinimizeIcon {
    pub const SPAN: f32 = 11.0;
    pub const DEFAULT_STROKE: f32 = 1.2;

    pub fn draw<S: IconSink>(sink: &mut S, cx: f32, cy: f32, stroke: f32, color: [f32; 4]) {
        let half_span = Self::SPAN * 0.5;
        let half_stroke = stroke * 0.5;
        sink.push_quad(
            cx - half_span,
            cy - half_stroke,
            cx + half_span,
            cy + half_stroke,
            color,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sink::MockIconSink;

    #[test]
    fn test_window_minimize_icon_draw() {
        let mut sink = MockIconSink::default();
        WindowMinimizeIcon::draw(&mut sink, 100.0, 50.0, 1.2, [1.0, 1.0, 1.0, 1.0]);
        assert_eq!(sink.quads.len(), 1);
        let ([x0, _y0, x1, _y1], col) = sink.quads[0];
        assert_eq!(x1 - x0, 11.0);
        assert_eq!(col, [1.0, 1.0, 1.0, 1.0]);
    }
}
