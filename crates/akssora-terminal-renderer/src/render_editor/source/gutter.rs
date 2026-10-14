use akssora_git::{LineGitChange, LineGitStatus};
use akssora_terminal_config::{LineNumberMode, Theme};

use crate::font::FontManager;
use crate::glyph::GlyphRenderer;
use crate::quad::{push_quad, push_rounded_rect};
use crate::vertex::GlyphVertex;

// INFO: Renders line numbers in the gutter taking relative/absolute mode into account.
#[allow(clippy::too_many_arguments)]
pub fn render_line_number(
    renderer: &mut GlyphRenderer,
    queue: &wgpu::Queue,
    fonts: &FontManager,
    verts: &mut Vec<GlyphVertex>,
    logical_row: usize,
    cursor_row: usize,
    mode: LineNumberMode,
    src_x: f32,
    line_y: f32,
    gutter_w: f32,
    theme: &Theme,
    screen_w: f32,
    screen_h: f32,
) {
    let num_str = match mode {
        LineNumberMode::Relative => {
            if logical_row == cursor_row {
                format!("{:>3} ", logical_row + 1)
            } else {
                format!("{:>3} ", logical_row.abs_diff(cursor_row))
            }
        }
        _ => format!("{:>3} ", logical_row + 1),
    };
    let num_col = if logical_row == cursor_row {
        theme.text_primary
    } else {
        theme.text_secondary
    };
    renderer.render_text(
        queue,
        fonts,
        verts,
        &num_str,
        src_x + 4.0,
        line_y,
        gutter_w - 6.0,
        num_col,
        logical_row == cursor_row,
        screen_w,
        screen_h,
    );
}

