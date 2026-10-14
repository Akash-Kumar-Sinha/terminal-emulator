use akssora_terminal_config::{Config, Theme};
use akssora_terminal_ui::{SettingsAction, SettingsView};

use crate::font::FontManager;
use crate::glyph::GlyphRenderer;
use crate::quad::push_rounded_rect;
use crate::vertex::GlyphVertex;

#[allow(clippy::too_many_arguments)]
pub fn render_appearance_tab(
    renderer: &mut GlyphRenderer,
    queue: &wgpu::Queue,
    fonts: &FontManager,
    theme: &Theme,
    config: &Config,
    hovered_action: Option<SettingsAction>,
    workspace_x: f32,
    workspace_y: f32,
    content_title_x: f32,
    sec_y: f32,
    cell_h: f32,
    verts: &mut Vec<GlyphVertex>,
    solid_uv: [f32; 2],
    circle_uv: [[f32; 2]; 2],
    screen_w: f32,
    screen_h: f32,
) {
    renderer.render_text(
        queue,
        fonts,
        verts,
        "Terminal Cursor",
        content_title_x,
        sec_y,
        300.0,
        theme.text_secondary,
        false,
        screen_w,
        screen_h,
    );

    let (cx, cy, cw, ch) = SettingsView::cursor_style_rect(workspace_x, workspace_y, cell_h);
    let bg_cur = if hovered_action == Some(SettingsAction::CycleCursorStyle) {
        theme.session_hover_background
    } else {
        theme.session_background
    };
    push_rounded_rect(
        verts,
        cx,
        cy,
        cx + cw,
        cy + ch,
        4.0,
        circle_uv,
        solid_uv,
        bg_cur,
        screen_w,
        screen_h,
    );
    let cur_label = format!("Style: {}", config.cursor_style);
    renderer.render_text(
        queue,
        fonts,
        verts,
        &cur_label,
        cx + 14.0,
        cy + (ch - cell_h) / 2.0,
        cw - 20.0,
        theme.text_primary,
        false,
        screen_w,
        screen_h,
    );

    let (kx, ky, kw, kh) = SettingsView::cursor_blink_rect(workspace_x, workspace_y, cell_h);
    let bg_blink = if hovered_action == Some(SettingsAction::ToggleCursorBlink) {
        theme.session_hover_background
    } else {
        theme.session_background
    };
    push_rounded_rect(
        verts,
        kx,
        ky,
        kx + kw,
        ky + kh,
        4.0,
        circle_uv,
        solid_uv,
        bg_blink,
        screen_w,
        screen_h,
    );
    let blink_label = if config.cursor_blink {
        "Blink: Enabled"
    } else {
        "Blink: Disabled"
    };
    renderer.render_text(
        queue,
        fonts,
        verts,
        blink_label,
        kx + 14.0,
        ky + (kh - cell_h) / 2.0,
        kw - 20.0,
        theme.text_primary,
        false,
        screen_w,
        screen_h,
    );
}
