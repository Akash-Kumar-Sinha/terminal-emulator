use akssora_terminal_config::Theme;
use akssora_terminal_ui::ContextMenu;

use crate::font::FontManager;
use crate::glyph::GlyphRenderer;
use crate::quad::push_rounded_rect;
use crate::vertex::GlyphVertex;

// INFO: Renders floating terminal context menu popup with item hover states and shortcut labels.
#[allow(clippy::too_many_arguments)]
pub fn render_context_menu(
    renderer: &mut GlyphRenderer,
    queue: &wgpu::Queue,
    fonts: &FontManager,
    theme: &Theme,
    menu: &ContextMenu,
    verts: &mut Vec<GlyphVertex>,
    circle_uv: [[f32; 2]; 2],
    solid_uv: [f32; 2],
    screen_w: f32,
    screen_h: f32,
) {
    let (mx, my, mw, mh) = menu.bounds();

    // Menu background shadow / outer border
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

    // Menu background surface
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

    let mut item_y = my + 4.0;
    for (idx, item) in menu.items.iter().enumerate() {
        let is_hovered = menu.hovered_idx == Some(idx) && item.enabled;

        if is_hovered {
            push_rounded_rect(
                verts,
                mx + 4.0,
                item_y,
                mx + mw - 4.0,
                item_y + menu.item_height,
                2.0,
                circle_uv,
                solid_uv,
                theme.hover_background,
                screen_w,
                screen_h,
            );
        }

        let text_color = if !item.enabled {
            theme.text_muted
        } else if is_hovered {
            theme.foreground
        } else {
            theme.text_secondary
        };

        // Render item label
        let text_x = mx + 12.0;
        let text_y = item_y + (menu.item_height - fonts.cell_h) * 0.5;
        renderer.render_text(
            queue,
            fonts,
            verts,
            item.label,
            text_x,
            text_y,
            mw - 50.0,
            text_color,
            false,
            screen_w,
            screen_h,
        );

        // Render item shortcut label
        if !item.shortcut.is_empty() {
            let sc_w = item.shortcut.len() as f32 * fonts.cell_w;
            let sc_x = mx + mw - sc_w - 12.0;
            renderer.render_text(
                queue,
                fonts,
                verts,
                item.shortcut,
                sc_x,
                text_y,
                sc_w,
                theme.text_muted,
                false,
                screen_w,
                screen_h,
            );
        }

        item_y += menu.item_height;
    }
}
