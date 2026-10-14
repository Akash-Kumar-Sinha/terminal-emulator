use crate::sink::IconSink;

// INFO: Minimal outline icon representing the file explorer toggle button.
pub struct ExplorerToggleIcon;

impl ExplorerToggleIcon {
    pub const WIDTH: f32 = 16.0;
    pub const HEIGHT: f32 = 13.0;
    pub const DEFAULT_STROKE: f32 = 1.0;

    pub fn draw<S: IconSink>(sink: &mut S, ix: f32, iy: f32, stroke: f32, stroke_col: [f32; 4]) {
        let icon_w = Self::WIDTH;
        let icon_h = Self::HEIGHT;
        let tab_w = 6.0;
        let tab_h = 2.0;

        // Folder tab (top-left)
        sink.push_quad(ix, iy, ix + tab_w, iy + stroke, stroke_col);
        sink.push_quad(ix, iy, ix + stroke, iy + tab_h, stroke_col);

        // Folder main top edge (from tab_w to icon_w)
        sink.push_quad(
            ix + tab_w - stroke,
            iy + tab_h,
            ix + icon_w,
            iy + tab_h + stroke,
            stroke_col,
        );

        // Folder bottom edge
        sink.push_quad(
            ix,
            iy + icon_h - stroke,
            ix + icon_w,
            iy + icon_h,
            stroke_col,
        );

        // Folder left edge
        sink.push_quad(ix, iy + tab_h, ix + stroke, iy + icon_h, stroke_col);

        // Folder right edge
        sink.push_quad(
            ix + icon_w - stroke,
            iy + tab_h,
            ix + icon_w,
            iy + icon_h,
            stroke_col,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sink::MockIconSink;

    #[test]
    fn test_explorer_toggle_draw() {
        let mut sink = MockIconSink::default();
        ExplorerToggleIcon::draw(&mut sink, 10.0, 10.0, 1.0, [1.0; 4]);
        assert_eq!(sink.quads.len(), 6);
    }
}
