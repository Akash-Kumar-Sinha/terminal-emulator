use akssora_terminal_config::Theme;

use crate::font::FontManager;
use crate::glyph::GlyphRenderer;
use crate::quad::{push_quad, push_rounded_rect};
use crate::vertex::GlyphVertex;

// INFO: Renders the modal prompt dialog when attempting to close editor tabs with unsaved changes.
#[allow(clippy::too_many_arguments)]
pub fn render_unsaved_changes_dialog(
    renderer: &mut GlyphRenderer,
    queue: &wgpu::Queue,
    ui_fonts: &FontManager,
    theme: &Theme,
    screen_w: f32,
    screen_h: f32,
    file_title: &str,
    verts: &mut Vec<GlyphVertex>,
    solid_uv: [f32; 2],
) {
    // 1. Dimmed backdrop
    push_quad(
        verts,
        0.0,
        0.0,
        screen_w,
        screen_h,
        solid_uv,
        solid_uv,
        theme.modal_backdrop,
        screen_w,
        screen_h,
    );

    let modal_w = 480.0f32.min(screen_w - 40.0);
    let modal_h = 160.0f32;
    let mx = (screen_w - modal_w) / 2.0;
    let my = (screen_h - modal_h) / 2.0;

    // 2. Card background (rgb(1,1,1), rounded rect, refined visual proportions)
    push_rounded_rect(
        verts,
        mx,
        my,
        mx + modal_w,
        my + modal_h,
        10.0,
        [[solid_uv[0], solid_uv[1]]; 2],
        solid_uv,
        theme.modal_card_background,
        screen_w,
        screen_h,
    );

    let is_all = file_title == "all modified files";

    // 3. Title
    let title_text = if is_all {
        "Save All Changes?"
    } else {
        "Save Changes?"
    };
    renderer.render_text(
        queue,
        ui_fonts,
        verts,
        title_text,
        mx + 24.0,
        my + 22.0,
        modal_w - 48.0,
        theme.text_primary,
        true,
        screen_w,
        screen_h,
    );

    // 4. Message
    let msg = if is_all {
        "Do you want to save changes made to all modified files?".to_string()
    } else {
        format!("Do you want to save changes made to \"{file_title}\"?")
    };
    renderer.render_text(
        queue,
        ui_fonts,
        verts,
        &msg,
        mx + 24.0,
        my + 54.0,
        modal_w - 48.0,
        theme.text_secondary,
        false,
        screen_w,
        screen_h,
    );

    // 5. Centered Buttons: [Save All (Enter) / Save (Enter)]  [Don't Save (D)]  [Cancel (Esc)]
    let btn_y = my + modal_h - 48.0;
    let btn_h = 32.0;
    let btn_gap = 12.0;

    let save_w = if is_all { 145.0 } else { 125.0 };
    let dont_w = 135.0;
    let cancel_w = 125.0;
    let total_buttons_w = save_w + dont_w + cancel_w + (btn_gap * 2.0);
    let start_btn_x = mx + (modal_w - total_buttons_w) / 2.0;

    // Save button
    let save_x = start_btn_x;
    push_rounded_rect(
        verts,
        save_x,
        btn_y,
        save_x + save_w,
        btn_y + btn_h,
        6.0,
        [[solid_uv[0], solid_uv[1]]; 2],
        solid_uv,
        theme.modal_button_background,
        screen_w,
        screen_h,
    );
    let save_text = if is_all {
        "Save All (Enter)"
    } else {
        "Save (Enter)"
    };
    let save_text_w = save_text.len() as f32 * ui_fonts.cell_w;
    let save_text_x = save_x + (save_w - save_text_w).max(0.0) / 2.0;
    renderer.render_text(
        queue,
        ui_fonts,
        verts,
        save_text,
        save_text_x,
        btn_y + (btn_h - ui_fonts.cell_h) * 0.5,
        save_w,
        theme.text_primary,
        true,
        screen_w,
        screen_h,
    );

    // Don't Save button
    let dont_x = save_x + save_w + btn_gap;
    push_rounded_rect(
        verts,
        dont_x,
        btn_y,
        dont_x + dont_w,
        btn_y + btn_h,
        6.0,
        [[solid_uv[0], solid_uv[1]]; 2],
        solid_uv,
        theme.modal_button_background,
        screen_w,
        screen_h,
    );
    let dont_text = "Don't Save (D)";
    let dont_text_w = dont_text.len() as f32 * ui_fonts.cell_w;
    let dont_text_x = dont_x + (dont_w - dont_text_w).max(0.0) / 2.0;
    renderer.render_text(
        queue,
        ui_fonts,
        verts,
        dont_text,
        dont_text_x,
        btn_y + (btn_h - ui_fonts.cell_h) * 0.5,
        dont_w,
        theme.text_secondary,
        false,
        screen_w,
        screen_h,
    );

    // Cancel button
    let cancel_x = dont_x + dont_w + btn_gap;
    push_rounded_rect(
        verts,
        cancel_x,
        btn_y,
        cancel_x + cancel_w,
        btn_y + btn_h,
        6.0,
        [[solid_uv[0], solid_uv[1]]; 2],
        solid_uv,
        theme.modal_button_background,
        screen_w,
        screen_h,
    );
    let cancel_text = "Cancel (Esc)";
    let cancel_text_w = cancel_text.len() as f32 * ui_fonts.cell_w;
    let cancel_text_x = cancel_x + (cancel_w - cancel_text_w).max(0.0) / 2.0;
    renderer.render_text(
        queue,
        ui_fonts,
        verts,
        cancel_text,
        cancel_text_x,
        btn_y + (btn_h - ui_fonts.cell_h) * 0.5,
        cancel_w,
        theme.text_secondary,
        false,
        screen_w,
        screen_h,
    );
}
