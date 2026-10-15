use akssora_terminal_config::Theme;
use akssora_terminal_ui::{CodeReviewAction, CodeReviewLayout, CodeReviewModel};

use crate::font::FontManager;
use crate::glyph::GlyphRenderer;
use crate::quad::{push_quad, push_rounded_rect};
use crate::vertex::GlyphVertex;

// INFO: Renders the Code Review top header bar with Uncommitted changes, Commit, and Close button.
#[allow(clippy::too_many_arguments)]
pub fn render_header(
    renderer: &mut GlyphRenderer,
    queue: &wgpu::Queue,
    ui_fonts: &FontManager,
    theme: &Theme,
    model: &CodeReviewModel,
    hovered_action: Option<&CodeReviewAction>,
    ws_x: f32,
    ws_y: f32,
    ws_w: f32,
    verts: &mut Vec<GlyphVertex>,
    solid_uv: [f32; 2],
    circle_uv: [[f32; 2]; 2],
    screen_w: f32,
    screen_h: f32,
) {
    let header_h = CodeReviewLayout::HEADER_HEIGHT;
    let ui_cell_h = ui_fonts.cell_h;

    // Header background
    push_quad(
        verts,
        ws_x,
        ws_y,
        ws_x + ws_w,
        ws_y + header_h,
        solid_uv,
        solid_uv,
        theme.code_review_header_background,
        screen_w,
        screen_h,
    );
    // Divider line below header
    push_quad(
        verts,
        ws_x,
        ws_y + header_h - 1.0,
        ws_x + ws_w,
        ws_y + header_h,
        solid_uv,
        solid_uv,
        theme.code_review_header_divider,
        screen_w,
        screen_h,
    );

    // INFO: Sidebar collapse/expand toggle button.
    let (sx, sy, sw, sh) = CodeReviewLayout::sidebar_toggle_btn_rect(ws_x, ws_y);
    let is_sb_hovered = hovered_action == Some(&CodeReviewAction::ToggleSidebar);
    let is_expanded = model.sidebar_expanded;
    if is_sb_hovered || is_expanded {
        let sb_bg = if is_sb_hovered {
            theme.code_review_header_button_hover
        } else {
            theme.header_button_active_background
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
            sb_bg,
            screen_w,
            screen_h,
        );
    }
    let sb_icon = if is_expanded {
        akssora_icons::PanelExpandedIcon::GLYPH
    } else {
        akssora_icons::PanelCollapsedIcon::GLYPH
    };
    let sb_icon_col = if is_sb_hovered || is_expanded {
        theme.text_primary
    } else {
        theme.text_secondary
    };
    renderer.render_text(
        queue,
        ui_fonts,
        verts,
        sb_icon,
        sx + (sw - ui_fonts.cell_w) * 0.5,
        sy + (sh - ui_cell_h) * 0.5,
        sw,
        sb_icon_col,
        false,
        screen_w,
        screen_h,
    );

    let is_git = akssora_git::GitInspector::is_git_repo(&model.repo_root);

    // INFO: Single toggle button to expand or collapse all file diff accordions.
    let (tx, ty, tw, th) = CodeReviewLayout::toggle_collapse_all_btn_rect(ws_x, ws_y);
    let is_toggle_hovered = hovered_action == Some(&CodeReviewAction::ToggleCollapseAll)
        || hovered_action == Some(&CodeReviewAction::CollapseAll)
        || hovered_action == Some(&CodeReviewAction::ExpandAll);
    let toggle_bg = if is_toggle_hovered {
        theme.code_review_header_button_hover
    } else {
        theme.code_review_header_button_background
    };
    push_rounded_rect(
        verts,
        tx,
        ty,
        tx + tw,
        ty + th,
        4.0,
        circle_uv,
        solid_uv,
        toggle_bg,
        screen_w,
        screen_h,
    );
    let all_collapsed =
        !model.diff.files.is_empty() && model.collapsed_files.len() >= model.diff.files.len();
    let toggle_label = if all_collapsed {
        "Expand All"
    } else {
        "Collapse All"
    };
    let text_w = toggle_label.len() as f32 * ui_fonts.cell_w;
    let text_x = tx + (tw - text_w) * 0.5;
    let text_y = ty + (th - ui_cell_h) * 0.5;
    renderer.render_text(
        queue,
        ui_fonts,
        verts,
        toggle_label,
        text_x,
        text_y,
        tw,
        theme.text_primary,
        false,
        screen_w,
        screen_h,
    );

    // Right: "Commit" button
    if is_git {
        let (cx, cy, cw, ch) = CodeReviewLayout::commit_btn_rect(ws_x, ws_y, ws_w);
        let is_commit_hovered = hovered_action == Some(&CodeReviewAction::OpenCommitDialog);
        let commit_bg = if is_commit_hovered {
            theme.code_review_header_button_hover
        } else {
            theme.code_review_header_button_background
        };
        push_rounded_rect(
            verts,
            cx,
            cy,
            cx + cw,
            cy + ch,
            4.0,
            circle_uv,
            solid_uv,
            commit_bg,
            screen_w,
            screen_h,
        );
        let commit_text = "Commit";
        let tw = commit_text.len() as f32 * ui_fonts.cell_w;
        let tx = cx + ((cw - tw) * 0.5).max(4.0);
        renderer.render_text(
            queue,
            ui_fonts,
            verts,
            commit_text,
            tx,
            cy + (ch - ui_cell_h) * 0.5,
            cw - 8.0,
            theme.text_primary,
            false,
            screen_w,
            screen_h,
        );
    }

    // Right: Expand / Restore toggle button
    let (ex, ey, ew, eh) = CodeReviewLayout::expand_btn_rect(ws_x, ws_y, ws_w);
    let is_expand_hovered = hovered_action == Some(&CodeReviewAction::ToggleExpandView);
    if is_expand_hovered {
        push_rounded_rect(
            verts,
            ex,
            ey,
            ex + ew,
            ey + eh,
            4.0,
            circle_uv,
            solid_uv,
            theme.code_review_header_button_hover,
            screen_w,
            screen_h,
        );
    }
    let expand_cx = (ex + ew * 0.5).round();
    let expand_cy = (ey + eh * 0.5).round();
    let expand_icon_col = if is_expand_hovered {
        theme.text_primary
    } else {
        theme.text_secondary
    };
    {
        let mut sink = crate::VertexIconSink::new(verts, solid_uv, circle_uv, screen_w, screen_h);
        if model.is_full_view {
            akssora_icons::WindowRestoreIcon::draw(
                &mut sink,
                expand_cx,
                expand_cy,
                1.2,
                expand_icon_col,
            );
        } else {
            akssora_icons::WindowMaximizeIcon::draw(
                &mut sink,
                expand_cx,
                expand_cy,
                1.2,
                expand_icon_col,
            );
        }
    }

    // Right: Close button
    let (clx, cly, clw, clh) = CodeReviewLayout::close_btn_rect(ws_x, ws_y, ws_w);
    let is_close_hovered = hovered_action == Some(&CodeReviewAction::ClosePanel);
    if is_close_hovered {
        push_rounded_rect(
            verts,
            clx,
            cly,
            clx + clw,
            cly + clh,
            4.0,
            circle_uv,
            solid_uv,
            theme.code_review_header_button_hover,
            screen_w,
            screen_h,
        );
    }
    let close_cx = (clx + clw * 0.5).round();
    let close_cy = (cly + clh * 0.5).round();
    let close_icon_col = if is_close_hovered {
        theme.text_primary
    } else {
        theme.text_secondary
    };
    {
        let mut sink = crate::VertexIconSink::new(verts, solid_uv, circle_uv, screen_w, screen_h);
        akssora_icons::WindowCloseIcon::draw(&mut sink, close_cx, close_cy, 1.2, close_icon_col);
    }
}
