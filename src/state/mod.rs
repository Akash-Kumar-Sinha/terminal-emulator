use crate::fonts::TextPipeline;
use crate::renderer::rect::RectPipeline;

pub struct WgpuState {
    surface: wgpu::Surface<'static>,
    #[allow(dead_code)]
    adapter: wgpu::Adapter,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    rect: RectPipeline,
}

impl WgpuState {
    pub fn new(
        surface: wgpu::Surface<'static>,
        adapter: wgpu::Adapter,
        device: wgpu::Device,
        queue: wgpu::Queue,
        config: wgpu::SurfaceConfiguration,
        rect: RectPipeline,
    ) -> Self {
        Self { surface, adapter, device, queue, config, rect }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if width == 0 || height == 0 {
            return;
        }
        self.config.width = width;
        self.config.height = height;
        self.surface.configure(&self.device, &self.config);
    }

    pub fn render(&mut self, text_pipeline: &mut TextPipeline) {
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

        text_pipeline.prepare(
            &self.device,
            &self.queue,
            self.config.width,
            self.config.height,
            12.0,
        );

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());

        let sw = self.config.width as f32;
        let sh = self.config.height as f32;

        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("main pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color { r: 0.0, g: 0.0, b: 0.0, a: 1.0 }),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });

            // bottom bar: x=0, y=screen_h-80, width=screen_w, height=80
            self.rect.draw(
                &self.queue,
                &mut pass,
                0.0,              // x
                sh - 80.0,        // y (top-left of rect, pinned to bottom)
                sw,               // width
                80.0,             // height
                1.0,              // border_width
                [0.0, 0.0, 0.0, 0.0],    // bg_color
                [0.3, 0.3, 0.3, 1.0],    // border_color
                sw,
                sh,
            );

            text_pipeline.render(&mut pass);
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        frame.present();
    }
}
