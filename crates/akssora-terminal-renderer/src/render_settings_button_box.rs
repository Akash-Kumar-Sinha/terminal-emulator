use akssora_terminal_config::Theme;

use crate::font::FontManager;
use crate::glyph::GlyphRenderer;
use crate::quad::push_quad;
use crate::vertex::GlyphVertex;

// INFO: Draws a rectangular button with 1px border and optional glyph icon for settings controls.
#[allow(clippy::too_many_arguments)]
pub(crate) fn render_button_box(
    renderer: &mut GlyphRenderer,
    queue: &wgpu::Queue,
    fonts: &FontManager,
    theme: &Theme,
    verts: &mut Vec<GlyphVertex>,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    is_hovered: bool,
    ch: Option<char>,
    label: Option<&str>,
    solid_uv: [f32; 2],
    screen_w: f32,
    screen_h: f32,
) {
    let bg = if is_hovered {
        theme.button_hover_background
    } else {
        theme.session_background
    };
    crate::quad::push_rounded_rect(
        verts,
        x,
        y,
        x + w,
        y + h,
        4.0,
        [[solid_uv[0], solid_uv[1]]; 2],
        solid_uv,
        bg,
        screen_w,
        screen_h,
    );

    let glyph_info = ch.and_then(|c| {
        renderer.glyph(queue, fonts, c, false, false).or_else(|| {
            if c == '\u{2212}' {
                renderer.glyph(queue, fonts, '-', false, false)
            } else {
                None
            }
        })
    });
    if let Some(info) = glyph_info {
        let gx = x + (w - info.width) / 2.0;
        let gy = y + (h - info.height) / 2.0;
        let col = if is_hovered {
            theme.accent_hover
        } else {
            theme.text_primary
        };
        push_quad(
            verts,
            gx,
            gy,
            gx + info.width,
            gy + info.height,
            info.uv_min,
            info.uv_max,
            col,
            screen_w,
            screen_h,
        );
    }

    if let Some(text) = label {
        let text_y = y + (h - fonts.cell_h) / 2.0;
        let col = if is_hovered {
            theme.accent_hover
        } else {
            theme.text_secondary
        };
        renderer.render_text(
            queue,
            fonts,
            verts,
            text,
            x + 12.0,
            text_y,
            w - 16.0,
            col,
            false,
            screen_w,
            screen_h,
        );
    }
}
