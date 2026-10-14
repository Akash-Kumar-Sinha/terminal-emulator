use std::collections::HashMap;

use akssora_terminal_config::{TabLayoutPreference, Theme};
use akssora_terminal_ui::{Header, HeaderItem, PaneTree, TabManager};

use crate::font::FontManager;
use crate::glyph::GlyphRenderer;
use crate::quad::push_quad;
use crate::vertex::GlyphVertex;

// INFO: Renders header titlebar, transparent sidebar toggle, titlebar tabs, single corner settings button, and window buttons.
#[allow(clippy::too_many_arguments)]
pub fn render_header_bar(
    renderer: &mut GlyphRenderer,
    queue: &wgpu::Queue,
    fonts: &FontManager,
    theme: &Theme,
    header: &Header,
    tab_manager: &TabManager,
    tab_layout: TabLayoutPreference,
    split_trees: &HashMap<u64, PaneTree>,
    code_review: Option<&akssora_terminal_ui::CodeReviewModel>,
    verts: &mut Vec<GlyphVertex>,
    solid_uv: [f32; 2],
    circle_uv: [[f32; 2]; 2],
    screen_w: f32,
    screen_h: f32,
) {
    let cell_h = fonts.cell_h;
    let show_sidebar_toggle = tab_layout.show_sidebar_toggle();

    // ---------------------------------------------------------
    // Tier 1: Existing Application Header (Always Visible)
    // ---------------------------------------------------------
    push_quad(
        verts,
        0.0,
        0.0,
        screen_w,
        header.height,
        solid_uv,
        solid_uv,
        theme.header_background,
        screen_w,
        screen_h,
    );

    // 1. Sidebar Toggle button (only displayed when tab layout includes sidebar)
    if show_sidebar_toggle {
        crate::render_sidebar_toggle_button(
            verts, theme, header, solid_uv, circle_uv, screen_w, screen_h,
        );
    }

    // 1.1 Explorer Toggle button (beside sidebar toggle button)
    crate::render_explorer_toggle::render_explorer_toggle_button(
        verts, theme, header, solid_uv, circle_uv, screen_w, screen_h,
    );

    // 2. Akssora branding & application name (visible at all times)
    let (bx, _, _, _) = header.branding_rect_ex(show_sidebar_toggle, true);
    let brand_y = (header.height - cell_h) / 2.0;
    renderer.render_text(
        queue,
        fonts,
        verts,
        akssora_icons::AppLogoIcon::WORDMARK,
        bx,
        brand_y,
        80.0,
        theme.text_primary,
        false,
        screen_w,
        screen_h,
    );

    // 2.05 Left-side Git branch indicator and branch dropdown menu
    crate::render_code_review_button::render_branch_header_button(
        renderer,
        queue,
        fonts,
        verts,
        theme,
        header,
        show_sidebar_toggle,
        solid_uv,
        circle_uv,
        screen_w,
        screen_h,
    );

    // INFO: 2.1 Center Quick File Search bar (always accessible in header center)
    let (qs_x, qs_y, qs_w, qs_h) = header.quick_search_rect(screen_w);
    let is_qs_hovered = header.hovered_item == Some(HeaderItem::QuickSearch);
    let qs_bg = if is_qs_hovered {
        theme.search_bar_hover_background
    } else {
        theme.search_bar_background
    };
    crate::quad::push_rounded_rect(
        verts,
        qs_x,
        qs_y,
        qs_x + qs_w,
        qs_y + qs_h,
        4.0,
        circle_uv,
        solid_uv,
        qs_bg,
        screen_w,
        screen_h,
    );
    let qs_text_y = qs_y + (qs_h - cell_h) * 0.5;
    renderer.render_text(
        queue,
        fonts,
        verts,
        akssora_icons::SearchIcon::HEADER_LABEL,
        qs_x + 10.0,
        qs_text_y,
        qs_w - 20.0,
        theme.text_secondary,
        false,
        screen_w,
        screen_h,
    );

    // 2.2 Code Review header button (beside Settings button)
    crate::render_code_review_header_button(
        renderer,
        queue,
        fonts,
        verts,
        theme,
        header,
        code_review,
        solid_uv,
        circle_uv,
        screen_w,
        screen_h,
    );

    // 3. Single permanent Settings button in the corner (next to window control buttons)
    crate::render_settings_button(
        verts, theme, header, solid_uv, circle_uv, screen_w, screen_h,
    );

    // 4. Native window controls (Minimize, Maximize/Restore, Close)
    crate::render_window_controls(
        verts, theme, header, solid_uv, circle_uv, screen_w, screen_h,
    );

    // ---------------------------------------------------------
    // Tier 2: Terminal Tab Bar (Directly below Header when TitleBar layout is active)
    // ---------------------------------------------------------
    if tab_layout == TabLayoutPreference::TitleBar {
        let (tbx, tby, tbw, tbh) = header.tab_bar_rect(screen_w);

        // Tab bar background
        push_quad(
            verts,
            tbx,
            tby,
            tbx + tbw,
            tby + tbh,
            solid_uv,
            solid_uv,
            theme.title_bar_background,
            screen_w,
            screen_h,
        );

        // Horizontal tabs
        for (i, tab) in tab_manager.tabs().iter().enumerate() {
            if let Some((tx, ty, tw, th)) = header.tab_rect(i, tab_manager.len(), screen_w) {
                let is_active = i == tab_manager.active_index();
                let is_hovered = header.hovered_item == Some(HeaderItem::Tab(i));

                // Tab pill background
                let tab_bg = if is_active {
                    theme.tab_selected_background
                } else if is_hovered {
                    theme.tab_hover_background
                } else {
                    theme.tab_background
                };

                push_quad(
                    verts,
                    tx,
                    ty,
                    tx + tw,
                    ty + th,
                    solid_uv,
                    solid_uv,
                    tab_bg,
                    screen_w,
                    screen_h,
                );

                let split_count = split_trees
                    .get(&tab.id)
                    .map(|t| t.all_leaf_ids().len())
                    .unwrap_or(0);
                let title_text = if split_count > 1 {
                    format!("{} [{}]", tab.display_name(), split_count)
                } else {
                    tab.display_name().to_string()
                };

                let text_x = tx + 8.0;
                let text_y = ty + (th - cell_h) / 2.0;
                let text_w = (tw - 32.0).max(10.0);
                let text_col = if is_active {
                    theme.text_primary
                } else {
                    theme.text_secondary
                };
                renderer.render_text(
                    queue,
                    fonts,
                    verts,
                    &title_text,
                    text_x,
                    text_y,
                    text_w,
                    text_col,
                    false,
                    screen_w,
                    screen_h,
                );

                // Tab close button - displayed on active tab or when hovered
                let is_close_hovered = header.hovered_item == Some(HeaderItem::TabClose(i));
                let should_show_close = is_active || is_hovered || is_close_hovered;
                if should_show_close
                    && let Some((cx, cy, cw, ch)) =
                        header.tab_close_rect(i, tab_manager.len(), screen_w)
                {
                    if is_close_hovered {
                        push_quad(
                            verts,
                            cx,
                            cy,
                            cx + cw,
                            cy + ch,
                            circle_uv[0],
                            circle_uv[1],
                            theme.close_button_hover_background,
                            screen_w,
                            screen_h,
                        );
                    }
                    let close_col = if is_close_hovered {
                        theme.close_button_foreground
                    } else if is_active {
                        theme.text_secondary
                    } else {
                        theme.text_muted
                    };
                    if let Some(info) =
                        renderer.glyph(queue, fonts, akssora_icons::CloseIcon::CHAR, false, false)
                    {
                        let gx = cx + (cw - info.width) / 2.0;
                        let gy = cy + (ch - info.height) / 2.0;
                        push_quad(
                            verts,
                            gx,
                            gy,
                            gx + info.width,
                            gy + info.height,
                            info.uv_min,
                            info.uv_max,
                            close_col,
                            screen_w,
                            screen_h,
                        );
                    }
                }
            }
        }

        // New Tab '+' button directly after tabs
        let (nx, ny, nw, nh) = header.new_tab_rect(tab_manager.len(), screen_w);
        let is_new_hovered = header.hovered_item == Some(HeaderItem::NewTab);
        if is_new_hovered {
            push_quad(
                verts,
                nx,
                ny,
                nx + nw,
                ny + nh,
                circle_uv[0],
                circle_uv[1],
                theme.header_button_hover_background,
                screen_w,
                screen_h,
            );
        }
        let new_col = if is_new_hovered {
            theme.accent_hover
        } else {
            theme.text_secondary
        };
        if let Some(info) = renderer.glyph(queue, fonts, akssora_icons::AddIcon::CHAR, false, false)
        {
            let gx = nx + (nw - info.width) / 2.0;
            let gy = ny + (nh - info.height) / 2.0;
            push_quad(
                verts,
                gx,
                gy,
                gx + info.width,
                gy + info.height,
                info.uv_min,
                info.uv_max,
                new_col,
                screen_w,
                screen_h,
            );
        }
    }
}
