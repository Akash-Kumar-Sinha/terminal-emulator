use akssora_terminal_config::Theme;
use akssora_terminal_ui::{
    CodeReviewAction, CodeReviewLayout, CodeReviewModel, CodeReviewSidebarMode, CodeReviewTreeItem,
};

use crate::font::FontManager;
use crate::glyph::GlyphRenderer;
use crate::quad::{push_quad, push_rounded_rect};
use crate::vertex::GlyphVertex;

// INFO: Renders the left sidebar containing the explorer-like tree/list of modified files, git status indicators, and toolbar.
#[allow(clippy::too_many_arguments)]
pub fn render_sidebar(
    renderer: &mut GlyphRenderer,
    queue: &wgpu::Queue,
    ui_fonts: &FontManager,
    theme: &Theme,
    model: &CodeReviewModel,
    hovered_action: Option<&CodeReviewAction>,
    ws_x: f32,
    ws_y: f32,
    ws_w: f32,
    ws_h: f32,
    verts: &mut Vec<GlyphVertex>,
    solid_uv: [f32; 2],
    circle_uv: [[f32; 2]; 2],
    screen_w: f32,
    screen_h: f32,
) {
    let Some((sbx, sby, sbw, sbh)) = CodeReviewLayout::sidebar_rect(model, ws_x, ws_y, ws_w, ws_h)
    else {
        return;
    };

    let ui_cell_h = ui_fonts.cell_h;

    // Sidebar Background (strictly within sby..sby + sbh)
    push_quad(
        verts,
        sbx,
        sby,
        sbx + sbw,
        sby + sbh,
        solid_uv,
        solid_uv,
        theme.code_review_sidebar_background,
        screen_w,
        screen_h,
    );

    // Sidebar Right Divider
    let divider_col = if model.hovered_sidebar_edge || model.is_resizing_sidebar {
        theme.header_button_active_background
    } else {
        theme.code_review_sidebar_border
    };
    push_quad(
        verts,
        sbx + sbw - 1.0,
        sby,
        sbx + sbw,
        sby + sbh,
        solid_uv,
        solid_uv,
        divider_col,
        screen_w,
        screen_h,
    );

    // ── 1. Top Sidebar Toolbar (CHANGED FILES label + Tree/List view toggle) ───
    let tb_h = CodeReviewLayout::SIDEBAR_TOOLBAR_HEIGHT;
    let tb_bottom = sby + tb_h;

    // Toolbar background
    push_quad(
        verts,
        sbx,
        sby,
        sbx + sbw,
        tb_bottom,
        solid_uv,
        solid_uv,
        theme.sidebar_panel_background,
        screen_w,
        screen_h,
    );
    // Divider below toolbar
    push_quad(
        verts,
        sbx,
        tb_bottom - 1.0,
        sbx + sbw,
        tb_bottom,
        solid_uv,
        solid_uv,
        theme.border_subtle,
        screen_w,
        screen_h,
    );

    // Section title: "CHANGED FILES ({count})"
    let files_count = model.filtered_files().len();
    let title_text = format!("CHANGED FILES ({files_count})");
    let title_y = sby + (tb_h - ui_cell_h) * 0.5;
    renderer.render_text(
        queue,
        ui_fonts,
        verts,
        &title_text,
        sbx + 12.0,
        title_y,
        sbw - 60.0,
        theme.text_secondary,
        true,
        screen_w,
        screen_h,
    );

    // Tree / List View Toggle Button on the right of toolbar
    let (tx, ty, tw, th) = CodeReviewLayout::sidebar_mode_toggle_rect(sbx, sby, sbw);
    let is_toggle_hovered = hovered_action == Some(&CodeReviewAction::ToggleSidebarMode);
    if is_toggle_hovered {
        push_rounded_rect(
            verts,
            tx,
            ty,
            tx + tw,
            ty + th,
            3.0,
            circle_uv,
            solid_uv,
            theme.button_hover_background,
            screen_w,
            screen_h,
        );
    }
    let mode_icon = match model.sidebar_mode {
        CodeReviewSidebarMode::Tree => akssora_icons::SidebarListIcon::GLYPH,
        CodeReviewSidebarMode::FlatList => akssora_icons::SidebarTreeIcon::GLYPH,
    };
    renderer.render_text(
        queue,
        ui_fonts,
        verts,
        mode_icon,
        tx + 6.0,
        ty + (th - ui_cell_h) * 0.5,
        14.0,
        if is_toggle_hovered {
            theme.text_primary
        } else {
            theme.text_secondary
        },
        false,
        screen_w,
        screen_h,
    );

    // ── 2. Tree / Flat List Items ─────────────────────────────────────────────
    let items = model.visible_tree_items();
    let total_items_h = items.len() as f32 * CodeReviewLayout::FILE_ROW_HEIGHT + 16.0;
    let list_viewport_h = (sbh - tb_h).max(0.0);
    let max_sidebar_scroll = (total_items_h - list_viewport_h).max(0.0);
    let clamped_scroll_y = model.sidebar_scroll_y.clamp(0.0, max_sidebar_scroll);

    let list_top = tb_bottom + 4.0;
    let mut row_y = list_top - clamped_scroll_y;
    let row_h = CodeReviewLayout::FILE_ROW_HEIGHT;

    for item in items {
        if row_y + row_h > list_top && row_y < sby + sbh {
            let y0 = row_y.max(list_top);
            let y1 = (row_y + row_h).min(sby + sbh);

            match item {
                CodeReviewTreeItem::Directory {
                    path,
                    display_name,
                    depth,
                    is_collapsed,
                } => {
                    let is_hovered =
                        hovered_action == Some(&CodeReviewAction::ToggleTreeDir(path.clone()));
                    if is_hovered {
                        push_rounded_rect(
                            verts,
                            sbx + 4.0,
                            y0 + 1.0,
                            sbx + sbw - 4.0,
                            y1 - 1.0,
                            3.0,
                            circle_uv,
                            solid_uv,
                            theme.code_review_sidebar_item_hover,
                            screen_w,
                            screen_h,
                        );
                    }

                    let text_y = row_y + (row_h - ui_cell_h) * 0.5;
                    if text_y >= list_top && text_y + ui_cell_h <= sby + sbh {
                        let indent_x = sbx + 8.0 + depth as f32 * 14.0;
                        let chevron = if is_collapsed {
                            akssora_icons::ChevronRightIcon::GLYPH_LARGE
                        } else {
                            akssora_icons::ChevronDownIcon::GLYPH_LARGE
                        };
                        renderer.render_text(
                            queue,
                            ui_fonts,
                            verts,
                            chevron,
                            indent_x,
                            text_y,
                            12.0,
                            theme.text_muted,
                            false,
                            screen_w,
                            screen_h,
                        );

                        let folder_icon = akssora_icons::FolderIcon::FOLDER_EMOJI;
                        renderer.render_text(
                            queue,
                            ui_fonts,
                            verts,
                            folder_icon,
                            indent_x + 14.0,
                            text_y,
                            16.0,
                            theme.text_secondary,
                            false,
                            screen_w,
                            screen_h,
                        );

                        renderer.render_text(
                            queue,
                            ui_fonts,
                            verts,
                            &display_name,
                            indent_x + 32.0,
                            text_y,
                            (sbw - (indent_x - sbx) - 40.0).max(30.0),
                            theme.text_primary,
                            true,
                            screen_w,
                            screen_h,
                        );
                    }
                }
                CodeReviewTreeItem::File {
                    orig_idx,
                    path: _,
                    display_name,
                    depth,
                } => {
                    let is_selected = orig_idx == model.selected_file_idx;
                    let is_hovered = hovered_action
                        == Some(&CodeReviewAction::SelectFile(orig_idx))
                        || hovered_action == Some(&CodeReviewAction::SelectFileForEdit(orig_idx));
                    let row_bg = if is_selected {
                        theme.code_review_sidebar_item_selected
                    } else if is_hovered {
                        theme.code_review_sidebar_item_hover
                    } else {
                        [0.0, 0.0, 0.0, 0.0]
                    };

                    if is_selected || is_hovered {
                        push_rounded_rect(
                            verts,
                            sbx + 4.0,
                            y0 + 1.0,
                            sbx + sbw - 4.0,
                            y1 - 1.0,
                            4.0,
                            circle_uv,
                            solid_uv,
                            row_bg,
                            screen_w,
                            screen_h,
                        );
                    }

                    let text_y = row_y + (row_h - ui_cell_h) * 0.5;
                    if text_y >= list_top && text_y + ui_cell_h <= sby + sbh {
                        let indent_x = sbx + 8.0 + depth as f32 * 14.0;
                        let file = model.diff.files.get(orig_idx);

                        // Status indicator
                        let (status_icon, status_col) = match file.map(|f| f.status) {
                            Some(akssora_git::FileGitStatus::Modified) => {
                                ("M", theme.diff_modified_yellow)
                            }
                            Some(akssora_git::FileGitStatus::Added)
                            | Some(akssora_git::FileGitStatus::Untracked) => {
                                ("A", theme.diff_add_green)
                            }
                            Some(akssora_git::FileGitStatus::Deleted) => {
                                ("D", theme.diff_delete_red)
                            }
                            _ => ("•", theme.text_muted),
                        };

                        renderer.render_text(
                            queue,
                            ui_fonts,
                            verts,
                            status_icon,
                            indent_x + 4.0,
                            text_y,
                            14.0,
                            status_col,
                            false,
                            screen_w,
                            screen_h,
                        );

                        // File name
                        let name_w = (sbw - (indent_x - sbx) - 70.0).max(30.0);
                        renderer.render_text(
                            queue,
                            ui_fonts,
                            verts,
                            &display_name,
                            indent_x + 22.0,
                            text_y,
                            name_w,
                            if is_selected {
                                theme.text_primary
                            } else {
                                theme.text_secondary
                            },
                            false,
                            screen_w,
                            screen_h,
                        );

                        // Stats count pill on the right of the row
                        if let Some(f) = file {
                            let stats_str = format!("+{} -{}", f.additions, f.deletions);
                            renderer.render_text(
                                queue,
                                ui_fonts,
                                verts,
                                &stats_str,
                                sbx + sbw - 56.0,
                                text_y,
                                50.0,
                                theme.text_muted,
                                false,
                                screen_w,
                                screen_h,
                            );
                        }
                    }
                }
            }
        }
        row_y += row_h;
    }
}
