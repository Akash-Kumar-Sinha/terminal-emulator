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

// INFO: Renders modal confirmation dialog when discarding unstaged file changes.
#[allow(clippy::too_many_arguments)]
pub fn render_discard_dialog(
    renderer: &mut GlyphRenderer,
    queue: &wgpu::Queue,
    fonts: &FontManager,
    ui_fonts: &FontManager,
    theme: &Theme,
    target: &akssora_terminal_ui::DiscardTarget,
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
        theme.modal_backdrop,
        screen_w,
        screen_h,
    );

    let (modal_x, modal_y, modal_w, modal_h) =
        akssora_terminal_ui::CodeReviewLayout::discard_dialog_rect(ws_x, ws_y, ws_w, ws_h);
    let (cancel_rect, confirm_rect) =
        akssora_terminal_ui::CodeReviewLayout::discard_dialog_buttons_rect(ws_x, ws_y, ws_w, ws_h);

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
        theme.modal_card_background,
        screen_w,
        screen_h,
    );

    let (dialog_title, prompt_msg) = match target {
        akssora_terminal_ui::DiscardTarget::File(path) => {
            let filename = path
                .file_name()
                .map(|n| n.to_string_lossy())
                .unwrap_or_default();
            (
                "Discard File Changes?".to_string(),
                format!("Discard all unstaged changes in \"{filename}\"?"),
            )
        }
        akssora_terminal_ui::DiscardTarget::Directory(dir) => (
            "Discard Folder Changes?".to_string(),
            format!("Discard all unstaged changes in \"{}\"?", dir.display()),
        ),
        akssora_terminal_ui::DiscardTarget::All => (
            "Discard All Changes?".to_string(),
            "Discard all unstaged changes in the repository?".to_string(),
        ),
    };

    // Dialog Header Title
    renderer.render_text(
        queue,
        ui_fonts,
        verts,
        &dialog_title,
        modal_x + 20.0,
        modal_y + 18.0,
        modal_w - 40.0,
        theme.text_primary,
        false,
        screen_w,
        screen_h,
    );

    // Confirmation Body Text
    renderer.render_text(
        queue,
        fonts,
        verts,
        &prompt_msg,
        modal_x + 20.0,
        modal_y + 52.0,
        modal_w - 40.0,
        theme.text_secondary,
        false,
        screen_w,
        screen_h,
    );

    renderer.render_text(
        queue,
        ui_fonts,
        verts,
        "This operation is destructive and cannot be undone.",
        modal_x + 20.0,
        modal_y + 78.0,
        modal_w - 40.0,
        theme.danger_foreground,
        false,
        screen_w,
        screen_h,
    );

    // Cancel Button
    push_rounded_rect(
        verts,
        cancel_rect.0,
        cancel_rect.1,
        cancel_rect.0 + cancel_rect.2,
        cancel_rect.1 + cancel_rect.3,
        4.0,
        circle_uv,
        solid_uv,
        theme.modal_cancel_button_background,
        screen_w,
        screen_h,
    );
    let cancel_text_w = 46.0;
    renderer.render_text(
        queue,
        ui_fonts,
        verts,
        "Cancel",
        cancel_rect.0 + (cancel_rect.2 - cancel_text_w) * 0.5,
        cancel_rect.1 + (cancel_rect.3 - ui_fonts.cell_h) * 0.5,
        cancel_rect.2,
        theme.text_primary,
        false,
        screen_w,
        screen_h,
    );

    // Confirm Discard Button (Destructive action)
    push_rounded_rect(
        verts,
        confirm_rect.0,
        confirm_rect.1,
        confirm_rect.0 + confirm_rect.2,
        confirm_rect.1 + confirm_rect.3,
        4.0,
        circle_uv,
        solid_uv,
        theme.modal_danger_button_background,
        screen_w,
        screen_h,
    );
    let discard_text_w = 48.0;
    renderer.render_text(
        queue,
        ui_fonts,
        verts,
        "Discard",
        confirm_rect.0 + (confirm_rect.2 - discard_text_w) * 0.5,
        confirm_rect.1 + (confirm_rect.3 - ui_fonts.cell_h) * 0.5,
        confirm_rect.2,
        theme.text_primary,
        false,
        screen_w,
        screen_h,
    );
}
