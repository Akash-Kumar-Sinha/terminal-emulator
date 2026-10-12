use akssora_terminal_config::{Action, ConflictWarning, Keybindings, Theme};
use akssora_terminal_ui::{SettingsAction, SettingsView};

use crate::font::FontManager;
use crate::glyph::GlyphRenderer;
use crate::quad::push_rounded_rect;
use crate::vertex::GlyphVertex;

// INFO: Renders the searchable keyboard shortcuts panel with category/scope indicators and conflict warnings.
#[allow(clippy::too_many_arguments)]
pub fn render_shortcuts_section(
    renderer: &mut GlyphRenderer,
    queue: &wgpu::Queue,
    fonts: &FontManager,
    theme: &Theme,
    keybindings: &Keybindings,
    search_query: &str,
    hovered_action: Option<SettingsAction>,
    conflicts: &[ConflictWarning],
    workspace_x: f32,
    workspace_y: f32,
    workspace_w: f32,
    workspace_h: f32,
    title_x: f32,
    verts: &mut Vec<GlyphVertex>,
    solid_uv: [f32; 2],
    circle_uv: [[f32; 2]; 2],
    screen_w: f32,
    screen_h: f32,
) {
    let cell_h = fonts.cell_h;

    // 1. Search Box
    let (sx, sy, sw, sh) =
        SettingsView::shortcuts_search_rect(workspace_x, workspace_y, workspace_w, cell_h);
    let search_bg = if hovered_action == Some(SettingsAction::KeybindingSearchClick) {
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
        search_bg,
        screen_w,
        screen_h,
    );

    let search_text = if search_query.is_empty() {
        "Type to filter keybindings...".to_string()
    } else {
        format!("Search: {search_query}")
    };
    let search_col = if search_query.is_empty() {
        theme.text_muted
    } else {
        theme.text_primary
    };
    renderer.render_text(
        queue,
        fonts,
        verts,
        &search_text,
        sx + 10.0,
        sy + (sh - cell_h) / 2.0,
        sw - 20.0,
        search_col,
        false,
        screen_w,
        screen_h,
    );

    // 2. Reset All Button
    let (rx, ry, rw, rh) =
        SettingsView::shortcuts_reset_rect(workspace_x, workspace_y, workspace_w, cell_h);
    let reset_bg = if hovered_action == Some(SettingsAction::KeybindingResetAll) {
        theme.button_hover_background
    } else {
        theme.session_background
    };
    push_rounded_rect(
        verts,
        rx,
        ry,
        rx + rw,
        ry + rh,
        4.0,
        circle_uv,
        solid_uv,
        reset_bg,
        screen_w,
        screen_h,
    );
    renderer.render_text(
        queue,
        fonts,
        verts,
        "Reset All",
        rx + 16.0,
        ry + (rh - cell_h) / 2.0,
        rw - 20.0,
        theme.text_secondary,
        false,
        screen_w,
        screen_h,
    );

    // 3. Conflict Warnings Banner
    let mut banner_offset_y = 0.0;
    if !conflicts.is_empty() {
        let warn_y = sy + sh + 10.0;
        let warn_w = (workspace_w - 80.0).clamp(300.0, 600.0);
        let warn_h = (cell_h + 8.0).max(24.0);
        push_rounded_rect(
            verts,
            sx,
            warn_y,
            sx + warn_w,
            warn_y + warn_h,
            4.0,
            circle_uv,
            solid_uv,
            [0.35, 0.15, 0.05, 1.0], // subtle amber warning bg
            screen_w,
            screen_h,
        );
        let warn_msg = format!(
            "Conflict: \"{}\" mapped to multiple actions in {} scope!",
            conflicts[0].shortcut,
            conflicts[0].scope.label()
        );
        renderer.render_text(
            queue,
            fonts,
            verts,
            &warn_msg,
            sx + 10.0,
            warn_y + (warn_h - cell_h) / 2.0,
            warn_w - 20.0,
            [1.0, 0.8, 0.4, 1.0],
            true,
            screen_w,
            screen_h,
        );
        banner_offset_y = warn_h + 10.0;
    }

    // 4. Keybinding Rows
    let query_lower = search_query.to_lowercase();
    let mut visible_idx = 0;
    for &act in Action::ALL {
        let desc = act.description();
        let shortcut = keybindings.shortcut_for(act).unwrap_or("Unassigned");
        let scope_lbl = act.scope().label();

        if !query_lower.is_empty()
            && !desc.to_lowercase().contains(&query_lower)
            && !shortcut.to_lowercase().contains(&query_lower)
            && !scope_lbl.to_lowercase().contains(&query_lower)
        {
            continue;
        }

        let row_y =
            SettingsView::shortcuts_row_y(workspace_y, cell_h, visible_idx) + banner_offset_y;
        if row_y + cell_h > workspace_y + workspace_h - 10.0 {
            break;
        }

        // Highlight row if rebind target hovered
        if hovered_action == Some(SettingsAction::KeybindingRebind(act)) {
            push_rounded_rect(
                verts,
                title_x,
                row_y - 2.0,
                title_x + 560.0,
                row_y + cell_h + 4.0,
                4.0,
                circle_uv,
                solid_uv,
                theme.session_hover_background,
                screen_w,
                screen_h,
            );
        }

        // Shortcut box
        let is_conflict = conflicts
            .iter()
            .any(|c| c.first_action == act || c.second_action == act);
        let key_col = if is_conflict {
            [1.0, 0.4, 0.4, 1.0] // Warning Red
        } else {
            theme.text_primary
        };

        renderer.render_text(
            queue,
            fonts,
            verts,
            shortcut,
            title_x + 8.0,
            row_y,
            160.0,
            key_col,
            true,
            screen_w,
            screen_h,
        );

        renderer.render_text(
            queue,
            fonts,
            verts,
            desc,
            title_x + 180.0,
            row_y,
            240.0,
            theme.text_secondary,
            false,
            screen_w,
            screen_h,
        );

        // Scope badge
        renderer.render_text(
            queue,
            fonts,
            verts,
            scope_lbl,
            title_x + 440.0,
            row_y,
            100.0,
            theme.text_muted,
            false,
            screen_w,
            screen_h,
        );

        visible_idx += 1;
    }
}
