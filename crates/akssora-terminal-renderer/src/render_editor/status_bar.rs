use akssora_editor::{Document, VimMode};
use akssora_terminal_config::Theme;

use crate::font::FontManager;
use crate::glyph::GlyphRenderer;
use crate::quad::push_quad;
use crate::vertex::GlyphVertex;

// INFO: Renders the editor bottom status bar with Vim mode, file title, and line/column metrics.
#[allow(clippy::too_many_arguments)]
pub fn render_editor_status_bar(
    renderer: &mut GlyphRenderer,
    queue: &wgpu::Queue,
    fonts: &FontManager,
    theme: &Theme,
    doc: &Document,
    ws_x: f32,
    ws_y: f32,
    ws_w: f32,
    ws_h: f32,
    status_bar_h: f32,
    verts: &mut Vec<GlyphVertex>,
    solid_uv: [f32; 2],
    screen_w: f32,
    screen_h: f32,
) {
    let cell_h = fonts.cell_h;
    let cell_w = fonts.cell_w;
    let bar_y = ws_y + ws_h - status_bar_h;

    push_quad(
        verts,
        ws_x,
        bar_y,
        ws_x + ws_w,
        ws_y + ws_h,
        solid_uv,
        solid_uv,
        theme.header_background,
        screen_w,
        screen_h,
    );
    push_quad(
        verts,
        ws_x,
        bar_y,
        ws_x + ws_w,
        bar_y + 1.0,
        solid_uv,
        solid_uv,
        theme.border_subtle,
        screen_w,
        screen_h,
    );

    // Left status info: Vim Mode / Title / Markdown mode
    let mode_str = if doc.vim_enabled {
        match doc.vim.mode {
            VimMode::Normal => "-- NORMAL --",
            VimMode::Insert => "-- INSERT --",
            VimMode::Visual => "-- VISUAL --",
            VimMode::VisualLine => "-- V-LINE --",
            VimMode::VisualBlock => "-- V-BLOCK --",
            VimMode::Replace => "-- REPLACE --",
            VimMode::Command => ":",
        }
    } else {
        "EDIT"
    };

    let title_display = doc.display_title();
    let md_status = if doc.is_markdown() {
        format!(" [{}]", doc.markdown_mode.label())
    } else {
        String::new()
    };

    let left_status = if let Some(msg) = &doc.vim.status_message {
        format!("{mode_str} | {title_display}{md_status} | {msg}")
    } else {
        format!("{mode_str} | {title_display}{md_status}")
    };

    renderer.render_text(
        queue,
        fonts,
        verts,
        &left_status,
        ws_x + 8.0,
        bar_y + (status_bar_h - cell_h) * 0.5,
        ws_w - 140.0,
        theme.text_primary,
        false,
        screen_w,
        screen_h,
    );

    // Right status info: Ln X, Col Y
    let (cursor_row, cursor_col) = doc.cursor;
    let right_status = format!("Ln {}, Col {}", cursor_row + 1, cursor_col + 1);
    let right_w = right_status.len() as f32 * cell_w;
    renderer.render_text(
        queue,
        fonts,
        verts,
        &right_status,
        ws_x + ws_w - right_w - 12.0,
        bar_y + (status_bar_h - cell_h) * 0.5,
        right_w + 10.0,
        theme.text_secondary,
        false,
        screen_w,
        screen_h,
    );
}
