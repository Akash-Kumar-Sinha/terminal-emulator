use akssora_terminal_config::Theme;
use akssora_terminal_ui::QuickSearchUi;

use crate::font::FontManager;
use crate::glyph::GlyphRenderer;
use crate::quad::{push_quad, push_rounded_rect};
use crate::vertex::GlyphVertex;

// INFO: Renders the command-palette quick file search overlay centered below the header bar.
#[allow(clippy::too_many_arguments)]
pub fn render_quick_search_overlay(
    renderer: &mut GlyphRenderer,
    queue: &wgpu::Queue,
    fonts: &FontManager,
    theme: &Theme,
    quick_search: &QuickSearchUi,
    verts: &mut Vec<GlyphVertex>,
    solid_uv: [f32; 2],
    circle_uv: [[f32; 2]; 2],
    screen_w: f32,
    screen_h: f32,
    top_y: f32,
) {
    if !quick_search.is_open {
        return;
    }

    // INFO: Command palette uses refined font size scaled proportionally with UI font size.
    let palette_size = (fonts.size() * (12.0 / 14.0)).max(10.0);
    let mut palette_fonts = fonts.clone();
    palette_fonts.set_size(palette_size);

    // 1. Semi-transparent backdrop overlay
    push_quad(
        verts,
        0.0,
        0.0,
        screen_w,
        screen_h,
        solid_uv,
        solid_uv,
        theme.modal_backdrop,
        screen_w,
        screen_h,
    );

    let (ox, oy, ow, oh) = QuickSearchUi::overlay_rect(screen_w, top_y, quick_search.results.len());

    // 2. Palette background container using theme.palette_background
    push_rounded_rect(
        verts,
        ox,
        oy,
        ox + ow,
        oy + oh,
        6.0,
        circle_uv,
        solid_uv,
        theme.palette_background,
        screen_w,
        screen_h,
    );

    // 3. Search Input Header Row
    let input_h = QuickSearchUi::INPUT_HEIGHT;

    let text_y = oy + (input_h - palette_fonts.cell_h) * 0.5;
    renderer.render_text(
        queue,
        &palette_fonts,
        verts,
        akssora_icons::SearchIcon::GLYPH,
        ox + 12.0,
        text_y,
        18.0,
        theme.text_secondary,
        false,
        screen_w,
        screen_h,
    );

    let query_text = if quick_search.query.is_empty() {
        "Type a filename or partial path to open...".to_string()
    } else {
        format!("{}_", quick_search.query)
    };
    let query_col = if quick_search.query.is_empty() {
        theme.text_secondary
    } else {
        theme.text_primary
    };

    renderer.render_text(
        queue,
        &palette_fonts,
        verts,
        &query_text,
        ox + 34.0,
        text_y,
        ow - 48.0,
        query_col,
        false,
        screen_w,
        screen_h,
    );

    // 4. Candidate Results List
    let item_h = QuickSearchUi::ITEM_HEIGHT;
    let list_start_y = oy + input_h + 2.0;

    let visible_count = quick_search
        .results
        .len()
        .saturating_sub(quick_search.scroll_offset)
        .min(QuickSearchUi::MAX_VISIBLE_ITEMS);

    if quick_search.results.is_empty() {
        let empty_msg = if quick_search.query.is_empty() {
            "No files in workspace"
        } else {
            "No matching files found"
        };
        renderer.render_text(
            queue,
            &palette_fonts,
            verts,
            empty_msg,
            ox + 16.0,
            list_start_y + 8.0,
            ow - 32.0,
            theme.text_secondary,
            false,
            screen_w,
            screen_h,
        );
    } else {
        for i in 0..visible_count {
            let actual_idx = quick_search.scroll_offset + i;
            let item = &quick_search.results[actual_idx];
            let iy = list_start_y + (i as f32 * item_h);
            let is_selected = actual_idx == quick_search.selected_index;

            if is_selected {
                push_rounded_rect(
                    verts,
                    ox + 4.0,
                    iy,
                    ox + ow - 4.0,
                    iy + item_h,
                    3.0,
                    circle_uv,
                    solid_uv,
                    theme.tab_hover_background,
                    screen_w,
                    screen_h,
                );
            }

            // File icon
            let item_text_y = iy + (item_h - palette_fonts.cell_h) * 0.5;
            let file_icon = akssora_icons::file_type_icon(&item.filename);
            renderer.render_text(
                queue,
                &palette_fonts,
                verts,
                file_icon,
                ox + 12.0,
                item_text_y,
                14.0,
                if is_selected {
                    theme.text_primary
                } else {
                    theme.text_secondary
                },
                false,
                screen_w,
                screen_h,
            );

            // Filename (regular font weight, no bold)
            let name_w = (item.filename.len() as f32 * palette_fonts.cell_w) + 12.0;
            renderer.render_text(
                queue,
                &palette_fonts,
                verts,
                &item.filename,
                ox + 30.0,
                item_text_y,
                name_w,
                theme.text_primary,
                false,
                screen_w,
                screen_h,
            );

            // Relative directory path
            let path_x = ox + 30.0 + name_w + 6.0;
            let path_w = (ox + ow - 16.0 - path_x).max(10.0);
            renderer.render_text(
                queue,
                &palette_fonts,
                verts,
                &item.relative_path,
                path_x,
                item_text_y,
                path_w,
                theme.text_secondary,
                false,
                screen_w,
                screen_h,
            );
        }
    }
}
