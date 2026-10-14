use akssora_git::LineGitStatus;
use akssora_terminal_config::Theme;

use crate::font::FontManager;
use crate::glyph::GlyphRenderer;
use crate::quad::push_rounded_rect;
use crate::vertex::GlyphVertex;

// INFO: Renders an inline diff chip displaying removed text from git status.
#[allow(clippy::too_many_arguments)]
pub fn render_inline_diff_chip(
    renderer: &mut GlyphRenderer,
    queue: &wgpu::Queue,
    fonts: &FontManager,
    verts: &mut Vec<GlyphVertex>,
    theme: &Theme,
    git_status: &LineGitStatus,
    text_x: f32,
    rendered_len: usize,
    line_y: f32,
    cell_w: f32,
    cell_h: f32,
    src_x: f32,
    src_w: f32,
    v_sb_w: f32,
    solid_uv: [f32; 2],
    circle_uv: [[f32; 2]; 2],
    screen_w: f32,
    screen_h: f32,
) {
    let chip_info = if let Some(ref orig) = git_status.original_line {
        Some(format!("− {}", orig))
    } else if git_status.deleted_above && !git_status.deleted_lines.is_empty() {
        let first = &git_status.deleted_lines[0];
        if git_status.deleted_lines.len() == 1 {
            Some(format!("− {}", first))
        } else {
            Some(format!(
                "− {} (+{} more)",
                first,
                git_status.deleted_lines.len() - 1
            ))
        }
    } else {
        None
    };

    if let Some(chip_text) = chip_info {
        let chip_x = text_x + rendered_len as f32 * cell_w + 16.0;
        let max_chip_w = (src_x + src_w - v_sb_w - chip_x - 8.0).max(0.0);
        let chip_w = (chip_text.chars().count() as f32 * cell_w + 12.0).min(max_chip_w);
        if chip_w > 24.0 {
            push_rounded_rect(
                verts,
                chip_x,
                line_y + 1.0,
                chip_x + chip_w,
                line_y + cell_h - 1.0,
                3.0,
                circle_uv,
                solid_uv,
                theme.diff_chip_delete_background,
                screen_w,
                screen_h,
            );
            renderer.render_text(
                queue,
                fonts,
                verts,
                &chip_text,
                chip_x + 6.0,
                line_y,
                chip_w - 8.0,
                theme.diff_chip_delete_foreground,
                false,
                screen_w,
                screen_h,
            );
        }
    }
}
