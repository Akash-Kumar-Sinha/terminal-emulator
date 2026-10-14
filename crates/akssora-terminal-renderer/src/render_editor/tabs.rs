use akssora_editor::{Document, MarkdownViewMode};
use akssora_terminal_config::Theme;
use akssora_terminal_ui::{EditorFileTab, EditorTabBar, EditorToolbarAction};

use crate::font::FontManager;
use crate::glyph::GlyphRenderer;
use crate::quad::{push_quad, push_rounded_rect};
use crate::vertex::GlyphVertex;

// INFO: Renders the fixed top file-tab bar with tabs, close buttons, and markdown/diff toolbar icons.
#[allow(clippy::too_many_arguments)]
pub fn render_editor_tabs(
    renderer: &mut GlyphRenderer,
    queue: &wgpu::Queue,
    ui_fonts: &FontManager,
    theme: &Theme,
    doc: &Document,
    file_tabs: &[EditorFileTab],
    hovered_tab: Option<usize>,
    hovered_close: Option<usize>,
    hovered_toolbar: Option<EditorToolbarAction>,
    ws_x: f32,
    ws_y: f32,
    ws_w: f32,
    tab_bar_h: f32,
    verts: &mut Vec<GlyphVertex>,
    solid_uv: [f32; 2],
    circle_uv: [[f32; 2]; 2],
    screen_w: f32,
    screen_h: f32,
) {
    if tab_bar_h <= 0.0 {
        return;
    }

    // Tab bar container background
    push_quad(
        verts,
        ws_x,
        ws_y,
        ws_x + ws_w,
        ws_y + tab_bar_h,
        solid_uv,
        solid_uv,
        theme.sidebar_background,
        screen_w,
        screen_h,
    );

    for (i, tab) in file_tabs.iter().enumerate() {
        let (tx, ty, tw, th) = EditorTabBar::tab_rect(i, file_tabs.len(), ws_x, ws_y, ws_w);

        let tab_bg = if tab.is_active {
            theme.session_selected_background
        } else if hovered_tab == Some(i) {
            theme.session_hover_background
        } else {
            [0.0, 0.0, 0.0, 0.0]
        };

        // INFO: Highlight selected file tab with rounded pill inspired by sidebar session styling
        if tab.is_active || hovered_tab == Some(i) {
            push_rounded_rect(
                verts,
                tx + 2.0,
                ty + 2.0,
                tx + tw - 2.0,
                ty + th - 2.0,
                4.0,
                circle_uv,
                solid_uv,
                tab_bg,
                screen_w,
                screen_h,
            );
        }

        // Tab label with file icon, unsaved change indicator, preview style, pin indicator
        let mut label = String::new();
        if tab.is_dirty {
            label.push_str(akssora_icons::DirtyCircleIcon::GLYPH_SPACE);
        }
        if tab.is_pinned {
            label.push_str(akssora_icons::PinIcon::GLYPH_SPACE);
        }
        let icon = akssora_icons::file_type_icon(&tab.title);
        label.push_str(icon);
        label.push(' ');
        label.push_str(&tab.title);
        if tab.is_preview {
            label.push_str(" (preview)");
        }

        let text_col = if tab.is_active {
            theme.text_primary
        } else {
            theme.text_secondary
        };

        let badge_reserve = if tab.git_status.is_some() { 16.0 } else { 0.0 };
        let label_max_w = (tw - 32.0 - badge_reserve).max(10.0);
        renderer.render_text(
            queue,
            ui_fonts,
            verts,
            &label,
            tx + 8.0,
            ty + (th - ui_fonts.cell_h) * 0.5,
            label_max_w,
            text_col,
            false,
            screen_w,
            screen_h,
        );

        // Git badge if available
        if let Some(status) = &tab.git_status {
            let badge_x = tx + tw - 32.0;
            renderer.render_text(
                queue,
                ui_fonts,
                verts,
                status.badge_label(),
                badge_x,
                ty + (th - ui_fonts.cell_h) * 0.5,
                14.0,
                status.badge_color(),
                true,
                screen_w,
                screen_h,
            );
        }

        // Individual close button (pinned tabs omit close button or show subtle pin)
        if !tab.is_pinned {
            let (cx, cy, cw, ch) = EditorTabBar::close_rect(i, file_tabs.len(), ws_x, ws_y, ws_w);
            if hovered_close == Some(i) {
                push_rounded_rect(
                    verts,
                    cx,
                    cy,
                    cx + cw,
                    cy + ch,
                    3.0,
                    circle_uv,
                    solid_uv,
                    theme.close_button_hover_background,
                    screen_w,
                    screen_h,
                );
            }

            let close_col = if hovered_close == Some(i) {
                theme.close_button_foreground
            } else if tab.is_active {
                theme.text_secondary
            } else {
                theme.text_muted
            };
            renderer.render_text(
                queue,
                ui_fonts,
                verts,
                akssora_icons::CloseIcon::GLYPH,
                cx + 3.0,
                cy + (ch - ui_fonts.cell_h) * 0.5,
                cw,
                close_col,
                false,
                screen_w,
                screen_h,
            );
        }
    }

    // INFO: Editor toolbar controls rendered on the right side of the tab bar
    let mut actions = Vec::new();
    if doc.is_markdown() {
        actions.push((
            EditorToolbarAction::TogglePreview,
            akssora_icons::PreviewIcon::GLYPH,
            doc.markdown_mode == MarkdownViewMode::Preview,
        ));
        actions.push((
            EditorToolbarAction::ToggleSplitPreview,
            akssora_icons::SplitPreviewIcon::GLYPH,
            doc.markdown_mode == MarkdownViewMode::Split,
        ));
    }
    actions.push((
        EditorToolbarAction::ToggleInlineDiff,
        akssora_icons::InlineDiffIcon::GLYPH,
        doc.inline_diff,
    ));

    for (action, icon_text, is_selected) in actions {
        let (bx, by, bw, bh) = EditorTabBar::toolbar_rect(action, ws_x, ws_y, ws_w);
        let is_hovered = hovered_toolbar == Some(action);

        let btn_bg = if is_selected {
            theme.session_selected_background
        } else if is_hovered {
            theme.header_button_hover_background
        } else {
            [0.0, 0.0, 0.0, 0.0]
        };

        if is_selected || is_hovered {
            push_rounded_rect(
                verts,
                bx,
                by,
                bx + bw,
                by + bh,
                3.0,
                circle_uv,
                solid_uv,
                btn_bg,
                screen_w,
                screen_h,
            );
        }

        let icon_col = if is_selected {
            theme.text_primary
        } else if is_hovered {
            theme.header_foreground
        } else {
            theme.text_secondary
        };

        let text_x = bx + (bw - ui_fonts.cell_w * icon_text.len() as f32) * 0.5;
        let text_y = by + (bh - ui_fonts.cell_h) * 0.5;
        renderer.render_text(
            queue,
            ui_fonts,
            verts,
            icon_text,
            text_x,
            text_y,
            bw,
            icon_col,
            is_selected,
            screen_w,
            screen_h,
        );
    }
}
