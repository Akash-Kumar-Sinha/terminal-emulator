use crate::fonts::FontManager;
use crate::renderer::glyph::{ChromeRect, ChromeText, GlyphRenderer};
use crate::terminal::TerminalGrid;
use crate::theme::Theme;

pub struct WgpuState {
    surface: wgpu::Surface<'static>,
    #[allow(dead_code)]
    adapter: wgpu::Adapter,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    glyphs: GlyphRenderer,
}

impl WgpuState {
    pub fn new(
        surface: wgpu::Surface<'static>,
        adapter: wgpu::Adapter,
        device: wgpu::Device,
        queue: wgpu::Queue,
        config: wgpu::SurfaceConfiguration,
        glyphs: GlyphRenderer,
    ) -> Self {
        Self {
            surface,
            adapter,
            device,
            queue,
            config,
            glyphs,
        }
    }

    pub fn size(&self) -> (u32, u32) {
        (self.config.width, self.config.height)
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if width == 0 || height == 0 {
            return;
        }
        self.config.width = width;
        self.config.height = height;
        self.surface.configure(&self.device, &self.config);
    }

    #[allow(clippy::too_many_arguments)]
    pub fn render(
        &mut self,
        fonts: &FontManager,
        theme: &Theme,
        grid: Option<&TerminalGrid>,
        cursor: Option<(usize, usize)>,
        rects: &[ChromeRect],
        texts: &[ChromeText],
        origin_x: f32,
        origin_y: f32,
    ) {
        let frame = match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(frame) => frame,
            wgpu::CurrentSurfaceTexture::Suboptimal(frame) => {
                self.surface.configure(&self.device, &self.config);
                frame
            }
            wgpu::CurrentSurfaceTexture::Outdated => {
                self.surface.configure(&self.device, &self.config);
                return;
            }
            wgpu::CurrentSurfaceTexture::Timeout
            | wgpu::CurrentSurfaceTexture::Occluded
            | wgpu::CurrentSurfaceTexture::Validation => {
                return;
            }
            wgpu::CurrentSurfaceTexture::Lost => {
                panic!("surface lost");
            }
        };

        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        self.glyphs.prepare(
            &self.device,
            &self.queue,
            grid,
            fonts,
            theme,
            cursor,
            rects,
            texts,
            origin_x,
            origin_y,
            self.config.width as f32,
            self.config.height as f32,
        );

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());

        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("main pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(theme.clear_color()),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });

            self.glyphs.draw(&mut pass);
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        frame.present();
    }
}
