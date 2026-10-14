use akssora_terminal_config::Theme;
use akssora_terminal_ui::{CodeReviewLayout, CodeReviewModel};

use crate::font::FontManager;
use crate::glyph::GlyphRenderer;
use crate::quad::push_rounded_rect;
use crate::vertex::GlyphVertex;

// INFO: Renders floating tooltip-style popup above hovered changed line number with Add and Remove buttons.
#[allow(clippy::too_many_arguments)]
pub fn render_line_action_popup(
    renderer: &mut GlyphRenderer,
    queue: &wgpu::Queue,
    ui_fonts: &FontManager,
    theme: &Theme,
    model: &CodeReviewModel,
    file_idx: usize,
    row: usize,
    is_deletion: bool,
    action: Option<akssora_editor::GutterActionKind>,
    ws_x: f32,
    ws_y: f32,
    ws_w: f32,
    ws_h: f32,
    verts: &mut Vec<GlyphVertex>,
    solid_uv: [f32; 2],
    circle_uv: [[f32; 2]; 2],
    screen_w: f32,
    screen_h: f32,
) {
    let geom = match CodeReviewLayout::line_action_popup_geometry(
        model,
        ws_x,
        ws_y,
        ws_w,
        ws_h,
        file_idx,
        row,
        is_deletion,
    ) {
        Some(g) => g,
        None => return,
    };

    let (px, py, pw, ph) = geom.popup_rect;
    if pw <= 0.0 || ph <= 0.0 {
        return;
    }

    // Outer popup container border (1px border using theme.tooltip_border)
    push_rounded_rect(
        verts,
        px,
        py,
        px + pw,
        py + ph,
        4.0,
        circle_uv,
        solid_uv,
        theme.tooltip_border,
        screen_w,
        screen_h,
    );

    // Inner popup background (theme.tooltip_background)
    push_rounded_rect(
        verts,
        px + 1.0,
        py + 1.0,
        px + pw - 1.0,
        py + ph - 1.0,
        3.0,
        circle_uv,
        solid_uv,
        theme.tooltip_background,
        screen_w,
        screen_h,
    );

    let (ax, ay, aw, ah) = geom.add_btn_rect;
    let is_add_hovered = action == Some(akssora_editor::GutterActionKind::StageLine);
    let add_bg = if is_add_hovered {
        theme.code_review_header_button_hover
    } else {
        theme.code_review_header_button_background
    };
    let add_fg = if is_add_hovered {
        theme.diff_gutter_add_badge_fg
    } else {
        theme.tooltip_foreground
    };

    // Stage Change button
    push_rounded_rect(
        verts,
        ax,
        ay,
        ax + aw,
        ay + ah,
        3.0,
        circle_uv,
        solid_uv,
        add_bg,
        screen_w,
        screen_h,
    );

    let add_text = "Stage Change";
    let add_text_w = add_text.len() as f32 * ui_fonts.cell_w;
    let add_tx = ax + ((aw - add_text_w) * 0.5).max(2.0);
    let add_ty = ay + (ah - ui_fonts.cell_h) * 0.5;
    renderer.render_text(
        queue, ui_fonts, verts, add_text, add_tx, add_ty, aw, add_fg, false, screen_w, screen_h,
    );

    // Revert Changes button
    let (rx, ry, rw, rh) = geom.remove_btn_rect;
    let is_rem_hovered = action == Some(akssora_editor::GutterActionKind::RevertLine);
    let rem_bg = if is_rem_hovered {
        theme.code_review_header_button_hover
    } else {
        theme.code_review_header_button_background
    };
    let rem_fg = if is_rem_hovered {
        theme.diff_gutter_del_badge_fg
    } else {
        theme.tooltip_foreground
    };

    push_rounded_rect(
        verts,
        rx,
        ry,
        rx + rw,
        ry + rh,
        3.0,
        circle_uv,
        solid_uv,
        rem_bg,
        screen_w,
        screen_h,
    );

    let rem_text = "Revert Changes";
    let rem_text_w = rem_text.len() as f32 * ui_fonts.cell_w;
    let rem_tx = rx + ((rw - rem_text_w) * 0.5).max(2.0);
    let rem_ty = ry + (rh - ui_fonts.cell_h) * 0.5;
    renderer.render_text(
        queue, ui_fonts, verts, rem_text, rem_tx, rem_ty, rw, rem_fg, false, screen_w, screen_h,
    );
}
