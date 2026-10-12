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
        [0.0, 0.0, 0.0, 0.78],
        screen_w,
        screen_h,
    );

    let modal_w = 460.0f32.min(screen_w - 40.0);
    let modal_h = 160.0f32;
    let mx = (screen_w - modal_w) / 2.0;
    let my = (screen_h - modal_h) / 2.0;

    // 2. Card background (borderless, rounded rect, subtle background separation)
    crate::quad::push_rounded_rect(
        verts,
        mx,
        my,
        mx + modal_w,
        my + modal_h,
        8.0,
        [[solid_uv[0], solid_uv[1]]; 2],
        solid_uv,
        theme.session_selected_background,
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

    // 5. Confirm button [Exit (Enter)]
    let btn_y = my + modal_h - 48.0;
    let btn_h = 32.0;
    let c_x = mx + 24.0;
    let c_w = 170.0;
    crate::quad::push_rounded_rect(
        verts,
        c_x,
        btn_y,
        c_x + c_w,
        btn_y + btn_h,
        4.0,
        [[solid_uv[0], solid_uv[1]]; 2],
        solid_uv,
        theme.button_hover_background,
        screen_w,
        screen_h,
    );
    renderer.render_text(
        queue,
        ui_fonts,
        verts,
        "Exit (Enter)",
        c_x + 36.0,
        btn_y + 8.0,
        c_w - 40.0,
        theme.text_primary,
        true,
        screen_w,
        screen_h,
    );

    // 6. Cancel button [Cancel (Esc)]
    let can_x = c_x + c_w + 16.0;
    let can_w = 150.0;
    crate::quad::push_rounded_rect(
        verts,
        can_x,
        btn_y,
        can_x + can_w,
        btn_y + btn_h,
        4.0,
        [[solid_uv[0], solid_uv[1]]; 2],
        solid_uv,
        theme.session_background,
        screen_w,
        screen_h,
    );
    renderer.render_text(
        queue,
        ui_fonts,
        verts,
        "Cancel (Esc)",
        can_x + 30.0,
        btn_y + 8.0,
        can_w - 40.0,
        theme.text_secondary,
        false,
        screen_w,
        screen_h,
    );
}
