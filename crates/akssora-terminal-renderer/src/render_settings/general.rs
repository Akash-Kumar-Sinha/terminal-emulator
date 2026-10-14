use akssora_terminal_config::{TabLayoutPreference, Theme};
use akssora_terminal_ui::SettingsAction;

use crate::font::FontManager;
use crate::glyph::GlyphRenderer;
use crate::render_settings_font::{render_font_size_section, render_ui_font_size_section};
use crate::render_settings_layout::render_tab_layout_trigger;
use crate::vertex::GlyphVertex;

#[allow(clippy::too_many_arguments)]
pub fn render_general_tab(
    renderer: &mut GlyphRenderer,
    queue: &wgpu::Queue,
    fonts: &FontManager,
    terminal_font_size: f32,
    ui_font_size: f32,
    theme: &Theme,
    current_layout: TabLayoutPreference,
    hovered_action: Option<SettingsAction>,
    layout_dropdown_open: bool,
    workspace_x: f32,
    workspace_y: f32,
    workspace_w: f32,
    content_title_x: f32,
    sec_y: f32,
    verts: &mut Vec<GlyphVertex>,
    solid_uv: [f32; 2],
    screen_w: f32,
    screen_h: f32,
) {
    render_tab_layout_trigger(
        renderer,
        queue,
        fonts,
        theme,
        current_layout,
        hovered_action,
        layout_dropdown_open,
        workspace_x,
        workspace_y,
        workspace_w,
        content_title_x,
        sec_y,
        verts,
        solid_uv,
        screen_w,
        screen_h,
    );

    render_font_size_section(
        renderer,
        queue,
        fonts,
        terminal_font_size,
        theme,
        hovered_action,
        workspace_x,
        workspace_y,
        workspace_w,
        content_title_x,
        verts,
        solid_uv,
        screen_w,
        screen_h,
    );

    render_ui_font_size_section(
        renderer,
        queue,
        fonts,
        ui_font_size,
        theme,
        hovered_action,
        workspace_x,
        workspace_y,
        workspace_w,
        content_title_x,
        verts,
        solid_uv,
        screen_w,
        screen_h,
    );
}
