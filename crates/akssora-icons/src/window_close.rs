use crate::sink::IconSink;

// INFO: Minimal native outline icon representing the window Close control.
pub struct WindowCloseIcon;

impl WindowCloseIcon {
    pub const CHAR: char = '\u{00D7}';
    pub const GLYPH: &'static str = "\u{00D7}";
    pub const SPAN: f32 = 4.5;
    pub const DEFAULT_STROKE: f32 = 1.2;

    pub fn draw<S: IconSink>(sink: &mut S, cx: f32, cy: f32, stroke: f32, color: [f32; 4]) {
        let span = Self::SPAN;
        sink.push_line(cx - span, cy - span, cx + span, cy + span, stroke, color);
        sink.push_line(cx - span, cy + span, cx + span, cy - span, stroke, color);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sink::MockIconSink;

    #[test]
    fn test_window_close_icon_draw() {
        let mut sink = MockIconSink::default();
        WindowCloseIcon::draw(&mut sink, 100.0, 50.0, 1.2, [1.0, 1.0, 1.0, 1.0]);
        assert_eq!(sink.lines.len(), 2);
    }
}
