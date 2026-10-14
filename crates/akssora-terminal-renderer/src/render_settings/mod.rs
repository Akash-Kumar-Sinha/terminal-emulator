use akssora_terminal_config::{Config, ConflictWarning, TabLayoutPreference, Theme};
use akssora_terminal_ui::{SettingsAction, SettingsTab, SettingsView};

use crate::font::FontManager;
use crate::glyph::GlyphRenderer;
use crate::quad::{push_quad, push_rounded_rect};
use crate::render_settings_shortcuts::render_shortcuts_section;
use crate::vertex::GlyphVertex;

pub mod appearance;
pub mod code_review;
pub mod dropdown;
pub mod editor;
pub mod general;

pub use dropdown::render_line_number_dropdown_list;

// INFO: Renders the full multi-section settings view with category tabs.
#[allow(clippy::too_many_arguments)]
pub fn render_settings_panel(
    renderer: &mut GlyphRenderer,
    queue: &wgpu::Queue,
    fonts: &FontManager,
    terminal_font_size: f32,
    theme: &Theme,
    config: &Config,
    active_tab: SettingsTab,
    keybinding_search: &str,
    conflicts: &[ConflictWarning],
    current_layout: TabLayoutPreference,
    hovered_action: Option<SettingsAction>,
    layout_dropdown_open: bool,
    line_number_dropdown_open: bool,
    workspace_x: f32,
    workspace_y: f32,
    workspace_w: f32,
    workspace_h: f32,
    verts: &mut Vec<GlyphVertex>,
    solid_uv: [f32; 2],
    circle_uv: [[f32; 2]; 2],
    screen_w: f32,
    screen_h: f32,
) {
    if workspace_w <= 0.0 || workspace_h <= 0.0 {
        return;
    }

    let cell_h = fonts.cell_h;

    // ── 1. Workspace background ───────────────────────────────────────────────
    push_quad(
        verts,
        workspace_x,
        workspace_y,
        workspace_x + workspace_w,
        workspace_y + workspace_h,
        solid_uv,
        solid_uv,
        theme.background,
        screen_w,
        screen_h,
    );

    // ── 2. Page title ─────────────────────────────────────────────────────────
    let title_x = workspace_x + 40.0;
    let title_y = workspace_y + 24.0;
    renderer.render_text(
        queue,
        fonts,
        verts,
        "Settings",
        title_x,
        title_y,
        300.0,
        theme.text_primary,
        true,
        screen_w,
        screen_h,
    );

    // ── 3. Subtitle ───────────────────────────────────────────────────────────
    let sub_y = title_y + cell_h + 4.0;
    renderer.render_text(
        queue,
        fonts,
        verts,
        "Preferences, appearance, editor, and keybindings",
        title_x,
        sub_y,
        500.0,
        theme.text_muted,
        false,
        screen_w,
        screen_h,
    );

    // ── 4. Category Tabs (Vertical on the left) ──────────────────────────────
    for (idx, &tab) in SettingsTab::ALL.iter().enumerate() {
        let (tx, ty, tw, th) = SettingsView::tab_button_rect(idx, workspace_x, workspace_y, cell_h);
        let is_selected = tab == active_tab;
        let is_hovered = hovered_action == Some(SettingsAction::SelectTab(tab));

        let tab_bg = if is_selected {
            theme.session_selected_background
        } else if is_hovered {
            theme.session_hover_background
        } else {
            theme.session_background
        };

        push_rounded_rect(
            verts,
            tx,
            ty,
            tx + tw,
            ty + th,
            4.0,
            circle_uv,
            solid_uv,
            tab_bg,
            screen_w,
            screen_h,
        );

        let tab_col = if is_selected {
            theme.text_primary
        } else if is_hovered {
            theme.text_secondary
        } else {
            theme.text_muted
        };

        renderer.render_text(
            queue,
            fonts,
            verts,
            tab.label(),
            tx + 14.0,
            ty + (th - cell_h) / 2.0,
            tw - 20.0,
            tab_col,
            is_selected,
            screen_w,
            screen_h,
        );
    }

    let content_title_x = SettingsView::content_x(workspace_x);
    let sec_y = SettingsView::tab_content_start_y(workspace_y, cell_h);

    // ── 5. Tab Content Panes ──────────────────────────────────────────────────
    match active_tab {
        SettingsTab::General => {
            general::render_general_tab(
                renderer,
                queue,
                fonts,
                terminal_font_size,
                config.ui_font_size,
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
        }

        SettingsTab::Appearance => {
            appearance::render_appearance_tab(
                renderer,
                queue,
                fonts,
                theme,
                config,
                hovered_action,
                workspace_x,
                workspace_y,
                content_title_x,
                sec_y,
                cell_h,
                verts,
                solid_uv,
                circle_uv,
                screen_w,
                screen_h,
            );
        }

        SettingsTab::Editor => {
            editor::render_editor_tab(
                renderer,
                queue,
                fonts,
                theme,
                config,
                hovered_action,
                line_number_dropdown_open,
                workspace_x,
                workspace_y,
                content_title_x,
                sec_y,
                cell_h,
                verts,
                solid_uv,
                circle_uv,
                screen_w,
                screen_h,
            );
        }

        SettingsTab::CodeReview => {
            code_review::render_code_review_tab(
                renderer,
                queue,
                fonts,
                theme,
                config,
                hovered_action,
                workspace_x,
                workspace_y,
                content_title_x,
                sec_y,
                cell_h,
                verts,
                solid_uv,
                circle_uv,
                screen_w,
                screen_h,
            );
        }

        SettingsTab::Keybindings => {
            render_shortcuts_section(
                renderer,
                queue,
                fonts,
                theme,
                &config.keybindings,
                keybinding_search,
                hovered_action,
                conflicts,
                workspace_x,
                workspace_y,
                workspace_w,
                workspace_h,
                content_title_x,
                verts,
                solid_uv,
                circle_uv,
                screen_w,
                screen_h,
            );
        }
    }
}