// INFO: Renders Git status change indicators in gutter and line background tint.
#[allow(clippy::too_many_arguments)]
pub fn render_git_gutter_indicator(
    renderer: &mut GlyphRenderer,
    queue: &wgpu::Queue,
    fonts: &FontManager,
    verts: &mut Vec<GlyphVertex>,
    git_status: &LineGitStatus,
    inline_diff: bool,
    is_continuation: bool,
    logical_row: usize,
    hovered_gutter: Option<(usize, Option<akssora_editor::GutterActionKind>)>,
    src_x: f32,
    src_w: f32,
    line_y: f32,
    gutter_w: f32,
    v_sb_w: f32,
    cell_h: f32,
    theme: &Theme,
    solid_uv: [f32; 2],
    circle_uv: [[f32; 2]; 2],
    screen_w: f32,
    screen_h: f32,
) {
    if inline_diff && let Some(change) = git_status.change {
        let bg_col = match change {
            LineGitChange::Added => theme.diff_add_green_background,
            LineGitChange::Modified => theme.diff_modified_yellow_background,
            LineGitChange::Deleted => theme.diff_delete_red_background,
        };
        let bg_x1 = src_x + gutter_w;
        let bg_x2 = (src_x + src_w - v_sb_w).max(bg_x1);
        push_quad(
            verts,
            bg_x1,
            line_y,
            bg_x2,
            line_y + cell_h,
            solid_uv,
            solid_uv,
            bg_col,
            screen_w,
            screen_h,
        );
    }

    if gutter_w <= 0.0 {
        return;
    }

    let is_line_hovered = hovered_gutter.map(|(r, _)| r) == Some(logical_row);
    let has_git_change =
        git_status.change.is_some() || git_status.deleted_above || git_status.deleted_below;

    if !is_continuation && is_line_hovered && has_git_change {
        let btn_y = line_y + (cell_h - 15.0) * 0.5;
        let btn_h = 15.0;

        // Button 1: Add This Line (+)
        let btn_add_x = src_x + gutter_w - 33.0;
        let btn_add_w = 15.0;
        let is_add_hovered = hovered_gutter.and_then(|(_, a)| a)
            == Some(akssora_editor::GutterActionKind::StageLine);
        let add_bg = if is_add_hovered {
            theme.diff_gutter_badge_hover
        } else {
            theme.button_default_background
        };
        let add_fg = if is_add_hovered {
            theme.diff_gutter_add_badge_fg
        } else {
            theme.text_secondary
        };
        push_rounded_rect(
            verts,
            btn_add_x,
            btn_y,
            btn_add_x + btn_add_w,
            btn_y + btn_h,
            3.0,
            circle_uv,
            solid_uv,
            add_bg,
            screen_w,
            screen_h,
        );
        renderer.render_text(
            queue,
            fonts,
            verts,
            akssora_icons::AddIcon::GLYPH,
            btn_add_x + 3.5,
            btn_y + (btn_h - fonts.cell_h) * 0.5,
            btn_add_w,
            add_fg,
            false,
            screen_w,
            screen_h,
        );

        // Button 2: Revert This Line (↶)
        let btn_rev_x = src_x + gutter_w - 17.0;
        let btn_rev_w = 15.0;
        let is_rev_hovered = hovered_gutter.and_then(|(_, a)| a)
            == Some(akssora_editor::GutterActionKind::RevertLine);
        let rev_bg = if is_rev_hovered {
            theme.diff_gutter_badge_hover
        } else {
            theme.button_default_background
        };
        let rev_fg = if is_rev_hovered {
            theme.diff_gutter_del_badge_fg
        } else {
            theme.text_secondary
        };
        push_rounded_rect(
            verts,
            btn_rev_x,
            btn_y,
            btn_rev_x + btn_rev_w,
            btn_y + btn_h,
            3.0,
            circle_uv,
            solid_uv,
            rev_bg,
            screen_w,
            screen_h,
        );
        renderer.render_text(
            queue,
            fonts,
            verts,
            akssora_icons::ArrowRevertIcon::GLYPH,
            btn_rev_x + 2.5,
            btn_y + (btn_h - fonts.cell_h) * 0.5,
            btn_rev_w,
            rev_fg,
            false,
            screen_w,
            screen_h,
        );
        return;
    }

    if inline_diff {
        if !is_continuation {
            if let Some(change) = git_status.change {
                let badge_x = src_x + gutter_w - 18.0;
                let badge_y = line_y + (cell_h - 14.0) * 0.5;
                let (badge_bg, badge_col, badge_sym) = match change {
                    LineGitChange::Added => (
                        theme.diff_gutter_add_badge_bg,
                        theme.diff_gutter_add_badge_fg,
                        "+",
                    ),
                    LineGitChange::Modified => (
                        theme.diff_gutter_mod_badge_bg,
                        theme.diff_gutter_mod_badge_fg,
                        "~",
                    ),
                    LineGitChange::Deleted => (
                        theme.diff_gutter_del_badge_bg,
                        theme.diff_gutter_del_badge_fg,
                        "-",
                    ),
                };
                push_rounded_rect(
                    verts,
                    badge_x,
                    badge_y,
                    badge_x + 14.0,
                    badge_y + 14.0,
                    3.0,
                    circle_uv,
                    solid_uv,
                    badge_bg,
                    screen_w,
                    screen_h,
                );
                renderer.render_text(
                    queue,
                    fonts,
                    verts,
                    badge_sym,
                    badge_x + 3.0,
                    badge_y + (14.0 - fonts.cell_h) * 0.5,
                    14.0,
                    badge_col,
                    false,
                    screen_w,
                    screen_h,
                );
            }
            if git_status.deleted_above {
                let badge_x = src_x + gutter_w - 18.0;
                let badge_y = line_y;
                push_rounded_rect(
                    verts,
                    badge_x,
                    badge_y,
                    badge_x + 14.0,
                    badge_y + 12.0,
                    3.0,
                    circle_uv,
                    solid_uv,
                    theme.diff_gutter_del_badge_bg,
                    screen_w,
                    screen_h,
                );
                renderer.render_text(
                    queue,
                    fonts,
                    verts,
                    "-",
                    badge_x + 3.5,
                    badge_y,
                    10.0,
                    theme.diff_gutter_del_badge_fg,
                    false,
                    screen_w,
                    screen_h,
                );
                push_quad(
                    verts,
                    src_x + gutter_w,
                    line_y,
                    src_x + src_w - v_sb_w,
                    line_y + 2.0,
                    solid_uv,
                    solid_uv,
                    theme.diff_gutter_del_badge_fg,
                    screen_w,
                    screen_h,
                );
            }
        }
        if git_status.deleted_below {
            let badge_x = src_x + gutter_w - 18.0;
            let badge_y = line_y + cell_h - 12.0;
            push_rounded_rect(
                verts,
                badge_x,
                badge_y,
                badge_x + 14.0,
                badge_y + 12.0,
                3.0,
                circle_uv,
                solid_uv,
                theme.diff_gutter_del_badge_bg,
                screen_w,
                screen_h,
            );
            renderer.render_text(
                queue,
                fonts,
                verts,
                "-",
                badge_x + 3.5,
                badge_y,
                10.0,
                theme.diff_gutter_del_badge_fg,
                false,
                screen_w,
                screen_h,
            );
            push_quad(
                verts,
                src_x + gutter_w,
                line_y + cell_h - 2.0,
                src_x + src_w - v_sb_w,
                line_y + cell_h,
                solid_uv,
                solid_uv,
                theme.diff_gutter_del_badge_fg,
                screen_w,
                screen_h,
            );
        }
    } else {
        // Standard indicator bar along right edge of gutter
        if let Some(change) = git_status.change {
            let ind_x = src_x + gutter_w - 3.0;
            let ind_w = 2.5;
            let bar_col = match change {
                LineGitChange::Added => theme.diff_gutter_add_badge_fg,
                LineGitChange::Modified => theme.diff_gutter_mod_badge_fg,
                LineGitChange::Deleted => theme.diff_gutter_del_badge_fg,
            };
            push_quad(
                verts,
                ind_x,
                line_y,
                ind_x + ind_w,
                line_y + cell_h,
                solid_uv,
                solid_uv,
                bar_col,
                screen_w,
                screen_h,
            );
        }
        if git_status.deleted_above && !is_continuation {
            let del_x = src_x + gutter_w - 4.5;
            let del_w = 4.5;
            push_quad(
                verts,
                del_x,
                line_y,
                del_x + del_w,
                line_y + 3.0,
                solid_uv,
                solid_uv,
                theme.diff_gutter_del_badge_fg,
                screen_w,
                screen_h,
            );
        }
        if git_status.deleted_below {
            let del_x = src_x + gutter_w - 4.5;
            let del_w = 4.5;
            push_quad(
                verts,
                del_x,
                line_y + cell_h - 3.0,
                del_x + del_w,
                line_y + cell_h,
                solid_uv,
                solid_uv,
                theme.diff_gutter_del_badge_fg,
                screen_w,
                screen_h,
            );
        }
    }
}
