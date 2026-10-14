use akssora_terminal_config::Theme;

use crate::quad::push_quad;
use crate::vertex::GlyphVertex;

// INFO: Renders the editor cursor (block or line) within visible boundaries.
#[allow(clippy::too_many_arguments)]
pub fn render_editor_cursor(
    verts: &mut Vec<GlyphVertex>,
    rendered_vis_row: usize,
    rendered_vis_col: usize,
    scroll_row: usize,
    visible_rows: usize,
    is_block_cursor: bool,
    text_x: f32,
    src_y: f32,
    max_text_w: f32,
    cell_w: f32,
    cell_h: f32,
    line_h: f32,
    theme: &Theme,
    solid_uv: [f32; 2],
    screen_w: f32,
    screen_h: f32,
) {
    if rendered_vis_row < scroll_row || rendered_vis_row >= scroll_row + visible_rows {
        return;
    }

    let line_y = src_y + (rendered_vis_row - scroll_row) as f32 * line_h;
    let cy = line_y + (line_h - cell_h) * 0.5;
    let cx = text_x + rendered_vis_col as f32 * cell_w;
    if cx + 2.0 <= text_x + max_text_w {
        let cur_w = if is_block_cursor {
            (cell_w - 1.5).max(2.0)
        } else {
            1.5
        };
        let cur_pad_y = (cell_h * 0.1).clamp(1.0, 2.5);
        push_quad(
            verts,
            cx,
            cy + cur_pad_y,
            cx + cur_w,
            cy + cell_h - cur_pad_y,
            solid_uv,
            solid_uv,
            theme.cursor,
            screen_w,
            screen_h,
        );
    }
}
