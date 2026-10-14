// INFO: Rendering abstraction sink receiving 2D vector primitives for geometric icons.
pub trait IconSink {
    fn push_quad(&mut self, x0: f32, y0: f32, x1: f32, y1: f32, color: [f32; 4]);
    fn push_line(&mut self, x0: f32, y0: f32, x1: f32, y1: f32, stroke: f32, color: [f32; 4]);
    fn push_rounded_rect(
        &mut self,
        x0: f32,
        y0: f32,
        x1: f32,
        y1: f32,
        radius: f32,
        color: [f32; 4],
    );
}

// INFO: In-memory primitive collector for tests and headless validation.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct MockIconSink {
    pub quads: Vec<([f32; 4], [f32; 4])>,
    pub lines: Vec<([f32; 4], f32, [f32; 4])>,
    pub rounded_rects: Vec<([f32; 4], f32, [f32; 4])>,
}

impl IconSink for MockIconSink {
    fn push_quad(&mut self, x0: f32, y0: f32, x1: f32, y1: f32, color: [f32; 4]) {
        self.quads.push(([x0, y0, x1, y1], color));
    }

    fn push_line(&mut self, x0: f32, y0: f32, x1: f32, y1: f32, stroke: f32, color: [f32; 4]) {
        self.lines.push(([x0, y0, x1, y1], stroke, color));
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
        self.rounded_rects.push(([x0, y0, x1, y1], radius, color));
    }
}
