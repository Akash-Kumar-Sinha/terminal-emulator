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

    // ── 1. Sidebar Background ────────────────────────────────────────────────
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

    // ── 3. Top Sidebar Toolbar (CHANGED FILES + Tree/List toggle) ────────────
    let tb_h = CodeReviewLayout::SIDEBAR_TOOLBAR_HEIGHT;
    let tb_bottom = sby + tb_h;

    // Toolbar header background
    push_quad(
        verts,
        sbx,
        sby,
        sbx + sbw,
        tb_bottom,
        solid_uv,
        solid_uv,
        theme.header_background,
        screen_w,
        screen_h,
    );

    // Bottom border below toolbar header
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

    // Section title & total files count pill
    let files_count = model.filtered_files().len();
    let title_text = "CHANGED FILES";
    let title_y = sby + (tb_h - ui_cell_h) * 0.5;
    renderer.render_text(
        queue,
        ui_fonts,
        verts,
        title_text,
        sbx + 12.0,
        title_y,
        sbw - 60.0,
        theme.text_secondary,
        true,
        screen_w,
        screen_h,
    );

    // Subtle count badge pill right after "CHANGED FILES"
    let title_w = title_text.len() as f32 * ui_fonts.cell_w;
    let count_str = format!("{files_count}");
    let count_w = count_str.len() as f32 * ui_fonts.cell_w + 8.0;
    let count_x = sbx + 12.0 + title_w + 6.0;
    if count_x + count_w < sbx + sbw - 38.0 {
        let count_y = sby + (tb_h - 16.0) * 0.5;
        push_rounded_rect(
            verts,
            count_x,
            count_y,
            count_x + count_w,
            count_y + 16.0,
            3.0,
            circle_uv,
            solid_uv,
            theme.header_button_hover_background,
            screen_w,
            screen_h,
        );
        renderer.render_text(
            queue,
            ui_fonts,
            verts,
            &count_str,
            count_x + 4.0,
            title_y,
            count_w,
            theme.text_muted,
            false,
            screen_w,
            screen_h,
        );
    }

    // Tree / Flat List View Toggle Button on the right of toolbar
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
        tx + (tw - ui_fonts.cell_w) * 0.5,
        ty + (th - ui_cell_h) * 0.5,
        tw,
        if is_toggle_hovered {
            theme.text_primary
        } else {
            theme.text_secondary
        },
        false,
        screen_w,
        screen_h,
    );

    // ── 4. Tree / Flat List Rows ──────────────────────────────────────────────
    let items = model.visible_tree_items();
    let row_h = CodeReviewLayout::FILE_ROW_HEIGHT;
    let list_viewport_h = (sbh - tb_h).max(0.0);
    let total_items_h = items.len() as f32 * row_h + 16.0;
    let max_sidebar_scroll = (total_items_h - list_viewport_h).max(0.0);
    let clamped_scroll_y = model.sidebar_scroll_y.clamp(0.0, max_sidebar_scroll);

    let list_top = tb_bottom;
    let mut row_y = list_top + 4.0 - clamped_scroll_y;

    // Fixed right-hand reservation for contextual actions/badges to prevent any layout shifting
    const ACTION_RESERVE: f32 = 86.0;

    for item in items {
        if row_y + row_h > list_top && row_y < sby + sbh {
            let y0 = row_y.max(list_top);
            let y1 = (row_y + row_h).min(sby + sbh);
            let text_y = row_y + (row_h - ui_cell_h) * 0.5;

            match item {
                CodeReviewTreeItem::SectionHeader {
                    is_staged,
                    count,
                    is_collapsed,
                } => {
                    let is_row_hovered = match hovered_action {
                        Some(CodeReviewAction::ToggleSectionCollapse(st)) => *st == is_staged,
                        Some(CodeReviewAction::UnstageAll) => is_staged,
                        Some(CodeReviewAction::StageAll)
                        | Some(CodeReviewAction::PromptDiscardAll) => !is_staged,
                        _ => false,
                    };

                    if is_row_hovered {
                        push_quad(
                            verts,
                            sbx + 2.0,
                            y0 + 1.0,
                            sbx + sbw - 3.0,
                            y1 - 1.0,
                            solid_uv,
                            solid_uv,
                            theme.code_review_sidebar_item_hover,
                            screen_w,
                            screen_h,
                        );
                    }

                    if text_y >= list_top && text_y + ui_cell_h <= sby + sbh {
                        // Section chevron
                        let chevron = if is_collapsed {
                            akssora_icons::ChevronRightIcon::GLYPH_SMALL
                        } else {
                            akssora_icons::ChevronDownIcon::GLYPH_SMALL
                        };
                        renderer.render_text(
                            queue,
                            ui_fonts,
                            verts,
                            chevron,
                            sbx + 8.0,
                            text_y,
                            14.0,
                            theme.text_secondary,
                            false,
                            screen_w,
                            screen_h,
                        );

                        // Section title
                        let header_title = if is_staged {
                            "STAGED CHANGES"
                        } else {
                            "CHANGES"
                        };
                        let title_w = header_title.len() as f32 * ui_fonts.cell_w;
                        renderer.render_text(
                            queue,
                            ui_fonts,
                            verts,
                            header_title,
                            sbx + 22.0,
                            text_y,
                            (sbw - ACTION_RESERVE - 24.0).max(30.0),
                            if is_row_hovered {
                                theme.text_primary
                            } else {
                                theme.text_secondary
                            },
                            true,
                            screen_w,
                            screen_h,
                        );

                        // Permanent count pill next to section label
                        let count_str = format!("{count}");
                        let count_badge_w = count_str.len() as f32 * ui_fonts.cell_w + 8.0;
                        let count_badge_x = sbx + 24.0 + title_w;
                        if count_badge_x + count_badge_w < sbx + sbw - ACTION_RESERVE {
                            let badge_pill_y = row_y + (row_h - 16.0) * 0.5;
                            push_rounded_rect(
                                verts,
                                count_badge_x,
                                badge_pill_y,
                                count_badge_x + count_badge_w,
                                badge_pill_y + 16.0,
                                3.0,
                                circle_uv,
                                solid_uv,
                                theme.header_button_hover_background,
                                screen_w,
                                screen_h,
                            );
                            renderer.render_text(
                                queue,
                                ui_fonts,
                                verts,
                                &count_str,
                                count_badge_x + 4.0,
                                text_y,
                                count_badge_w,
                                theme.text_muted,
                                false,
                                screen_w,
                                screen_h,
                            );
                        }

                        // Contextual action buttons on section header hover
                        if is_row_hovered {
                            let (first_rect, second_opt) =
                                CodeReviewLayout::sidebar_group_action_rects(
                                    sbx, sbw, row_y, is_staged,
                                );
                            if is_staged {
                                let is_unstage_hov =
                                    hovered_action == Some(&CodeReviewAction::UnstageAll);
                                if is_unstage_hov {
                                    push_rounded_rect(
                                        verts,
                                        first_rect.0,
                                        first_rect.1,
                                        first_rect.0 + first_rect.2,
                                        first_rect.1 + first_rect.3,
                                        3.0,
                                        circle_uv,
                                        solid_uv,
                                        theme.button_hover_background,
                                        screen_w,
                                        screen_h,
                                    );
                                }
                                renderer.render_text(
                                    queue,
                                    ui_fonts,
                                    verts,
                                    akssora_icons::RemoveIcon::GLYPH,
                                    first_rect.0 + (first_rect.2 - ui_fonts.cell_w) * 0.5,
                                    first_rect.1 + (first_rect.3 - ui_cell_h) * 0.5,
                                    first_rect.2,
                                    if is_unstage_hov {
                                        theme.text_primary
                                    } else {
                                        theme.text_secondary
                                    },
                                    false,
                                    screen_w,
                                    screen_h,
                                );
                            } else {
                                let is_revert_hov =
                                    hovered_action == Some(&CodeReviewAction::PromptDiscardAll);
                                if is_revert_hov {
                                    push_rounded_rect(
                                        verts,
                                        first_rect.0,
                                        first_rect.1,
                                        first_rect.0 + first_rect.2,
                                        first_rect.1 + first_rect.3,
                                        3.0,
                                        circle_uv,
                                        solid_uv,
                                        theme.button_hover_background,
                                        screen_w,
                                        screen_h,
                                    );
                                }
                                renderer.render_text(
                                    queue,
                                    ui_fonts,
                                    verts,
                                    akssora_icons::ArrowRevertIcon::GLYPH,
                                    first_rect.0 + (first_rect.2 - ui_fonts.cell_w) * 0.5,
                                    first_rect.1 + (first_rect.3 - ui_cell_h) * 0.5,
                                    first_rect.2,
                                    if is_revert_hov {
                                        theme.text_primary
                                    } else {
                                        theme.text_secondary
                                    },
                                    false,
                                    screen_w,
                                    screen_h,
                                );

                                if let Some(stage_rect) = second_opt {
                                    let is_stage_hov =
                                        hovered_action == Some(&CodeReviewAction::StageAll);
                                    if is_stage_hov {
                                        push_rounded_rect(
                                            verts,
                                            stage_rect.0,
                                            stage_rect.1,
                                            stage_rect.0 + stage_rect.2,
                                            stage_rect.1 + stage_rect.3,
                                            3.0,
                                            circle_uv,
                                            solid_uv,
                                            theme.button_hover_background,
                                            screen_w,
                                            screen_h,
                                        );
                                    }
                                    renderer.render_text(
                                        queue,
                                        ui_fonts,
                                        verts,
                                        akssora_icons::AddIcon::GLYPH,
                                        stage_rect.0 + (stage_rect.2 - ui_fonts.cell_w) * 0.5,
                                        stage_rect.1 + (stage_rect.3 - ui_cell_h) * 0.5,
                                        stage_rect.2,
                                        if is_stage_hov {
                                            theme.text_primary
                                        } else {
                                            theme.text_secondary
                                        },
                                        false,
                                        screen_w,
                                        screen_h,
                                    );
                                }
                            }
                        }
                    }
                }
                CodeReviewTreeItem::Directory {
                    is_staged,
                    path,
                    display_name,
                    depth,
                    is_collapsed,
                } => {
                    let is_row_hovered = match hovered_action {
                        Some(CodeReviewAction::ToggleTreeDir(p)) => p == &path,
                        Some(CodeReviewAction::UnstageDirectoryByPath(p)) => {
                            p == &path && is_staged
                        }
                        Some(CodeReviewAction::StageDirectoryByPath(p))
                        | Some(CodeReviewAction::PromptDiscardDirectory(p)) => {
                            p == &path && !is_staged
                        }
                        _ => false,
                    };

                    if is_row_hovered {
                        push_quad(
                            verts,
                            sbx + 2.0,
                            y0 + 1.0,
                            sbx + sbw - 3.0,
                            y1 - 1.0,
                            solid_uv,
                            solid_uv,
                            theme.code_review_sidebar_item_hover,
                            screen_w,
                            screen_h,
                        );
                    }

                    if text_y >= list_top && text_y + ui_cell_h <= sby + sbh {
                        let max_indent = (sbw - ACTION_RESERVE - 60.0).max(8.0);
                        let depth_indent = (depth as f32 * 14.0).min(max_indent);
                        let indent_x = sbx + 8.0 + depth_indent;

                        // Directory arrow toggle
                        let chevron = if is_collapsed {
                            akssora_icons::ChevronRightIcon::GLYPH_SMALL
                        } else {
                            akssora_icons::ChevronDownIcon::GLYPH_SMALL
                        };
                        renderer.render_text(
                            queue,
                            ui_fonts,
                            verts,
                            chevron,
                            indent_x,
                            text_y,
                            14.0,
                            theme.text_secondary,
                            false,
                            screen_w,
                            screen_h,
                        );

                        // Directory folder icon (consistent with File Explorer)
                        renderer.render_text(
                            queue,
                            ui_fonts,
                            verts,
                            akssora_icons::FolderIcon::GLYPH,
                            indent_x + 14.0,
                            text_y,
                            14.0,
                            theme.explorer_directory_icon,
                            false,
                            screen_w,
                            screen_h,
                        );

                        // Directory name aligned with file labels
                        let text_start_x = indent_x + 30.0;
                        let max_text_w = (sbx + sbw - text_start_x - ACTION_RESERVE).max(0.0);
                        renderer.render_text(
                            queue,
                            ui_fonts,
                            verts,
                            &display_name,
                            text_start_x,
                            text_y,
                            max_text_w,
                            if is_row_hovered {
                                theme.text_primary
                            } else {
                                theme.foreground
                            },
                            false,
                            screen_w,
                            screen_h,
                        );

                        // Directory contextual actions on hover
                        if is_row_hovered {
                            let (first_rect, second_opt) =
                                CodeReviewLayout::sidebar_group_action_rects(
                                    sbx, sbw, row_y, is_staged,
                                );
                            if is_staged {
                                let is_unstage_hov = hovered_action
                                    == Some(&CodeReviewAction::UnstageDirectoryByPath(
                                        path.clone(),
                                    ));
                                if is_unstage_hov {
                                    push_rounded_rect(
                                        verts,
                                        first_rect.0,
                                        first_rect.1,
                                        first_rect.0 + first_rect.2,
                                        first_rect.1 + first_rect.3,
                                        3.0,
                                        circle_uv,
                                        solid_uv,
                                        theme.button_hover_background,
                                        screen_w,
                                        screen_h,
                                    );
                                }
                                renderer.render_text(
                                    queue,
                                    ui_fonts,
                                    verts,
                                    akssora_icons::RemoveIcon::GLYPH,
                                    first_rect.0 + (first_rect.2 - ui_fonts.cell_w) * 0.5,
                                    first_rect.1 + (first_rect.3 - ui_cell_h) * 0.5,
                                    first_rect.2,
                                    if is_unstage_hov {
                                        theme.text_primary
                                    } else {
                                        theme.text_secondary
                                    },
                                    false,
                                    screen_w,
                                    screen_h,
                                );
                            } else {
                                let is_revert_hov = hovered_action
                                    == Some(&CodeReviewAction::PromptDiscardDirectory(
                                        path.clone(),
                                    ));
                                if is_revert_hov {
                                    push_rounded_rect(
                                        verts,
                                        first_rect.0,
                                        first_rect.1,
                                        first_rect.0 + first_rect.2,
                                        first_rect.1 + first_rect.3,
                                        3.0,
                                        circle_uv,
                                        solid_uv,
                                        theme.button_hover_background,
                                        screen_w,
                                        screen_h,
                                    );
                                }
                                renderer.render_text(
                                    queue,
                                    ui_fonts,
                                    verts,
                                    akssora_icons::ArrowRevertIcon::GLYPH,
                                    first_rect.0 + (first_rect.2 - ui_fonts.cell_w) * 0.5,
                                    first_rect.1 + (first_rect.3 - ui_cell_h) * 0.5,
                                    first_rect.2,
                                    if is_revert_hov {
                                        theme.text_primary
                                    } else {
                                        theme.text_secondary
                                    },
                                    false,
                                    screen_w,
                                    screen_h,
                                );

                                if let Some(stage_rect) = second_opt {
                                    let is_stage_hov = hovered_action
                                        == Some(&CodeReviewAction::StageDirectoryByPath(
                                            path.clone(),
                                        ));
                                    if is_stage_hov {
                                        push_rounded_rect(
                                            verts,
                                            stage_rect.0,
                                            stage_rect.1,
                                            stage_rect.0 + stage_rect.2,
                                            stage_rect.1 + stage_rect.3,
                                            3.0,
                                            circle_uv,
                                            solid_uv,
                                            theme.button_hover_background,
                                            screen_w,
                                            screen_h,
                                        );
                                    }
                                    renderer.render_text(
                                        queue,
                                        ui_fonts,
                                        verts,
                                        akssora_icons::AddIcon::GLYPH,
                                        stage_rect.0 + (stage_rect.2 - ui_fonts.cell_w) * 0.5,
                                        stage_rect.1 + (stage_rect.3 - ui_cell_h) * 0.5,
                                        stage_rect.2,
                                        if is_stage_hov {
                                            theme.text_primary
                                        } else {
                                            theme.text_secondary
                                        },
                                        false,
                                        screen_w,
                                        screen_h,
                                    );
                                }
                            }
                        }
                    }
                }
                CodeReviewTreeItem::File {
                    is_staged,
                    orig_idx,
                    path,
                    display_name,
                    depth,
                    status,
                    additions,
                    deletions,
                } => {
                    let full_path = model.repo_root.join(&path);
                    let is_active_editor = model.is_staged == is_staged
                        && (model.active_file_idx == Some(orig_idx)
                            || model.active_editing_file.as_ref() == Some(&full_path));

                    let is_selected = model.is_staged == is_staged
                        && (is_active_editor
                            || (model.active_editing_file.is_none()
                                && model.active_file_idx.is_none()
                                && model.selected_file_idx == orig_idx
                                && model.selected_file().map(|f| &f.path) == Some(&path)));

                    let is_row_hovered = match hovered_action {
                        Some(CodeReviewAction::SelectTreeFile {
                            path: p,
                            is_staged: st,
                            ..
                        }) => p == &path && *st == is_staged,
                        Some(CodeReviewAction::OpenFileInEditorByPath(p)) => p == &path,
                        Some(CodeReviewAction::UnstageFileByPath(p)) => p == &path && is_staged,
                        Some(CodeReviewAction::StageFileByPath(p))
                        | Some(CodeReviewAction::PromptDiscardFile(p))
                        | Some(CodeReviewAction::DiscardFileByPath(p)) => p == &path && !is_staged,
                        Some(CodeReviewAction::SelectFile(idx))
                        | Some(CodeReviewAction::SelectFileForEdit(idx)) => {
                            *idx == orig_idx && model.is_staged == is_staged
                        }
                        _ => false,
                    };

                    // Selection and hover backgrounds matching File Explorer
                    if is_selected || is_active_editor {
                        push_quad(
                            verts,
                            sbx + 2.0,
                            y0 + 1.0,
                            sbx + sbw - 3.0,
                            y1 - 1.0,
                            solid_uv,
                            solid_uv,
                            theme.code_review_sidebar_item_selected,
                            screen_w,
                            screen_h,
                        );
                    } else if is_row_hovered {
                        push_quad(
                            verts,
                            sbx + 2.0,
                            y0 + 1.0,
                            sbx + sbw - 3.0,
                            y1 - 1.0,
                            solid_uv,
                            solid_uv,
                            theme.code_review_sidebar_item_hover,
                            screen_w,
                            screen_h,
                        );
                    }

                    if text_y >= list_top && text_y + ui_cell_h <= sby + sbh {
                        let max_indent = (sbw - ACTION_RESERVE - 60.0).max(8.0);
                        let depth_indent = (depth as f32 * 14.0).min(max_indent);
                        let indent_x = sbx + 8.0 + depth_indent;

                        // File type icon aligned with folder icons (14px spacer precedes it)
                        let file_icon = akssora_icons::file_type_icon(&display_name);
                        renderer.render_text(
                            queue,
                            ui_fonts,
                            verts,
                            file_icon,
                            indent_x + 14.0,
                            text_y,
                            14.0,
                            theme.explorer_file_icon,
                            false,
                            screen_w,
                            screen_h,
                        );

                        // File name with stable width reservation (prevents shifting on hover)
                        let text_start_x = indent_x + 30.0;
                        let max_text_w = (sbx + sbw - text_start_x - ACTION_RESERVE).max(0.0);

                        let name_col = if is_selected || is_active_editor {
                            theme.text_primary
                        } else {
                            status.text_color()
                        };

                        renderer.render_text(
                            queue,
                            ui_fonts,
                            verts,
                            &display_name,
                            text_start_x,
                            text_y,
                            max_text_w,
                            name_col,
                            is_selected || is_active_editor,
                            screen_w,
                            screen_h,
                        );

                        // Right-hand area: action buttons when hovered, Git status badge & diff stats when idle
                        if is_row_hovered {
                            let (open_rect, mid_rect, stage_opt) =
                                CodeReviewLayout::sidebar_file_action_rects(
                                    sbx, sbw, row_y, is_staged,
                                );

                            // 1. Open File in Editor Action (↗)
                            let is_open_hov = hovered_action
                                == Some(&CodeReviewAction::OpenFileInEditorByPath(path.clone()));
                            if is_open_hov {
                                push_rounded_rect(
                                    verts,
                                    open_rect.0,
                                    open_rect.1,
                                    open_rect.0 + open_rect.2,
                                    open_rect.1 + open_rect.3,
                                    3.0,
                                    circle_uv,
                                    solid_uv,
                                    theme.button_hover_background,
                                    screen_w,
                                    screen_h,
                                );
                            }
                            renderer.render_text(
                                queue,
                                ui_fonts,
                                verts,
                                akssora_icons::ArrowUpRightIcon::GLYPH,
                                open_rect.0 + (open_rect.2 - ui_fonts.cell_w) * 0.5,
                                open_rect.1 + (open_rect.3 - ui_cell_h) * 0.5,
                                open_rect.2,
                                if is_open_hov {
                                    theme.text_primary
                                } else {
                                    theme.text_secondary
                                },
                                false,
                                screen_w,
                                screen_h,
                            );

                            if is_staged {
                                // 2. Unstage File Action (−)
                                let is_unstage_hov = hovered_action
                                    == Some(&CodeReviewAction::UnstageFileByPath(path.clone()));
                                if is_unstage_hov {
                                    push_rounded_rect(
                                        verts,
                                        mid_rect.0,
                                        mid_rect.1,
                                        mid_rect.0 + mid_rect.2,
                                        mid_rect.1 + mid_rect.3,
                                        3.0,
                                        circle_uv,
                                        solid_uv,
                                        theme.button_hover_background,
                                        screen_w,
                                        screen_h,
                                    );
                                }
                                renderer.render_text(
                                    queue,
                                    ui_fonts,
                                    verts,
                                    akssora_icons::RemoveIcon::GLYPH,
                                    mid_rect.0 + (mid_rect.2 - ui_fonts.cell_w) * 0.5,
                                    mid_rect.1 + (mid_rect.3 - ui_cell_h) * 0.5,
                                    mid_rect.2,
                                    if is_unstage_hov {
                                        theme.text_primary
                                    } else {
                                        theme.text_secondary
                                    },
                                    false,
                                    screen_w,
                                    screen_h,
                                );
                            } else {
                                // 2. Revert / Discard Action (↺)
                                let is_revert_hov = hovered_action
                                    == Some(&CodeReviewAction::PromptDiscardFile(path.clone()))
                                    || hovered_action
                                        == Some(&CodeReviewAction::DiscardFileByPath(path.clone()));
                                if is_revert_hov {
                                    push_rounded_rect(
                                        verts,
                                        mid_rect.0,
                                        mid_rect.1,
                                        mid_rect.0 + mid_rect.2,
                                        mid_rect.1 + mid_rect.3,
                                        3.0,
                                        circle_uv,
                                        solid_uv,
                                        theme.button_hover_background,
                                        screen_w,
                                        screen_h,
                                    );
                                }
                                renderer.render_text(
                                    queue,
                                    ui_fonts,
                                    verts,
                                    akssora_icons::ArrowRevertIcon::GLYPH,
                                    mid_rect.0 + (mid_rect.2 - ui_fonts.cell_w) * 0.5,
                                    mid_rect.1 + (mid_rect.3 - ui_cell_h) * 0.5,
                                    mid_rect.2,
                                    if is_revert_hov {
                                        theme.text_primary
                                    } else {
                                        theme.text_secondary
                                    },
                                    false,
                                    screen_w,
                                    screen_h,
                                );

                                // 3. Stage Action (+)
                                if let Some(stage_rect) = stage_opt {
                                    let is_stage_hov = hovered_action
                                        == Some(&CodeReviewAction::StageFileByPath(path.clone()));
                                    if is_stage_hov {
                                        push_rounded_rect(
                                            verts,
                                            stage_rect.0,
                                            stage_rect.1,
                                            stage_rect.0 + stage_rect.2,
                                            stage_rect.1 + stage_rect.3,
                                            3.0,
                                            circle_uv,
                                            solid_uv,
                                            theme.button_hover_background,
                                            screen_w,
                                            screen_h,
                                        );
                                    }
                                    renderer.render_text(
                                        queue,
                                        ui_fonts,
                                        verts,
                                        akssora_icons::AddIcon::GLYPH,
                                        stage_rect.0 + (stage_rect.2 - ui_fonts.cell_w) * 0.5,
                                        stage_rect.1 + (stage_rect.3 - ui_cell_h) * 0.5,
                                        stage_rect.2,
                                        if is_stage_hov {
                                            theme.text_primary
                                        } else {
                                            theme.text_secondary
                                        },
                                        false,
                                        screen_w,
                                        screen_h,
                                    );
                                }
                            }
                        } else {
                            // Non-hovered state: Git status indicator badge on the far right
                            let badge_x = sbx + sbw - 20.0;
                            let badge_str = status.badge_label();
                            renderer.render_text(
                                queue,
                                ui_fonts,
                                verts,
                                badge_str,
                                badge_x,
                                text_y,
                                14.0,
                                status.badge_color(),
                                true,
                                screen_w,
                                screen_h,
                            );

                            // Additions / deletions stats immediately to the left of the badge
                            if additions > 0 && deletions > 0 {
                                let add_str = format!("+{additions}");
                                let del_str = format!("-{deletions}");
                                let add_w = add_str.len() as f32 * ui_fonts.cell_w;
                                let del_w = del_str.len() as f32 * ui_fonts.cell_w;
                                let space_w = ui_fonts.cell_w * 0.5;
                                let total_stats_w = add_w + space_w + del_w;
                                let stats_x = badge_x - total_stats_w - 6.0;

                                if stats_x > text_start_x + 20.0 {
                                    renderer.render_text(
                                        queue,
                                        ui_fonts,
                                        verts,
                                        &add_str,
                                        stats_x,
                                        text_y,
                                        add_w + 1.0,
                                        theme.diff_add_green,
                                        false,
                                        screen_w,
                                        screen_h,
                                    );
                                    renderer.render_text(
                                        queue,
                                        ui_fonts,
                                        verts,
                                        &del_str,
                                        stats_x + add_w + space_w,
                                        text_y,
                                        del_w + 1.0,
                                        theme.diff_delete_red,
                                        false,
                                        screen_w,
                                        screen_h,
                                    );
                                }
                            } else if additions > 0 {
                                let add_str = format!("+{additions}");
                                let add_w = add_str.len() as f32 * ui_fonts.cell_w;
                                let stats_x = badge_x - add_w - 6.0;
                                if stats_x > text_start_x + 20.0 {
                                    renderer.render_text(
                                        queue,
                                        ui_fonts,
                                        verts,
                                        &add_str,
                                        stats_x,
                                        text_y,
                                        add_w + 1.0,
                                        theme.diff_add_green,
                                        false,
                                        screen_w,
                                        screen_h,
                                    );
                                }
                            } else if deletions > 0 {
                                let del_str = format!("-{deletions}");
                                let del_w = del_str.len() as f32 * ui_fonts.cell_w;
                                let stats_x = badge_x - del_w - 6.0;
                                if stats_x > text_start_x + 20.0 {
                                    renderer.render_text(
                                        queue,
                                        ui_fonts,
                                        verts,
                                        &del_str,
                                        stats_x,
                                        text_y,
                                        del_w + 1.0,
                                        theme.diff_delete_red,
                                        false,
                                        screen_w,
                                        screen_h,
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }
        row_y += row_h;
    }
}
