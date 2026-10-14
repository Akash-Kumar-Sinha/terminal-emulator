use akssora_editor::{Document, VimMode};
use akssora_git::{DiffLineType, FileDiff, LineGitChange, LineGitStatus};
use akssora_terminal_config::Theme;
use akssora_terminal_ui::{CodeReviewAction, CodeReviewLayout, CodeReviewModel};

use crate::font::FontManager;
use crate::glyph::GlyphRenderer;
use crate::quad::push_quad;
use crate::render_editor::source::GUTTER_WIDTH;
use crate::render_editor::source::cursor::render_editor_cursor;
use crate::render_editor::source::gutter::{render_git_gutter_indicator, render_line_number};
use crate::vertex::GlyphVertex;

// INFO: Renders the card body using the actual live editor in editing mode with Git effects and viewport-based block virtualization.
#[allow(clippy::too_many_arguments)]
pub fn render_card_body(
    renderer: &mut GlyphRenderer,
    queue: &wgpu::Queue,
    fonts: &FontManager,
    ui_fonts: &FontManager,
    theme: &Theme,
    model: &CodeReviewModel,
    file: &FileDiff,
    file_idx: usize,
    card_doc: Option<&Document>,
    _hovered_action: Option<&CodeReviewAction>,
    _hovered_gutter: Option<(usize, Option<akssora_editor::GutterActionKind>)>,
    cx: f32,
    cy: f32,
    cw: f32,
    ch: f32,
    header_y: f32,
    header_h: f32,
    clip_top: f32,
    clip_h: f32,
    verts: &mut Vec<GlyphVertex>,
    solid_uv: [f32; 2],
    circle_uv: [[f32; 2]; 2],
    screen_w: f32,
    screen_h: f32,
) {
    let body_top = (header_y + header_h).max(clip_top);
    let body_bot = (cy + ch).min(clip_top + clip_h);
    let body_h = (body_bot - body_top).max(0.0);
    if body_h <= 0.0 {
        return;
    }

    // Small rendering buffer (~200px before and after viewport) for smooth scrolling
    const BUFFER_PX: f32 = 200.0;
    let render_min_y = (clip_top - BUFFER_PX).max(0.0);
    let render_max_y = clip_top + clip_h + BUFFER_PX;

    let cell_w = fonts.cell_w;
    let cell_h = fonts.cell_h;
    let ui_cell_h = ui_fonts.cell_h;
    let line_h = CodeReviewLayout::DIFF_LINE_HEIGHT;
    let gutter_w = GUTTER_WIDTH;

    let full_path = model.repo_root.join(&file.path);
    let is_active_editing_file = model.active_file_idx == Some(file_idx)
        || model.active_editing_file.as_ref() == Some(&full_path);

    // 1. Gutter Background & Dividing Line (strictly clamped to [body_top, body_bot])
    push_quad(
        verts,
        cx,
        body_top,
        cx + gutter_w,
        body_bot,
        solid_uv,
        solid_uv,
        theme.sidebar_background,
        screen_w,
        screen_h,
    );
    push_quad(
        verts,
        cx + gutter_w - 1.0,
        body_top,
        cx + gutter_w,
        body_bot,
        solid_uv,
        solid_uv,
        theme.border_subtle,
        screen_w,
        screen_h,
    );

    let text_x = cx + gutter_w + 8.0;
    let max_text_w = (cw - gutter_w - 16.0).max(0.0);
    let is_full_file = model.is_full_file(&file.path);

    // ── Mode A: Full File View ───────────────────────────────────────────────
    if is_full_file {
        if let Some(doc) = card_doc {
            let line_count = doc.buffer.line_count();
            let start_y = cy + header_h + 8.0;

            let first_row = ((render_min_y - start_y).max(0.0) / line_h).floor() as usize;
            let last_row =
                (((render_max_y - start_y).max(0.0) / line_h).ceil() as usize).min(line_count);

            let selection_range = doc.selection_range();

            doc.with_git_line_statuses(|git_statuses| {
                for row in first_row..last_row {
                    let line_y = start_y + row as f32 * line_h;
                    if line_y + line_h <= body_top || line_y >= body_bot {
                        continue;
                    }

                    // Hovered line highlight & current line highlight (behind text)
                    let is_hovered_line =
                        model.hovered_gutter.map(|h| (h.file_idx, h.row)) == Some((file_idx, row));
                    if is_hovered_line {
                        push_quad(
                            verts,
                            cx,
                            line_y,
                            cx + cw,
                            line_y + line_h,
                            solid_uv,
                            solid_uv,
                            theme.hover_background,
                            screen_w,
                            screen_h,
                        );
                    } else if is_active_editing_file && doc.cursor.0 == row {
                        push_quad(
                            verts,
                            cx + gutter_w,
                            line_y,
                            cx + cw,
                            line_y + line_h,
                            solid_uv,
                            solid_uv,
                            theme.current_line_background,
                            screen_w,
                            screen_h,
                        );
                    }

                    // Line number
                    render_line_number(
                        renderer,
                        queue,
                        fonts,
                        verts,
                        row,
                        doc.cursor.0,
                        doc.line_number_mode,
                        cx,
                        line_y + (line_h - cell_h) * 0.5,
                        gutter_w,
                        theme,
                        screen_w,
                        screen_h,
                    );

                    // Git gutter indicator
                    if let Some(git_status) = git_statuses.get(row) {
                        render_git_gutter_indicator(
                            renderer, queue, fonts, verts, git_status, false, false, row, None, cx,
                            cw, line_y, gutter_w, 0.0, line_h, theme, solid_uv, circle_uv,
                            screen_w, screen_h,
                        );
                    }

                    // Selection highlight
                    if let Some(((s_row, s_col), (e_row, e_col))) = selection_range
                        && row >= s_row
                        && row <= e_row
                    {
                        let line_len = doc.buffer.line_len(row);
                        let col_start = if row == s_row { s_col } else { 0 };
                        let col_end = if row == e_row { e_col } else { line_len + 1 };
                        if col_end > col_start {
                            let sel_x = text_x + col_start as f32 * cell_w;
                            let sel_w = ((col_end - col_start) as f32 * cell_w)
                                .min((text_x + max_text_w - sel_x).max(0.0));
                            if sel_w > 0.0 {
                                push_quad(
                                    verts,
                                    sel_x,
                                    line_y,
                                    sel_x + sel_w,
                                    line_y + line_h,
                                    solid_uv,
                                    solid_uv,
                                    theme.selection_background,
                                    screen_w,
                                    screen_h,
                                );
                            }
                        }
                    }

                    // Text glyphs
                    if let Some(line_content) = doc.buffer.get_line(row) {
                        let clean = line_content.trim_end_matches(['\r', '\n']);
                        if !clean.is_empty() {
                            renderer.render_text(
                                queue,
                                fonts,
                                verts,
                                clean,
                                text_x,
                                line_y + (line_h - cell_h) * 0.5,
                                max_text_w,
                                theme.foreground,
                                false,
                                screen_w,
                                screen_h,
                            );
                        }
                    }

                    // Cursor
                    if is_active_editing_file && doc.cursor.0 == row {
                        let is_block = doc.vim_enabled && doc.vim.mode == VimMode::Normal;
                        render_editor_cursor(
                            verts,
                            row,
                            doc.cursor.1,
                            0,
                            line_count,
                            is_block,
                            text_x,
                            line_y,
                            max_text_w,
                            cell_w,
                            cell_h,
                            line_h,
                            theme,
                            solid_uv,
                            screen_w,
                            screen_h,
                        );
                    }
                }
            });
        }
        return;
    }

    // ── Mode B: Hunk-Based Diff View (5 Lines of Surrounding Context) ────────
    let hunk_header_h = CodeReviewLayout::UNMODIFIED_LINES_HEIGHT;
    let mut cur_y = cy + header_h + 8.0;

    for hunk in &file.hunks {
        let hunk_start_y = cur_y;
        let can_top = hunk.can_expand_top();
        let can_bot = hunk.can_expand_bottom(file.line_count);

        let top_bar_y = cur_y;
        cur_y += hunk_header_h;

        let lines_start_y = cur_y;
        cur_y += hunk.lines.len() as f32 * line_h;
        let hunk_end_y = cur_y;

        // Viewport virtualization: skip hunks completely outside visible viewport + buffer
        if hunk_end_y < render_min_y || hunk_start_y > render_max_y {
            continue;
        }

        // 1. Hunk Header Bar with Context Expansion Arrows (▲ / ▼)
        if top_bar_y + hunk_header_h > body_top && top_bar_y < body_bot {
            let bar_y0 = top_bar_y.max(body_top);
            let bar_y1 = (top_bar_y + hunk_header_h).min(body_bot);

            push_quad(
                verts,
                cx,
                bar_y0,
                cx + cw,
                bar_y1,
                solid_uv,
                solid_uv,
                theme.code_review_accordion_body_background,
                screen_w,
                screen_h,
            );
            push_quad(
                verts,
                cx,
                bar_y1 - 1.0,
                cx + cw,
                bar_y1,
                solid_uv,
                solid_uv,
                theme.code_review_card_border,
                screen_w,
                screen_h,
            );

            if top_bar_y >= body_top && top_bar_y + ui_cell_h <= body_bot {
                let bar_text_y = top_bar_y + (hunk_header_h - ui_cell_h) * 0.5;
                // Hunk range header (e.g. @@ -12,5 +12,12 @@)
                renderer.render_text(
                    queue,
                    ui_fonts,
                    verts,
                    &hunk.header,
                    cx + 12.0,
                    bar_text_y,
                    (cw - 80.0).max(40.0),
                    theme.text_secondary,
                    false,
                    screen_w,
                    screen_h,
                );

                // Up arrow context expansion button (▲)
                if can_top {
                    let btn_x = cx + cw - 48.0;
                    let btn_w = 20.0;
                    renderer.render_text(
                        queue,
                        ui_fonts,
                        verts,
                        akssora_icons::ChevronUpIcon::GLYPH_LARGE,
                        btn_x + 4.0,
                        bar_text_y,
                        btn_w,
                        theme.text_secondary,
                        false,
                        screen_w,
                        screen_h,
                    );
                }

                // Down arrow context expansion button (▼)
                if can_bot {
                    let btn_x = cx + cw - 24.0;
                    let btn_w = 20.0;
                    renderer.render_text(
                        queue,
                        ui_fonts,
                        verts,
                        akssora_icons::ChevronDownIcon::GLYPH_LARGE,
                        btn_x + 4.0,
                        bar_text_y,
                        btn_w,
                        theme.text_secondary,
                        false,
                        screen_w,
                        screen_h,
                    );
                }
            }
        }

        // 2. Lines within this hunk (virtualized slice)
        let first_line = ((render_min_y - lines_start_y).max(0.0) / line_h).floor() as usize;
        let last_line = (((render_max_y - lines_start_y).max(0.0) / line_h).ceil() as usize)
            .min(hunk.lines.len());

        let doc_selection = card_doc.and_then(|d| d.selection_range());

        for line_idx in first_line..last_line {
            let diff_line = &hunk.lines[line_idx];
            let line_y = lines_start_y + line_idx as f32 * line_h;
            if line_y + line_h <= body_top || line_y >= body_bot {
                continue;
            }

            let text_y = line_y + (line_h - cell_h) * 0.5;
            let logical_row_opt = diff_line.new_line_num.map(|n| n.saturating_sub(1));
            let target_row_opt = diff_line
                .new_line_num
                .or(diff_line.old_line_num)
                .map(|n| n.saturating_sub(1));
            let is_del = diff_line.line_type == DiffLineType::Delete;
            let is_hovered_line = target_row_opt
                .map(|r| {
                    model
                        .hovered_gutter
                        .map(|h| (h.file_idx, h.row, h.is_deletion))
                        == Some((file_idx, r, is_del))
                })
                .unwrap_or(false);

            let is_active_line = is_active_editing_file
                && card_doc
                    .map(|d| {
                        if diff_line.line_type == DiffLineType::Delete {
                            diff_line.old_line_num.map(|n| n.saturating_sub(1)) == Some(d.cursor.0)
                        } else {
                            logical_row_opt == Some(d.cursor.0)
                        }
                    })
                    .unwrap_or(false);

            // A. Line Background Tint & Active/Hovered Line Highlight (rendered behind text)
            if is_hovered_line {
                push_quad(
                    verts,
                    cx,
                    line_y,
                    cx + cw,
                    line_y + line_h,
                    solid_uv,
                    solid_uv,
                    theme.hover_background,
                    screen_w,
                    screen_h,
                );
            } else if is_active_line {
                push_quad(
                    verts,
                    cx + gutter_w,
                    line_y,
                    cx + cw,
                    line_y + line_h,
                    solid_uv,
                    solid_uv,
                    theme.current_line_background,
                    screen_w,
                    screen_h,
                );
            }

            match diff_line.line_type {
                DiffLineType::Add => {
                    push_quad(
                        verts,
                        cx + gutter_w,
                        line_y,
                        cx + cw,
                        line_y + line_h,
                        solid_uv,
                        solid_uv,
                        theme.diff_add_green_background,
                        screen_w,
                        screen_h,
                    );
                }
                DiffLineType::Delete => {
                    push_quad(
                        verts,
                        cx + gutter_w,
                        line_y,
                        cx + cw,
                        line_y + line_h,
                        solid_uv,
                        solid_uv,
                        theme.diff_delete_red_background,
                        screen_w,
                        screen_h,
                    );
                }
                DiffLineType::Context => {}
            }

            // B. Line Number in Gutter
            let num_str = match diff_line.line_type {
                DiffLineType::Delete => diff_line
                    .old_line_num
                    .map(|n| format!("{:>3} ", n))
                    .unwrap_or_else(|| "    ".to_string()),
                _ => diff_line
                    .new_line_num
                    .map(|n| format!("{:>3} ", n))
                    .unwrap_or_else(|| "    ".to_string()),
            };
            let num_color = match diff_line.line_type {
                DiffLineType::Delete => {
                    if is_active_line {
                        theme.text_primary
                    } else {
                        theme.diff_delete_red
                    }
                }
                _ => {
                    if is_active_line {
                        theme.text_primary
                    } else {
                        theme.text_secondary
                    }
                }
            };
            renderer.render_text(
                queue,
                fonts,
                verts,
                &num_str,
                cx + 4.0,
                text_y,
                gutter_w - 6.0,
                num_color,
                false,
                screen_w,
                screen_h,
            );

            // C. Git Indicators and Action Buttons in Gutter
            let status_opt = match diff_line.line_type {
                DiffLineType::Add => Some(LineGitStatus {
                    change: Some(LineGitChange::Added),
                    deleted_above: false,
                    deleted_below: false,
                    deleted_lines: Vec::new(),
                    original_line: None,
                }),
                DiffLineType::Delete => Some(LineGitStatus {
                    change: Some(LineGitChange::Deleted),
                    deleted_above: false,
                    deleted_below: false,
                    deleted_lines: Vec::new(),
                    original_line: None,
                }),
                DiffLineType::Context => None,
            };

            if let Some(status) = status_opt {
                let target_line = diff_line
                    .new_line_num
                    .or(diff_line.old_line_num)
                    .map(|n| n.saturating_sub(1))
                    .unwrap_or(0);
                render_git_gutter_indicator(
                    renderer,
                    queue,
                    fonts,
                    verts,
                    &status,
                    false,
                    false,
                    target_line,
                    None,
                    cx,
                    cw,
                    line_y,
                    gutter_w,
                    0.0,
                    line_h,
                    theme,
                    solid_uv,
                    circle_uv,
                    screen_w,
                    screen_h,
                );
            }

            // D. Selection Highlight for active editor
            if is_active_editing_file
                && let Some(((s_row, s_col), (e_row, e_col))) = doc_selection
                && let Some(row) = logical_row_opt
                && row >= s_row
                && row <= e_row
            {
                let line_len = card_doc.map(|d| d.buffer.line_len(row)).unwrap_or(0);
                let col_start = if row == s_row { s_col } else { 0 };
                let col_end = if row == e_row { e_col } else { line_len + 1 };
                if col_end > col_start {
                    let sel_x = text_x + col_start as f32 * cell_w;
                    let sel_w = ((col_end - col_start) as f32 * cell_w)
                        .min((text_x + max_text_w - sel_x).max(0.0));
                    if sel_w > 0.0 {
                        push_quad(
                            verts,
                            sel_x,
                            line_y,
                            sel_x + sel_w,
                            line_y + line_h,
                            solid_uv,
                            solid_uv,
                            theme.selection_background,
                            screen_w,
                            screen_h,
                        );
                    }
                }
            }

            // E. Text Glyphs (Using live editor buffer if modified, or diff hunk text)
            let raw_text = if let Some(curr_row) = logical_row_opt
                && let Some(doc) = card_doc
                && let Some(l) = doc.buffer.get_line(curr_row)
            {
                l.trim_end_matches(['\r', '\n']).to_string()
            } else {
                diff_line.content.trim_end_matches(['\r', '\n']).to_string()
            };

            if !raw_text.is_empty() {
                renderer.render_text(
                    queue,
                    fonts,
                    verts,
                    &raw_text,
                    text_x,
                    text_y,
                    max_text_w,
                    theme.foreground,
                    false,
                    screen_w,
                    screen_h,
                );
            }

            // F. Live Editor Cursor (Block cursor for Vim normal mode, Bar cursor for insert mode)
            if is_active_editing_file
                && let Some(doc) = card_doc
                && logical_row_opt == Some(doc.cursor.0)
            {
                let cursor_col = doc.cursor.1;
                let cursor_x = text_x + cursor_col as f32 * cell_w;
                if cursor_x >= text_x && cursor_x < cx + cw - 16.0 {
                    let is_block = doc.vim_enabled && doc.vim.mode == VimMode::Normal;
                    let cursor_w = if is_block {
                        (cell_w - 1.5).max(2.0)
                    } else {
                        2.0
                    };
                    let cur_pad_y = (cell_h * 0.1).clamp(1.0, 2.5);
                    let cy_cursor = line_y + (line_h - cell_h) * 0.5;
                    push_quad(
                        verts,
                        cursor_x,
                        cy_cursor + cur_pad_y,
                        cursor_x + cursor_w,
                        cy_cursor + cell_h - cur_pad_y,
                        solid_uv,
                        solid_uv,
                        theme.cursor,
                        screen_w,
                        screen_h,
                    );
                }
            }
        }
    }
}
