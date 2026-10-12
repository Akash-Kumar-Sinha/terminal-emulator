use akssora_terminal_config::{Config, ConflictWarning, TabLayoutPreference, Theme};
use akssora_terminal_ui::{SettingsAction, SettingsTab, SettingsView};

use crate::font::FontManager;
use crate::glyph::GlyphRenderer;
use crate::quad::{push_quad, push_rounded_rect};
use crate::render_settings_font::render_font_size_section;
use crate::render_settings_layout::{render_tab_layout_dropdown_list, render_tab_layout_trigger};
use crate::render_settings_shortcuts::render_shortcuts_section;
use crate::vertex::GlyphVertex;

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
    let sub_y = title_y + cell_h + 2.0;
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

    // ── 4. Category Tabs ──────────────────────────────────────────────────────
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

    // Horizontal divider below category tabs
    let div_y = SettingsView::tab_content_start_y(workspace_y, cell_h) - 8.0;
    push_quad(
        verts,
        title_x,
        div_y,
        (workspace_x + workspace_w - 40.0).max(title_x + 100.0),
        div_y + 1.0,
        solid_uv,
        solid_uv,
        theme.border_subtle,
        screen_w,
        screen_h,
    );

    // ── 5. Tab Content Panes ──────────────────────────────────────────────────
    match active_tab {
        SettingsTab::General => {
            let sec_y = div_y + 16.0;
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
                title_x,
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
                title_x,
                verts,
                solid_uv,
                screen_w,
                screen_h,
            );

            if layout_dropdown_open {
                render_tab_layout_dropdown_list(
                    renderer,
                    queue,
                    fonts,
                    theme,
                    current_layout,
                    hovered_action,
                    workspace_x,
                    workspace_y,
                    workspace_w,
                    verts,
                    solid_uv,
                    screen_w,
                    screen_h,
                );
            }
        }

        SettingsTab::Appearance => {
            let sec_y = div_y + 16.0;
            renderer.render_text(
                queue,
                fonts,
                verts,
                "Theme Selection",
                title_x,
                sec_y,
                300.0,
                theme.text_secondary,
                false,
                screen_w,
                screen_h,
            );

            let (bx, by, bw, bh) =
                SettingsView::theme_monochrome_rect(workspace_x, workspace_y, cell_h);
            let is_pure_black = config.theme_name.contains("Black");
            let bg_mono = if is_pure_black {
                theme.session_selected_background
            } else if hovered_action == Some(SettingsAction::SelectThemeMonochrome) {
                theme.session_hover_background
            } else {
                theme.session_background
            };
            push_rounded_rect(
                verts,
                bx,
                by,
                bx + bw,
                by + bh,
                4.0,
                circle_uv,
                solid_uv,
                bg_mono,
                screen_w,
                screen_h,
            );
            renderer.render_text(
                queue,
                fonts,
                verts,
                "Pure Black",
                bx + 14.0,
                by + (bh - cell_h) / 2.0,
                bw - 20.0,
                if is_pure_black {
                    theme.text_primary
                } else {
                    theme.text_secondary
                },
                is_pure_black,
                screen_w,
                screen_h,
            );

            let (sx, sy, sw, sh) = SettingsView::theme_slate_rect(workspace_x, workspace_y, cell_h);
            let is_slate = config.theme_name.contains("Slate");
            let bg_slate = if is_slate {
                theme.session_selected_background
            } else if hovered_action == Some(SettingsAction::SelectThemeSlate) {
                theme.session_hover_background
            } else {
                theme.session_background
            };
            push_rounded_rect(
                verts,
                sx,
                sy,
                sx + sw,
                sy + sh,
                4.0,
                circle_uv,
                solid_uv,
                bg_slate,
                screen_w,
                screen_h,
            );
            renderer.render_text(
                queue,
                fonts,
                verts,
                "Slate Dark",
                sx + 14.0,
                sy + (sh - cell_h) / 2.0,
                sw - 20.0,
                if is_slate {
                    theme.text_primary
                } else {
                    theme.text_secondary
                },
                is_slate,
                screen_w,
                screen_h,
            );

            // Cursor Style & Cursor Blink
            let cur_sec_y = sec_y + cell_h * 2.0 + 20.0;
            renderer.render_text(
                queue,
                fonts,
                verts,
                "Terminal Cursor",
                title_x,
                cur_sec_y,
                300.0,
                theme.text_secondary,
                false,
                screen_w,
                screen_h,
            );

            let (cx, cy, cw, ch) =
                SettingsView::cursor_style_rect(workspace_x, workspace_y, cell_h);
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

            let (kx, ky, kw, kh) =
                SettingsView::cursor_blink_rect(workspace_x, workspace_y, cell_h);
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

        SettingsTab::Editor => {
            let sec_y = div_y + 16.0;
            renderer.render_text(
                queue,
                fonts,
                verts,
                "Editor Configuration",
                title_x,
                sec_y,
                300.0,
                theme.text_secondary,
                false,
                screen_w,
                screen_h,
            );

            let (tx, ty, tw, th) = SettingsView::tab_size_rect(workspace_x, workspace_y, cell_h);
            let bg_tab = if hovered_action == Some(SettingsAction::CycleTabSize) {
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
                bg_tab,
                screen_w,
                screen_h,
            );
            let tab_label = format!("Tab Size: {} spaces", config.tab_size);
            renderer.render_text(
                queue,
                fonts,
                verts,
                &tab_label,
                tx + 14.0,
                ty + (th - cell_h) / 2.0,
                tw - 20.0,
                theme.text_primary,
                false,
                screen_w,
                screen_h,
            );

            let (lx, ly, lw, lh) =
                SettingsView::line_numbers_rect(workspace_x, workspace_y, cell_h);
            let bg_ln = if hovered_action == Some(SettingsAction::ToggleLineNumbers) {
                theme.session_hover_background
            } else {
                theme.session_background
            };
            push_rounded_rect(
                verts,
                lx,
                ly,
                lx + lw,
                ly + lh,
                4.0,
                circle_uv,
                solid_uv,
                bg_ln,
                screen_w,
                screen_h,
            );
            let ln_label = if config.line_numbers {
                "Line Numbers: Shown"
            } else {
                "Line Numbers: Hidden"
            };
            renderer.render_text(
                queue,
                fonts,
                verts,
                ln_label,
                lx + 14.0,
                ly + (lh - cell_h) / 2.0,
                lw - 20.0,
                theme.text_primary,
                false,
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
                title_x,
                verts,
                solid_uv,
                circle_uv,
                screen_w,
                screen_h,
            );
        }
    }
}
