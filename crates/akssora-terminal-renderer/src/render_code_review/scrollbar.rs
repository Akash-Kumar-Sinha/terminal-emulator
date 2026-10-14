use akssora_terminal_config::Theme;

use crate::quad::push_pill_thumb;
use crate::vertex::GlyphVertex;

// INFO: Renders a smooth, subtle vertical scrollbar pill indicator for scrollable viewports in Code Review.
#[allow(clippy::too_many_arguments)]
pub fn render_vertical_scrollbar(
    verts: &mut Vec<GlyphVertex>,
    theme: &Theme,
    viewport_x: f32,
    viewport_y: f32,
    viewport_w: f32,
    viewport_h: f32,
    content_h: f32,
    scroll_y: f32,
    solid_uv: [f32; 2],
    circle_uv: [[f32; 2]; 2],
    screen_w: f32,
    screen_h: f32,
) {
    if content_h <= viewport_h || viewport_h <= 0.0 {
        return;
    }

    let scrollbar_w = 6.0;
    let track_x = viewport_x + viewport_w - scrollbar_w - 4.0;
    let track_y = viewport_y + 4.0;
    let track_h = (viewport_h - 8.0).max(10.0);

    // Track line (subtle background using theme, transparent if alpha is 0)
    if theme.scrollbar_track[3] > 0.0 {
        crate::quad::push_quad(
            verts,
            track_x,
            track_y,
            track_x + scrollbar_w,
            track_y + track_h,
            solid_uv,
            solid_uv,
            theme.scrollbar_track,
            screen_w,
            screen_h,
        );
    }

    // Compute thumb dimensions and position
    let visible_ratio = (viewport_h / content_h).clamp(0.05, 1.0);
    let thumb_h = (track_h * visible_ratio).max(24.0).min(track_h);
    let max_scroll = (content_h - viewport_h).max(1.0);
    let scroll_ratio = (scroll_y / max_scroll).clamp(0.0, 1.0);
    let thumb_y = track_y + (track_h - thumb_h) * scroll_ratio;

    let thumb_color = theme.scrollbar_thumb;

    push_pill_thumb(
        verts,
        track_x,
        thumb_y,
        scrollbar_w,
        thumb_h,
        circle_uv,
        solid_uv,
        thumb_color,
        screen_w,
        screen_h,
    );
}
