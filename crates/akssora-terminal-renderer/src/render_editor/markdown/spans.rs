use akssora_editor::InlineSpan;
use akssora_terminal_config::Theme;

use crate::font::FontManager;
use crate::glyph::GlyphRenderer;
use crate::quad::{push_quad, push_rounded_rect};
use crate::vertex::GlyphVertex;

// INFO: Helper to convert list of InlineSpan to plain text string.
pub fn spans_to_plain_text(spans: &[InlineSpan]) -> String {
    let mut out = String::new();
    for span in spans {
        match span {
            InlineSpan::Text(t)
            | InlineSpan::Bold(t)
            | InlineSpan::Italic(t)
            | InlineSpan::BoldItalic(t)
            | InlineSpan::Code(t)
            | InlineSpan::Strikethrough(t) => out.push_str(t),
            InlineSpan::Link { text, .. } => out.push_str(text),
            InlineSpan::Image { alt, .. } => {
                out.push_str("[Image: ");
                out.push_str(alt);
                out.push(']');
            }
        }
    }
    out
}

// INFO: Splits inline spans across lines based on width wrapping.
pub fn wrap_spans_to_lines(spans: &[InlineSpan], max_w: f32, cell_w: f32) -> Vec<Vec<InlineSpan>> {
    let max_chars = (max_w / cell_w.max(1.0)).floor() as usize;
    if max_chars == 0 {
        return vec![spans.to_vec()];
    }

    let mut lines = Vec::new();
    let mut cur_line = Vec::new();
    let mut cur_len = 0;

    for span in spans {
        let span_len = match span {
            InlineSpan::Text(t)
            | InlineSpan::Bold(t)
            | InlineSpan::Italic(t)
            | InlineSpan::BoldItalic(t)
            | InlineSpan::Code(t)
            | InlineSpan::Strikethrough(t) => t.len(),
            InlineSpan::Link { text, .. } => text.len(),
            InlineSpan::Image { alt, .. } => alt.len() + 9,
        };

        if cur_len + span_len > max_chars && cur_len > 0 {
            lines.push(std::mem::take(&mut cur_line));
            cur_len = 0;
        }

        cur_line.push(span.clone());
        cur_len += span_len;
    }

    if !cur_line.is_empty() {
        lines.push(cur_line);
    }

    if lines.is_empty() {
        vec![Vec::new()]
    } else {
        lines
    }
}

// INFO: Renders formatted inline spans across horizontal line.
#[allow(clippy::too_many_arguments)]
pub fn render_inline_spans(
    renderer: &mut GlyphRenderer,
    queue: &wgpu::Queue,
    fonts: &FontManager,
    verts: &mut Vec<GlyphVertex>,
    spans: &[InlineSpan],
    start_x: f32,
    y: f32,
    max_w: f32,
    theme: &Theme,
    circle_uv: [[f32; 2]; 2],
    solid_uv: [f32; 2],
    screen_w: f32,
    screen_h: f32,
) {
    let mut cur_x = start_x;
    let cell_w = fonts.cell_w;
    let cell_h = fonts.cell_h;

    for span in spans {
        match span {
            InlineSpan::Text(t) => {
                let w = t.len() as f32 * cell_w;
                renderer.render_text(
                    queue,
                    fonts,
                    verts,
                    t,
                    cur_x,
                    y,
                    (start_x + max_w - cur_x).max(0.0),
                    theme.foreground,
                    false,
                    screen_w,
                    screen_h,
                );
                cur_x += w;
            }
            InlineSpan::Bold(t) => {
                let w = t.len() as f32 * cell_w;
                renderer.render_text(
                    queue,
                    fonts,
                    verts,
                    t,
                    cur_x,
                    y,
                    (start_x + max_w - cur_x).max(0.0),
                    theme.text_primary,
                    true,
                    screen_w,
                    screen_h,
                );
                cur_x += w;
            }
            InlineSpan::Italic(t) => {
                let w = t.len() as f32 * cell_w;
                renderer.render_text(
                    queue,
                    fonts,
                    verts,
                    t,
                    cur_x,
                    y,
                    (start_x + max_w - cur_x).max(0.0),
                    theme.text_secondary,
                    false,
                    screen_w,
                    screen_h,
                );
                cur_x += w;
            }
            InlineSpan::BoldItalic(t) => {
                let w = t.len() as f32 * cell_w;
                renderer.render_text(
                    queue,
                    fonts,
                    verts,
                    t,
                    cur_x,
                    y,
                    (start_x + max_w - cur_x).max(0.0),
                    theme.text_primary,
                    true,
                    screen_w,
                    screen_h,
                );
                cur_x += w;
            }
            InlineSpan::Code(c) => {
                let code_w = c.len() as f32 * cell_w + 8.0;
                // Code pill background
                push_rounded_rect(
                    verts,
                    cur_x,
                    y - 1.0,
                    cur_x + code_w,
                    y + cell_h + 1.0,
                    3.0,
                    circle_uv,
                    solid_uv,
                    theme.sidebar_background,
                    screen_w,
                    screen_h,
                );
                renderer.render_text(
                    queue,
                    fonts,
                    verts,
                    c,
                    cur_x + 4.0,
                    y,
                    (start_x + max_w - cur_x).max(0.0),
                    theme.accent,
                    false,
                    screen_w,
                    screen_h,
                );
                cur_x += code_w + 4.0;
            }
            InlineSpan::Link {
                text,
                url: _,
                safe: _,
            } => {
                let w = text.len() as f32 * cell_w;
                renderer.render_text(
                    queue,
                    fonts,
                    verts,
                    text,
                    cur_x,
                    y,
                    (start_x + max_w - cur_x).max(0.0),
                    theme.accent,
                    false,
                    screen_w,
                    screen_h,
                );
                // Underline
                push_quad(
                    verts,
                    cur_x,
                    y + cell_h - 1.0,
                    cur_x + w,
                    y + cell_h,
                    solid_uv,
                    solid_uv,
                    theme.accent,
                    screen_w,
                    screen_h,
                );
                cur_x += w;
            }
            InlineSpan::Image {
                alt,
                url: _,
                resolved_path: _,
                safe: _,
            } => {
                let img_label = format!("[Image: {alt}]");
                let w = img_label.len() as f32 * cell_w;
                renderer.render_text(
                    queue,
                    fonts,
                    verts,
                    &img_label,
                    cur_x,
                    y,
                    (start_x + max_w - cur_x).max(0.0),
                    theme.text_muted,
                    false,
                    screen_w,
                    screen_h,
                );
                cur_x += w;
            }
            InlineSpan::Strikethrough(t) => {
                let w = t.len() as f32 * cell_w;
                renderer.render_text(
                    queue,
                    fonts,
                    verts,
                    t,
                    cur_x,
                    y,
                    (start_x + max_w - cur_x).max(0.0),
                    theme.text_muted,
                    false,
                    screen_w,
                    screen_h,
                );
                // Strikethrough line
                push_quad(
                    verts,
                    cur_x,
                    y + cell_h * 0.5,
                    cur_x + w,
                    y + cell_h * 0.5 + 1.0,
                    solid_uv,
                    solid_uv,
                    theme.text_muted,
                    screen_w,
                    screen_h,
                );
                cur_x += w;
            }
        }
    }
}
