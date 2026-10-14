use akssora_icons::IconSink;

use crate::quad::{push_line, push_quad, push_rounded_rect};
use crate::vertex::GlyphVertex;

// INFO: Adapts GlyphVertex buffer and shader UV coordinates to the centralized IconSink trait.
pub struct VertexIconSink<'a> {
    pub verts: &'a mut Vec<GlyphVertex>,
    pub solid_uv: [f32; 2],
    pub circle_uv: [[f32; 2]; 2],
    pub screen_w: f32,
    pub screen_h: f32,
}

impl<'a> VertexIconSink<'a> {
    pub fn new(
        verts: &'a mut Vec<GlyphVertex>,
        solid_uv: [f32; 2],
        circle_uv: [[f32; 2]; 2],
        screen_w: f32,
        screen_h: f32,
    ) -> Self {
        Self {
            verts,
            solid_uv,
            circle_uv,
            screen_w,
            screen_h,
        }
    }
}

impl<'a> IconSink for VertexIconSink<'a> {
    fn push_quad(&mut self, x0: f32, y0: f32, x1: f32, y1: f32, color: [f32; 4]) {
        push_quad(
            self.verts,
            x0,
            y0,
            x1,
            y1,
            self.solid_uv,
            self.solid_uv,
            color,
            self.screen_w,
            self.screen_h,
        );
    }

    fn push_line(&mut self, x0: f32, y0: f32, x1: f32, y1: f32, stroke: f32, color: [f32; 4]) {
        push_line(
            self.verts,
            x0,
            y0,
            x1,
            y1,
            stroke,
            self.solid_uv,
            color,
            self.screen_w,
            self.screen_h,
        );
    }

    fn push_rounded_rect(
        &mut self,
        x0: f32,
        y0: f32,
        x1: f32,
        y1: f32,
        radius: f32,
        color: [f32; 4],
    ) {
        push_rounded_rect(
            self.verts,
            x0,
            y0,
            x1,
            y1,
            radius,
            self.circle_uv,
            self.solid_uv,
            color,
            self.screen_w,
            self.screen_h,
        );
    }
}
