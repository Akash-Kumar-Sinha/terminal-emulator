use std::sync::Arc;

use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::{ElementState, KeyEvent, Modifiers, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::{Key, ModifiersState, NamedKey};
use winit::window::{Window, WindowId};

use crate::fonts::FontManager;
use crate::renderer::glyph::GlyphRenderer;
use crate::state::WgpuState;
use crate::terminal::Terminal;
use crate::theme::Theme;

const MARGIN: f32 = 8.0;

#[derive(Default)]
pub struct App {
    window: Option<Arc<Window>>,
    state: Option<WgpuState>,
    fonts: Option<FontManager>,
    theme: Theme,
    terminal: Option<Terminal>,
    modifiers: ModifiersState,
}

fn grid_dims(width: u32, height: u32, fonts: &FontManager) -> (usize, usize) {
    let usable_w = (width as f32 - 2.0 * MARGIN).max(0.0);
    let usable_h = (height as f32 - 2.0 * MARGIN).max(0.0);
    let cols = (usable_w / fonts.cell_w).floor() as usize;
    let rows = (usable_h / fonts.cell_h).floor() as usize;
    (rows.max(1), cols.max(1))
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }

        let window_size = PhysicalSize::new(1200, 800);
        let window = event_loop
            .create_window(
                Window::default_attributes()
                    .with_title("AKS emulator")
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
        surface.configure(&device, &config);

        let fonts = FontManager::new();
        let glyphs = GlyphRenderer::new(&device, &queue, format);

        let (rows, cols) = grid_dims(config.width, config.height, &fonts);
        let terminal = match Terminal::new(rows, cols) {
            Ok(t) => Some(t),
            Err(e) => {
                eprintln!("failed to start shell: {e:?}");
                None
            }
        };

        self.window = Some(window.clone());
        self.state = Some(WgpuState::new(surface, adapter, device, queue, config, glyphs));
        self.fonts = Some(fonts);
        self.terminal = terminal;

        window.request_redraw();
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),

            WindowEvent::ModifiersChanged(mods) => {
                self.modifiers = Modifiers::state(&mods);
            }

            WindowEvent::KeyboardInput { event, .. } => {
                if event.state == ElementState::Pressed {
                    if let (Some(terminal), Some(bytes)) =
                        (self.terminal.as_mut(), encode_key(&event, self.modifiers))
                    {
                        terminal.send(&bytes);
                    }
                }
            }

            WindowEvent::Resized(new_size) => {
                if let Some(state) = self.state.as_mut() {
                    state.resize(new_size.width, new_size.height);
                }
                if let (Some(state), Some(fonts), Some(terminal)) =
                    (self.state.as_ref(), self.fonts.as_ref(), self.terminal.as_mut())
                {
                    let (w, h) = state.size();
                    let (rows, cols) = grid_dims(w, h, fonts);
                    terminal.resize(rows, cols);
                }
            }

            WindowEvent::RedrawRequested => {
                if let Some(terminal) = self.terminal.as_mut() {
                    terminal.pump();
                    if terminal.is_closed() {
                        event_loop.exit();
                        return;
                    }
                }

                if let (Some(state), Some(fonts)) = (self.state.as_mut(), self.fonts.as_ref()) {
                    let (grid, cursor) = match self.terminal.as_ref() {
                        Some(t) => (Some(t.grid()), Some(t.cursor())),
                        None => (None, None),
                    };
                    state.render(
                        fonts,
                        &self.theme,
                        grid,
                        cursor,
                        &[],
                        &[],
                        MARGIN,
                        MARGIN,
                    );
                }

                if let Some(window) = self.window.as_ref() {
                    window.request_redraw();
                }
            }

            _ => {}
        }
    }
}

fn encode_key(event: &KeyEvent, mods: ModifiersState) -> Option<Vec<u8>> {
    if mods.control_key() {
        if let Key::Character(s) = &event.logical_key {
            if let Some(c) = s.chars().next() {
                let up = c.to_ascii_uppercase();
                if up.is_ascii_alphabetic() {
                    return Some(vec![(up as u8) & 0x1f]);
                }
                match c {
                    ' ' => return Some(vec![0]),
                    '[' => return Some(vec![0x1b]),
                    '\\' => return Some(vec![0x1c]),
                    ']' => return Some(vec![0x1d]),
                    _ => {}
                }
            }
        }
    }

    match &event.logical_key {
        Key::Named(named) => match named {
            NamedKey::Enter => Some(b"\r".to_vec()),
            NamedKey::Backspace => Some(vec![0x7f]),
            NamedKey::Tab => Some(b"\t".to_vec()),
            NamedKey::Escape => Some(vec![0x1b]),
            NamedKey::ArrowUp => Some(b"\x1b[A".to_vec()),
            NamedKey::ArrowDown => Some(b"\x1b[B".to_vec()),
            NamedKey::ArrowRight => Some(b"\x1b[C".to_vec()),
            NamedKey::ArrowLeft => Some(b"\x1b[D".to_vec()),
            NamedKey::Home => Some(b"\x1b[H".to_vec()),
            NamedKey::End => Some(b"\x1b[F".to_vec()),
            NamedKey::Delete => Some(b"\x1b[3~".to_vec()),
            NamedKey::PageUp => Some(b"\x1b[5~".to_vec()),
            NamedKey::PageDown => Some(b"\x1b[6~".to_vec()),
            NamedKey::Space => Some(b" ".to_vec()),
            _ => None,
        },
        _ => event.text.as_ref().map(|t| t.as_bytes().to_vec()),
    }
}
