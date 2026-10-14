use akssora_editor::{Document, MarkdownBlock};
use akssora_terminal_config::Theme;
use akssora_terminal_ui::Scrollbar;

use crate::font::FontManager;
use crate::glyph::GlyphRenderer;
use crate::quad::{push_pill_thumb, push_quad, push_rounded_rect};
use crate::vertex::GlyphVertex;

pub mod code;
pub mod spans;

pub use code::render_highlighted_code_line;
pub use spans::{render_inline_spans, spans_to_plain_text, wrap_spans_to_lines};

// INFO: Renders rich formatted Markdown preview output with typography hierarchy, code blocks, tables, task lists, and scrollbar.
#[allow(clippy::too_many_arguments)]
pub fn render_markdown_preview(
    renderer: &mut GlyphRenderer,
    queue: &wgpu::Queue,
    fonts: &FontManager,
    ui_fonts: &FontManager,
    theme: &Theme,
    doc: &Document,
    prev_x: f32,
    prev_y: f32,
    prev_w: f32,
    prev_h: f32,
    verts: &mut Vec<GlyphVertex>,
    solid_uv: [f32; 2],
    circle_uv: [[f32; 2]; 2],
    screen_w: f32,
    screen_h: f32,
) {
    if prev_w <= 0.0 || prev_h <= 0.0 {
        return;
    }

    // Background surface for preview
    push_quad(
        verts,
        prev_x,
        prev_y,
        prev_x + prev_w,
        prev_y + prev_h,
        solid_uv,
        solid_uv,
        theme.background,
        screen_w,
        screen_h,
    );

    let parsed = doc.parse_markdown_preview(None);
    let pad_x = 20.0;
    let pad_y = 16.0;
    let scroll_y = doc.preview_scroll_row as f32 * fonts.cell_h;
    let max_text_w = (prev_w - pad_x * 2.0 - 12.0).max(10.0);
    let cell_h = fonts.cell_h;
    let cell_w = fonts.cell_w;

    let mut cur_y = prev_y + pad_y - scroll_y;
    let view_bottom = prev_y + prev_h;

    // Error alert banner if parsing failed
    if let Some(err) = &parsed.error {
        let err_h = cell_h * 2.0 + 8.0;
        if cur_y + err_h >= prev_y && cur_y <= view_bottom {
            push_rounded_rect(
                verts,
                prev_x + pad_x,
                cur_y,
                prev_x + prev_w - pad_x,
                cur_y + err_h,
                4.0,
                circle_uv,
                solid_uv,
                theme.danger,
                screen_w,
                screen_h,
            );
            renderer.render_text(
                queue,
                ui_fonts,
                verts,
                &format!("Error: {err}"),
                prev_x + pad_x + 8.0,
                cur_y + 4.0,
                max_text_w,
                theme.header_foreground,
                true,
                screen_w,
                screen_h,
            );
        }
        cur_y += err_h + 12.0;
    }

    for block in &parsed.blocks {
        match block {
            MarkdownBlock::Heading { level, spans } => {
                cur_y += 8.0;
                let text = spans_to_plain_text(spans);
                let is_major = *level <= 2;
                let heading_h = if is_major { cell_h * 1.5 } else { cell_h * 1.2 };

                if cur_y + heading_h >= prev_y && cur_y <= view_bottom {
                    let col = if is_major {
                        theme.text_primary
                    } else {
                        theme.text_secondary
                    };
                    renderer.render_text(
                        queue,
                        if is_major { ui_fonts } else { fonts },
                        verts,
                        &text,
                        prev_x + pad_x,
                        cur_y,
                        max_text_w,
                        col,
                        true,
                        screen_w,
                        screen_h,
                    );

                    // Underline rule for H1 and H2
                    if is_major {
                        let rule_y = cur_y + heading_h + 2.0;
                        push_quad(
                            verts,
                            prev_x + pad_x,
                            rule_y,
                            prev_x + prev_w - pad_x,
                            rule_y + 1.0,
                            solid_uv,
                            solid_uv,
                            theme.border_subtle,
                            screen_w,
                            screen_h,
                        );
                    }
                }
                cur_y += heading_h + (if is_major { 8.0 } else { 4.0 });
            }
            MarkdownBlock::Paragraph(spans) => {
                let lines = wrap_spans_to_lines(spans, max_text_w, cell_w);
                for line_spans in lines {
                    if cur_y + cell_h >= prev_y && cur_y <= view_bottom {
                        render_inline_spans(
                            renderer,
                            queue,
                            fonts,
                            verts,
                            &line_spans,
                            prev_x + pad_x,
                            cur_y,
                            max_text_w,
                            theme,
                            circle_uv,
                            solid_uv,
                            screen_w,
                            screen_h,
                        );
                    }
                    cur_y += cell_h * 1.25;
                }
                cur_y += 6.0;
            }
            MarkdownBlock::Blockquote(inner_blocks) => {
                let bq_start_y = cur_y;
                let bq_pad_x = 14.0;

                for ib in inner_blocks {
                    if let MarkdownBlock::Paragraph(spans) = ib {
                        let lines = wrap_spans_to_lines(spans, max_text_w - bq_pad_x, cell_w);
                        for line_spans in lines {
                            if cur_y + cell_h >= prev_y && cur_y <= view_bottom {
                                render_inline_spans(
                                    renderer,
                                    queue,
                                    fonts,
                                    verts,
                                    &line_spans,
                                    prev_x + pad_x + bq_pad_x,
                                    cur_y,
                                    max_text_w - bq_pad_x,
                                    theme,
                                    circle_uv,
                                    solid_uv,
                                    screen_w,
                                    screen_h,
                                );
                            }
                            cur_y += cell_h * 1.25;
                        }
                    }
                }

                // Vertical accent bar for blockquote
                if cur_y > bq_start_y && cur_y >= prev_y && bq_start_y <= view_bottom {
                    let bar_y0 = bq_start_y.max(prev_y);
                    let bar_y1 = cur_y.min(view_bottom);
                    push_quad(
                        verts,
                        prev_x + pad_x + 2.0,
                        bar_y0,
                        prev_x + pad_x + 5.0,
                        bar_y1,
                        solid_uv,
                        solid_uv,
                        theme.accent,
                        screen_w,
                        screen_h,
                    );
                }
                cur_y += 6.0;
            }
            MarkdownBlock::List {
                ordered,
                start,
                items,
            } => {
                for (idx, item_spans) in items.iter().enumerate() {
                    let bullet = if *ordered {
                        format!("{}. ", start + idx as u64)
                    } else {
                        akssora_icons::DirtyCircleIcon::GLYPH_SPACE.to_string()
                    };
                    let bullet_w = bullet.len() as f32 * cell_w;

                    if cur_y + cell_h >= prev_y && cur_y <= view_bottom {
                        renderer.render_text(
                            queue,
                            fonts,
                            verts,
                            &bullet,
                            prev_x + pad_x + 4.0,
                            cur_y,
                            bullet_w + 4.0,
                            theme.accent,
                            true,
                            screen_w,
                            screen_h,
                        );

                        render_inline_spans(
                            renderer,
                            queue,
                            fonts,
                            verts,
                            item_spans,
                            prev_x + pad_x + 4.0 + bullet_w,
                            cur_y,
                            max_text_w - bullet_w - 4.0,
                            theme,
                            circle_uv,
                            solid_uv,
                            screen_w,
                            screen_h,
                        );
                    }
                    cur_y += cell_h * 1.3;
                }
                cur_y += 6.0;
            }
            MarkdownBlock::TaskList { items } => {
                for (checked, item_spans) in items {
                    let box_glyph = if *checked {
                        akssora_icons::CheckboxCheckedIcon::GLYPH
                    } else {
                        akssora_icons::CheckboxUncheckedIcon::GLYPH
                    };
                    let box_w = box_glyph.len() as f32 * cell_w;
                    let box_col = if *checked {
                        theme.accent
                    } else {
                        theme.text_muted
                    };

                    if cur_y + cell_h >= prev_y && cur_y <= view_bottom {
                        renderer.render_text(
                            queue,
                            fonts,
                            verts,
                            box_glyph,
                            prev_x + pad_x + 4.0,
                            cur_y,
                            box_w + 4.0,
                            box_col,
                            true,
                            screen_w,
                            screen_h,
                        );

                        render_inline_spans(
                            renderer,
                            queue,
                            fonts,
                            verts,
                            item_spans,
                            prev_x + pad_x + 4.0 + box_w,
                            cur_y,
                            max_text_w - box_w - 4.0,
                            theme,
                            circle_uv,
                            solid_uv,
                            screen_w,
                            screen_h,
                        );
                    }
                    cur_y += cell_h * 1.3;
                }
                cur_y += 6.0;
            }
            MarkdownBlock::CodeBlock { language, code } => {
                let block_h = code.len() as f32 * cell_h + 16.0;
                let cb_x = prev_x + pad_x;
                let cb_w = max_text_w;

                if cur_y + block_h >= prev_y && cur_y <= view_bottom {
                    // Code block container
                    push_rounded_rect(
                        verts,
                        cb_x,
                        cur_y,
                        cb_x + cb_w,
                        cur_y + block_h,
                        4.0,
                        circle_uv,
                        solid_uv,
                        theme.sidebar_background,
                        screen_w,
                        screen_h,
                    );

                    // Language label badge in upper right
                    if let Some(lang) = language {
                        let lang_w = lang.len() as f32 * ui_fonts.cell_w;
                        renderer.render_text(
                            queue,
                            ui_fonts,
                            verts,
                            lang,
                            cb_x + cb_w - lang_w - 8.0,
                            cur_y + 4.0,
                            lang_w + 4.0,
                            theme.text_muted,
                            false,
                            screen_w,
                            screen_h,
                        );
                    }

                    // Render code lines
                    for (line_idx, line) in code.iter().enumerate() {
                        let line_y = cur_y + 8.0 + line_idx as f32 * cell_h;
                        if line_y >= prev_y && line_y + cell_h <= view_bottom {
                            render_highlighted_code_line(
                                renderer,
                                queue,
                                fonts,
                                verts,
                                line,
                                cb_x + 10.0,
                                line_y,
                                cb_w - 20.0,
                                theme,
                                screen_w,
                                screen_h,
                            );
                        }
                    }
                }
                cur_y += block_h + 10.0;
            }
            MarkdownBlock::Table {
                headers,
                alignments: _,
                rows,
            } => {
                let col_count = headers.len();
                if col_count > 0 {
                    let col_w = (max_text_w / col_count as f32).max(40.0);
                    let row_h = cell_h * 1.4;
                    let table_h = (rows.len() + 1) as f32 * row_h;

                    if cur_y + table_h >= prev_y && cur_y <= view_bottom {
                        // Header row background
                        push_quad(
                            verts,
                            prev_x + pad_x,
                            cur_y,
                            prev_x + pad_x + max_text_w,
                            cur_y + row_h,
                            solid_uv,
                            solid_uv,
                            theme.header_background,
                            screen_w,
                            screen_h,
                        );

                        // Header cells
                        for (col_idx, h_text) in headers.iter().enumerate() {
                            let cell_x = prev_x + pad_x + col_idx as f32 * col_w + 6.0;
                            renderer.render_text(
                                queue,
                                fonts,
                                verts,
                                h_text,
                                cell_x,
                                cur_y + (row_h - cell_h) * 0.5,
                                col_w - 12.0,
                                theme.text_primary,
                                true,
                                screen_w,
                                screen_h,
                            );
                        }

                        // Divider rule below header
                        push_quad(
                            verts,
                            prev_x + pad_x,
                            cur_y + row_h,
                            prev_x + pad_x + max_text_w,
                            cur_y + row_h + 1.0,
                            solid_uv,
                            solid_uv,
                            theme.border,
                            screen_w,
                            screen_h,
                        );

                        // Data rows
                        for (r_idx, row) in rows.iter().enumerate() {
                            let row_y = cur_y + (r_idx + 1) as f32 * row_h;
                            for (col_idx, cell_text) in row.iter().enumerate() {
                                if col_idx < col_count {
                                    let cell_x = prev_x + pad_x + col_idx as f32 * col_w + 6.0;
                                    renderer.render_text(
                                        queue,
                                        fonts,
                                        verts,
                                        cell_text,
                                        cell_x,
                                        row_y + (row_h - cell_h) * 0.5,
                                        col_w - 12.0,
                                        theme.foreground,
                                        false,
                                        screen_w,
                                        screen_h,
                                    );
                                }
                            }
                            // Row bottom border
                            push_quad(
                                verts,
                                prev_x + pad_x,
                                row_y + row_h,
                                prev_x + pad_x + max_text_w,
                                row_y + row_h + 1.0,
                                solid_uv,
                                solid_uv,
                                theme.border_subtle,
                                screen_w,
                                screen_h,
                            );
                        }
                    }
                    cur_y += table_h + 10.0;
                }
            }
            MarkdownBlock::ThematicBreak => {
                cur_y += 6.0;
                if cur_y >= prev_y && cur_y <= view_bottom {
                    push_quad(
                        verts,
                        prev_x + pad_x,
                        cur_y,
                        prev_x + prev_w - pad_x,
                        cur_y + 1.0,
                        solid_uv,
                        solid_uv,
                        theme.border_subtle,
                        screen_w,
                        screen_h,
                    );
                }
                cur_y += 8.0;
            }
            MarkdownBlock::RawHtml(html) => {
                if cur_y + cell_h >= prev_y && cur_y <= view_bottom {
                    renderer.render_text(
                        queue,
                        fonts,
                        verts,
                        html,
                        prev_x + pad_x,
                        cur_y,
                        max_text_w,
                        theme.text_muted,
                        false,
                        screen_w,
                        screen_h,
                    );
                }
                cur_y += cell_h * 1.25;
            }
        }
    }

    // Render Preview Scrollbar if content exceeds viewport
    let total_content_h = cur_y - prev_y + scroll_y + pad_y;
    if total_content_h > prev_h && prev_h > 0.0 {
        let max_scroll = (total_content_h - prev_h).max(1.0);
        let ratio = (prev_h / total_content_h).clamp(0.05, 1.0);
        let thumb_h = (prev_h * ratio)
            .max(Scrollbar::MIN_THUMB_HEIGHT)
            .min(prev_h);
        let travel = prev_h - thumb_h;
        let norm = (scroll_y / max_scroll).clamp(0.0, 1.0);
        let thumb_y = prev_y + travel * norm;
        let sb_x = prev_x + prev_w - Scrollbar::VISIBLE_WIDTH - 2.0;

        push_quad(
            verts,
            sb_x,
            prev_y,
            sb_x + Scrollbar::VISIBLE_WIDTH,
            prev_y + prev_h,
            solid_uv,
            solid_uv,
            theme.scrollbar_background,
            screen_w,
            screen_h,
        );

        push_pill_thumb(
            verts,
            sb_x,
            thumb_y,
            Scrollbar::VISIBLE_WIDTH,
            thumb_h,
            circle_uv,
            solid_uv,
            theme.session_selected_background,
            screen_w,
            screen_h,
        );
    }
}
