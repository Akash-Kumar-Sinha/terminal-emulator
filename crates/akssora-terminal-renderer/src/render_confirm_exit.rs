use akssora_terminal_config::Theme;

use crate::font::FontManager;
use crate::glyph::GlyphRenderer;
use crate::quad::push_quad;
use crate::vertex::GlyphVertex;

// INFO: Renders the modal exit confirmation dialog and backdrop when closing the last tab.
#[allow(clippy::too_many_arguments)]
pub fn render_confirm_exit_dialog(
    renderer: &mut GlyphRenderer,
    queue: &wgpu::Queue,
    ui_fonts: &FontManager,
    theme: &Theme,
    screen_w: f32,
    screen_h: f32,
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

    let modal_w = 460.0f32.min(screen_w - 40.0);
    let modal_h = 160.0f32;
    let mx = (screen_w - modal_w) / 2.0;
    let my = (screen_h - modal_h) / 2.0;

    // 2. Card background (rgb(1,1,1), rounded rect, refined visual proportions)
    crate::quad::push_rounded_rect(
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

    // 3. Title
    renderer.render_text(
        queue,
        ui_fonts,
        verts,
        "Close Akssora?",
        mx + 24.0,
        my + 22.0,
        modal_w - 48.0,
        theme.text_primary,
        true,
        screen_w,
        screen_h,
    );

    // 4. Message
    renderer.render_text(
        queue,
        ui_fonts,
        verts,
        "Closing the last tab will exit the application.",
        mx + 24.0,
        my + 54.0,
        modal_w - 48.0,
        theme.text_secondary,
        false,
        screen_w,
        screen_h,
    );

    // 5. Centered Buttons: [Exit (Enter)] [Cancel (Esc)]
    let btn_y = my + modal_h - 48.0;
    let btn_h = 32.0;
    let btn_gap = 16.0;
    let c_w = 160.0;
    let can_w = 140.0;
    let total_buttons_w = c_w + can_w + btn_gap;
    let start_btn_x = mx + (modal_w - total_buttons_w) / 2.0;

    let c_x = start_btn_x;
    crate::quad::push_rounded_rect(
        verts,
        c_x,
        btn_y,
        c_x + c_w,
        btn_y + btn_h,
        6.0,
        [[solid_uv[0], solid_uv[1]]; 2],
        solid_uv,
        theme.modal_button_background,
        screen_w,
        screen_h,
    );
    let exit_text = "Exit (Enter)";
    let exit_text_w = exit_text.len() as f32 * ui_fonts.cell_w;
    let exit_text_x = c_x + (c_w - exit_text_w).max(0.0) / 2.0;
    renderer.render_text(
        queue,
        ui_fonts,
        verts,
        exit_text,
        exit_text_x,
        btn_y + (btn_h - ui_fonts.cell_h) * 0.5,
        c_w,
        theme.text_primary,
        true,
        screen_w,
        screen_h,
    );

    // Cancel button
    let can_x = c_x + c_w + btn_gap;
    crate::quad::push_rounded_rect(
        verts,
        can_x,
        btn_y,
        can_x + can_w,
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
    let cancel_text_x = can_x + (can_w - cancel_text_w).max(0.0) / 2.0;
    renderer.render_text(
        queue,
        ui_fonts,
        verts,
        cancel_text,
        cancel_text_x,
        btn_y + (btn_h - ui_fonts.cell_h) * 0.5,
        can_w,
        theme.text_secondary,
        false,
        screen_w,
        screen_h,
    );
}
