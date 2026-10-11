use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable, Debug)]
pub struct GlyphVertex {
    pub pos: [f32; 2],
    pub uv: [f32; 2],
    pub color: [f32; 4],
}

impl GlyphVertex {
    pub fn new(pos: [f32; 2], uv: [f32; 2], color: [f32; 4]) -> Self {
        Self { pos, uv, color }
    }
}
