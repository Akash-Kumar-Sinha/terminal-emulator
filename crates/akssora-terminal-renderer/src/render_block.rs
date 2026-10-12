use akssora_terminal_config::Theme;
use akssora_terminal_ui::Block;

use crate::font::FontManager;
use crate::glyph::GlyphRenderer;
use crate::quad::{push_pill_thumb, push_quad};
use crate::vertex::GlyphVertex;

// INFO: Renders terminal buffer cells, bottom-up offsets, active cursor, borders, and pill scrollbar.
#[allow(clippy::too_many_arguments)]
pub fn render_terminal_blocks(
    renderer: &mut GlyphRenderer,
    queue: &wgpu::Queue,
    fonts: &FontManager,
    theme: &Theme,
    blocks: &[&Block],
    active_pane_id: Option<u64>,
    verts: &mut Vec<GlyphVertex>,
    solid_uv: [f32; 2],
    circle_uv: [[f32; 2]; 2],
    screen_w: f32,
    screen_h: f32,
) {
    let cell_w = fonts.cell_w;
    let cell_h = fonts.cell_h;
    let ascent = fonts.ascent_px;

    for block in blocks {
        let is_active_pane = if blocks.len() > 1 && active_pane_id.is_some() {
            block.pane_id == active_pane_id
        } else {
            true
        };

        let grid = block.shell.grid();
        let lines = grid.visible_lines();
        let cursor = if is_active_pane && block.shell.scroll_offset() == 0 {
            Some(block.shell.cursor())
        } else {
            None
        };

        let (y_offset, active_count) = if block.bottom_up && block.shell.scroll_offset() == 0 {
            let (crow, _) = block.shell.cursor();
            let active = (crow + 1).max(grid.content_rows()).min(lines.len());
            let offset = (lines.len().saturating_sub(active)) as f32 * cell_h;
            (offset, active)
        } else {
            (0.0, lines.len())
        };

        // Block background
        push_quad(
            verts,
            block.x,
            block.y,
            block.x + block.width as f32,
            block.y + block.height as f32,
            solid_uv,
            solid_uv,
            block.bg_color,
            screen_w,
            screen_h,
        );

        let margin = 2.0;
        let pad_x = margin;
        let pad_y = margin;
        let max_x = block.x + block.width as f32 - pad_x;
        let max_y = block.y + block.height as f32 - pad_y;
        let selection = block.shell.selection();

        // INFO: Render cell backgrounds and selection highlights, strictly clipping within block boundaries.
        for (row_idx, row) in lines.iter().take(active_count).enumerate() {
            let y0 = block.y + pad_y + y_offset + row_idx as f32 * cell_h;
            if y0 >= max_y {
                break;
            }
            for (col_idx, cell) in row.iter().enumerate() {
                let x0 = block.x + pad_x + col_idx as f32 * cell_w;
                if x0 >= max_x {
                    break;
                }
                let is_selected = selection
                    .map(|s| s.contains(row_idx, col_idx))
                    .unwrap_or(false);
                let bg_opt = if is_selected {
                    Some(theme.selection_background)
                } else {
                    theme.resolve_bg(cell.bg)
                };
                if let Some(bg) = bg_opt {
                    let x1 = (x0 + cell_w).min(max_x);
                    let y1 = (y0 + cell_h).min(max_y);
                    push_quad(
                        verts, x0, y0, x1, y1, solid_uv, solid_uv, bg, screen_w, screen_h,
                    );
                }
            }
        }

        // INFO: Render active terminal cursor within block bounds.
        if let Some((crow, ccol)) = cursor
            && crow < active_count
        {
            let x0 = block.x + pad_x + ccol as f32 * cell_w;
            let y0 = block.y + pad_y + y_offset + crow as f32 * cell_h;
            if x0 < max_x && y0 < max_y {
                let x1 = (x0 + cell_w).min(max_x);
                let y1 = (y0 + cell_h).min(max_y);
                push_quad(
                    verts,
                    x0,
                    y0,
                    x1,
                    y1,
                    solid_uv,
                    solid_uv,
                    theme.cursor,
                    screen_w,
                    screen_h,
                );
            }
        }

        // INFO: Render terminal glyphs, discarding any character that overflows block bounds.
        for (row_idx, row) in lines.iter().take(active_count).enumerate() {
            let row_y0 = block.y + pad_y + y_offset + row_idx as f32 * cell_h;
            if row_y0 >= max_y {
                break;
            }
            for (col_idx, cell) in row.iter().enumerate() {
                let cell_x0 = block.x + pad_x + col_idx as f32 * cell_w;
                if cell_x0 >= max_x {
                    break;
                }
                if cell.ch == ' ' || cell.ch == '\0' {
                    continue;
                }
                let Some(info) = renderer.glyph(queue, fonts, cell.ch, cell.bold, cell.italic)
                else {
                    continue;
                };
                let is_selected = selection
                    .map(|s| s.contains(row_idx, col_idx))
                    .unwrap_or(false);
                let fg = if cursor == Some((row_idx, col_idx)) {
                    theme.cursor_foreground
                } else if is_selected {
                    theme.foreground
                } else {
                    theme.resolve_fg(cell.fg, cell.bold)
                };
                let x0 = cell_x0 + info.bearing_left;
                let y0 = row_y0 + ascent + info.bearing_top;
                if x0 < max_x && y0 < max_y {
                    push_quad(
                        verts,
                        x0,
                        y0,
                        (x0 + info.width).min(max_x),
                        (y0 + info.height).min(max_y),
                        info.uv_min,
                        info.uv_max,
                        fg,
                        screen_w,
                        screen_h,
                    );
                }
            }
        }

        // Scrollbar
        let total_lines = grid.scrollback_len() + grid.rows();
        let visible_lines = grid.rows();
        let scroll_offset = grid.scroll_offset() as f32;
        let mut sb_ui = block.scrollbar.clone();
        sb_ui.update_geometry(block.x, block.y, block.width as f32, block.height as f32);
        if let Some((sx, sy, sw, sh)) = sb_ui.thumb_rect(total_lines, visible_lines, scroll_offset)
        {
            push_quad(
                verts,
                sx,
                sb_ui.y,
                sx + sw,
                sb_ui.y + sb_ui.height,
                solid_uv,
                solid_uv,
                sb_ui.track_color(theme),
                screen_w,
                screen_h,
            );

            let thumb_col = sb_ui.thumb_color(theme);
            push_pill_thumb(
                verts, sx, sy, sw, sh, circle_uv, solid_uv, thumb_col, screen_w, screen_h,
            );
        }

        // Borders
        if block.border_top {
            push_quad(
                verts,
                block.x,
                block.y,
                block.x + block.width as f32,
                block.y + block.border_thickness,
                solid_uv,
                solid_uv,
                block.border_color,
                screen_w,
                screen_h,
            );
        }
        if block.border_bottom {
            let y1 = block.y + block.height as f32;
            push_quad(
                verts,
                block.x,
                y1 - block.border_thickness,
                block.x + block.width as f32,
                y1,
                solid_uv,
                solid_uv,
                block.border_color,
                screen_w,
                screen_h,
            );
        }
    }

    // Render split divider lines between adjacent panes if multiple blocks are visible
    if blocks.len() > 1 {
        let divider_color = theme.border_subtle;
        for i in 0..blocks.len() {
            for j in (i + 1)..blocks.len() {
                let b1 = blocks[i];
                let b2 = blocks[j];
                // Vertical divider check (adjacent horizontally)
                if (b1.x + b1.width as f32 - b2.x).abs() <= 6.0 {
                    let div_x = b1.x + b1.width as f32;
                    let top_y = b1.y.max(b2.y);
                    let bot_y = (b1.y + b1.height as f32).min(b2.y + b2.height as f32);
                    if bot_y > top_y {
                        push_quad(
                            verts,
                            div_x - 1.0,
                            top_y,
                            div_x + 1.0,
                            bot_y,
                            solid_uv,
                            solid_uv,
                            divider_color,
                            screen_w,
                            screen_h,
                        );
                    }
                } else if (b2.x + b2.width as f32 - b1.x).abs() <= 6.0 {
                    let div_x = b2.x + b2.width as f32;
                    let top_y = b1.y.max(b2.y);
                    let bot_y = (b1.y + b1.height as f32).min(b2.y + b2.height as f32);
                    if bot_y > top_y {
                        push_quad(
                            verts,
                            div_x - 1.0,
                            top_y,
                            div_x + 1.0,
                            bot_y,
                            solid_uv,
                            solid_uv,
                            divider_color,
                            screen_w,
                            screen_h,
                        );
                    }
                }
                // Horizontal divider check (adjacent vertically)
                if (b1.y + b1.height as f32 - b2.y).abs() <= 6.0 {
                    let div_y = b1.y + b1.height as f32;
                    let left_x = b1.x.max(b2.x);
                    let right_x = (b1.x + b1.width as f32).min(b2.x + b2.width as f32);
                    if right_x > left_x {
                        push_quad(
                            verts,
                            left_x,
                            div_y - 1.0,
                            right_x,
                            div_y + 1.0,
                            solid_uv,
                            solid_uv,
                            divider_color,
                            screen_w,
                            screen_h,
                        );
                    }
                } else if (b2.y + b2.height as f32 - b1.y).abs() <= 6.0 {
                    let div_y = b2.y + b2.height as f32;
                    let left_x = b1.x.max(b2.x);
                    let right_x = (b1.x + b1.width as f32).min(b2.x + b2.width as f32);
                    if right_x > left_x {
                        push_quad(
                            verts,
                            left_x,
                            div_y - 1.0,
                            right_x,
                            div_y + 1.0,
                            solid_uv,
                            solid_uv,
                            divider_color,
                            screen_w,
                            screen_h,
                        );
                    }
                }
            }
        }
    }
}

