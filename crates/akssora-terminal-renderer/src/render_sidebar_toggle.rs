use akssora_terminal_config::Theme;
use akssora_terminal_ui::{Header, HeaderItem};

use crate::quad::{push_quad, push_rounded_rect};
use crate::vertex::GlyphVertex;

// INFO: Subtle zinc hover background and monochrome neutral accents for the sidebar toggle button.
const ZINC_HOVER_BG: [f32; 4] = [0.18, 0.18, 0.20, 0.85];
const ZINC_ACTIVE_BG: [f32; 4] = [0.15, 0.15, 0.16, 0.90];
const WHITE_ACCENT: [f32; 4] = [1.0, 1.0, 1.0, 0.95];
const WHITE_FOCUS_RING: [f32; 4] = [0.70, 0.70, 0.70, 0.80];

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
            ZINC_ACTIVE_BG
        } else {
            ZINC_HOVER_BG
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

    // 2. Restrained blue accent focus ring on keyboard focus
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
            WHITE_FOCUS_RING,
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
            WHITE_FOCUS_RING,
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
            WHITE_FOCUS_RING,
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
            WHITE_FOCUS_RING,
            screen_w,
            screen_h,
        );
    }

    // 3. Crisp outline icon geometry (18px x 14px, balanced proportions)
    let icon_w = 18.0;
    let icon_h = 14.0;
    let stroke = 1.0;

    let ix = (bx + (btn_w - icon_w) * 0.5).round();
    let iy = (by + (btn_h - icon_h) * 0.5).round();

    let stroke_col = if is_focused || is_active {
        WHITE_ACCENT
    } else if is_hovered {
        theme.text_primary
    } else {
        theme.text_secondary
    };

    // Outer rectangle frame
    push_quad(
        verts,
        ix,
        iy,
        ix + icon_w,
        iy + stroke,
        solid_uv,
        solid_uv,
        stroke_col,
        screen_w,
        screen_h,
    );
    push_quad(
        verts,
        ix,
        iy + icon_h - stroke,
        ix + icon_w,
        iy + icon_h,
        solid_uv,
        solid_uv,
        stroke_col,
        screen_w,
        screen_h,
    );
    push_quad(
        verts,
        ix,
        iy + stroke,
        ix + stroke,
        iy + icon_h - stroke,
        solid_uv,
        solid_uv,
        stroke_col,
        screen_w,
        screen_h,
    );
    push_quad(
        verts,
        ix + icon_w - stroke,
        iy + stroke,
        ix + icon_w,
        iy + icon_h - stroke,
        solid_uv,
        solid_uv,
        stroke_col,
        screen_w,
        screen_h,
    );

    // Vertical divider separating the narrow left sidebar from larger content area
    let div_x = ix + 5.5;
    push_quad(
        verts,
        div_x,
        iy + stroke,
        div_x + stroke,
        iy + icon_h - stroke,
        solid_uv,
        solid_uv,
        stroke_col,
        screen_w,
        screen_h,
    );

    // 4. Panel state and directional indicator
    if header.is_sidebar_expanded {
        // Open state: Left sidebar is filled with restrained white accent to represent open state
        let panel_col = if is_hovered || is_focused || is_active {
            WHITE_ACCENT
        } else {
            [WHITE_ACCENT[0], WHITE_ACCENT[1], WHITE_ACCENT[2], 0.85]
        };
        push_quad(
            verts,
            ix + stroke,
            iy + stroke,
            div_x,
            iy + icon_h - stroke,
            solid_uv,
            solid_uv,
            panel_col,
            screen_w,
            screen_h,
        );
    } else {
        // Closed state: Left sidebar is hollow with subtle directional chevron pointing right to indicate expansion
        let cy = (iy + icon_h * 0.5).round();
        let cx = (ix + 2.5).round();
        let chevron_col = if is_focused || is_active {
            WHITE_ACCENT
        } else if is_hovered {
            theme.text_primary
        } else {
            theme.text_muted
        };
        push_quad(
            verts,
            cx - 1.0,
            cy - 2.5,
            cx,
            cy - 1.0,
            solid_uv,
            solid_uv,
            chevron_col,
            screen_w,
            screen_h,
        );
        push_quad(
            verts,
            cx,
            cy - 1.0,
            cx + 1.0,
            cy + 1.0,
            solid_uv,
            solid_uv,
            chevron_col,
            screen_w,
            screen_h,
        );
        push_quad(
            verts,
            cx - 1.0,
            cy + 1.0,
            cx,
            cy + 2.5,
            solid_uv,
            solid_uv,
            chevron_col,
            screen_w,
            screen_h,
        );
    }
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
