use akssora_terminal_config::Theme;
use akssora_terminal_ui::CodeReviewModel;

use crate::font::FontManager;
use crate::glyph::GlyphRenderer;
use crate::quad::{push_quad, push_rounded_rect};
use crate::vertex::GlyphVertex;

// INFO: Renders the commit confirmation dialog modal overlay.
#[allow(clippy::too_many_arguments)]
pub fn render_commit_dialog(
    renderer: &mut GlyphRenderer,
    queue: &wgpu::Queue,
    fonts: &FontManager,
    ui_fonts: &FontManager,
    theme: &Theme,
    model: &CodeReviewModel,
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
    // Backdrop dimming
    push_quad(
        verts,
        ws_x,
        ws_y,
        ws_x + ws_w,
        ws_y + ws_h,
        solid_uv,
        solid_uv,
        [0.0, 0.0, 0.0, 0.65],
        screen_w,
        screen_h,
    );

    let modal_w = 460.0;
    let modal_h = 200.0;
    let modal_x = ws_x + (ws_w - modal_w) * 0.5;
    let modal_y = ws_y + (ws_h - modal_h) * 0.5;

    // Modal Card
    push_rounded_rect(
        verts,
        modal_x,
        modal_y,
        modal_x + modal_w,
        modal_y + modal_h,
        8.0,
        circle_uv,
        solid_uv,
        [0.12, 0.12, 0.14, 1.0],
        screen_w,
        screen_h,
    );

    // Title
    renderer.render_text(
        queue,
        ui_fonts,
        verts,
        "Commit Changes",
        modal_x + 20.0,
        modal_y + 16.0,
        300.0,
        theme.text_primary,
        false,
        screen_w,
        screen_h,
    );

    // Input box
    let input_y = modal_y + 50.0;
    let input_h = 60.0;
    push_rounded_rect(
        verts,
        modal_x + 20.0,
        input_y,
        modal_x + modal_w - 20.0,
        input_y + input_h,
        4.0,
        circle_uv,
        solid_uv,
        theme.background,
        screen_w,
        screen_h,
    );
    let msg_display = if model.commit_message.is_empty() {
        "Enter commit message...".to_string()
    } else {
        model.commit_message.clone()
    };
    let msg_col = if model.commit_message.is_empty() {
        theme.text_muted
    } else {
        theme.text_primary
    };
    renderer.render_text(
        queue,
        fonts,
        verts,
        &msg_display,
        modal_x + 30.0,
        input_y + 12.0,
        modal_w - 60.0,
        msg_col,
        false,
        screen_w,
        screen_h,
    );

    // Action buttons
    let btn_y = modal_y + modal_h - 44.0;
    // Submit Commit Button
    push_rounded_rect(
        verts,
        modal_x + modal_w - 120.0,
        btn_y,
        modal_x + modal_w - 20.0,
        btn_y + 28.0,
        4.0,
        circle_uv,
        solid_uv,
        theme.code_review_commit_dialog_button,
        screen_w,
        screen_h,
    );
    renderer.render_text(
        queue,
        ui_fonts,
        verts,
        "Commit",
        modal_x + modal_w - 100.0,
        btn_y + 6.0,
        80.0,
        theme.text_primary,
        false,
        screen_w,
        screen_h,
    );

    // Cancel Button
    push_rounded_rect(
        verts,
        modal_x + modal_w - 210.0,
        btn_y,
        modal_x + modal_w - 130.0,
        btn_y + 28.0,
        4.0,
        circle_uv,
        solid_uv,
        theme.modal_button_background,
        screen_w,
        screen_h,
    );
    renderer.render_text(
        queue,
        ui_fonts,
        verts,
        "Cancel",
        modal_x + modal_w - 195.0,
        btn_y + 6.0,
        70.0,
        theme.text_muted,
        false,
        screen_w,
        screen_h,
    );
}
