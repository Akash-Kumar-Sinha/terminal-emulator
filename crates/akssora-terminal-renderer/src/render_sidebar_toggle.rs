use akssora_terminal_config::Theme;
use akssora_terminal_ui::{Header, HeaderItem};

use crate::quad::{push_quad, push_rounded_rect};
use crate::vertex::GlyphVertex;

// INFO: Renders the minimal outline sidebar toggle button with zinc hover, blue accent, and directional state.
pub fn render_sidebar_toggle_button(
    verts: &mut Vec<GlyphVertex>,
    theme: &Theme,
    header: &Header,
    solid_uv: [f32; 2],
    circle_uv: [[f32; 2]; 2],
    screen_w: f32,
    screen_h: f32,
) {
    let (bx, by, btn_w, btn_h) = header.sidebar_toggle_visual_rect();
    let is_hovered = header.hovered_item == Some(HeaderItem::SidebarToggle);
    let is_focused = header.is_sidebar_toggle_focused;
    let is_active = header.is_sidebar_toggle_active;

    // 1. Button container background (completely transparent in default state)
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

    // 2. Restrained accent focus ring on keyboard focus
    if is_focused {
        let ring_pad = 1.0;
        let rx0 = bx - ring_pad;
        let ry0 = by - ring_pad;
        let rx1 = bx + btn_w + ring_pad;
        let ry1 = by + btn_h + ring_pad;
        let r = 5.0 + ring_pad;
        push_quad(
            verts,
            rx0 + r,
            ry0,
            rx1 - r,
            ry0 + 1.0,
            solid_uv,
            solid_uv,
            theme.header_button_focus_ring,
            screen_w,
            screen_h,
        );
        push_quad(
            verts,
            rx0 + r,
            ry1 - 1.0,
            rx1 - r,
            ry1,
            solid_uv,
            solid_uv,
            theme.header_button_focus_ring,
            screen_w,
            screen_h,
        );
        push_quad(
            verts,
            rx0,
            ry0 + r,
            rx0 + 1.0,
            ry1 - r,
            solid_uv,
            solid_uv,
            theme.header_button_focus_ring,
            screen_w,
            screen_h,
        );
        push_quad(
            verts,
            rx1 - 1.0,
            ry0 + r,
            rx1,
            ry1 - r,
            solid_uv,
            solid_uv,
            theme.header_button_focus_ring,
            screen_w,
            screen_h,
        );
    }

    // 3. Crisp outline icon geometry
    let stroke = 1.0;
    let ix = (bx + (btn_w - akssora_icons::SidebarToggleIcon::WIDTH) * 0.5).round();
    let iy = (by + (btn_h - akssora_icons::SidebarToggleIcon::HEIGHT) * 0.5).round();

    let stroke_col = if is_focused || is_active || is_hovered {
        theme.text_primary
    } else {
        theme.text_secondary
    };

    let panel_col = if is_hovered || is_focused || is_active {
        theme.text_primary
    } else {
        theme.text_secondary
    };

    let mut sink =
        crate::icon_sink::VertexIconSink::new(verts, solid_uv, circle_uv, screen_w, screen_h);
    akssora_icons::SidebarToggleIcon::draw(
        &mut sink,
        ix,
        iy,
        stroke,
        header.is_sidebar_expanded,
        stroke_col,
        panel_col,
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    // INFO: Verify transparent default background, zinc hover, blue accent, and open/closed vertex generation.
    #[test]
    fn test_render_sidebar_toggle_states() {
        let theme = Theme::monochrome_black();
        let mut header = Header::new("Akssora");
        let solid_uv = [0.0, 0.0];
        let circle_uv = [[0.0, 0.0], [1.0, 1.0]];

        // 1. Default State: No background quad emitted (completely transparent)
        let mut default_verts = Vec::new();
        header.hovered_item = None;
        header.is_sidebar_toggle_focused = false;
        header.is_sidebar_toggle_active = false;
        header.is_sidebar_expanded = true;
        render_sidebar_toggle_button(
            &mut default_verts,
            &theme,
            &header,
            solid_uv,
            circle_uv,
            800.0,
            600.0,
        );
        assert!(!default_verts.is_empty());
        let default_count = default_verts.len();

        // 2. Hover State: Zinc rounded rectangle background is emitted, increasing vertex count
        let mut hover_verts = Vec::new();
        header.hovered_item = Some(HeaderItem::SidebarToggle);
        render_sidebar_toggle_button(
            &mut hover_verts,
            &theme,
            &header,
            solid_uv,
            circle_uv,
            800.0,
            600.0,
        );
        assert!(hover_verts.len() > default_count);

        // 3. Focus State: Restrained blue accent focus ring adds additional border quads
        let mut focus_verts = Vec::new();
        header.hovered_item = None;
        header.is_sidebar_toggle_focused = true;
        render_sidebar_toggle_button(
            &mut focus_verts,
            &theme,
            &header,
            solid_uv,
            circle_uv,
            800.0,
            600.0,
        );
        assert!(focus_verts.len() > hover_verts.len());

        // 4. Closed vs Open State: Directional chevron vs panel fill quad
        let mut closed_verts = Vec::new();
        header.is_sidebar_toggle_focused = false;
        header.is_sidebar_expanded = false;
        render_sidebar_toggle_button(
            &mut closed_verts,
            &theme,
            &header,
            solid_uv,
            circle_uv,
            800.0,
            600.0,
        );
        assert!(!closed_verts.is_empty());
    }
}
