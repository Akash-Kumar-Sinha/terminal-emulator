use crate::sink::IconSink;

// INFO: Minimal outline icon representing the collapsible Sidebar toggle button.
pub struct SidebarToggleIcon;

impl SidebarToggleIcon {
    pub const WIDTH: f32 = 18.0;
    pub const HEIGHT: f32 = 14.0;
    pub const DEFAULT_STROKE: f32 = 1.0;
    pub const DIVIDER_OFFSET: f32 = 5.5;

    pub fn draw<S: IconSink>(
        sink: &mut S,
        ix: f32,
        iy: f32,
        stroke: f32,
        is_open: bool,
        stroke_col: [f32; 4],
        panel_col: [f32; 4],
    ) {
        let icon_w = Self::WIDTH;
        let icon_h = Self::HEIGHT;

        // Outer rectangle frame
        sink.push_quad(ix, iy, ix + icon_w, iy + stroke, stroke_col);
        sink.push_quad(
            ix,
            iy + icon_h - stroke,
            ix + icon_w,
            iy + icon_h,
            stroke_col,
        );
        sink.push_quad(
            ix,
            iy + stroke,
            ix + stroke,
            iy + icon_h - stroke,
            stroke_col,
        );
        sink.push_quad(
            ix + icon_w - stroke,
            iy + stroke,
            ix + icon_w,
            iy + icon_h - stroke,
            stroke_col,
        );

        // Vertical divider separating narrow left sidebar from content area
        let div_x = ix + Self::DIVIDER_OFFSET;
        sink.push_quad(
            div_x,
            iy + stroke,
            div_x + stroke,
            iy + icon_h - stroke,
            stroke_col,
        );

        // Panel state indicator: filled left panel when sidebar is open
        if is_open {
            sink.push_quad(
                ix + stroke,
                iy + stroke,
                div_x,
                iy + icon_h - stroke,
                panel_col,
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sink::MockIconSink;

    #[test]
    fn test_sidebar_toggle_draw_closed_and_open() {
        let mut closed_sink = MockIconSink::default();
        SidebarToggleIcon::draw(&mut closed_sink, 10.0, 10.0, 1.0, false, [1.0; 4], [1.0; 4]);
        assert_eq!(closed_sink.quads.len(), 5);

        let mut open_sink = MockIconSink::default();
        SidebarToggleIcon::draw(&mut open_sink, 10.0, 10.0, 1.0, true, [1.0; 4], [1.0; 4]);
        assert_eq!(open_sink.quads.len(), 6);
    }
}