// INFO: Renders drop indicator preview quad and outline when dragging a terminal pane.
pub fn render_drop_indicator(
    verts: &mut Vec<GlyphVertex>,
    rect: (f32, f32, f32, f32),
    theme: &Theme,
    solid_uv: [f32; 2],
    screen_w: f32,
    screen_h: f32,
) {
    let (x, y, w, h) = rect;
    if w <= 0.0 || h <= 0.0 {
        return;
    }
    // Translucent drop target fill
    let fill_color = [
        theme.selection_background[0],
        theme.selection_background[1],
        theme.selection_background[2],
        0.25,
    ];
    push_quad(
        verts,
        x,
        y,
        x + w,
        y + h,
        solid_uv,
        solid_uv,
        fill_color,
        screen_w,
        screen_h,
    );

    // 2.0px drop indicator border
    let border_color = [
        theme.foreground[0],
        theme.foreground[1],
        theme.foreground[2],
        0.7,
    ];
    let t = 2.0;
    // top
    push_quad(
        verts,
        x,
        y,
        x + w,
        y + t,
        solid_uv,
        solid_uv,
        border_color,
        screen_w,
        screen_h,
    );
    // bottom
    push_quad(
        verts,
        x,
        y + h - t,
        x + w,
        y + h,
        solid_uv,
        solid_uv,
        border_color,
        screen_w,
        screen_h,
    );
    // left
    push_quad(
        verts,
        x,
        y,
        x + t,
        y + h,
        solid_uv,
        solid_uv,
        border_color,
        screen_w,
        screen_h,
    );
    // right
    push_quad(
        verts,
        x + w - t,
        y,
        x + w,
        y + h,
        solid_uv,
        solid_uv,
        border_color,
        screen_w,
        screen_h,
    );
}
