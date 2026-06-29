use std::sync::Arc;

use cosmic_text::{Attrs, Buffer, Family, FontSystem, Metrics, Shaping, SwashCache};
use glyphon::{Cache, TextAtlas, TextRenderer};
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::window::{Window, WindowId};

use crate::fonts::TextPipeline;
use crate::state::WgpuState;
use crate::renderer::rect::RectPipeline;

pub struct App {
    window: Option<Arc<Window>>,
    state: Option<WgpuState>,
    text_pipeline: Option<TextPipeline>,
}

impl Default for App {
    fn default() -> Self {
        Self {
            window: None,
            state: None,
            text_pipeline: None,
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window_size = PhysicalSize::new(1200, 800);
        let title = "AKS emulator";

        let window = event_loop
            .create_window(
                Window::default_attributes()
                    .with_title(title)
                    .with_inner_size(window_size),
            )
            .unwrap();

        let window = Arc::new(window);

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle());

        let surface = instance.create_surface(window.clone()).unwrap();

        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            compatible_surface: Some(&surface),
            ..Default::default()
        }))
        .unwrap();

        let (device, queue) =
            pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor::default())).unwrap();

        let size = window.inner_size();
        let surface_caps = surface.get_capabilities(&adapter);
        let format = surface_caps.formats[0];

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        let mut font_system = FontSystem::new();
        let swash_cache = SwashCache::new();

        let metrics = Metrics::new(20.0, 24.0);
        let mut buffer = Buffer::new(&mut font_system, metrics);

        buffer.set_size(
            &mut font_system,
            Some(config.width as f32),
            Some(config.height as f32),
        );

        buffer.set_text(
            &mut font_system,
            "Hello, world!\nThis is a test of the terminal emulator.",
            &Attrs::new().family(Family::Monospace),
            Shaping::Advanced,
            None,
        );
        buffer.shape_until_scroll(&mut font_system, false);

        let glyph_cache = Cache::new(&device);
        let mut atlas = TextAtlas::new(&device, &queue, &glyph_cache, config.format);
        let text = TextRenderer::new(&mut atlas, &device, wgpu::MultisampleState::default(), None);

        let rect = RectPipeline::new(&device, config.format);

        surface.configure(&device, &config);

        self.window = Some(window);
        self.text_pipeline = Some(TextPipeline::new(
            font_system,
            swash_cache,
            buffer,
            glyph_cache,
            atlas,
            text,
            &device,
        ));
        self.state = Some(WgpuState::new(
            surface, adapter, device, queue, config, rect,
        ));
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                if let (Some(state), Some(text_pipeline)) =
                    (self.state.as_mut(), self.text_pipeline.as_mut())
                {
                    state.render(text_pipeline);
                }
                if let Some(window) = self.window.as_ref() {
                    window.request_redraw(); // ask for another frame, keeps the loop going
                }
            }
            WindowEvent::Resized(new_size) => {
                if let Some(state) = self.state.as_mut() {
                    state.resize(new_size.width, new_size.height);
                }
            }
            _ => {}
        }
    }
}
