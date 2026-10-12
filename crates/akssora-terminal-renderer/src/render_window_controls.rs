use akssora_terminal_config::Theme;
use akssora_terminal_ui::{Header, HeaderButton};

use crate::quad::{push_line, push_quad, push_rounded_rect};
use crate::vertex::GlyphVertex;

// INFO: Subtle zinc hover background for minimal native-feeling window controls.
pub const ZINC_HOVER_BG: [f32; 4] = [0.20, 0.20, 0.23, 0.80];

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

        // 1. Transparent default background, subtle zinc rounded pill on hover / focus (no red on close)
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
                ZINC_HOVER_BG,
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

        // 3. Crisp outline icon geometry with slightly reduced, compact proportions (~10-12px)
        match btn {
            HeaderButton::Minimize => {
                // Horizontal line: 11px span, crisp stroke
                let half_span = 5.5;
                let half_stroke = stroke * 0.5;
                push_quad(
                    verts,
                    cx - half_span,
                    cy - half_stroke,
                    cx + half_span,
                    cy + half_stroke,
                    solid_uv,
                    solid_uv,
                    icon_col,
                    screen_w,
                    screen_h,
                );
            }
            HeaderButton::Maximize => {
                if header.is_maximized {
                    // Restore: Overlapping double-square icon (footprint ~11x11px)
                    // Front square: 8x8 px at bottom-left
                    let f_l = cx - 5.0;
                    let f_r = cx + 3.0;
                    let f_t = cy - 3.0;
                    let f_b = cy + 5.0;

                    // Front square outline
                    push_quad(
                        verts,
                        f_l,
                        f_t,
                        f_r,
                        f_t + stroke,
                        solid_uv,
                        solid_uv,
                        icon_col,
                        screen_w,
                        screen_h,
                    );
                    push_quad(
                        verts,
                        f_l,
                        f_b - stroke,
                        f_r,
                        f_b,
                        solid_uv,
                        solid_uv,
                        icon_col,
                        screen_w,
                        screen_h,
                    );
                    push_quad(
                        verts,
                        f_l,
                        f_t + stroke,
                        f_l + stroke,
                        f_b - stroke,
                        solid_uv,
                        solid_uv,
                        icon_col,
                        screen_w,
                        screen_h,
                    );
                    push_quad(
                        verts,
                        f_r - stroke,
                        f_t + stroke,
                        f_r,
                        f_b - stroke,
                        solid_uv,
                        solid_uv,
                        icon_col,
                        screen_w,
                        screen_h,
                    );

                    // Back square: 8x8 px offset top-right (cx - 3.0 .. cx + 5.0, cy - 5.0 .. cy + 3.0)
                    let b_l = cx - 2.5;
                    let b_r = cx + 5.5;
                    let b_t = cy - 5.5;
                    let b_b = cy + 2.5;

                    // Back square visible edges
                    push_quad(
                        verts,
                        b_l,
                        b_t,
                        b_r,
                        b_t + stroke,
                        solid_uv,
                        solid_uv,
                        icon_col,
                        screen_w,
                        screen_h,
                    );
                    push_quad(
                        verts,
                        b_r - stroke,
                        b_t + stroke,
                        b_r,
                        b_b,
                        solid_uv,
                        solid_uv,
                        icon_col,
                        screen_w,
                        screen_h,
                    );
                    // Left edge segment above front square
                    if f_t > b_t + stroke {
                        push_quad(
                            verts,
                            b_l,
                            b_t + stroke,
                            b_l + stroke,
                            f_t,
                            solid_uv,
                            solid_uv,
                            icon_col,
                            screen_w,
                            screen_h,
                        );
                    }
                    // Bottom edge segment to the right of front square
                    if b_r - stroke > f_r {
                        push_quad(
                            verts,
                            f_r,
                            b_b - stroke,
                            b_r - stroke,
                            b_b,
                            solid_uv,
                            solid_uv,
                            icon_col,
                            screen_w,
                            screen_h,
                        );
                    }
                } else {
                    // Maximize: Square outline 11x11 px, crisp stroke
                    let half = 5.5;
                    let left = cx - half;
                    let right = cx + half;
                    let top = cy - half;
                    let bottom = cy + half;

                    // Top
                    push_quad(
                        verts,
                        left,
                        top,
                        right,
                        top + stroke,
                        solid_uv,
                        solid_uv,
                        icon_col,
                        screen_w,
                        screen_h,
                    );
                    // Bottom
                    push_quad(
                        verts,
                        left,
                        bottom - stroke,
                        right,
                        bottom,
                        solid_uv,
                        solid_uv,
                        icon_col,
                        screen_w,
                        screen_h,
                    );
                    // Left
                    push_quad(
                        verts,
                        left,
                        top + stroke,
                        left + stroke,
                        bottom - stroke,
                        solid_uv,
                        solid_uv,
                        icon_col,
                        screen_w,
                        screen_h,
                    );
                    // Right
                    push_quad(
                        verts,
                        right - stroke,
                        top + stroke,
                        right,
                        bottom - stroke,
                        solid_uv,
                        solid_uv,
                        icon_col,
                        screen_w,
                        screen_h,
                    );
                }
            }
            HeaderButton::Close => {
                // Crisp diagonal cross 'X' with ~9-10px span
                let span = 4.5;
                push_line(
                    verts,
                    cx - span,
                    cy - span,
                    cx + span,
                    cy + span,
                    stroke,
                    solid_uv,
                    icon_col,
                    screen_w,
                    screen_h,
                );
                push_line(
                    verts,
                    cx - span,
                    cy + span,
                    cx + span,
                    cy - span,
                    stroke,
                    solid_uv,
                    icon_col,
                    screen_w,
                    screen_h,
                );
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
