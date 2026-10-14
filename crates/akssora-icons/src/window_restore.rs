use crate::sink::IconSink;

// INFO: Minimal native outline icon representing the window Restore control (overlapping double squares).
pub struct WindowRestoreIcon;

impl WindowRestoreIcon {
    pub const DEFAULT_STROKE: f32 = 1.2;

    pub fn draw<S: IconSink>(sink: &mut S, cx: f32, cy: f32, stroke: f32, color: [f32; 4]) {
        // Front square: 8x8 px at bottom-left
        let f_l = cx - 5.0;
        let f_r = cx + 3.0;
        let f_t = cy - 3.0;
        let f_b = cy + 5.0;

        // Front square outline
        sink.push_quad(f_l, f_t, f_r, f_t + stroke, color);
        sink.push_quad(f_l, f_b - stroke, f_r, f_b, color);
        sink.push_quad(f_l, f_t + stroke, f_l + stroke, f_b - stroke, color);
        sink.push_quad(f_r - stroke, f_t + stroke, f_r, f_b - stroke, color);

        // Back square: 8x8 px offset top-right
        let b_l = cx - 2.5;
        let b_r = cx + 5.5;
        let b_t = cy - 5.5;
        let b_b = cy + 2.5;

        // Back square visible edges
        sink.push_quad(b_l, b_t, b_r, b_t + stroke, color);
        sink.push_quad(b_r - stroke, b_t + stroke, b_r, b_b, color);
        if f_t > b_t + stroke {
            sink.push_quad(b_l, b_t + stroke, b_l + stroke, f_t, color);
        }
        if b_r - stroke > f_r {
            sink.push_quad(f_r, b_b - stroke, b_r - stroke, b_b, color);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sink::MockIconSink;

    #[test]
    fn test_window_restore_icon_draw() {
        let mut sink = MockIconSink::default();
        WindowRestoreIcon::draw(&mut sink, 100.0, 50.0, 1.2, [1.0, 1.0, 1.0, 1.0]);
        assert_eq!(sink.quads.len(), 8);
    }
}
