use akssora_terminal_config::{Config, Theme};
use akssora_terminal_ui::{SettingsAction, SettingsView};

use crate::font::FontManager;
use crate::glyph::GlyphRenderer;
use crate::quad::push_rounded_rect;
use crate::vertex::GlyphVertex;

#[allow(clippy::too_many_arguments)]
pub fn render_code_review_tab(
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
        "Code Review & Diff Preferences",
        content_title_x,
        sec_y,
        400.0,
        theme.text_secondary,
        false,
        screen_w,
        screen_h,
    );

    let (dx, dy, dw, dh) =
        SettingsView::code_review_diff_mode_rect(workspace_x, workspace_y, cell_h);
    let bg_dm = if hovered_action == Some(SettingsAction::ToggleCodeReviewDiffMode) {
        theme.session_hover_background
    } else {
        theme.session_background
    };
    push_rounded_rect(
        verts,
        dx,
        dy,
        dx + dw,
        dy + dh,
        4.0,
        circle_uv,
        solid_uv,
        bg_dm,
        screen_w,
        screen_h,
    );
    let diff_mode_str = format!("Default Diff Mode: {}", config.code_review_diff_mode);
    renderer.render_text(
        queue,
        fonts,
        verts,
        &diff_mode_str,
        dx + 14.0,
        dy + (dh - cell_h) / 2.0,
        dw - 28.0,
        theme.text_primary,
        false,
        screen_w,
        screen_h,
    );

    let (fx, fy, fw, fh) =
        SettingsView::code_review_find_bar_rect(workspace_x, workspace_y, cell_h);
    let bg_fb = if hovered_action == Some(SettingsAction::ToggleCodeReviewFindBar) {
        theme.session_hover_background
    } else {
        theme.session_background
    };
    push_rounded_rect(
        verts,
        fx,
        fy,
        fx + fw,
        fy + fh,
        4.0,
        circle_uv,
        solid_uv,
        bg_fb,
        screen_w,
        screen_h,
    );
    let fb_str = format!(
        "Show Find Bar: {}",
        if config.code_review_show_find_bar {
            "ON"
        } else {
            "OFF"
        }
    );
    renderer.render_text(
        queue,
        fonts,
        verts,
        &fb_str,
        fx + 14.0,
        fy + (fh - cell_h) / 2.0,
        fw - 28.0,
        if config.code_review_show_find_bar {
            theme.text_primary
        } else {
            theme.text_muted
        },
        config.code_review_show_find_bar,
        screen_w,
        screen_h,
    );
}
