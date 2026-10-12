use std::collections::HashMap;

use akssora_terminal_config::Theme;
use akssora_terminal_ui::{PaneTree, Sidebar, SidebarRowKind, TabManager};

use crate::font::FontManager;
use crate::glyph::GlyphRenderer;
use crate::quad::push_quad;
use crate::vertex::GlyphVertex;

// INFO: Renders vertical navigation panel, hierarchical tab groups, split sessions, and new tab trigger.
#[allow(clippy::too_many_arguments)]
pub fn render_sidebar_panel(
    renderer: &mut GlyphRenderer,
    queue: &wgpu::Queue,
    fonts: &FontManager,
    theme: &Theme,
    sidebar: &Sidebar,
    tab_manager: &TabManager,
    split_trees: &HashMap<u64, PaneTree>,
    active_pane_id: Option<u64>,
    sessions: Option<&[akssora_terminal_ui::SessionRowData]>,
    header_h: f32,
    verts: &mut Vec<GlyphVertex>,
    solid_uv: [f32; 2],
    circle_uv: [[f32; 2]; 2],
    screen_w: f32,
    screen_h: f32,
) {
    if !sidebar.is_expanded || sidebar.width <= 0.0 {
        return;
    }

    let sidebar_w = sidebar.width;
    let sb_bg = sidebar.bg_color(theme);

    // Sidebar background from below header to bottom of screen
    push_quad(
        verts, 0.0, header_h, sidebar_w, screen_h, solid_uv, solid_uv, sb_bg, screen_w, screen_h,
    );

    let content_start_y = header_h + 8.0;
    let rows = if let Some(session_items) = sessions {
        sidebar.compute_session_rows(session_items, content_start_y)
    } else {
        sidebar.compute_rows(tab_manager.tabs(), split_trees, content_start_y)
    };
    let active_tab_idx = tab_manager.active_index();

    for row in &rows {
        let is_tab_active = row.tab_index == active_tab_idx;
        match &row.kind {
            SidebarRowKind::ProjectHeader {
                project_name,
                session_count,
                is_collapsed,
            } => {
                crate::render_sidebar_session::render_project_header(
                    renderer,
                    queue,
                    fonts,
                    theme,
                    project_name.as_str(),
                    *session_count,
                    *is_collapsed,
                    row.y,
                    row.height,
                    sidebar_w,
                    verts,
                    screen_w,
                    screen_h,
                );
            }
            SidebarRowKind::Session {
                pane_id,
                session_name,
                secondary_meta,
                is_split,
                icon,
            } => {
                let is_active = if *is_split {
                    is_tab_active && active_pane_id == Some(*pane_id)
                } else {
                    is_tab_active
                };
                crate::render_sidebar_session::render_session_entry(
                    renderer,
                    queue,
                    fonts,
                    theme,
                    sidebar,
                    *pane_id,
                    session_name.as_str(),
                    secondary_meta.as_str(),
                    *icon,
                    is_active,
                    *is_split,
                    row.y,
                    row.height,
                    sidebar_w,
                    verts,
                    solid_uv,
                    circle_uv,
                    screen_w,
                    screen_h,
                );
            }
            SidebarRowKind::NormalTab => {
                render_tab_entry(
                    renderer,
                    queue,
                    fonts,
                    theme,
                    sidebar,
                    tab_manager,
                    row.tab_index,
                    row.y,
                    row.height,
                    sidebar_w,
                    is_tab_active,
                    None,
                    verts,
                    solid_uv,
                    circle_uv,
                    screen_w,
                    screen_h,
                );
            }
            SidebarRowKind::GroupHeader {
                is_collapsed,
                split_count,
            } => {
                if sessions.is_some() {
                    let tab_title = tab_manager
                        .tabs()
                        .get(row.tab_index)
                        .map(|t| t.title.as_str());
                    crate::render_sidebar_session::render_split_group_header(
                        renderer,
                        queue,
                        fonts,
                        theme,
                        tab_title,
                        *split_count,
                        *is_collapsed,
                        row.y,
                        row.height,
                        sidebar_w,
                        verts,
                        screen_w,
                        screen_h,
                    );
                } else {
                    render_tab_entry(
                        renderer,
                        queue,
                        fonts,
                        theme,
                        sidebar,
                        tab_manager,
                        row.tab_index,
                        row.y,
                        row.height,
                        sidebar_w,
                        is_tab_active,
                        Some(*is_collapsed),
                        verts,
                        solid_uv,
                        circle_uv,
                        screen_w,
                        screen_h,
                    );
                }
            }
            SidebarRowKind::SplitChild {
                pane_id,
                child_index,
                is_last,
            } => {
                render_split_entry(
                    renderer,
                    queue,
                    fonts,
                    theme,
                    sidebar,
                    row.y,
                    row.height,
                    sidebar_w,
                    *pane_id,
                    *child_index,
                    *is_last,
                    is_tab_active,
                    active_pane_id,
                    verts,
                    solid_uv,
                    circle_uv,
                    screen_w,
                    screen_h,
                );
            }
        }
    }

    // New Tab '+' button in sidebar directly below items
    render_new_tab_entry(
        renderer,
        queue,
        fonts,
        theme,
        sidebar,
        &rows,
        content_start_y,
        verts,
        solid_uv,
        screen_w,
        screen_h,
    );
}

