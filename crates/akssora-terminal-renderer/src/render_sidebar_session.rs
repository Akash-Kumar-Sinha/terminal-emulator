use akssora_terminal_config::Theme;
use akssora_terminal_ui::Sidebar;

use crate::font::FontManager;
use crate::glyph::GlyphRenderer;
use crate::quad::{push_quad, push_rounded_rect};
use crate::vertex::GlyphVertex;

// INFO: Renders project heading with toggle chevron and session count badge.
#[allow(clippy::too_many_arguments)]
pub fn render_project_header(
    renderer: &mut GlyphRenderer,
    queue: &wgpu::Queue,
    fonts: &FontManager,
    theme: &Theme,
    project_name: &str,
    session_count: usize,
    is_collapsed: bool,
    y: f32,
    h: f32,
    sidebar_w: f32,
    verts: &mut Vec<GlyphVertex>,
    screen_w: f32,
    screen_h: f32,
) {
    let cell_h = fonts.cell_h;
    let ascent = fonts.ascent_px;

    // Chevron icon
    let chevron_char = if is_collapsed {
        akssora_icons::ChevronRightIcon::CHAR_SMALL
    } else {
        akssora_icons::ChevronDownIcon::CHAR_SMALL
    };
    let chevron_col = theme.text_secondary;
    if let Some(info) = renderer.glyph(queue, fonts, chevron_char, false, false) {
        let gx = 10.0 + info.bearing_left;
        let gy = y + (h - cell_h) / 2.0 + ascent + info.bearing_top;
        push_quad(
            verts,
            gx,
            gy,
            gx + info.width,
            gy + info.height,
            info.uv_min,
            info.uv_max,
            chevron_col,
            screen_w,
            screen_h,
        );
    }

    // Project name in uppercase
    let name_x = 24.0;
    let name_y = y + (h - cell_h) / 2.0;
    let display_title = project_name.to_uppercase();
    let text_col = theme.text_primary;
    let title_max_w = (sidebar_w - 70.0).max(10.0);

    renderer.render_text(
        queue,
        fonts,
        verts,
        &display_title,
        name_x,
        name_y,
        title_max_w,
        text_col,
        false,
        screen_w,
        screen_h,
    );

    // Session count indicator e.g. "(2)"
    let count_label = format!("{session_count}");
    let count_col = theme.text_muted;
    let count_x = sidebar_w - 24.0;
    renderer.render_text(
        queue,
        fonts,
        verts,
        &count_label,
        count_x,
        name_y,
        20.0,
        count_col,
        false,
        screen_w,
        screen_h,
    );
}

// INFO: Renders split-view group header with toggle chevron and child count indicator.
#[allow(clippy::too_many_arguments)]
pub fn render_split_group_header(
    renderer: &mut GlyphRenderer,
    queue: &wgpu::Queue,
    fonts: &FontManager,
    theme: &Theme,
    tab_title: Option<&str>,
    split_count: usize,
    is_collapsed: bool,
    y: f32,
    h: f32,
    sidebar_w: f32,
    verts: &mut Vec<GlyphVertex>,
    screen_w: f32,
    screen_h: f32,
) {
    let cell_h = fonts.cell_h;
    let ascent = fonts.ascent_px;

    // Chevron icon
    let chevron_char = if is_collapsed {
        akssora_icons::ChevronRightIcon::CHAR_SMALL
    } else {
        akssora_icons::ChevronDownIcon::CHAR_SMALL
    };
    let chevron_col = theme.text_secondary;
    if let Some(info) = renderer.glyph(queue, fonts, chevron_char, false, false) {
        let gx = 8.0 + info.bearing_left;
        let gy = y + (h - cell_h) / 2.0 + ascent + info.bearing_top;
        push_quad(
            verts,
            gx,
            gy,
            gx + info.width,
            gy + info.height,
            info.uv_min,
            info.uv_max,
            chevron_col,
            screen_w,
            screen_h,
        );
    }

    // Group title
    let name_x = 22.0;
    let name_y = y + (h - cell_h) / 2.0;
    let title = tab_title.unwrap_or("Split View");
    let text_col = theme.text_secondary;
    let title_max_w = (sidebar_w - 65.0).max(10.0);

    renderer.render_text(
        queue,
        fonts,
        verts,
        title,
        name_x,
        name_y,
        title_max_w,
        text_col,
        false,
        screen_w,
        screen_h,
    );

    // Split count badge e.g. "[2]"
    let count_label = format!("[{split_count}]");
    let count_col = theme.text_muted;
    let count_x = sidebar_w - 32.0;
    renderer.render_text(
        queue,
        fonts,
        verts,
        &count_label,
        count_x,
        name_y,
        24.0,
        count_col,
        false,
        screen_w,
        screen_h,
    );
}

