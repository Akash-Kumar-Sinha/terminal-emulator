use crate::vertex::GlyphVertex;

// INFO: Emits 2D textured quad vertices transformed into normalized device coordinates.
#[allow(clippy::too_many_arguments)]
#[inline]
pub fn push_quad(
    verts: &mut Vec<GlyphVertex>,
    x0: f32,
    y0: f32,
    x1: f32,
    y1: f32,
    uv_min: [f32; 2],
    uv_max: [f32; 2],
    c: [f32; 4],
    screen_w: f32,
    screen_h: f32,
) {
    let l = (x0 / screen_w) * 2.0 - 1.0;
    let r = (x1 / screen_w) * 2.0 - 1.0;
    let t = 1.0 - (y0 / screen_h) * 2.0;
    let b = 1.0 - (y1 / screen_h) * 2.0;
    let v = |pos: [f32; 2], uv: [f32; 2]| GlyphVertex { pos, uv, color: c };
    verts.push(v([l, t], [uv_min[0], uv_min[1]]));
    verts.push(v([r, t], [uv_max[0], uv_min[1]]));
    verts.push(v([l, b], [uv_min[0], uv_max[1]]));
    verts.push(v([r, t], [uv_max[0], uv_min[1]]));
    verts.push(v([r, b], [uv_max[0], uv_max[1]]));
    verts.push(v([l, b], [uv_min[0], uv_max[1]]));
}

// INFO: Emits a pill-shaped scrollbar thumb quad sequence with antialiased rounded ends.
#[inline]
#[allow(clippy::too_many_arguments)]
pub fn push_pill_thumb(
    verts: &mut Vec<GlyphVertex>,
    sx: f32,
    sy: f32,
    sw: f32,
    sh: f32,
    circle_uv: [[f32; 2]; 2],
    solid_uv: [f32; 2],
    thumb_col: [f32; 4],
    screen_w: f32,
    screen_h: f32,
) {
    let radius = sw / 2.0;
    let mid_v = (circle_uv[0][1] + circle_uv[1][1]) * 0.5;

    if sh <= sw {
        push_quad(
            verts,
            sx,
            sy,
            sx + sw,
            sy + sh,
            circle_uv[0],
            circle_uv[1],
            thumb_col,
            screen_w,
            screen_h,
        );
    } else {
        // Top rounded cap
        push_quad(
            verts,
            sx,
            sy,
            sx + sw,
            sy + radius,
            circle_uv[0],
            [circle_uv[1][0], mid_v],
            thumb_col,
            screen_w,
            screen_h,
        );
        // Middle solid rectangular body
        push_quad(
            verts,
            sx,
            sy + radius,
            sx + sw,
            sy + sh - radius,
            solid_uv,
            solid_uv,
            thumb_col,
            screen_w,
            screen_h,
        );
        // Bottom rounded cap
        push_quad(
            verts,
            sx,
            sy + sh - radius,
            sx + sw,
            sy + sh,
            [circle_uv[0][0], mid_v],
            circle_uv[1],
            thumb_col,
            screen_w,
            screen_h,
        );
    }
}

// INFO: Emits a 9-slice rounded rectangle quad sequence with antialiased corner quadrants.
#[inline]
#[allow(clippy::too_many_arguments)]
pub fn push_rounded_rect(
    verts: &mut Vec<GlyphVertex>,
    x0: f32,
    y0: f32,
    x1: f32,
    y1: f32,
    radius: f32,
    circle_uv: [[f32; 2]; 2],
    solid_uv: [f32; 2],
    col: [f32; 4],
    screen_w: f32,
    screen_h: f32,
) {
    let w = x1 - x0;
    let h = y1 - y0;
    if w <= 0.0 || h <= 0.0 {
        return;
    }
    let r = radius.min(w * 0.5).min(h * 0.5).max(0.0);
    if r <= 0.5 {
        push_quad(
            verts, x0, y0, x1, y1, solid_uv, solid_uv, col, screen_w, screen_h,
        );
        return;
    }

    let u_mid = (circle_uv[0][0] + circle_uv[1][0]) * 0.5;
    let v_mid = (circle_uv[0][1] + circle_uv[1][1]) * 0.5;

    // Corner 1: Top-left
    push_quad(
        verts,
        x0,
        y0,
        x0 + r,
        y0 + r,
        circle_uv[0],
        [u_mid, v_mid],
        col,
        screen_w,
        screen_h,
    );
    // Corner 2: Top-right
    push_quad(
        verts,
        x1 - r,
        y0,
        x1,
        y0 + r,
        [u_mid, circle_uv[0][1]],
        [circle_uv[1][0], v_mid],
        col,
        screen_w,
        screen_h,
    );
    // Corner 3: Bottom-left
    push_quad(
        verts,
        x0,
        y1 - r,
        x0 + r,
        y1,
        [circle_uv[0][0], v_mid],
        [u_mid, circle_uv[1][1]],
        col,
        screen_w,
        screen_h,
    );
    // Corner 4: Bottom-right
    push_quad(
        verts,
        x1 - r,
        y1 - r,
        x1,
        y1,
        [u_mid, v_mid],
        circle_uv[1],
        col,
        screen_w,
        screen_h,
    );

    // Top center edge
    if x1 - r > x0 + r {
        push_quad(
            verts,
            x0 + r,
            y0,
            x1 - r,
            y0 + r,
            solid_uv,
            solid_uv,
            col,
            screen_w,
            screen_h,
        );
        // Bottom center edge
        push_quad(
            verts,
            x0 + r,
            y1 - r,
            x1 - r,
            y1,
            solid_uv,
            solid_uv,
            col,
            screen_w,
            screen_h,
        );
    }
    // Middle body
    if y1 - r > y0 + r {
        push_quad(
            verts,
            x0,
            y0 + r,
            x1,
            y1 - r,
            solid_uv,
            solid_uv,
            col,
            screen_w,
            screen_h,
        );
    }
}

// INFO: Emits a crisp line with specified thickness as an oriented quad in NDC coordinates.
#[allow(clippy::too_many_arguments)]
#[inline]
pub fn push_line(
    verts: &mut Vec<GlyphVertex>,
    x0: f32,
    y0: f32,
    x1: f32,
    y1: f32,
    thickness: f32,
    solid_uv: [f32; 2],
    c: [f32; 4],
    screen_w: f32,
    screen_h: f32,
) {
    let dx = x1 - x0;
    let dy = y1 - y0;
    let len = (dx * dx + dy * dy).sqrt();
    if len < 1e-4 {
        return;
    }
    let half = thickness * 0.5;
    let nx = -dy / len * half;
    let ny = dx / len * half;

    let p0 = [x0 + nx, y0 + ny];
    let p1 = [x1 + nx, y1 + ny];
    let p2 = [x0 - nx, y0 - ny];
    let p3 = [x1 - nx, y1 - ny];

    let to_ndc = |p: [f32; 2]| [(p[0] / screen_w) * 2.0 - 1.0, 1.0 - (p[1] / screen_h) * 2.0];
    let v0 = GlyphVertex {
        pos: to_ndc(p0),
        uv: solid_uv,
        color: c,
    };
    let v1 = GlyphVertex {
        pos: to_ndc(p1),
        uv: solid_uv,
        color: c,
    };
    let v2 = GlyphVertex {
        pos: to_ndc(p2),
        uv: solid_uv,
        color: c,
    };
    let v3 = GlyphVertex {
        pos: to_ndc(p3),
        uv: solid_uv,
        color: c,
    };

    verts.push(v0);
    verts.push(v1);
    verts.push(v2);
    verts.push(v1);
    verts.push(v3);
    verts.push(v2);
}
