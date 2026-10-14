pub mod body;
pub mod header;

use akssora_editor::Document;
use akssora_git::FileDiff;
use akssora_terminal_config::Theme;
use akssora_terminal_ui::{CodeReviewAction, CodeReviewLayout, CodeReviewModel};

use crate::font::FontManager;
use crate::glyph::GlyphRenderer;
use crate::quad::{push_quad, push_rounded_rect};
use crate::vertex::GlyphVertex;

pub use body::render_card_body;
pub use header::render_card_header;

// INFO: Renders an individual file diff card container with header actions, formatted diff hunks, or embedded live editor.
#[allow(clippy::too_many_arguments)]
pub fn render_file_diff_card(
    renderer: &mut GlyphRenderer,
    queue: &wgpu::Queue,
    fonts: &FontManager,
    ui_fonts: &FontManager,
    theme: &Theme,
    model: &CodeReviewModel,
    file: &FileDiff,
    file_idx: usize,
    card_doc: Option<&Document>,
    hovered_action: Option<&CodeReviewAction>,
    hovered_gutter: Option<(usize, Option<akssora_editor::GutterActionKind>)>,
    is_collapsed: bool,
    cx: f32,
    cy: f32,
    cw: f32,
    ch: f32,
    clip_top: f32,
    clip_h: f32,
    verts: &mut Vec<GlyphVertex>,
    solid_uv: [f32; 2],
    circle_uv: [[f32; 2]; 2],
    screen_w: f32,
    screen_h: f32,
) {
    let header_h = CodeReviewLayout::CARD_HEADER_HEIGHT;

    // Strict vertical clipping for the card against diff viewport bounds
    let card_top = cy.max(clip_top);
    let card_bot = (cy + ch).min(clip_top + clip_h);
    if card_bot <= card_top {
        return;
    }

    // Card Outer Background (strictly clamped to viewport)
    push_rounded_rect(
        verts,
        cx,
        card_top,
        cx + cw,
        card_bot,
        6.0,
        circle_uv,
        solid_uv,
        theme.code_review_accordion_body_background,
        screen_w,
        screen_h,
    );

    // Subtle card top border quad if top is visible
    if cy >= clip_top && cy < clip_top + clip_h {
        push_quad(
            verts,
            cx,
            cy,
            cx + cw,
            cy + 1.0,
            solid_uv,
            solid_uv,
            theme.code_review_card_border,
            screen_w,
            screen_h,
        );
    }

    let header_y = CodeReviewLayout::card_header_y(cy, ch, is_collapsed, clip_top);

    // Card Body: rendered first so sticky header floats on top in painter's algorithm
    if !is_collapsed {
        render_card_body(
            renderer,
            queue,
            fonts,
            ui_fonts,
            theme,
            model,
            file,
            file_idx,
            card_doc,
            hovered_action,
            hovered_gutter,
            cx,
            cy,
            cw,
            ch,
            header_y,
            header_h,
            clip_top,
            clip_h,
            verts,
            solid_uv,
            circle_uv,
            screen_w,
            screen_h,
        );
    }

    // Card Header Bar: rendered after body with fully opaque background so code moves beneath it
    render_card_header(
        renderer,
        queue,
        ui_fonts,
        theme,
        model,
        file,
        file_idx,
        hovered_action,
        is_collapsed,
        cx,
        header_y,
        cw,
        header_h,
        clip_top,
        clip_h,
        verts,
        solid_uv,
        circle_uv,
        screen_w,
        screen_h,
    );
}
