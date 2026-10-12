use crate::quad::push_quad;
use crate::vertex::GlyphVertex;

// INFO: Draws teeth quads for the crisp 16x16 outline gear settings icon.
#[allow(clippy::too_many_arguments)]
pub(crate) fn render_gear_teeth(
    verts: &mut Vec<GlyphVertex>,
    cx: f32,
    cy: f32,
    stroke_col: [f32; 4],
    solid_uv: [f32; 2],
    screen_w: f32,
    screen_h: f32,
) {
    // Cardinal gear teeth (outer radius 8.0 px, width 3.0 px, stroke 1.0 px)
    // Top tooth
    push_quad(
        verts,
        cx - 1.5,
        cy - 8.0,
        cx + 1.5,
        cy - 7.0,
        solid_uv,
        solid_uv,
        stroke_col,
        screen_w,
        screen_h,
    );
    push_quad(
        verts,
        cx - 1.5,
        cy - 7.0,
        cx - 0.5,
        cy - 5.5,
        solid_uv,
        solid_uv,
        stroke_col,
        screen_w,
        screen_h,
    );
    push_quad(
        verts,
        cx + 0.5,
        cy - 7.0,
        cx + 1.5,
        cy - 5.5,
        solid_uv,
        solid_uv,
        stroke_col,
        screen_w,
        screen_h,
    );

    // Bottom tooth
    push_quad(
        verts,
        cx - 1.5,
        cy + 7.0,
        cx + 1.5,
        cy + 8.0,
        solid_uv,
        solid_uv,
        stroke_col,
        screen_w,
        screen_h,
    );
    push_quad(
        verts,
        cx - 1.5,
        cy + 5.5,
        cx - 0.5,
        cy + 7.0,
        solid_uv,
        solid_uv,
        stroke_col,
        screen_w,
        screen_h,
    );
    push_quad(
        verts,
        cx + 0.5,
        cy + 5.5,
        cx + 1.5,
        cy + 7.0,
        solid_uv,
        solid_uv,
        stroke_col,
        screen_w,
        screen_h,
    );

    // Left tooth
    push_quad(
        verts,
        cx - 8.0,
        cy - 1.5,
        cx - 7.0,
        cy + 1.5,
        solid_uv,
        solid_uv,
        stroke_col,
        screen_w,
        screen_h,
    );
    push_quad(
        verts,
        cx - 7.0,
        cy - 1.5,
        cx - 5.5,
        cy - 0.5,
        solid_uv,
        solid_uv,
        stroke_col,
        screen_w,
        screen_h,
    );
    push_quad(
        verts,
        cx - 7.0,
        cy + 0.5,
        cx - 5.5,
        cy + 1.5,
        solid_uv,
        solid_uv,
        stroke_col,
        screen_w,
        screen_h,
    );

    // Right tooth
    push_quad(
        verts,
        cx + 7.0,
        cy - 1.5,
        cx + 8.0,
        cy + 1.5,
        solid_uv,
        solid_uv,
        stroke_col,
        screen_w,
        screen_h,
    );
    push_quad(
        verts,
        cx + 5.5,
        cy - 1.5,
        cx + 7.0,
        cy - 0.5,
        solid_uv,
        solid_uv,
        stroke_col,
        screen_w,
        screen_h,
    );
    push_quad(
        verts,
        cx + 5.5,
        cy + 0.5,
        cx + 7.0,
        cy + 1.5,
        solid_uv,
        solid_uv,
        stroke_col,
        screen_w,
        screen_h,
    );

    // Diagonal gear teeth caps (at 45-degree angles)
    // Top-right
    push_quad(
        verts,
        cx + 4.5,
        cy - 6.5,
        cx + 6.5,
        cy - 4.5,
        solid_uv,
        solid_uv,
        stroke_col,
        screen_w,
        screen_h,
    );
    push_quad(
        verts,
        cx + 3.0,
        cy - 5.0,
        cx + 4.5,
        cy - 4.0,
        solid_uv,
        solid_uv,
        stroke_col,
        screen_w,
        screen_h,
    );
    push_quad(
        verts,
        cx + 4.0,
        cy - 4.5,
        cx + 5.0,
        cy - 3.0,
        solid_uv,
        solid_uv,
        stroke_col,
        screen_w,
        screen_h,
    );

    // Top-left
    push_quad(
        verts,
        cx - 6.5,
        cy - 6.5,
        cx - 4.5,
        cy - 4.5,
        solid_uv,
        solid_uv,
        stroke_col,
        screen_w,
        screen_h,
    );
    push_quad(
        verts,
        cx - 4.5,
        cy - 5.0,
        cx - 3.0,
        cy - 4.0,
        solid_uv,
        solid_uv,
        stroke_col,
        screen_w,
        screen_h,
    );
    push_quad(
        verts,
        cx - 5.0,
        cy - 4.5,
        cx - 4.0,
        cy - 3.0,
        solid_uv,
        solid_uv,
        stroke_col,
        screen_w,
        screen_h,
    );

    // Bottom-right
    push_quad(
        verts,
        cx + 4.5,
        cy + 4.5,
        cx + 6.5,
        cy + 6.5,
        solid_uv,
        solid_uv,
        stroke_col,
        screen_w,
        screen_h,
    );
    push_quad(
        verts,
        cx + 3.0,
        cy + 4.0,
        cx + 4.5,
        cy + 5.0,
        solid_uv,
        solid_uv,
        stroke_col,
        screen_w,
        screen_h,
    );
    push_quad(
        verts,
        cx + 4.0,
        cy + 3.0,
        cx + 5.0,
        cy + 4.5,
        solid_uv,
        solid_uv,
        stroke_col,
        screen_w,
        screen_h,
    );

    // Bottom-left
    push_quad(
        verts,
        cx - 6.5,
        cy + 4.5,
        cx - 4.5,
        cy + 6.5,
        solid_uv,
        solid_uv,
        stroke_col,
        screen_w,
        screen_h,
    );
    push_quad(
        verts,
        cx - 4.5,
        cy + 4.0,
        cx - 3.0,
        cy + 5.0,
        solid_uv,
        solid_uv,
        stroke_col,
        screen_w,
        screen_h,
    );
    push_quad(
        verts,
        cx - 5.0,
        cy + 3.0,
        cx - 4.0,
        cy + 4.5,
        solid_uv,
        solid_uv,
        stroke_col,
        screen_w,
        screen_h,
    );

    // Outer connecting rim segments between teeth
    push_quad(
        verts,
        cx + 1.5,
        cy - 5.5,
        cx + 3.5,
        cy - 4.5,
        solid_uv,
        solid_uv,
        stroke_col,
        screen_w,
        screen_h,
    );
    push_quad(
        verts,
        cx - 3.5,
        cy - 5.5,
        cx - 1.5,
        cy - 4.5,
        solid_uv,
        solid_uv,
        stroke_col,
        screen_w,
        screen_h,
    );
    push_quad(
        verts,
        cx + 1.5,
        cy + 4.5,
        cx + 3.5,
        cy + 5.5,
        solid_uv,
        solid_uv,
        stroke_col,
        screen_w,
        screen_h,
    );
    push_quad(
        verts,
        cx - 3.5,
        cy + 4.5,
        cx - 1.5,
        cy + 5.5,
        solid_uv,
        solid_uv,
        stroke_col,
        screen_w,
        screen_h,
    );
}
