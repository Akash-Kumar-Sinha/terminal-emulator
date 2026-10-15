pub mod cards;
pub mod dialog;
pub mod header;
pub mod line_popup;
pub mod scrollbar;
pub mod sidebar;

use akssora_terminal_config::Theme;
use akssora_terminal_ui::{CodeReviewAction, CodeReviewLayout, CodeReviewModel};

use crate::font::FontManager;
use crate::glyph::GlyphRenderer;
use crate::quad::push_quad;
use crate::vertex::GlyphVertex;

pub use cards::render_file_diff_card;
pub use dialog::{render_commit_dialog, render_discard_dialog};
pub use header::render_header;
pub use line_popup::render_line_action_popup;
pub use scrollbar::render_vertical_scrollbar;
pub use sidebar::render_sidebar;

// INFO: Main coordinator function rendering the full Code Review panel with painter's algorithm Z-index.
#[allow(clippy::too_many_arguments)]
pub fn render_code_review_panel(
    renderer: &mut GlyphRenderer,
    queue: &wgpu::Queue,
    fonts: &FontManager,
    ui_fonts: &FontManager,
    theme: &Theme,
    model: &CodeReviewModel,
    editor_documents: &std::collections::HashMap<std::path::PathBuf, akssora_editor::Document>,
    active_editor: Option<&akssora_editor::Document>,
    hovered_action: Option<&CodeReviewAction>,
    hovered_gutter: Option<(usize, Option<akssora_editor::GutterActionKind>)>,
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
    if ws_w <= 0.0 || ws_h <= 0.0 {
        return;
    }

    // ── 1. Main Background ────────────────────────────────────────────────────
    push_quad(
        verts,
        ws_x,
        ws_y,
        ws_x + ws_w,
        ws_y + ws_h,
        solid_uv,
        solid_uv,
        theme.code_review_background,
        screen_w,
        screen_h,
    );

    // ── 2. Main Diff Viewport (Multi-File Cards on right) ──────────────────────
    let (dvx, dvy, dvw, dvh) = CodeReviewLayout::diff_viewport_rect(model, ws_x, ws_y, ws_w, ws_h);

    let card_x = dvx + 16.0;
    let card_w = (dvw - 32.0).max(100.0);
    let total_content_h = model.total_diff_content_height(dvh);
    let max_diff_scroll = (total_content_h - dvh).max(0.0);
    let clamped_diff_scroll = model.diff_scroll_y.clamp(0.0, max_diff_scroll);
    let mut curr_y = dvy + 12.0 - clamped_diff_scroll;

    for (orig_idx, file) in model.diff.files.iter().enumerate() {
        let is_collapsed = model.is_file_collapsed(&file.path);
        let full_path = model.repo_root.join(&file.path);
        let is_editing = model.active_file_idx == Some(orig_idx)
            || model.active_editing_file.as_ref() == Some(&full_path);
        let is_full = model.is_full_file(&file.path);
        let card_h = CodeReviewLayout::card_height_for_file_state_ex(
            file,
            is_collapsed,
            is_editing,
            is_full,
            dvh,
        );

        // Small buffer (~200px) above and below viewport for smooth scrolling without blank flickers
        const BUFFER_PX: f32 = 200.0;
        if curr_y + card_h > dvy - BUFFER_PX && curr_y < dvy + dvh + BUFFER_PX {
            let card_doc =
                editor_documents
                    .get(&full_path)
                    .or(if is_editing { active_editor } else { None });

            let card_hovered_gutter = if model.hovered_gutter.map(|h| h.file_idx) == Some(orig_idx)
            {
                model.hovered_gutter.map(|h| (h.row, h.action))
            } else if hovered_gutter.is_some() && model.active_file_idx == Some(orig_idx) {
                hovered_gutter
            } else {
                None
            };

            cards::render_file_diff_card(
                renderer,
                queue,
                fonts,
                ui_fonts,
                theme,
                model,
                file,
                orig_idx,
                card_doc,
                hovered_action,
                card_hovered_gutter,
                is_collapsed,
                card_x,
                curr_y,
                card_w,
                card_h,
                dvy,
                dvh,
                verts,
                solid_uv,
                circle_uv,
                screen_w,
                screen_h,
            );
        }

        curr_y += card_h + CodeReviewLayout::CARD_MARGIN_BOTTOM;
    }

    // Empty or loading banner feedback
    if model.diff.files.is_empty() {
        let msg = if model.is_loading {
            "Retrieving Git diff in background worker..."
        } else {
            "No changes detected in repository"
        };
        let tw = msg.len() as f32 * 8.0;
        let tx = (dvx + (dvw - tw) / 2.0).max(dvx + 16.0);
        let ty = dvy + dvh / 2.0 - 8.0;
        renderer.render_text(
            queue,
            ui_fonts,
            verts,
            msg,
            tx,
            ty,
            dvw - 32.0,
            theme.foreground,
            false,
            screen_w,
            screen_h,
        );
    }

    // 2.1 Diff Viewport Scrollbar (Vertical pill indicator)
    scrollbar::render_vertical_scrollbar(
        verts,
        theme,
        dvx,
        dvy,
        dvw,
        dvh,
        total_content_h,
        clamped_diff_scroll,
        solid_uv,
        circle_uv,
        screen_w,
        screen_h,
    );

    // ── 3. Left Sidebar (Changed Files List & Tree View) ───────────────────────
    sidebar::render_sidebar(
        renderer,
        queue,
        ui_fonts,
        theme,
        model,
        hovered_action,
        ws_x,
        ws_y,
        ws_w,
        ws_h,
        verts,
        solid_uv,
        circle_uv,
        screen_w,
        screen_h,
    );

    // 3.1 Sidebar Scrollbar (if sidebar content overflows)
    if let Some((sbx, sby, sbw, sbh)) =
        CodeReviewLayout::sidebar_rect(model, ws_x, ws_y, ws_w, ws_h)
    {
        let tb_h = CodeReviewLayout::SIDEBAR_TOOLBAR_HEIGHT;
        let list_viewport_h = (sbh - tb_h).max(0.0);
        let sidebar_total_h =
            model.visible_tree_items().len() as f32 * CodeReviewLayout::FILE_ROW_HEIGHT + 16.0;
        let max_sb_scroll = (sidebar_total_h - list_viewport_h).max(0.0);
        let clamped_sb_scroll = model.sidebar_scroll_y.clamp(0.0, max_sb_scroll);
        scrollbar::render_vertical_scrollbar(
            verts,
            theme,
            sbx,
            sby + tb_h,
            sbw,
            list_viewport_h,
            sidebar_total_h,
            clamped_sb_scroll,
            solid_uv,
            circle_uv,
            screen_w,
            screen_h,
        );
    }

    // ── 4. Top Header Bar (Drawn LAST so it always appears above scrolled content)
    header::render_header(
        renderer,
        queue,
        ui_fonts,
        theme,
        model,
        hovered_action,
        ws_x,
        ws_y,
        ws_w,
        verts,
        solid_uv,
        circle_uv,
        screen_w,
        screen_h,
    );

    // ── 5. Line Action Popup (Drawn above cards, diff highlights, and sticky headers)
    if let Some(hover) = model.hovered_gutter {
        render_line_action_popup(
            renderer,
            queue,
            ui_fonts,
            theme,
            model,
            hover.file_idx,
            hover.row,
            hover.is_deletion,
            hover.action,
            ws_x,
            ws_y,
            ws_w,
            ws_h,
            verts,
            solid_uv,
            circle_uv,
            screen_w,
            screen_h,
        );
    }

    // ── 6. Commit Modal Dialog ────────────────────────────────────────────────
    if model.commit_dialog_open {
        dialog::render_commit_dialog(
            renderer, queue, fonts, ui_fonts, theme, model, ws_x, ws_y, ws_w, ws_h, verts,
            solid_uv, circle_uv, screen_w, screen_h,
        );
    }

    // ── 7. Discard Confirmation Modal Dialog ──────────────────────────────────
    if let Some(ref target) = model.confirm_discard {
        dialog::render_discard_dialog(
            renderer, queue, fonts, ui_fonts, theme, target, ws_x, ws_y, ws_w, ws_h, verts,
            solid_uv, circle_uv, screen_w, screen_h,
        );
    }
}