// INFO: Renders a top-level tab or collapsible group header row.
#[allow(clippy::too_many_arguments)]
fn render_tab_entry(
    renderer: &mut GlyphRenderer,
    queue: &wgpu::Queue,
    fonts: &FontManager,
    theme: &Theme,
    sidebar: &Sidebar,
    tab_manager: &TabManager,
    tab_idx: usize,
    ty: f32,
    th: f32,
    tw: f32,
    is_active: bool,
    collapsed: Option<bool>,
    verts: &mut Vec<GlyphVertex>,
    solid_uv: [f32; 2],
    circle_uv: [[f32; 2]; 2],
    screen_w: f32,
    screen_h: f32,
) {
    let cell_h = fonts.cell_h;
    let ascent = fonts.ascent_px;
    let is_hovered = sidebar.hovered_tab == Some(tab_idx);

    let tab_bg = if is_active && collapsed.is_none() {
        theme.session_selected_background
    } else if is_active || is_hovered {
        theme.session_hover_background
    } else {
        theme.session_background
    };

    crate::quad::push_rounded_rect(
        verts,
        6.0,
        ty + 2.0,
        tw - 6.0,
        ty + th - 2.0,
        5.0,
        circle_uv,
        solid_uv,
        tab_bg,
        screen_w,
        screen_h,
    );

    let mut icon_x = 14.0;
    if let Some(is_col) = collapsed {
        let ch = if is_col { '\u{25B8}' } else { '\u{25BE}' };
        let col_c = if is_active {
            theme.text_primary
        } else {
            theme.text_secondary
        };
        if let Some(info) = renderer.glyph(queue, fonts, ch, false, false) {
            let gx = 8.0 + info.bearing_left;
            let gy = ty + (th - cell_h) / 2.0 + ascent + info.bearing_top;
            push_quad(
                verts,
                gx,
                gy,
                gx + info.width,
                gy + info.height,
                info.uv_min,
                info.uv_max,
                col_c,
                screen_w,
                screen_h,
            );
        }
        icon_x = 22.0;
    }

    if let Some(tab) = tab_manager.tabs().get(tab_idx) {
        let icon_char = tab.kind.icon();
        let icon_y = ty + (th - cell_h) / 2.0;
        let icon_col = if is_active {
            theme.text_primary
        } else {
            theme.text_secondary
        };
        if let Some(info) = renderer.glyph(queue, fonts, icon_char, false, false) {
            let gx = icon_x + info.bearing_left;
            let gy = icon_y + ascent + info.bearing_top;
            push_quad(
                verts,
                gx,
                gy,
                gx + info.width,
                gy + info.height,
                info.uv_min,
                info.uv_max,
                icon_col,
                screen_w,
                screen_h,
            );
        }

        let title_x = icon_x + 18.0;
        let title_max_w = (tw - title_x - 30.0).max(10.0);
        let title_col = if is_active {
            theme.text_primary
        } else {
            theme.text_secondary
        };
        let char_limit = (title_max_w / fonts.cell_w.max(1.0)).floor() as usize;
        let title_str = if tab.title.chars().count() > char_limit && char_limit > 3 {
            let truncated: String = tab.title.chars().take(char_limit - 3).collect();
            format!("{}...", truncated)
        } else {
            tab.title.clone()
        };
        renderer.render_text(
            queue,
            fonts,
            verts,
            &title_str,
            title_x,
            icon_y,
            title_max_w,
            title_col,
            is_active,
            screen_w,
            screen_h,
        );
    }

    if is_active || is_hovered || sidebar.hovered_close == Some(tab_idx) {
        render_close_icon(
            renderer,
            queue,
            fonts,
            theme,
            tw,
            ty,
            th,
            sidebar.hovered_close == Some(tab_idx),
            verts,
            circle_uv,
            screen_w,
            screen_h,
        );
    }
}

