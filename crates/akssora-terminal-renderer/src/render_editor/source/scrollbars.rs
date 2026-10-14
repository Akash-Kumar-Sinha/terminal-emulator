use akssora_terminal_config::Theme;
use akssora_terminal_ui::Scrollbar;

use crate::quad::{push_pill_thumb, push_quad};
use crate::vertex::GlyphVertex;

// INFO: Renders pill-shaped vertical and horizontal scrollbars for the code editor pane.
#[allow(clippy::too_many_arguments)]
pub fn render_editor_scrollbars(
    verts: &mut Vec<GlyphVertex>,
    editor_scrollbars: Option<(&Scrollbar, &Scrollbar)>,
    v_overflow: bool,
    h_overflow: bool,
    total_lines: usize,
    visible_rows: usize,
    visible_cols: usize,
    scroll_row: usize,
    scroll_col: usize,
    max_line_len: usize,
    src_x: f32,
    src_y: f32,
    src_w: f32,
    editor_content_h: f32,
    gutter_w: f32,
    v_sb_w: f32,
    h_sb_h: f32,
    theme: &Theme,
    solid_uv: [f32; 2],
    circle_uv: [[f32; 2]; 2],
    screen_w: f32,
    screen_h: f32,
) {
    // Vertical Scrollbar (Right side of editor pane)
    if let Some((v_bar, _)) = editor_scrollbars
        && v_overflow
        && visible_rows > 0
    {
        let mut v_sb = (*v_bar).clone();
        v_sb.update_geometry(src_x, src_y, src_w, editor_content_h);
        if let Some((sx, sy, sw, sh)) =
            v_sb.thumb_rect_top_down(total_lines, visible_rows, scroll_row)
        {
            let track_col = v_sb.track_color(theme);
            if track_col[3] > 0.0 {
                push_quad(
                    verts,
                    v_sb.x,
                    v_sb.y,
                    v_sb.x + v_sb.width,
                    v_sb.y + v_sb.height,
                    solid_uv,
                    solid_uv,
                    track_col,
                    screen_w,
                    screen_h,
                );
            }
            let thumb_col = v_sb.thumb_color(theme);
            push_pill_thumb(
                verts, sx, sy, sw, sh, circle_uv, solid_uv, thumb_col, screen_w, screen_h,
            );
        }
    }

    // Horizontal Scrollbar (Bottom of editor pane)
    if let Some((_, h_bar)) = editor_scrollbars
        && h_overflow
        && visible_cols > 0
    {
        let mut h_sb = (*h_bar).clone();
        let h_y = src_y + editor_content_h;
        let h_x = src_x + gutter_w;
        let h_w = (src_w - gutter_w - v_sb_w).max(0.0);
        h_sb.update_geometry_horizontal(h_x, h_y, h_w, h_sb_h);
        if let Some((sx, sy, sw, sh)) =
            h_sb.thumb_rect_top_down(max_line_len, visible_cols, scroll_col)
        {
            let track_col = h_sb.track_color(theme);
            if track_col[3] > 0.0 {
                push_quad(
                    verts,
                    h_sb.x,
                    h_sb.y,
                    h_sb.x + h_sb.width,
                    h_sb.y + h_sb.height,
                    solid_uv,
                    solid_uv,
                    track_col,
                    screen_w,
                    screen_h,
                );
            }
            let thumb_col = h_sb.thumb_color(theme);
            push_pill_thumb(
                verts, sx, sy, sw, sh, circle_uv, solid_uv, thumb_col, screen_w, screen_h,
            );
        }
    }
}
