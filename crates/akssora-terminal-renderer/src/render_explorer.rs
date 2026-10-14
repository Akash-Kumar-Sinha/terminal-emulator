use akssora_file_explorer::EntryKind;
use akssora_terminal_config::Theme;
use akssora_terminal_ui::{
    ExplorerContextAction, ExplorerContextMenu, ExplorerUi, InlineInputMode,
};

use crate::font::FontManager;
use crate::glyph::GlyphRenderer;
use crate::quad::{push_pill_thumb, push_quad, push_rounded_rect};
use crate::vertex::GlyphVertex;
// INFO: Renders the hierarchical left-side file explorer panel including tree rows, toolbar, inline input, context menu, and confirmation dialogs.
#[allow(clippy::too_many_arguments)]
pub fn render_file_explorer_panel(
    renderer: &mut GlyphRenderer,
    queue: &wgpu::Queue,
    fonts: &FontManager,
    theme: &Theme,
    explorer: &ExplorerUi,
    explorer_x: f32,
    top_offset: f32,
    verts: &mut Vec<GlyphVertex>,
    solid_uv: [f32; 2],
    circle_uv: [[f32; 2]; 2],
    screen_w: f32,
    screen_h: f32,
) {
    if !explorer.is_expanded || explorer.width <= 0.0 {
        return;
    }

    let exp_w = explorer.width;

    // 1. Panel Background
    push_quad(
        verts,
        explorer_x,
        top_offset,
        explorer_x + exp_w,
        screen_h,
        solid_uv,
        solid_uv,
        theme.sidebar_background,
        screen_w,
        screen_h,
    );

    // INFO: 2. Toolbar Header (borderless design)
    let header_h = ExplorerUi::HEADER_HEIGHT;
    push_quad(
        verts,
        explorer_x,
        top_offset,
        explorer_x + exp_w,
        top_offset + header_h,
        solid_uv,
        solid_uv,
        theme.header_background,
        screen_w,
        screen_h,
    );

    // Root folder name
    let root_name = explorer
        .tree
        .root_path
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "WORKSPACE".to_string())
        .to_uppercase();

    let root_label = root_name;
    let text_y = top_offset + (header_h - fonts.cell_h) * 0.5;
    renderer.render_text(
        queue,
        fonts,
        verts,
        &root_label,
        explorer_x + 8.0,
        text_y,
        exp_w - 16.0,
        theme.text_secondary,
        true,
        screen_w,
        screen_h,
    );

    // 3. Tree Rows
    let rows = explorer.tree.flatten_rows();
    let viewport_h = (screen_h - top_offset - header_h).max(0.0);
    let total_content_h = rows.len() as f32 * ExplorerUi::ITEM_HEIGHT;
    let max_scroll = (total_content_h - viewport_h).max(0.0);
    let scroll_y = explorer.scroll_y.clamp(0.0, max_scroll);
    let content_start_y = top_offset + header_h - scroll_y;
    let item_h = ExplorerUi::ITEM_HEIGHT;

    for (idx, row) in rows.iter().enumerate() {
        let row_y = content_start_y + idx as f32 * item_h;
        if row_y + item_h < top_offset + header_h || row_y > screen_h {
            continue;
        }

        // Selection, Active editor file & Hover highlights
        let is_drop_target = explorer.tree.drop_target.as_ref() == Some(&row.path);
        let is_active_editor = explorer.active_editor_file.as_deref() == Some(&row.path);
        let is_selected = row.is_selected;
        let is_hovered = explorer.hovered_row == Some(idx);

        if is_drop_target {
            push_quad(
                verts,
                explorer_x + 2.0,
                row_y + 1.0,
                explorer_x + exp_w - 3.0,
                row_y + item_h - 1.0,
                solid_uv,
                solid_uv,
                theme.drop_highlight,
                screen_w,
                screen_h,
            );
        } else if is_active_editor {
            push_quad(
                verts,
                explorer_x + 2.0,
                row_y + 1.0,
                explorer_x + exp_w - 3.0,
                row_y + item_h - 1.0,
                solid_uv,
                solid_uv,
                theme.session_selected_background,
                screen_w,
                screen_h,
            );
        } else if is_selected {
            push_quad(
                verts,
                explorer_x + 2.0,
                row_y + 1.0,
                explorer_x + exp_w - 3.0,
                row_y + item_h - 1.0,
                solid_uv,
                solid_uv,
                theme.selection_background,
                screen_w,
                screen_h,
            );
        } else if is_hovered {
            push_quad(
                verts,
                explorer_x + 2.0,
                row_y + 1.0,
                explorer_x + exp_w - 3.0,
                row_y + item_h - 1.0,
                solid_uv,
                solid_uv,
                theme.tab_hover_background,
                screen_w,
                screen_h,
            );
        }

        // Git status lookup for this row
        let git_badge = explorer
            .git_status
            .as_ref()
            .and_then(|gs| gs.file_status(&explorer.tree.root_path, &row.path));

        // Indentation
        let indent = row.depth as f32 * 14.0 + 8.0;

        // Directory arrow toggle
        let mut cur_x = explorer_x + indent;
        if row.kind == EntryKind::Directory {
            let arrow = if row.is_expanded {
                akssora_icons::ChevronDownIcon::GLYPH_SMALL
            } else {
                akssora_icons::ChevronRightIcon::GLYPH_SMALL
            };
            renderer.render_text(
                queue,
                fonts,
                verts,
                arrow,
                cur_x,
                row_y + (item_h - fonts.cell_h) * 0.5,
                14.0,
                theme.text_secondary,
                false,
                screen_w,
                screen_h,
            );
            cur_x += 14.0;
        } else {
            cur_x += 14.0;
        }

        // File / Directory Icon
        let icon = match row.kind {
            EntryKind::Directory => akssora_icons::FolderIcon::GLYPH,
            EntryKind::File => akssora_icons::file_type_icon(&row.name),
            EntryKind::Symlink => akssora_icons::SymlinkIcon::GLYPH,
        };
        let icon_col = match row.kind {
            EntryKind::Directory => theme.explorer_directory_icon,
            _ => theme.explorer_file_icon,
        };
        renderer.render_text(
            queue,
            fonts,
            verts,
            icon,
            cur_x,
            row_y + (item_h - fonts.cell_h) * 0.5,
            14.0,
            icon_col,
            false,
            screen_w,
            screen_h,
        );
        cur_x += 14.0;

        // Entry Name Color (tinted with Git status if file/folder is modified/added/etc.)
        let name_col = if row.is_hidden {
            theme.text_secondary
        } else if is_active_editor || is_selected {
            theme.text_primary
        } else if let Some(ref st) = git_badge {
            st.text_color()
        } else {
            theme.foreground
        };

        let badge_reserve = if git_badge.is_some() { 22.0 } else { 0.0 };
        let max_text_w = (explorer_x + exp_w - cur_x - 8.0 - badge_reserve).max(0.0);
        renderer.render_text(
            queue,
            fonts,
            verts,
            &row.name,
            cur_x,
            row_y + (item_h - fonts.cell_h) * 0.5,
            max_text_w,
            name_col,
            is_active_editor || is_selected,
            screen_w,
            screen_h,
        );

        // Render subtle Git status indicator badge on the right if available
        if let Some(status) = git_badge {
            let badge_x = explorer_x + exp_w - 20.0;
            let badge_str = status.badge_label();
            renderer.render_text(
                queue,
                fonts,
                verts,
                badge_str,
                badge_x,
                row_y + (item_h - fonts.cell_h) * 0.5,
                16.0,
                status.badge_color(),
                true,
                screen_w,
                screen_h,
            );
        }
    }

    // 3.1 Explorer Scrollbar if content exceeds viewport
    if total_content_h > viewport_h && viewport_h > 0.0 {
        let ratio = (viewport_h / total_content_h).clamp(0.05, 1.0);
        let thumb_h = (viewport_h * ratio).max(16.0).min(viewport_h);
        let travel = viewport_h - thumb_h;
        let norm = (scroll_y / max_scroll).clamp(0.0, 1.0);
        let thumb_y = top_offset + header_h + travel * norm;
        let sb_x = explorer_x + exp_w - 5.0;
        if theme.scrollbar_track[3] > 0.0 {
            push_quad(
                verts,
                sb_x,
                top_offset + header_h,
                sb_x + 3.0,
                top_offset + header_h + viewport_h,
                solid_uv,
                solid_uv,
                theme.scrollbar_track,
                screen_w,
                screen_h,
            );
        }

        push_pill_thumb(
            verts,
            sb_x,
            thumb_y,
            3.0,
            thumb_h,
            circle_uv,
            solid_uv,
            theme.scrollbar_thumb,
            screen_w,
            screen_h,
        );
    }

    // 4. Inline Text Input (New File / New Folder / Rename / ChangeRoot)
    if let Some(input) = &explorer.inline_input {
        let box_y = top_offset + header_h + 4.0;
        let box_h = 28.0;
        let box_w = exp_w - 16.0;
        let box_x = explorer_x + 8.0;

        push_rounded_rect(
            verts,
            box_x,
            box_y,
            box_x + box_w,
            box_y + box_h,
            3.0,
            circle_uv,
            solid_uv,
            theme.header_background,
            screen_w,
            screen_h,
        );
        push_quad(
            verts,
            box_x,
            box_y + box_h - 1.0,
            box_x + box_w,
            box_y + box_h,
            solid_uv,
            solid_uv,
            theme.border,
            screen_w,
            screen_h,
        );

        let prefix = match &input.mode {
            InlineInputMode::NewFile => "File: ",
            InlineInputMode::NewFolder => "Folder: ",
            InlineInputMode::Rename { .. } => "Rename: ",
            InlineInputMode::ChangeRoot => "Root: ",
        };

        let text_with_cursor = format!("{prefix}{}_", input.text);
        renderer.render_text(
            queue,
            fonts,
            verts,
            &text_with_cursor,
            box_x + 6.0,
            box_y + (box_h - fonts.cell_h) * 0.5,
            box_w - 12.0,
            theme.text_primary,
            true,
            screen_w,
            screen_h,
        );

        if let Some(err_msg) = &input.error {
            renderer.render_text(
                queue,
                fonts,
                verts,
                err_msg,
                box_x + 6.0,
                box_y + box_h + 3.0,
                box_w - 12.0,
                theme.danger,
                false,
                screen_w,
                screen_h,
            );
        }
    }

    // 5. Context Menu Modal
    if let Some(menu) = &explorer.context_menu {
        // Menu background surface with rounded geometry
        push_rounded_rect(
            verts,
            menu.x,
            menu.y,
            menu.x + menu.width,
            menu.y + menu.height,
            6.0,
            circle_uv,
            solid_uv,
            theme.header_background,
            screen_w,
            screen_h,
        );

        for (i, action) in menu.items.iter().enumerate() {
            let iy = menu.y + 4.0 + i as f32 * ExplorerContextMenu::ITEM_HEIGHT;
            let is_item_hovered = menu.hovered_idx == Some(i);

            if is_item_hovered {
                push_rounded_rect(
                    verts,
                    menu.x + 4.0,
                    iy,
                    menu.x + menu.width - 4.0,
                    iy + ExplorerContextMenu::ITEM_HEIGHT,
                    4.0,
                    circle_uv,
                    solid_uv,
                    theme.tab_hover_background,
                    screen_w,
                    screen_h,
                );
            }

            let label = match action {
                ExplorerContextAction::OpenInEditor => "Open in Editor",
                ExplorerContextAction::OpenInSplit => "Open in Split Pane",
                ExplorerContextAction::OpenTerminalHere => "Open Terminal Here",
                ExplorerContextAction::NewFile => "New File (%)",
                ExplorerContextAction::NewFolder => "New Folder",
                ExplorerContextAction::Rename => "Rename",
                ExplorerContextAction::Copy => "Duplicate / Copy",
                ExplorerContextAction::CopyPath => "Copy Path",
                ExplorerContextAction::RevealInFileManager => "Reveal in File Manager",
                ExplorerContextAction::Delete => "Delete",
                ExplorerContextAction::ToggleHidden => "Toggle Hidden",
            };

            let item_col = if *action == ExplorerContextAction::Delete {
                theme.danger
            } else if is_item_hovered {
                theme.text_primary
            } else {
                theme.text_secondary
            };

            renderer.render_text(
                queue,
                fonts,
                verts,
                label,
                menu.x + 12.0,
                iy + (ExplorerContextMenu::ITEM_HEIGHT - fonts.cell_h) * 0.5,
                menu.width - 24.0,
                item_col,
                false,
                screen_w,
                screen_h,
            );
        }
    }

    // 6. Delete Confirmation Modal
    if let Some(to_delete) = &explorer.confirm_delete {
        push_quad(
            verts,
            0.0,
            0.0,
            screen_w,
            screen_h,
            solid_uv,
            solid_uv,
            theme.modal_backdrop,
            screen_w,
            screen_h,
        );

        let (dx, dy, dw, dh) = ExplorerUi::confirm_dialog_rect(screen_w, screen_h);
        push_rounded_rect(
            verts,
            dx,
            dy,
            dx + dw,
            dy + dh,
            10.0,
            circle_uv,
            solid_uv,
            theme.modal_card_background,
            screen_w,
            screen_h,
        );

        let title = "Confirm Deletion";
        renderer.render_text(
            queue,
            fonts,
            verts,
            title,
            dx + 16.0,
            dy + 16.0,
            dw - 32.0,
            theme.danger,
            true,
            screen_w,
            screen_h,
        );

        let count_str = if to_delete.len() == 1 {
            let name = to_delete[0]
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();
            format!("Permanently delete \"{name}\"?")
        } else {
            format!("Permanently delete {} selected items?", to_delete.len())
        };

        renderer.render_text(
            queue,
            fonts,
            verts,
            &count_str,
            dx + 16.0,
            dy + 42.0,
            dw - 32.0,
            theme.text_secondary,
            false,
            screen_w,
            screen_h,
        );

        // Centered Buttons: Delete and Cancel
        let btn_y = dy + dh - 38.0;
        let btn_h = 28.0;
        let del_w = 90.0;
        let can_w = 90.0;
        let btn_gap = 14.0;
        let total_buttons_w = del_w + can_w + btn_gap;
        let start_btn_x = dx + (dw - total_buttons_w) / 2.0;

        // Delete button
        let del_x = start_btn_x;
        push_rounded_rect(
            verts,
            del_x,
            btn_y,
            del_x + del_w,
            btn_y + btn_h,
            6.0,
            circle_uv,
            solid_uv,
            theme.danger,
            screen_w,
            screen_h,
        );
        let del_text = "Delete";
        let del_text_w = del_text.len() as f32 * fonts.cell_w;
        let del_text_x = del_x + (del_w - del_text_w).max(0.0) / 2.0;
        renderer.render_text(
            queue,
            fonts,
            verts,
            del_text,
            del_text_x,
            btn_y + (btn_h - fonts.cell_h) * 0.5,
            del_w,
            [1.0, 1.0, 1.0, 1.0],
            true,
            screen_w,
            screen_h,
        );

        // Cancel button
        let can_x = del_x + del_w + btn_gap;
        push_rounded_rect(
            verts,
            can_x,
            btn_y,
            can_x + can_w,
            btn_y + btn_h,
            6.0,
            circle_uv,
            solid_uv,
            theme.modal_button_background,
            screen_w,
            screen_h,
        );
        let can_text = "Cancel";
        let can_text_w = can_text.len() as f32 * fonts.cell_w;
        let can_text_x = can_x + (can_w - can_text_w).max(0.0) / 2.0;
        renderer.render_text(
            queue,
            fonts,
            verts,
            can_text,
            can_text_x,
            btn_y + (btn_h - fonts.cell_h) * 0.5,
            can_w,
            theme.text_secondary,
            false,
            screen_w,
            screen_h,
        );
    }
}
