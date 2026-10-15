use akssora_terminal_config::Theme;
use akssora_terminal_ui::{CodeReviewModel, Header, HeaderItem};

use crate::font::FontManager;
use crate::glyph::GlyphRenderer;
use crate::quad::push_rounded_rect;
use crate::vertex::GlyphVertex;

// INFO: Renders the Git branch indicator and branch switching dropdown on the left side beside branding.
#[allow(clippy::too_many_arguments)]
pub fn render_branch_header_button(
    renderer: &mut GlyphRenderer,
    queue: &wgpu::Queue,
    fonts: &FontManager,
    verts: &mut Vec<GlyphVertex>,
    theme: &Theme,
    header: &Header,
    show_sidebar_toggle: bool,
    solid_uv: [f32; 2],
    circle_uv: [[f32; 2]; 2],
    screen_w: f32,
    screen_h: f32,
) {
    if !header.is_git_repo {
        return;
    }

    let Some(branch) = &header.git_branch else {
        return;
    };

    let (bx, by, bw, bh) = header.branch_button_rect(show_sidebar_toggle, true);
    if bw <= 0.0 {
        return;
    }

    let is_hovered = header.hovered_item == Some(HeaderItem::BranchDropdown);
    let is_open = header.branch_dropdown_open;

    // Button container pill
    if is_hovered || is_open {
        let bg_col = if is_open {
            theme.header_button_active_background
        } else {
            theme.header_button_hover_background
        };
        push_rounded_rect(
            verts,
            bx,
            by,
            bx + bw,
            by + bh,
            4.0,
            circle_uv,
            solid_uv,
            bg_col,
            screen_w,
            screen_h,
        );
    }

    let cell_h = fonts.cell_h;
    let text_y = by + (bh - cell_h) * 0.5;
    let mut curr_x = bx + 6.0;

    let branch_label = akssora_icons::GitBranchIcon::format_branch(branch);
    let branch_w = branch_label.len() as f32 * fonts.cell_w;
    renderer.render_text(
        queue,
        fonts,
        verts,
        &branch_label,
        curr_x,
        text_y,
        branch_w + 4.0,
        if is_hovered || is_open {
            theme.text_primary
        } else {
            theme.text_secondary
        },
        false,
        screen_w,
        screen_h,
    );
    curr_x += branch_w + 4.0;

    // Dropdown chevron if multiple branches exist
    if header.git_branches.len() > 1 {
        let chevron = if header.branch_dropdown_open {
            akssora_icons::ChevronUpIcon::GLYPH
        } else {
            akssora_icons::ChevronDownIcon::GLYPH
        };
        renderer.render_text(
            queue,
            fonts,
            verts,
            chevron,
            curr_x,
            text_y,
            12.0,
            theme.text_muted,
            false,
            screen_w,
            screen_h,
        );
    }
}

// INFO: Renders the branch switcher dropdown list popup under the left branch button with top Z-index.
#[allow(clippy::too_many_arguments)]
pub fn render_branch_dropdown_list(
    renderer: &mut GlyphRenderer,
    queue: &wgpu::Queue,
    fonts: &FontManager,
    verts: &mut Vec<GlyphVertex>,
    theme: &Theme,
    header: &Header,
    show_sidebar_toggle: bool,
    solid_uv: [f32; 2],
    circle_uv: [[f32; 2]; 2],
    screen_w: f32,
    screen_h: f32,
) {
    let (mx, my, mw, mh) = header.branch_dropdown_rect(show_sidebar_toggle, true);

    // Dropdown border / shadow
    push_rounded_rect(
        verts,
        mx - 1.0,
        my - 1.0,
        mx + mw + 1.0,
        my + mh + 1.0,
        4.0,
        circle_uv,
        solid_uv,
        theme.border_subtle,
        screen_w,
        screen_h,
    );

    // Dropdown background
    push_rounded_rect(
        verts,
        mx,
        my,
        mx + mw,
        my + mh,
        3.0,
        circle_uv,
        solid_uv,
        theme.sidebar_background,
        screen_w,
        screen_h,
    );

    let cell_h = fonts.cell_h;
    for (i, branch_name) in header.git_branches.iter().enumerate() {
        let (ix, iy, iw, ih) = header.branch_dropdown_item_rect(i, show_sidebar_toggle, true);
        let is_current = header.git_branch.as_deref() == Some(branch_name.as_str());
        let is_hovered = header.hovered_item == Some(HeaderItem::BranchItem(i))
            || header.hovered_branch_idx == Some(i);

        let item_bg = if is_current {
            theme.session_selected_background
        } else if is_hovered {
            theme.session_hover_background
        } else {
            [0.0, 0.0, 0.0, 0.0]
        };

        if is_current || is_hovered {
            push_rounded_rect(
                verts,
                ix,
                iy,
                ix + iw,
                iy + ih,
                3.0,
                circle_uv,
                solid_uv,
                item_bg,
                screen_w,
                screen_h,
            );
        }

        let icon = if is_current {
            akssora_icons::CheckmarkIcon::GLYPH_SPACE
        } else {
            "  "
        };
        let label = format!("{icon}{branch_name}");
        let label_col = if is_current {
            theme.text_primary
        } else if is_hovered {
            theme.foreground
        } else {
            theme.text_secondary
        };

        let text_y = iy + (ih - cell_h) * 0.5;
        renderer.render_text(
            queue,
            fonts,
            verts,
            &label,
            ix + 8.0,
            text_y,
            iw - 16.0,
            label_col,
            is_current,
            screen_w,
            screen_h,
        );
    }
}

