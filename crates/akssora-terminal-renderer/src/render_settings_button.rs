use akssora_terminal_config::Theme;
use akssora_terminal_ui::{Header, HeaderItem};

use crate::quad::{push_quad, push_rounded_rect};
use crate::vertex::GlyphVertex;

// INFO: Renders the minimal outline gear settings button with zinc hover and monochrome focus accent.
pub fn render_settings_button(
    verts: &mut Vec<GlyphVertex>,
    theme: &Theme,
    header: &Header,
    solid_uv: [f32; 2],
    circle_uv: [[f32; 2]; 2],
    screen_w: f32,
    screen_h: f32,
) {
    let (sx, sy, sw, sh) = header.settings_button_rect(screen_w);
    let is_hovered = header.hovered_item == Some(HeaderItem::Settings);
    let is_focused = header.is_settings_focused;
    let is_active = header.is_settings_active;

    // 1. Button container background (completely transparent in default state)
    if is_hovered || is_focused || is_active {
        let bg_col = if is_active {
            theme.header_button_active_background
        } else {
            theme.header_button_hover_background
        };
        push_rounded_rect(
            verts,
            sx,
            sy,
            sx + sw,
            sy + sh,
            5.0,
            circle_uv,
            solid_uv,
            bg_col,
            screen_w,
            screen_h,
        );
    }

    // 2. Focus ring on keyboard focus
    if is_focused {
        let ring_pad = 1.0;
        let rx0 = sx - ring_pad;
        let ry0 = sy - ring_pad;
        let rx1 = sx + sw + ring_pad;
        let ry1 = sy + sh + ring_pad;
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

    // 3. Crisp outline-style gear icon (16x16 px)
    let cx = (sx + sw * 0.5).round();
    let cy = (sy + sh * 0.5).round();

    let stroke_col = if is_focused || is_active || is_hovered {
        theme.text_primary
    } else {
        theme.text_secondary
    };

    let mut sink =
        crate::icon_sink::VertexIconSink::new(verts, solid_uv, circle_uv, screen_w, screen_h);
    akssora_icons::SettingsGearIcon::draw(&mut sink, cx, cy, stroke_col);
}

#[cfg(test)]
mod tests {
    use super::*;

    // INFO: Verify transparent default background, zinc hover, white focus accent, and vertex count for settings gear button.
    #[test]
    fn test_render_settings_button_states() {
        let theme = Theme::monochrome_black();
        let mut header = Header::new("Akssora");
        let solid_uv = [0.0, 0.0];
        let circle_uv = [[0.0, 0.0], [1.0, 1.0]];

        // 1. Default State: No background quad emitted (completely transparent), only gear icon strokes
        let mut default_verts = Vec::new();
        header.hovered_item = None;
        header.is_settings_focused = false;
        header.is_settings_active = false;
        render_settings_button(
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
        header.hovered_item = Some(HeaderItem::Settings);
        render_settings_button(
            &mut hover_verts,
            &theme,
            &header,
            solid_uv,
            circle_uv,
            800.0,
            600.0,
        );
        assert!(hover_verts.len() > default_count);

        // 3. Focus State: Restrained focus accent ring adds additional border quads
        let mut focus_verts = Vec::new();
        header.hovered_item = None;
        header.is_settings_focused = true;
        render_settings_button(
            &mut focus_verts,
            &theme,
            &header,
            solid_uv,
            circle_uv,
            800.0,
            600.0,
        );
        assert!(focus_verts.len() > hover_verts.len());

        // 4. Alignment & Dimensions
        let (sx, sy, sw, sh) = header.settings_button_rect(800.0);
        assert!(sx > 0.0);
        assert_eq!(sw, 28.0);
        assert_eq!(sh, 26.0);
        assert_eq!(sy, (header.height - sh) * 0.5);
    }
}