// INFO: Renders individual terminal session item with icon, title, secondary metadata, and border.
#[allow(clippy::too_many_arguments)]
pub fn render_session_entry(
    renderer: &mut GlyphRenderer,
    queue: &wgpu::Queue,
    fonts: &FontManager,
    theme: &Theme,
    sidebar: &Sidebar,
    pane_id: u64,
    session_name: &str,
    secondary_meta: &str,
    icon_char: char,
    is_active: bool,
    is_split: bool,
    y: f32,
    h: f32,
    sidebar_w: f32,
    verts: &mut Vec<GlyphVertex>,
    solid_uv: [f32; 2],
    circle_uv: [[f32; 2]; 2],
    screen_w: f32,
    screen_h: f32,
) {
    let ascent = fonts.ascent_px;
    let is_hovered = sidebar.hovered_pane == Some(pane_id);

    let bg_color = if is_active {
        theme.session_selected_background
    } else if is_hovered {
        theme.session_hover_background
    } else {
        [0.0, 0.0, 0.0, 0.0]
    };

    let (left_x, right_x) = if is_split {
        (16.0, sidebar_w - 6.0)
    } else {
        (6.0, sidebar_w - 6.0)
    };

    // Draw background rounded rect
    push_rounded_rect(
        verts,
        left_x,
        y + 2.0,
        right_x,
        y + h - 2.0,
        5.0,
        circle_uv,
        solid_uv,
        bg_color,
        screen_w,
        screen_h,
    );

    // Session prompt/page icon
    let icon_x = left_x + 8.0;
    let icon_y = y + 4.0;
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

    // Line 1: Primary session name (e.g. "cargo run", "agy", "Terminal 1")
    let title_x = icon_x + 14.0;
    let title_y = y + 4.0;
    let title_max_w = (sidebar_w - title_x - 30.0).max(10.0);
    let title_col = if is_active {
        theme.text_primary
    } else {
        theme.text_secondary
    };

    let char_limit = (title_max_w / fonts.cell_w.max(1.0)).floor() as usize;
    let title_str = if session_name.chars().count() > char_limit && char_limit > 3 {
        let truncated: String = session_name.chars().take(char_limit - 3).collect();
        format!("{truncated}...")
    } else {
        session_name.to_string()
    };

    renderer.render_text(
        queue,
        fonts,
        verts,
        &title_str,
        title_x,
        title_y,
        title_max_w,
        title_col,
        false,
        screen_w,
        screen_h,
    );

    // Line 2: Secondary metadata (e.g. "~/dev/agy · main")
    let meta_y = y + 22.0;
    let meta_max_w = (sidebar_w - title_x - 10.0).max(10.0);
    let meta_col = theme.text_muted;

    let meta_limit = (meta_max_w / fonts.cell_w.max(1.0)).floor() as usize;
    let meta_str = if secondary_meta.chars().count() > meta_limit && meta_limit > 3 {
        let truncated: String = secondary_meta.chars().take(meta_limit - 3).collect();
        format!("{truncated}...")
    } else {
        secondary_meta.to_string()
    };

    renderer.render_text(
        queue, fonts, verts, &meta_str, title_x, meta_y, meta_max_w, meta_col, false, screen_w,
        screen_h,
    );

    // Close button '×' on hover or active
    if is_active || is_hovered || sidebar.hovered_pane_close == Some(pane_id) {
        let btn_size = 18.0;
        let cx = sidebar_w - btn_size - 8.0;
        let cy = y + (h - btn_size) / 2.0;
        let is_close_hovered = sidebar.hovered_pane_close == Some(pane_id);

        if is_close_hovered {
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
        if let Some(info) =
            renderer.glyph(queue, fonts, akssora_icons::CloseIcon::CHAR, false, false)
        {
            let gx = cx + (btn_size - info.width) / 2.0;
            let gy = cy + (btn_size - info.height) / 2.0;
            let close_col = if is_close_hovered {
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
}
