use akssora_terminal_config::Theme;
use akssora_terminal_ui::{Header, HeaderItem};

use crate::quad::push_rounded_rect;
use crate::vertex::GlyphVertex;

// INFO: Renders the minimal outline file explorer toggle button beside the sidebar toggle.
pub fn render_explorer_toggle_button(
    verts: &mut Vec<GlyphVertex>,
    theme: &Theme,
    header: &Header,
    solid_uv: [f32; 2],
    circle_uv: [[f32; 2]; 2],
    screen_w: f32,
    screen_h: f32,
) {
    let (bx, by, btn_w, btn_h) = header.explorer_toggle_visual_rect();
    let is_hovered = header.hovered_item == Some(HeaderItem::ExplorerToggle);
    let is_focused = header.is_explorer_toggle_focused;
    let is_active = header.is_explorer_toggle_active;

    // 1. Button container background
    if is_hovered || is_focused || is_active {
        let bg_col = if is_active {
            theme.header_button_active_background
        } else {
            theme.header_button_hover_background
        };
        push_rounded_rect(
            verts,
            bx,
            by,
            bx + btn_w,
            by + btn_h,
            5.0,
            circle_uv,
            solid_uv,
            bg_col,
            screen_w,
            screen_h,
        );
    }

    // 2. Folder outline icon geometry (16px x 13px)
    let stroke = 1.0;
    let ix = (bx + (btn_w - akssora_icons::ExplorerToggleIcon::WIDTH) * 0.5).round();
    let iy = (by + (btn_h - akssora_icons::ExplorerToggleIcon::HEIGHT) * 0.5).round();

    let stroke_col = if is_focused || is_active || is_hovered {
        theme.text_primary
    } else {
        theme.text_secondary
    };

    let mut sink =
        crate::icon_sink::VertexIconSink::new(verts, solid_uv, circle_uv, screen_w, screen_h);
    akssora_icons::ExplorerToggleIcon::draw(&mut sink, ix, iy, stroke, stroke_col);
}