// INFO: Renders the Code Review header button and git diff stats on the right side beside Settings.
#[allow(clippy::too_many_arguments)]
pub fn render_code_review_header_button(
    renderer: &mut GlyphRenderer,
    queue: &wgpu::Queue,
    fonts: &FontManager,
    verts: &mut Vec<GlyphVertex>,
    theme: &Theme,
    header: &Header,
    _code_review: Option<&CodeReviewModel>,
    solid_uv: [f32; 2],
    circle_uv: [[f32; 2]; 2],
    screen_w: f32,
    screen_h: f32,
) {
    if !header.is_git_repo {
        return;
    }

    let (additions, deletions) = (header.git_additions, header.git_deletions);
    let has_stats = additions > 0 || deletions > 0;
    let (bx, by, bw, bh) = header.code_review_button_rect(screen_w);
    if bw <= 0.0 {
        return;
    }

    let is_hovered = header.hovered_item == Some(HeaderItem::CodeReview);
    let is_focused = header.is_code_review_focused;
    let is_active = header.is_code_review_active;

    // 1. Button container background (subtle pill using dedicated theme variables)
    let bg_col = if is_active {
        theme.code_review_button_active
    } else if is_hovered || is_focused {
        theme.code_review_button_hover
    } else {
        theme.code_review_button_background
    };

    if bg_col[3] > 0.0 {
        push_rounded_rect(
            verts,
            bx,
            by,
            bx + bw,
            by + bh,
            4.0,
            circle_uv,
            solid_uv,
            bg_col,
            screen_w,
            screen_h,
        );
    }

    // 2. Diff Icon Design (viewBox: 0 0 14 14):
    // Plus sign on top, minus bar on bottom
    let icon_color = if is_active || is_focused || is_hovered {
        theme.text_primary
    } else {
        theme.text_secondary
    };

    let icon_x = if has_stats {
        bx + 8.0
    } else {
        (bx + (bw - akssora_icons::CodeReviewDiffIcon::SIZE) * 0.5).round()
    };
    let icon_y = by + (bh - akssora_icons::CodeReviewDiffIcon::SIZE) * 0.5;

    let mut sink =
        crate::icon_sink::VertexIconSink::new(verts, solid_uv, circle_uv, screen_w, screen_h);
    akssora_icons::CodeReviewDiffIcon::draw(&mut sink, icon_x, icon_y, icon_color);

    let cell_h = fonts.cell_h;
    let text_y = by + (bh - cell_h) * 0.5;
    let mut curr_x = icon_x + akssora_icons::CodeReviewDiffIcon::SIZE + 6.0;

    // 3. Stats badges: additions (+N in green) and deletions (-N in red)
    if has_stats {
        if additions > 0 {
            let add_str = format!("+{additions}");
            let str_w = add_str.len() as f32 * fonts.cell_w;
            renderer.render_text(
                queue,
                fonts,
                verts,
                &add_str,
                curr_x,
                text_y,
                str_w + 4.0,
                [0.35, 0.85, 0.40, 1.0],
                false,
                screen_w,
                screen_h,
            );
            curr_x += str_w + 4.0;
        }

        if additions > 0 && deletions > 0 {
            renderer.render_text(
                queue,
                fonts,
                verts,
                "·",
                curr_x,
                text_y,
                fonts.cell_w + 2.0,
                theme.text_muted,
                false,
                screen_w,
                screen_h,
            );
            curr_x += fonts.cell_w + 4.0;
        }

        if deletions > 0 {
            let del_str = format!("-{deletions}");
            let str_w = del_str.len() as f32 * fonts.cell_w;
            renderer.render_text(
                queue,
                fonts,
                verts,
                &del_str,
                curr_x,
                text_y,
                str_w + 4.0,
                [0.95, 0.35, 0.35, 1.0],
                false,
                screen_w,
                screen_h,
            );
        }
    }
}
