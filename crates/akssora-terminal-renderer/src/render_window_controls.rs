use akssora_terminal_config::Theme;
use akssora_terminal_ui::{Header, HeaderButton};

use crate::quad::push_rounded_rect;
use crate::vertex::GlyphVertex;

// INFO: Renders minimal, native-feeling window controls (Minimize, Maximize/Restore, Close) with outline icons.
pub fn render_window_controls(
    verts: &mut Vec<GlyphVertex>,
    theme: &Theme,
    header: &Header,
    solid_uv: [f32; 2],
    circle_uv: [[f32; 2]; 2],
    screen_w: f32,
    screen_h: f32,
) {
    for btn in [
        HeaderButton::Minimize,
        HeaderButton::Maximize,
        HeaderButton::Close,
    ] {
        let (bx, by, bw, bh) = header.button_rect(btn, screen_w);
        let is_hovered = header.hovered == Some(btn);
        let is_focused = header.focused_window_button == Some(btn);
        let is_interactive = is_hovered || is_focused;

        // 1. Transparent default background, subtle rounded pill on hover / focus (no red on close)
        if is_interactive {
            let (px, py, pw, ph) = header.button_visual_rect(btn, screen_w);
            push_rounded_rect(
                verts,
                px,
                py,
                px + pw,
                py + ph,
                4.0,
                circle_uv,
                solid_uv,
                theme.header_button_hover_background,
                screen_w,
                screen_h,
            );
        }

        // 2. Icon color determination (subtle secondary in default, clean primary text on hover)
        let icon_col = if is_interactive {
            theme.text_primary
        } else {
            theme.text_secondary
        };

        // Center coordinates of button
        let cx = (bx + bw * 0.5).round();
        let cy = (by + bh * 0.5).round();
        let stroke = 1.2;

        // 3. Crisp outline icon geometry using centralized Window Controls icons
        let mut sink = crate::VertexIconSink::new(verts, solid_uv, circle_uv, screen_w, screen_h);
        match btn {
            HeaderButton::Minimize => {
                akssora_icons::WindowMinimizeIcon::draw(&mut sink, cx, cy, stroke, icon_col);
            }
            HeaderButton::Maximize => {
                if header.is_maximized {
                    akssora_icons::WindowRestoreIcon::draw(&mut sink, cx, cy, stroke, icon_col);
                } else {
                    akssora_icons::WindowMaximizeIcon::draw(&mut sink, cx, cy, stroke, icon_col);
                }
            }
            HeaderButton::Close => {
                akssora_icons::WindowCloseIcon::draw(&mut sink, cx, cy, stroke, icon_col);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_window_controls_normal_and_maximized() {
        let mut header = Header::new("Akssora");
        let theme = Theme::default();
        let solid_uv = [0.0, 0.0];
        let circle_uv = [[0.0, 0.0], [1.0, 1.0]];

        // Normal state (unmaximized)
        let mut verts_normal = Vec::new();
        header.is_maximized = false;
        render_window_controls(
            &mut verts_normal,
            &theme,
            &header,
            solid_uv,
            circle_uv,
            1000.0,
            800.0,
        );
        assert!(!verts_normal.is_empty());

        // Maximized state (should render restore overlapping squares)
        let mut verts_maximized = Vec::new();
        header.is_maximized = true;
        render_window_controls(
            &mut verts_maximized,
            &theme,
            &header,
            solid_uv,
            circle_uv,
            1000.0,
            800.0,
        );
        assert!(!verts_maximized.is_empty());
        // Maximized produces more quads for overlapping squares than single square
        assert!(verts_maximized.len() > verts_normal.len());
    }

    #[test]
    fn test_render_window_controls_hover_and_focus() {
        let mut header = Header::new("Akssora");
        let theme = Theme::default();
        let solid_uv = [0.0, 0.0];
        let circle_uv = [[0.0, 0.0], [1.0, 1.0]];

        // Default state (no hover)
        let mut verts_default = Vec::new();
        render_window_controls(
            &mut verts_default,
            &theme,
            &header,
            solid_uv,
            circle_uv,
            1000.0,
            800.0,
        );

        // Hover Close button (emits background rounded rect)
        header.hovered = Some(HeaderButton::Close);
        let mut verts_hover = Vec::new();
        render_window_controls(
            &mut verts_hover,
            &theme,
            &header,
            solid_uv,
            circle_uv,
            1000.0,
            800.0,
        );
        assert!(verts_hover.len() > verts_default.len());

        // Focus Minimize button
        header.hovered = None;
        header.focused_window_button = Some(HeaderButton::Minimize);
        let mut verts_focus = Vec::new();
        render_window_controls(
            &mut verts_focus,
            &theme,
            &header,
            solid_uv,
            circle_uv,
            1000.0,
            800.0,
        );
        assert!(verts_focus.len() > verts_default.len());
    }
}
