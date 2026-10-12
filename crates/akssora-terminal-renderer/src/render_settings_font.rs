use akssora_terminal_config::Theme;
use akssora_terminal_ui::{SettingsAction, SettingsView};

use crate::font::FontManager;
use crate::glyph::GlyphRenderer;
use crate::quad::push_quad;
use crate::render_settings_button_box::render_button_box;
use crate::vertex::GlyphVertex;

// INFO: Renders the Terminal Font Size section with section heading, hint, and zoom controls.
// Button positions are defined in SettingsView::font_*_button_rect with enough room for the hint line.
#[allow(clippy::too_many_arguments)]
pub fn render_font_size_section(
    renderer: &mut GlyphRenderer,
    queue: &wgpu::Queue,
    fonts: &FontManager,
    terminal_font_size: f32,
    theme: &Theme,
    hovered_action: Option<SettingsAction>,
    workspace_x: f32,
    workspace_y: f32,
    workspace_w: f32,
    title_x: f32,
    verts: &mut Vec<GlyphVertex>,
    solid_uv: [f32; 2],
    screen_w: f32,
    screen_h: f32,
) {
    let cell_h = fonts.cell_h;
    let font_sec_y = SettingsView::font_section_y(workspace_y, cell_h);

    // Subtle divider above section
    let div_x2 = (workspace_x + workspace_w - 40.0).max(title_x + 100.0);
    push_quad(
        verts,
        title_x,
        font_sec_y - 14.0,
        div_x2,
        font_sec_y - 13.0,
        solid_uv,
        solid_uv,
        theme.border_subtle,
        screen_w,
        screen_h,
    );

    // Section heading
    renderer.render_text(
        queue,
        fonts,
        verts,
        "Terminal Font Size",
        title_x,
        font_sec_y,
        400.0,
        theme.text_secondary,
        false,
        screen_w,
        screen_h,
    );

    // Hint text — one short line below heading, well above the buttons
    let hint_y = font_sec_y + cell_h + 4.0;
    renderer.render_text(
        queue,
        fonts,
        verts,
        "Ctrl++  Ctrl+-  Ctrl+0 to zoom / reset",
        title_x,
        hint_y,
        500.0,
        theme.text_muted,
        false,
        screen_w,
        screen_h,
    );

    // ── Controls row ──────────────────────────────────────────────────────────
    // All four controls share btn_y = font_sec_y + 48 (room for heading + hint + gap).

    // 1. Minus button [−]
    let (mx, my, mw, mh) =
        SettingsView::font_minus_button_rect(workspace_x, workspace_y, workspace_w, cell_h);
    let is_minus_hovered = hovered_action == Some(SettingsAction::DecreaseFontSize);
    render_button_box(
        renderer,
        queue,
        fonts,
        theme,
        verts,
        mx,
        my,
        mw,
        mh,
        is_minus_hovered,
        Some('\u{2212}'),
        None,
        solid_uv,
        screen_w,
        screen_h,
    );

    // 2. Display box "14 pt"
    let (dx, dy, dw, dh) =
        SettingsView::font_display_rect(workspace_x, workspace_y, workspace_w, cell_h);
    crate::quad::push_rounded_rect(
        verts,
        dx,
        dy,
        dx + dw,
        dy + dh,
        4.0,
        [[solid_uv[0], solid_uv[1]]; 2],
        solid_uv,
        theme.session_background,
        screen_w,
        screen_h,
    );

    let size_label = format!("{:.0} pt", terminal_font_size);
    let size_text_y = dy + (dh - cell_h) / 2.0;
    renderer.render_text(
        queue,
        fonts,
        verts,
        &size_label,
        dx + 10.0,
        size_text_y,
        dw - 14.0,
        theme.text_primary,
        true,
        screen_w,
        screen_h,
    );

    // 3. Plus button [+]
    let (px, py, pw, ph) =
        SettingsView::font_plus_button_rect(workspace_x, workspace_y, workspace_w, cell_h);
    let is_plus_hovered = hovered_action == Some(SettingsAction::IncreaseFontSize);
    render_button_box(
        renderer,
        queue,
        fonts,
        theme,
        verts,
        px,
        py,
        pw,
        ph,
        is_plus_hovered,
        Some('+'),
        None,
        solid_uv,
        screen_w,
        screen_h,
    );

    // 4. Reset button
    let (rx, ry, rw, rh) =
        SettingsView::font_reset_button_rect(workspace_x, workspace_y, workspace_w, cell_h);
    let is_reset_hovered = hovered_action == Some(SettingsAction::ResetFontSize);
    render_button_box(
        renderer,
        queue,
        fonts,
        theme,
        verts,
        rx,
        ry,
        rw,
        rh,
        is_reset_hovered,
        None,
        Some("Reset"),
        solid_uv,
        screen_w,
        screen_h,
    );
}
