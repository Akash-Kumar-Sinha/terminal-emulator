use akssora_git::FileDiff;
use akssora_terminal_config::Theme;
use akssora_terminal_ui::{CodeReviewAction, CodeReviewLayout, CodeReviewModel};

use crate::font::FontManager;
use crate::glyph::GlyphRenderer;
use crate::quad::{push_quad, push_rounded_rect};
use crate::vertex::GlyphVertex;

// INFO: Renders file diff card header bar, title, stats pill, and accordion buttons.
#[allow(clippy::too_many_arguments)]
pub fn render_card_header(
    renderer: &mut GlyphRenderer,
    queue: &wgpu::Queue,
    ui_fonts: &FontManager,
    theme: &Theme,
    model: &CodeReviewModel,
    file: &FileDiff,
    file_idx: usize,
    hovered_action: Option<&CodeReviewAction>,
    is_collapsed: bool,
    cx: f32,
    header_y: f32,
    cw: f32,
    header_h: f32,
    clip_top: f32,
    clip_h: f32,
    verts: &mut Vec<GlyphVertex>,
    solid_uv: [f32; 2],
    circle_uv: [[f32; 2]; 2],
    screen_w: f32,
    screen_h: f32,
) {
    let ui_cell_h = ui_fonts.cell_h;
    let header_bot = (header_y + header_h).min(clip_top + clip_h);
    if header_bot <= header_y.max(clip_top) || header_y >= clip_top + clip_h {
        return;
    }

    let hdr_y0 = header_y.max(clip_top);
    push_rounded_rect(
        verts,
        cx,
        hdr_y0,
        cx + cw,
        header_bot,
        6.0,
        circle_uv,
        solid_uv,
        theme.code_review_accordion_trigger_background,
        screen_w,
        screen_h,
    );

    if !is_collapsed && header_y <= clip_top + 1.0 {
        push_quad(
            verts,
            cx,
            header_bot - 1.0,
            cx + cw,
            header_bot,
            solid_uv,
            solid_uv,
            theme.code_review_card_border,
            screen_w,
            screen_h,
        );
    }

    let header_text_y = header_y + (header_h - ui_cell_h) * 0.5;
    if header_text_y >= clip_top && header_text_y + ui_cell_h <= clip_top + clip_h {
        // Chevron icon: ▶ (collapsed) or ▼ (expanded)
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
            cx + 12.0,
            header_text_y,
            14.0,
            theme.text_secondary,
            false,
            screen_w,
            screen_h,
        );

        // File path
        let path_str = file.path.to_string_lossy().to_string();
        let path_w = (path_str.len() as f32 * ui_fonts.cell_w).min(cw * 0.45);
        let path_x = cx + 28.0;
        renderer.render_text(
            queue,
            ui_fonts,
            verts,
            &path_str,
            path_x,
            header_text_y,
            path_w + 4.0,
            theme.text_primary,
            false,
            screen_w,
            screen_h,
        );

        // Copy icon ⧉
        let copy_x = path_x + path_w + 8.0;
        renderer.render_text(
            queue,
            ui_fonts,
            verts,
            akssora_icons::CopyIcon::GLYPH,
            copy_x,
            header_text_y,
            16.0,
            theme.text_secondary,
            false,
            screen_w,
            screen_h,
        );

        // Stats pill
        let pill_x = copy_x + 24.0;
        let pill_w = 76.0;
        let pill_h = 20.0;
        let pill_y = header_y + (header_h - pill_h) * 0.5;
        push_rounded_rect(
            verts,
            pill_x,
            pill_y,
            pill_x + pill_w,
            pill_y + pill_h,
            3.0,
            circle_uv,
            solid_uv,
            theme.diff_stats_pill_background,
            screen_w,
            screen_h,
        );
        let add_str = format!("+{}", file.additions);
        let del_str = format!("-{}", file.deletions);
        let mut pill_inner_x = pill_x + 8.0;
        let pill_text_y = pill_y + (pill_h - ui_cell_h) * 0.5;

        let add_w = add_str.len() as f32 * ui_fonts.cell_w;
        renderer.render_text(
            queue,
            ui_fonts,
            verts,
            &add_str,
            pill_inner_x,
            pill_text_y,
            add_w + 2.0,
            theme.diff_add_green,
            false,
            screen_w,
            screen_h,
        );
        pill_inner_x += add_w + 4.0;

        renderer.render_text(
            queue,
            ui_fonts,
            verts,
            "·",
            pill_inner_x,
            pill_text_y,
            8.0,
            theme.text_muted,
            false,
            screen_w,
            screen_h,
        );
        pill_inner_x += 8.0;

        let del_w = del_str.len() as f32 * ui_fonts.cell_w;
        renderer.render_text(
            queue,
            ui_fonts,
            verts,
            &del_str,
            pill_inner_x,
            pill_text_y,
            del_w + 2.0,
            theme.diff_delete_red,
            false,
            screen_w,
            screen_h,
        );

        // Accordion trigger buttons: Full File, Discard (↺), Stage/Unstage (+/−), and Open file in editor (↗).
        let (fx, fy, fw, fh) = CodeReviewLayout::card_full_file_btn_rect(cx, header_y, cw);
        let (ox, oy, ow, oh) = CodeReviewLayout::card_open_file_btn_rect(cx, header_y, cw);
        let (sx, sy, sw, sh) = CodeReviewLayout::card_stage_btn_rect(cx, header_y, cw);
        let (dx, dy, dw, dh) = CodeReviewLayout::card_discard_btn_rect(cx, header_y, cw);

        let is_full_file_hovered =
            hovered_action == Some(&CodeReviewAction::ToggleFullFile(file_idx));
        let is_full_file_active = model.is_full_file(&file.path);
        let is_open_hovered = hovered_action == Some(&CodeReviewAction::OpenFileInEditor(file_idx));
        let is_stage_hovered = if model.is_staged {
            hovered_action == Some(&CodeReviewAction::UnstageFile(file_idx))
        } else {
            hovered_action == Some(&CodeReviewAction::StageFile(file_idx))
        };
        let is_discard_hovered = hovered_action == Some(&CodeReviewAction::DiscardFile(file_idx));

        // 0. Full File toggle button
        let full_file_bg = if is_full_file_hovered {
            theme.code_review_action_button_hover
        } else if is_full_file_active {
            theme.diff_stats_pill_background
        } else {
            theme.code_review_header_button_background
        };
        push_rounded_rect(
            verts,
            fx,
            fy,
            fx + fw,
            fy + fh,
            4.0,
            circle_uv,
            solid_uv,
            full_file_bg,
            screen_w,
            screen_h,
        );
        let full_file_label = if is_full_file_active {
            "Diff"
        } else {
            "Full File"
        };
        let ff_text_w = full_file_label.len() as f32 * ui_fonts.cell_w;
        let ff_color = if is_full_file_active || is_full_file_hovered {
            theme.text_primary
        } else {
            theme.text_secondary
        };
        renderer.render_text(
            queue,
            ui_fonts,
            verts,
            full_file_label,
            fx + ((fw - ff_text_w) * 0.5).max(4.0),
            fy + (fh - ui_cell_h) * 0.5,
            fw,
            ff_color,
            false,
            screen_w,
            screen_h,
        );

        // 1. Open file button ↗
        if is_open_hovered {
            push_rounded_rect(
                verts,
                ox,
                oy,
                ox + ow,
                oy + oh,
                4.0,
                circle_uv,
                solid_uv,
                theme.code_review_action_button_hover,
                screen_w,
                screen_h,
            );
        }
        let open_color = if is_open_hovered {
            theme.text_primary
        } else {
            theme.text_secondary
        };
        renderer.render_text(
            queue,
            ui_fonts,
            verts,
            akssora_icons::ArrowUpRightIcon::GLYPH,
            ox + (ow - 12.0) * 0.5,
            oy + (oh - ui_cell_h) * 0.5,
            ow,
            open_color,
            false,
            screen_w,
            screen_h,
        );

        // 2. Stage / Unstage button (+ / −)
        let stage_bg = if is_stage_hovered {
            if model.is_staged {
                theme.diff_delete_red_background
            } else {
                theme.diff_add_green_background
            }
        } else {
            theme.code_review_header_button_background
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
            stage_bg,
            screen_w,
            screen_h,
        );
        let stage_symbol = if model.is_staged {
            akssora_icons::RemoveIcon::GLYPH
        } else {
            akssora_icons::AddIcon::GLYPH
        };
        let stage_color = if is_stage_hovered {
            if model.is_staged {
                theme.diff_delete_red
            } else {
                theme.diff_add_green
            }
        } else {
            theme.text_primary
        };
        renderer.render_text(
            queue,
            ui_fonts,
            verts,
            stage_symbol,
            sx + (sw - 10.0) * 0.5,
            sy + (sh - ui_cell_h) * 0.5,
            sw,
            stage_color,
            false,
            screen_w,
            screen_h,
        );

        // 3. Discard button ↺
        if is_discard_hovered {
            push_rounded_rect(
                verts,
                dx,
                dy,
                dx + dw,
                dy + dh,
                4.0,
                circle_uv,
                solid_uv,
                theme.diff_delete_red_background,
                screen_w,
                screen_h,
            );
        }
        let discard_color = if is_discard_hovered {
            theme.diff_delete_red
        } else {
            theme.text_secondary
        };
        renderer.render_text(
            queue,
            ui_fonts,
            verts,
            akssora_icons::ArrowUndoIcon::GLYPH,
            dx + (dw - 12.0) * 0.5,
            dy + (dh - ui_cell_h) * 0.5,
            dw,
            discard_color,
            false,
            screen_w,
            screen_h,
        );
    }
}
