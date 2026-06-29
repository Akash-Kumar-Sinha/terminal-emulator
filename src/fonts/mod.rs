use cosmic_text::SwashCache;
use glyphon::{
    Buffer, Cache, Color, FontSystem, Metrics, Resolution, TextArea, TextAtlas, TextBounds,
    TextRenderer, Viewport,
};

pub struct TextPipeline {
    font_system: FontSystem,
    swash_cache: SwashCache,
    buffer: Buffer,
    #[allow(dead_code)]
    glyph_cache: Cache,
    atlas: TextAtlas,
    text: TextRenderer,
    viewport: Viewport,
}

impl TextPipeline {
    pub fn new(
        font_system: FontSystem,
        swash_cache: SwashCache,
        buffer: Buffer,
        glyph_cache: Cache,
        atlas: TextAtlas,
        text: TextRenderer,
        device: &wgpu::Device,
    ) -> Self {
        let viewport = Viewport::new(device, &glyph_cache);
        Self {
            font_system,
            swash_cache,
            buffer,
            glyph_cache,
            atlas,
            text,
            viewport,
        }
    }

    pub fn set_font_size(&mut self, size: f32) {
        self.buffer
            .set_metrics(&mut self.font_system, Metrics::new(size, size * 1.2));
    }

    pub fn prepare(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        width: u32,
        height: u32,
        font_size: f32,
    ) {
        self.set_font_size(font_size);

        self.viewport.update(queue, Resolution { width, height });

        self.text
            .prepare(
                device,
                queue,
                &mut self.font_system,
                &mut self.atlas,
                &self.viewport,
                [TextArea {
                    buffer: &self.buffer,
                    left: 10.0,
                    top: 10.0,
                    scale: 1.0,
                    bounds: TextBounds {
                        left: 0,
                        top: 0,
                        right: width as i32,
                        bottom: height as i32,
                    },
                    default_color: Color::rgb(234, 224, 224),
                    custom_glyphs: &[],
                }],
                &mut self.swash_cache,
            )
            .unwrap();
    }

    pub fn render<'pass>(&'pass self, render_pass: &mut wgpu::RenderPass<'pass>) {
        self.text
            .render(&self.atlas, &self.viewport, render_pass)
            .unwrap();
    }
}