// INFO: Renders an indented split pane session row with tree connector glyphs.
#[allow(clippy::too_many_arguments)]
fn render_split_entry(
    renderer: &mut GlyphRenderer,
    queue: &wgpu::Queue,
    fonts: &FontManager,
    theme: &Theme,
    sidebar: &Sidebar,
    ty: f32,
    th: f32,
    tw: f32,
    pane_id: u64,
    child_index: usize,
    is_last: bool,
    is_tab_active: bool,
    active_pane_id: Option<u64>,
    verts: &mut Vec<GlyphVertex>,
    solid_uv: [f32; 2],
    circle_uv: [[f32; 2]; 2],
    screen_w: f32,
    screen_h: f32,
) {
    let cell_h = fonts.cell_h;
    let is_pane_active = is_tab_active && active_pane_id == Some(pane_id);
    let is_hovered = sidebar.hovered_pane == Some(pane_id);

    let bg = if is_pane_active {
        theme.session_selected_background
    } else if is_hovered {
        theme.session_hover_background
    } else {
        [0.0, 0.0, 0.0, 0.0]
    };

    crate::quad::push_rounded_rect(
        verts,
        14.0,
        ty + 1.0,
        tw - 6.0,
        ty + th - 1.0,
        4.0,
        circle_uv,
        solid_uv,
        bg,
        screen_w,
        screen_h,
    );

    let branch = if is_last {
        "\u{2514}\u{2500} "
    } else {
        "\u{251C}\u{2500} "
    };
    let label = format!("{}Terminal {}", branch, child_index + 1);
    let text_x = 22.0;
    let text_y = ty + (th - cell_h) / 2.0;
    let text_max_w = (tw - 50.0).max(10.0);
    let text_col = if is_pane_active || is_hovered {
        theme.text_primary
    } else {
        theme.text_secondary
    };

    renderer.render_text(
        queue,
        fonts,
        verts,
        &label,
        text_x,
        text_y,
        text_max_w,
        text_col,
        is_pane_active,
        screen_w,
        screen_h,
    );

    if is_hovered || sidebar.hovered_pane_close == Some(pane_id) {
        render_close_icon(
            renderer,
            queue,
            fonts,
            theme,
            tw,
            ty,
            th,
            sidebar.hovered_pane_close == Some(pane_id),
            verts,
            circle_uv,
            screen_w,
            screen_h,
        );
    }
}

// INFO: Draws New Tab trigger button below the sidebar items.
#[allow(clippy::too_many_arguments)]
fn render_new_tab_entry(
    renderer: &mut GlyphRenderer,
    queue: &wgpu::Queue,
    fonts: &FontManager,
    theme: &Theme,
    sidebar: &Sidebar,
    rows: &[akssora_terminal_ui::SidebarRow],
    content_start_y: f32,
    verts: &mut Vec<GlyphVertex>,
    solid_uv: [f32; 2],
    screen_w: f32,
    screen_h: f32,
) {
    let cell_h = fonts.cell_h;
    let ascent = fonts.ascent_px;
    let (nx, ny, nw, nh) = sidebar.new_tab_button_rect_from_rows(rows, content_start_y);

    if sidebar.hovered_new_tab {
        push_quad(
            verts,
            nx + 4.0,
            ny + 2.0,
            nx + nw - 4.0,
            ny + nh - 2.0,
            solid_uv,
            solid_uv,
            theme.button_hover_background,
            screen_w,
            screen_h,
        );
    }

    let plus_x = nx + 14.0;
    let plus_y = ny + (nh - cell_h) / 2.0;
    let plus_col = if sidebar.hovered_new_tab {
        theme.text_primary
    } else {
        theme.text_secondary
    };

    if let Some(info) = renderer.glyph(queue, fonts, '+', false, false) {
        let gx = plus_x + info.bearing_left;
        let gy = plus_y + ascent + info.bearing_top;
        push_quad(
            verts,
            gx,
            gy,
            gx + info.width,
            gy + info.height,
            info.uv_min,
            info.uv_max,
            plus_col,
            screen_w,
            screen_h,
        );
    }

    renderer.render_text(
        queue,
        fonts,
        verts,
        "New Tab",
        nx + 32.0,
        plus_y,
        nw - 40.0,
        plus_col,
        false,
        screen_w,
        screen_h,
    );
}

// INFO: Draws close cross icon with optional hover highlight background.
#[allow(clippy::too_many_arguments)]
fn render_close_icon(
    renderer: &mut GlyphRenderer,
    queue: &wgpu::Queue,
    fonts: &FontManager,
    theme: &Theme,
    width: f32,
    y: f32,
    h: f32,
    is_hovered: bool,
    verts: &mut Vec<GlyphVertex>,
    circle_uv: [[f32; 2]; 2],
    screen_w: f32,
    screen_h: f32,
) {
    let btn_size = 18.0;
    let cx = width - btn_size - 8.0;
    let cy = y + (h - btn_size) / 2.0;

    if is_hovered {
        push_quad(
            verts,
            cx,
            cy,
            cx + btn_size,
            cy + btn_size,
            circle_uv[0],
            circle_uv[1],
            theme.close_button_hover_background,
            screen_w,
            screen_h,
        );
    }
    if let Some(info) = renderer.glyph(queue, fonts, '\u{00D7}', false, false) {
        let gx = cx + (btn_size - info.width) / 2.0;
        let gy = cy + (btn_size - info.height) / 2.0;
        let close_col = if is_hovered {
            theme.close_button_foreground
        } else {
            theme.text_muted
        };
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
