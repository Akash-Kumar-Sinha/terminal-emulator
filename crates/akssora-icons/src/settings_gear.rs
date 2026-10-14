use crate::sink::IconSink;

// INFO: Crisp 16x16 outline gear icon with central hollow hub and 8 teeth.
pub struct SettingsGearIcon;

impl SettingsGearIcon {
    pub const CHAR: char = '\u{2699}';
    pub const GLYPH: &'static str = "\u{2699}";
    pub const SIZE: f32 = 16.0;

    pub fn draw<S: IconSink>(sink: &mut S, cx: f32, cy: f32, stroke_col: [f32; 4]) {
        // 1. Central circular hub ring (r = 2.5 px, hollow center)
        sink.push_quad(cx - 1.5, cy - 2.5, cx + 1.5, cy - 1.5, stroke_col);
        sink.push_quad(cx - 1.5, cy + 1.5, cx + 1.5, cy + 2.5, stroke_col);
        sink.push_quad(cx - 2.5, cy - 1.5, cx - 1.5, cy + 1.5, stroke_col);
        sink.push_quad(cx + 1.5, cy - 1.5, cx + 2.5, cy + 1.5, stroke_col);

        // 2. Cardinal gear teeth (outer radius 8.0 px, width 3.0 px, stroke 1.0 px)
        // Top tooth
        sink.push_quad(cx - 1.5, cy - 8.0, cx + 1.5, cy - 7.0, stroke_col);
        sink.push_quad(cx - 1.5, cy - 7.0, cx - 0.5, cy - 5.5, stroke_col);
        sink.push_quad(cx + 0.5, cy - 7.0, cx + 1.5, cy - 5.5, stroke_col);

        // Bottom tooth
        sink.push_quad(cx - 1.5, cy + 7.0, cx + 1.5, cy + 8.0, stroke_col);
        sink.push_quad(cx - 1.5, cy + 5.5, cx - 0.5, cy + 7.0, stroke_col);
        sink.push_quad(cx + 0.5, cy + 5.5, cx + 1.5, cy + 7.0, stroke_col);

        // Left tooth
        sink.push_quad(cx - 8.0, cy - 1.5, cx - 7.0, cy + 1.5, stroke_col);
        sink.push_quad(cx - 7.0, cy - 1.5, cx - 5.5, cy - 0.5, stroke_col);
        sink.push_quad(cx - 7.0, cy + 0.5, cx - 5.5, cy + 1.5, stroke_col);

        // Right tooth
        sink.push_quad(cx + 7.0, cy - 1.5, cx + 8.0, cy + 1.5, stroke_col);
        sink.push_quad(cx + 5.5, cy - 1.5, cx + 7.0, cy - 0.5, stroke_col);
        sink.push_quad(cx + 5.5, cy + 0.5, cx + 7.0, cy + 1.5, stroke_col);

        // 3. Diagonal gear teeth caps (at 45-degree angles)
        // Top-right
        sink.push_quad(cx + 4.5, cy - 6.5, cx + 6.5, cy - 4.5, stroke_col);
        sink.push_quad(cx + 3.0, cy - 5.0, cx + 4.5, cy - 4.0, stroke_col);
        sink.push_quad(cx + 4.0, cy - 4.5, cx + 5.0, cy - 3.0, stroke_col);

        // Top-left
        sink.push_quad(cx - 6.5, cy - 6.5, cx - 4.5, cy - 4.5, stroke_col);
        sink.push_quad(cx - 4.5, cy - 5.0, cx - 3.0, cy - 4.0, stroke_col);
        sink.push_quad(cx - 5.0, cy - 4.5, cx - 4.0, cy - 3.0, stroke_col);

        // Bottom-right
        sink.push_quad(cx + 4.5, cy + 4.5, cx + 6.5, cy + 6.5, stroke_col);
        sink.push_quad(cx + 3.0, cy + 4.0, cx + 4.5, cy + 5.0, stroke_col);
        sink.push_quad(cx + 4.0, cy + 3.0, cx + 5.0, cy + 4.5, stroke_col);

        // Bottom-left
        sink.push_quad(cx - 6.5, cy + 4.5, cx - 4.5, cy + 6.5, stroke_col);
        sink.push_quad(cx - 4.5, cy + 4.0, cx - 3.0, cy + 5.0, stroke_col);
        sink.push_quad(cx - 5.0, cy + 3.0, cx - 4.0, cy + 4.5, stroke_col);

        // 4. Outer connecting rim segments between teeth
        sink.push_quad(cx + 1.5, cy - 5.5, cx + 3.5, cy - 4.5, stroke_col);
        sink.push_quad(cx - 3.5, cy - 5.5, cx - 1.5, cy - 4.5, stroke_col);
        sink.push_quad(cx + 1.5, cy + 4.5, cx + 3.5, cy + 5.5, stroke_col);
        sink.push_quad(cx - 3.5, cy + 4.5, cx - 1.5, cy + 5.5, stroke_col);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sink::MockIconSink;

    #[test]
    fn test_settings_gear_draw() {
        let mut sink = MockIconSink::default();
        SettingsGearIcon::draw(&mut sink, 100.0, 100.0, [1.0; 4]);
        assert_eq!(sink.quads.len(), 32);
        assert_eq!(SettingsGearIcon::CHAR, '\u{2699}');
    }
}
