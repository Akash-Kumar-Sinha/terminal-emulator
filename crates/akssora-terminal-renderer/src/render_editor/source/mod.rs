use akssora_editor::{Document, VimMode};
use akssora_terminal_config::Theme;
use akssora_terminal_ui::Scrollbar;

use crate::font::FontManager;
use crate::glyph::GlyphRenderer;
use crate::quad::push_quad;
use crate::vertex::GlyphVertex;

pub mod chip;
pub mod cursor;
pub mod gutter;
pub mod scrollbars;

use chip::render_inline_diff_chip;
use cursor::render_editor_cursor;
use gutter::{render_git_gutter_indicator, render_line_number};
use scrollbars::render_editor_scrollbars;

pub const GUTTER_WIDTH: f32 = 48.0;

// INFO: Renders the standard text / code source editor with line numbers, cursor, selections, and pill scrollbars.
#[allow(clippy::too_many_arguments)]
pub fn render_source_editor_view(
    renderer: &mut GlyphRenderer,
    queue: &wgpu::Queue,
    fonts: &FontManager,
    _ui_fonts: &FontManager,
    theme: &Theme,
    doc: &Document,
    editor_scrollbars: Option<(&Scrollbar, &Scrollbar)>,
    hovered_gutter: Option<(usize, Option<akssora_editor::GutterActionKind>)>,
    src_x: f32,
    src_y: f32,
    src_w: f32,
    src_h: f32,
    verts: &mut Vec<GlyphVertex>,
    solid_uv: [f32; 2],
    circle_uv: [[f32; 2]; 2],
    screen_w: f32,
    screen_h: f32,
) {
    if src_w <= 0.0 || src_h <= 0.0 {
        return;
    }

    let cell_h = fonts.cell_h;
    let cell_w = fonts.cell_w;
    let line_h = (cell_h * doc.line_height).max(cell_h);

    let show_line_numbers =
        doc.line_numbers && doc.line_number_mode != akssora_terminal_config::LineNumberMode::Off;
    let gutter_w = if show_line_numbers { GUTTER_WIDTH } else { 0.0 };
    let rough_content_w = (src_w - gutter_w - 16.0).max(0.0);
    let rough_visible_cols = (rough_content_w / cell_w).floor().max(1.0) as usize;
    let rough_visible_rows = (src_h / line_h).floor() as usize;

    let total_lines = if doc.line_wrap {
        doc.total_visual_rows(rough_visible_cols)
    } else {
        doc.buffer.line_count()
    };
    let max_line_len = doc.max_line_len();

    let v_overflow = total_lines > rough_visible_rows && rough_visible_rows > 0;
    let h_overflow = !doc.line_wrap && max_line_len > rough_visible_cols && rough_visible_cols > 0;

    let v_sb_w = if v_overflow { 8.0 } else { 0.0 };
    let h_sb_h = if h_overflow { 8.0 } else { 0.0 };

    let editor_content_h = (src_h - h_sb_h).max(0.0);
    let visible_rows = (editor_content_h / line_h).floor() as usize;

    // Gutter Background & Dividing Line
    if gutter_w > 0.0 {
        push_quad(
            verts,
            src_x,
            src_y,
            src_x + gutter_w,
            src_y + editor_content_h,
            solid_uv,
            solid_uv,
            theme.sidebar_background,
            screen_w,
            screen_h,
        );
        push_quad(
            verts,
            src_x + gutter_w - 1.0,
            src_y,
            src_x + gutter_w,
            src_y + editor_content_h,
            solid_uv,
            solid_uv,
            theme.border_subtle,
            screen_w,
            screen_h,
        );
    }

    let text_x = src_x + gutter_w + 8.0;
    let max_text_w = (src_w - gutter_w - 16.0 - v_sb_w).max(0.0);
    let visible_cols = (max_text_w / cell_w).floor().max(1.0) as usize;
    let (cursor_row, cursor_col) = doc.cursor;

    let wrap_layout = if doc.line_wrap {
        Some(doc.wrap_layout(visible_cols))
    } else {
        None
    };

    let (cursor_vis_row, cursor_vis_col) = if let Some(ref wl) = wrap_layout {
        wl.logical_to_visual(cursor_row, cursor_col)
    } else {
        (cursor_row, cursor_col)
    };

    // Current Line Highlight
    let highlight_row = if doc.line_wrap {
        cursor_vis_row
    } else {
        cursor_row
    };
    if highlight_row >= doc.scroll_row && highlight_row < doc.scroll_row + visible_rows {
        let cur_line_y = src_y + (highlight_row - doc.scroll_row) as f32 * line_h;
        push_quad(
            verts,
            src_x + gutter_w,
            cur_line_y,
            src_x + src_w - v_sb_w,
            cur_line_y + line_h,
            solid_uv,
            solid_uv,
            theme.current_line_background,
            screen_w,
            screen_h,
        );
    }

    // Render Lines (Line numbers, selections, and code glyphs)
    let selection_range = doc.selection_range();
    doc.with_git_line_statuses(|git_statuses| {
        if let Some(ref wl) = wrap_layout {
            for i in 0..visible_rows {
                let vis_idx = doc.scroll_row + i;
                if vis_idx >= total_lines {
                    break;
                }

                let Some(vr) = wl.get_visual_row(vis_idx) else {
                    break;
                };
                let line_y = src_y + i as f32 * line_h;
                let text_y = line_y + (line_h - cell_h) * 0.5;

                // Line number in gutter: only for non-continuation rows
                if gutter_w > 0.0 && !vr.is_continuation {
                    render_line_number(
                        renderer,
                        queue,
                        fonts,
                        verts,
                        vr.logical_row,
                        cursor_row,
                        doc.line_number_mode,
                        src_x,
                        text_y,
                        gutter_w,
                        theme,
                        screen_w,
                        screen_h,
                    );
                }

                // Inline Git Change Indicators & Diff Highlights
                if let Some(git_status) = git_statuses.get(vr.logical_row) {
                    render_git_gutter_indicator(
                        renderer,
                        queue,
                        fonts,
                        verts,
                        git_status,
                        doc.inline_diff,
                        vr.is_continuation,
                        vr.logical_row,
                        hovered_gutter,
                        src_x,
                        src_w,
                        line_y,
                        gutter_w,
                        v_sb_w,
                        line_h,
                        theme,
                        solid_uv,
                        circle_uv,
                        screen_w,
                        screen_h,
                    );
                }

                // Selection Highlight for this visual row
                if let Some(((s_row, s_col), (e_row, e_col))) = selection_range
                    && vr.logical_row >= s_row
                    && vr.logical_row <= e_row
                {
                    let line_len = doc.buffer.line_len(vr.logical_row);
                    let col_start = if vr.logical_row == s_row { s_col } else { 0 };
                    let col_end = if vr.logical_row == e_row {
                        e_col
                    } else {
                        line_len + 1
                    };

                    let vis_start = col_start.max(vr.char_start);
                    let vis_end = col_end.min(vr.char_end);
                    if vis_end > vis_start {
                        let sub_start = vis_start - vr.char_start;
                        let sub_end = vis_end - vr.char_start;
                        let sel_x = text_x + sub_start as f32 * cell_w;
                        let sel_w = ((sub_end - sub_start) as f32 * cell_w)
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

                // Text Glyphs for this visual row slice
                let mut vis_rendered_len = 0;
                if let Some(line_content) = doc.buffer.get_line(vr.logical_row) {
                    let clean_line = line_content.trim_end_matches(['\r', '\n']);
                    let display_text: String = clean_line
                        .chars()
                        .skip(vr.char_start)
                        .take(vr.char_end.saturating_sub(vr.char_start))
                        .collect();
                    vis_rendered_len = display_text.chars().count();
                    if !display_text.is_empty() {
                        renderer.render_text(
                            queue,
                            fonts,
                            verts,
                            &display_text,
                            text_x,
                            text_y,
                            max_text_w,
                            theme.foreground,
                            false,
                            screen_w,
                            screen_h,
                        );
                    }
                }

                // Inline Diff removed text inspection chip
                if doc.inline_diff
                    && !vr.is_continuation
                    && let Some(git_status) = git_statuses.get(vr.logical_row)
                {
                    render_inline_diff_chip(
                        renderer,
                        queue,
                        fonts,
                        verts,
                        theme,
                        git_status,
                        text_x,
                        vis_rendered_len,
                        text_y,
                        cell_w,
                        cell_h,
                        src_x,
                        src_w,
                        v_sb_w,
                        solid_uv,
                        circle_uv,
                        screen_w,
                        screen_h,
                    );
                }
            }
        } else {
            for i in 0..visible_rows {
                let line_idx = doc.scroll_row + i;
                if line_idx >= total_lines {
                    break;
                }

                let line_y = src_y + i as f32 * line_h;
                let text_y = line_y + (line_h - cell_h) * 0.5;

                // Line number in gutter
                if gutter_w > 0.0 {
                    render_line_number(
                        renderer,
                        queue,
                        fonts,
                        verts,
                        line_idx,
                        cursor_row,
                        doc.line_number_mode,
                        src_x,
                        text_y,
                        gutter_w,
                        theme,
                        screen_w,
                        screen_h,
                    );
                }

                // Inline Git Change Indicators & Diff Highlights
                if let Some(git_status) = git_statuses.get(line_idx) {
                    render_git_gutter_indicator(
                        renderer,
                        queue,
                        fonts,
                        verts,
                        git_status,
                        doc.inline_diff,
                        false,
                        line_idx,
                        hovered_gutter,
                        src_x,
                        src_w,
                        line_y,
                        gutter_w,
                        v_sb_w,
                        line_h,
                        theme,
                        solid_uv,
                        circle_uv,
                        screen_w,
                        screen_h,
                    );
                }

                // Selection Highlight for this line with horizontal scroll offset
                if let Some(((s_row, s_col), (e_row, e_col))) = selection_range
                    && line_idx >= s_row
                    && line_idx <= e_row
                {
                    let line_len = doc.buffer.line_len(line_idx);
                    let col_start = if line_idx == s_row { s_col } else { 0 };
                    let col_end = if line_idx == e_row {
                        e_col
                    } else {
                        line_len + 1
                    };
                    let vis_start = col_start.max(doc.scroll_col);
                    let vis_end = col_end.max(doc.scroll_col);
                    if vis_end > vis_start {
                        let sel_x = text_x + (vis_start - doc.scroll_col) as f32 * cell_w;
                        let sel_w = ((vis_end - vis_start) as f32 * cell_w)
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

                // Text Glyphs with horizontal scroll slicing
                let mut line_rendered_len = 0;
                if let Some(line_content) = doc.buffer.get_line(line_idx) {
                    let clean_line = line_content.trim_end_matches(['\r', '\n']);
                    let display_text = if doc.scroll_col > 0 {
                        clean_line.chars().skip(doc.scroll_col).collect::<String>()
                    } else {
                        clean_line.to_string()
                    };
                    line_rendered_len = display_text.chars().count();
                    if !display_text.is_empty() {
                        renderer.render_text(
                            queue,
                            fonts,
                            verts,
                            &display_text,
                            text_x,
                            text_y,
                            max_text_w,
                            theme.foreground,
                            false,
                            screen_w,
                            screen_h,
                        );
                    }
                }

                // Inline Diff removed text inspection chip
                if doc.inline_diff
                    && let Some(git_status) = git_statuses.get(line_idx)
                {
                    render_inline_diff_chip(
                        renderer,
                        queue,
                        fonts,
                        verts,
                        theme,
                        git_status,
                        text_x,
                        line_rendered_len,
                        text_y,
                        cell_w,
                        cell_h,
                        src_x,
                        src_w,
                        v_sb_w,
                        solid_uv,
                        circle_uv,
                        screen_w,
                        screen_h,
                    );
                }
            }
        }
    });

    // Cursor Rendering
    let (rendered_vis_row, rendered_vis_col) = if doc.line_wrap {
        (cursor_vis_row, cursor_vis_col)
    } else {
        (cursor_row, cursor_col.saturating_sub(doc.scroll_col))
    };

    if rendered_vis_row >= doc.scroll_row
        && rendered_vis_row < doc.scroll_row + visible_rows
        && (doc.line_wrap || cursor_col >= doc.scroll_col)
    {
        let is_block_cursor = doc.vim_enabled && doc.vim.mode == VimMode::Normal;
        render_editor_cursor(
            verts,
            rendered_vis_row,
            rendered_vis_col,
            doc.scroll_row,
            visible_rows,
            is_block_cursor,
            text_x,
            src_y,
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

    // Pill Scrollbars
    render_editor_scrollbars(
        verts,
        editor_scrollbars,
        v_overflow,
        h_overflow,
        total_lines,
        visible_rows,
        visible_cols,
        doc.scroll_row,
        doc.scroll_col,
        max_line_len,
        src_x,
        src_y,
        src_w,
        editor_content_h,
        gutter_w,
        v_sb_w,
        h_sb_h,
        theme,
        solid_uv,
        circle_uv,
        screen_w,
        screen_h,
    );
}
