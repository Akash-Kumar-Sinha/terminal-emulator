use std::sync::Arc;

use akssora_terminal_config::Theme;
use akssora_terminal_core::Terminal;
use akssora_terminal_input::{encode_key, scroll_delta_to_lines};
use akssora_terminal_renderer::{FontManager, GlyphRenderer};
use akssora_terminal_ui::{Block, Header, HeaderButton};
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::{ElementState, Modifiers, MouseButton, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::ModifiersState;
use winit::window::{Window, WindowId};

use crate::state::WgpuState;

const MARGIN: f32 = 2.0;
const MIN_BLOCK_HEIGHT: f32 = 100.0;

fn max_block_height(window_h: u32, header_h: f32) -> f32 {
    (window_h as f32 - header_h).max(MIN_BLOCK_HEIGHT)
}

fn grid_dims(width: u32, height: u32, fonts: &FontManager) -> (usize, usize) {
    let usable_w = (width as f32 - 2.0 * MARGIN).max(0.0);
    let usable_h = (height as f32 - 2.0 * MARGIN).max(0.0);
    let cols = (usable_w / fonts.cell_w).floor() as usize;
    let rows = (usable_h / fonts.cell_h).floor() as usize;
    (rows.max(1), cols.max(1))
}

// INFO: Window handler coordinating event dispatch, terminal state updates, and rendering loop.
pub struct App {
    window: Option<Arc<Window>>,
    state: Option<WgpuState>,
    fonts: Option<FontManager>,
    theme: Theme,
    modifiers: ModifiersState,
    block: Vec<Option<Block>>,
    header: Header,
    cursor_pos: (f32, f32),
}

impl Default for App {
    fn default() -> Self {
        Self {
            window: None,
            state: None,
            fonts: None,
            theme: Theme::default(),
            modifiers: ModifiersState::default(),
            block: Vec::new(),
            header: Header::new("Akssora"),
            cursor_pos: (0.0, 0.0),
        }
    }
}

impl App {
    pub fn new() -> Self {
        Self::default()
    }
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
                    .with_title("Akssora")
                    .with_decorations(false)
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

        let block_width = size.width;
        let max_h = max_block_height(size.height, self.header.height);
        let (rows, cols) = grid_dims(block_width, max_h as u32, &fonts);
        let terminal = match Terminal::new(rows, cols) {
            Ok(t) => Some(t),
            Err(e) => {
                eprintln!("failed to start shell: {e:?}");
                None
            }
        };

        let block = Some(Block::new(
            Some(block_width),
            Some(MIN_BLOCK_HEIGHT as u32),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            terminal.expect("terminal must initialize"),
        ));

        self.window = Some(window.clone());
        self.state = Some(WgpuState::new(
            surface, adapter, device, queue, config, glyphs,
        ));
        self.fonts = Some(fonts);
        self.block.push(block);

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
                if event.state == ElementState::Pressed
                    && let (Some(block), Some(bytes)) = (
                        self.block.first_mut().and_then(|b| b.as_mut()),
                        encode_key(&event, self.modifiers),
                    )
                {
                    block.shell.send(&bytes);
                }
            }

            WindowEvent::Resized(new_size) => {
                if let Some(state) = self.state.as_mut() {
                    state.resize(new_size.width, new_size.height);
                }
                let max_h = max_block_height(new_size.height, self.header.height);
                if let Some(fonts) = self.fonts.as_ref() {
                    for block in self.block.iter_mut().filter_map(|b| b.as_mut()) {
                        block.width = new_size.width;
                        let (rows, cols) = grid_dims(block.width, max_h as u32, fonts);
                        block.shell.resize(rows, cols);
                    }
                }
            }

            WindowEvent::MouseWheel { delta, .. } => {
                let cell_h = self.fonts.as_ref().map(|f| f.cell_h).unwrap_or(16.0);
                let lines = scroll_delta_to_lines(delta, cell_h);
                if let Some(block) = self.block.first_mut().and_then(|b| b.as_mut()) {
                    if lines > 0.0 {
                        block.shell.scroll_up(lines.ceil() as usize);
                    } else if lines < 0.0 {
                        block.shell.scroll_down((-lines).ceil() as usize);
                    }
                }
            }

            WindowEvent::CursorMoved { position, .. } => {
                self.cursor_pos = (position.x as f32, position.y as f32);
                let w = self.state.as_ref().map(|s| s.size().0).unwrap_or(0) as f32;
                let hovered = self
                    .header
                    .hit_test(self.cursor_pos.0, self.cursor_pos.1, w);
                if hovered != self.header.hovered {
                    self.header.hovered = hovered;
                    if let Some(window) = self.window.as_ref() {
                        window.request_redraw();
                    }
                }
            }

            WindowEvent::MouseInput {
                state: ElementState::Pressed,
                button: MouseButton::Left,
                ..
            } => {
                let w = self.state.as_ref().map(|s| s.size().0).unwrap_or(0) as f32;
                let (mx, my) = self.cursor_pos;
                match self.header.hit_test(mx, my, w) {
                    Some(HeaderButton::Minimize) => {
                        if let Some(win) = self.window.as_ref() {
                            win.set_minimized(true);
                        }
                    }
                    Some(HeaderButton::Maximize) => {
                        if let Some(win) = self.window.as_ref() {
                            win.set_maximized(!win.is_maximized());
                        }
                    }
                    Some(HeaderButton::Close) => event_loop.exit(),
                    None => {
                        if self.header.in_drag_region(mx, my, w)
                            && let Some(win) = self.window.as_ref()
                        {
                            let _ = win.drag_window();
                        }
                    }
                }
            }

            WindowEvent::RedrawRequested => {
                let screen_h = self.state.as_ref().map(|s| s.size().1).unwrap_or(0) as f32;
                let header_h = self.header.height;
                for block in self.block.iter_mut().filter_map(|b| b.as_mut()) {
                    block.shell.pump();
                    if block.shell.is_closed() {
                        event_loop.exit();
                        return;
                    }
                    block.y = header_h;
                    block.height = (screen_h - header_h).max(0.0) as u32;
                }

                if let (Some(state), Some(fonts)) = (self.state.as_mut(), self.fonts.as_ref()) {
                    let blocks: Vec<&Block> =
                        self.block.iter().filter_map(|b| b.as_ref()).collect();
                    state.render(fonts, &self.theme, &self.header, &blocks);
                }

                if let Some(window) = self.window.as_ref() {
                    window.request_redraw();
                }
            }

            _ => {}
        }
    }
}
