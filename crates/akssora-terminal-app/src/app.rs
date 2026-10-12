use std::collections::HashMap;
use std::sync::Arc;

use akssora_terminal_config::{Config, FontConfig, Keybindings, TabLayoutPreference, Theme};
use akssora_terminal_renderer::{FontManager, GlyphRenderer};
use akssora_terminal_ui::{Block, Header, PaneTree, SettingsAction, Sidebar, TabManager, Tooltip};
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::{Modifiers, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::ModifiersState;
use winit::window::{Window, WindowId};

use crate::state::WgpuState;

mod keyboard;
mod mouse;
mod mouse_click;
mod mouse_selection;
mod session_ops;
pub(crate) mod session_persist;
pub(crate) mod session_tracker;
mod split;
mod split_state;
mod tabs;
mod tooltips;

pub use split_state::{PaneDragState, SplitManager};

// INFO: Multi-tab window handler coordinating event dispatch, terminal state updates, and rendering loop.
pub struct App {
    pub(crate) window: Option<Arc<Window>>,
    pub(crate) state: Option<WgpuState>,
    pub(crate) fonts: Option<FontManager>,
    pub(crate) ui_fonts: Option<FontManager>,
    pub(crate) theme: Theme,
    pub config: Config,
    pub(crate) modifiers: ModifiersState,
    pub(crate) blocks: HashMap<u64, Block>,
    pub tab_manager: TabManager,
    pub split_trees: HashMap<u64, PaneTree>,
    pub split_manager: SplitManager,
    pub sidebar: Sidebar,
    pub header: Header,
    pub(crate) cursor_pos: (f32, f32),
    pub(crate) active_tooltip: Option<Tooltip>,
    pub(crate) command_buffers: HashMap<u64, String>,
    pub(crate) last_executed_commands: HashMap<u64, String>,
    pub(crate) pane_custom_names: HashMap<u64, String>,
    pub(crate) hovered_setting_action: Option<SettingsAction>,
    pub(crate) settings_layout_dropdown_open: bool,
    pub(crate) active_settings_tab: akssora_terminal_ui::SettingsTab,
    pub(crate) settings_search_query: String,
    pub prompt_confirm_exit: bool,
    pub session_file: Option<std::path::PathBuf>,
    pub context_menu: Option<akssora_terminal_ui::ContextMenu>,
    pub(crate) is_selecting_text: bool,
    pub(crate) selection_start_pane: Option<u64>,
    pub(crate) selection_start_pt: Option<akssora_terminal_screen::TerminalPoint>,
    pub(crate) mouse_press_pos: Option<(f32, f32)>,
    pub(crate) mouse_press_pane: Option<u64>,
    pub(crate) last_layout_time: Option<std::time::Instant>,
    pub(crate) last_render_time: Option<std::time::Instant>,
}

impl Default for App {
    fn default() -> Self {
        let config = Config::load();
        let mut header = Header::new("Akssora");
        let mut sidebar = Sidebar::new();
        let initial_width = config
            .sidebar_width
            .clamp(Sidebar::MIN_WIDTH, Sidebar::MAX_WIDTH);
        sidebar.saved_width = initial_width;
        sidebar.is_expanded = config.tab_layout.show_sidebar();
        sidebar.width = if sidebar.is_expanded {
            initial_width
        } else {
            0.0
        };
        header.is_sidebar_expanded = sidebar.is_expanded;
        Self {
            window: None,
            state: None,
            fonts: None,
            ui_fonts: None,
            theme: config.theme.clone(),
            config,
            modifiers: ModifiersState::default(),
            blocks: HashMap::new(),
            tab_manager: TabManager::new(),
            split_trees: HashMap::new(),
            split_manager: SplitManager::new(),
            sidebar,
            header,
            cursor_pos: (0.0, 0.0),
            active_tooltip: None,
            command_buffers: HashMap::new(),
            last_executed_commands: HashMap::new(),
            pane_custom_names: HashMap::new(),
            hovered_setting_action: None,
            settings_layout_dropdown_open: false,
            active_settings_tab: akssora_terminal_ui::SettingsTab::General,
            settings_search_query: String::new(),
            prompt_confirm_exit: false,
            session_file: None,
            context_menu: None,
            is_selecting_text: false,
            selection_start_pane: None,
            selection_start_pt: None,
            mouse_press_pos: None,
            mouse_press_pane: None,
            last_layout_time: None,
            last_render_time: None,
        }
    }
}

impl App {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn tab_layout(&self) -> TabLayoutPreference {
        self.config.tab_layout
    }

    pub fn set_tab_layout(&mut self, layout: TabLayoutPreference) {
        self.config.tab_layout = layout;
        let _ = self.config.save();
        self.sidebar.is_expanded = layout.show_sidebar();
        self.sidebar.width = if sidebar_width_needed(&self.sidebar) {
            if self.sidebar.saved_width >= Sidebar::MIN_WIDTH {
                self.sidebar.saved_width
            } else {
                Sidebar::DEFAULT_WIDTH
            }
        } else {
            0.0
        };
        self.header.is_sidebar_expanded = self.sidebar.is_expanded;
        self.layout_blocks();
        if let Some(win) = self.window.as_ref() {
            win.request_redraw();
        }
    }

    pub fn cycle_tab_layout(&mut self) {
        let next = self.config.tab_layout.next();
        self.set_tab_layout(next);
    }

    pub fn set_font_size(&mut self, size: f32) {
        let new_size = size.clamp(FontConfig::MIN_SIZE, FontConfig::MAX_SIZE);
        self.config.font.size = new_size;
        let _ = self.config.save();
        if let Some(fonts) = self.fonts.as_mut() {
            fonts.set_size(new_size);
        }
        // INFO: ui_fonts drives the header, titlebar, and sidebar — keep it independently controlled and untouched by terminal zoom.
        if let Some(state) = self.state.as_mut() {
            state.clear_glyph_cache();
        }
        self.layout_blocks();
        if let Some(win) = self.window.as_ref() {
            win.request_redraw();
        }
    }

    pub fn increase_font_size(&mut self) {
        let curr = self.config.font.size;
        self.set_font_size(curr + FontConfig::STEP);
    }

    pub fn decrease_font_size(&mut self) {
        let curr = self.config.font.size;
        self.set_font_size(curr - FontConfig::STEP);
    }

    pub fn reset_font_size(&mut self) {
        self.set_font_size(FontConfig::DEFAULT_SIZE);
    }

    pub fn set_ui_font_size(&mut self, size: f32) {
        let new_size = size.clamp(FontConfig::MIN_SIZE, FontConfig::MAX_SIZE);
        self.config.ui_font_size = new_size;
        let _ = self.config.save();
        if let Some(ui_fonts) = self.ui_fonts.as_mut() {
            ui_fonts.set_size(new_size);
        }
        if let Some(state) = self.state.as_mut() {
            state.clear_glyph_cache();
        }
        if let Some(win) = self.window.as_ref() {
            win.request_redraw();
        }
    }

    pub fn increase_ui_font_size(&mut self) {
        let curr = self.config.ui_font_size;
        self.set_ui_font_size(curr + FontConfig::STEP);
    }

    pub fn decrease_ui_font_size(&mut self) {
        let curr = self.config.ui_font_size;
        self.set_ui_font_size(curr - FontConfig::STEP);
    }

    pub fn reset_ui_font_size(&mut self) {
        self.set_ui_font_size(14.0);
    }

    pub fn set_test_fonts(&mut self, fonts: FontManager, ui_fonts: FontManager) {
        self.fonts = Some(fonts);
        self.ui_fonts = Some(ui_fonts);
    }

    pub fn terminal_font_size(&self) -> f32 {
        self.fonts
            .as_ref()
            .map(|f| f.size())
            .unwrap_or(self.config.font.size)
    }

    pub fn current_ui_font_size(&self) -> f32 {
        self.ui_fonts
            .as_ref()
            .map(|f| f.size())
            .unwrap_or(self.config.ui_font_size)
    }

    pub fn toggle_sidebar(&mut self) {
        if self.config.tab_layout.includes_sidebar() {
            self.sidebar.toggle();
            self.header.is_sidebar_expanded = self.sidebar.is_expanded;
            self.layout_blocks();
            if let Some(win) = self.window.as_ref() {
                win.request_redraw();
            }
        }
    }

    pub fn active_pane_id(&self) -> Option<u64> {
        let active_tab_id = self.tab_manager.active_id()?;
        if let Some(focused_pane) = self.split_manager.active_pane_id
            && let Some(tree) = self.split_trees.get(&active_tab_id)
            && tree.contains_leaf(focused_pane)
        {
            return Some(focused_pane);
        }
        Some(active_tab_id)
    }

    pub fn keybindings(&self) -> &Keybindings {
        &self.config.keybindings
    }

    pub fn set_test_cursor_pos(&mut self, x: f32, y: f32) {
        self.cursor_pos = (x, y);
    }

    pub fn set_test_modifiers(&mut self, mods: ModifiersState) {
        self.modifiers = mods;
    }

    pub fn handle_test_mouse_press(&mut self) {
        let (mx, my) = self.cursor_pos;
        self.focus_pane_at(mx, my);

        if let Some((pane_id, _, _, _, _)) = self.pane_at(mx, my) {
            self.mouse_press_pos = Some((mx, my));
            self.mouse_press_pane = Some(pane_id);
            self.selection_start_pane = Some(pane_id);

            let (cell_w, cell_h) = self
                .fonts
                .as_ref()
                .map(|f| (f.cell_w, f.cell_h))
                .unwrap_or((9.0, 18.0));

            if let Some(block) = self.blocks.get(&pane_id)
                && let Some((row, col)) =
                    crate::app::mouse_selection::pixel_to_grid_cell(block, mx, my, cell_w, cell_h)
            {
                self.selection_start_pt =
                    Some(akssora_terminal_screen::TerminalPoint::new(row, col));
            }

            let is_pane_drag_gesture = self.modifiers.control_key()
                || self
                    .blocks
                    .get(&pane_id)
                    .map(|b| my <= b.y + 24.0)
                    .unwrap_or(false);

            if is_pane_drag_gesture
                && let Some(active_tab_id) = self.tab_manager.active_id()
                && let Some(tree) = self.split_trees.get(&active_tab_id)
                && tree.all_leaf_ids().len() > 1
            {
                self.split_manager.pane_drag = Some(crate::app::split_state::PaneDragState {
                    source_pane_id: pane_id,
                    tab_id: active_tab_id,
                    start_x: mx,
                    start_y: my,
                    current_x: mx,
                    current_y: my,
                    hover_target: None,
                });
            }
        }
    }

    pub fn handle_test_mouse_release(&mut self) {
        self.split_manager.drag_state = None;
        if let Some(pane_drag) = self.split_manager.pane_drag.take() {
            let drag_dist = (pane_drag.current_x - pane_drag.start_x)
                .hypot(pane_drag.current_y - pane_drag.start_y);
            if drag_dist >= 10.0
                && let Some((target_id, zone, _)) = pane_drag.hover_target
            {
                self.reposition_pane(pane_drag.source_pane_id, target_id, zone);
            }
        }
        self.is_selecting_text = false;
        self.mouse_press_pos = None;
        self.mouse_press_pane = None;
        self.selection_start_pane = None;
        self.selection_start_pt = None;
    }
}

fn sidebar_width_needed(sidebar: &Sidebar) -> bool {
    sidebar.is_expanded
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }

        let window_size = PhysicalSize::new(self.config.window.width, self.config.window.height);
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

        let mut fonts = FontManager::new();
        fonts.set_size(self.config.font.size);
        let mut ui_fonts = FontManager::new();
        ui_fonts.set_size(self.config.ui_font_size);
        let glyphs = GlyphRenderer::new(&device, &queue, format);

        self.window = Some(window.clone());
        self.state = Some(WgpuState::new(
            surface, adapter, device, queue, config, glyphs,
        ));
        self.fonts = Some(fonts);
        self.ui_fonts = Some(ui_fonts);

        // Open initial default terminal tab (or restore persisted session state)
        if !self.restore_session_state() {
            self.open_terminal_tab();
        }

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
                self.handle_keyboard_input(event_loop, &event);
            }

            WindowEvent::Resized(new_size) => {
                if let Some(win) = self.window.as_ref() {
                    self.header.is_maximized = win.is_maximized();
                }
                if let Some(state) = self.state.as_mut() {
                    state.resize(new_size.width, new_size.height);
                }
                self.layout_blocks();
            }

            WindowEvent::MouseWheel { delta, .. } => {
                self.handle_mouse_wheel(delta);
            }

            WindowEvent::CursorMoved { position, .. } => {
                self.handle_cursor_moved(position);
            }

            WindowEvent::MouseInput { state, button, .. } => {
                self.handle_mouse_input(event_loop, state, button);
            }

            WindowEvent::RedrawRequested => {
                let mut should_exit = false;

                // Pump all terminal sessions with bounded batches
                let mut any_output = false;
                for block in self.blocks.values_mut() {
                    if block.shell.pump() {
                        any_output = true;
                    }
                }

                // Check closed terminal sessions
                let closed_ids: Vec<u64> = self
                    .blocks
                    .iter()
                    .filter(|(_, b)| b.shell.is_closed())
                    .map(|(id, _)| *id)
                    .collect();
                for id in closed_ids {
                    let is_split = self
                        .split_trees
                        .values()
                        .any(|t| t.contains_leaf(id) && t.all_leaf_ids().len() > 1);
                    if is_split {
                        self.close_split_pane(id);
                    } else {
                        if let Some(pos) = self.tab_manager.tabs().iter().position(|t| t.id == id) {
                            self.close_tab(pos);
                        } else {
                            let active = self.tab_manager.active_index();
                            self.close_tab(active);
                        }
                        if self.tab_manager.is_empty() {
                            should_exit = true;
                        }
                    }
                }

                if should_exit {
                    event_loop.exit();
                    return;
                }

                if let Some(win) = self.window.as_ref() {
                    self.header.is_maximized = win.is_maximized();
                }
                self.header.is_settings_active = self
                    .tab_manager
                    .active_tab()
                    .map(|t| matches!(t.kind, akssora_terminal_ui::TabKind::Settings))
                    .unwrap_or(false);

                let block_ids = self.active_tab_block_ids();
                let drop_indicator = self.split_manager.pane_drag.as_ref().and_then(|pd| {
                    pd.hover_target
                        .map(|(_, zone, (x, y, w, h))| zone.preview_rect(x, y, w, h))
                });
                let session_items =
                    if self.config.tab_layout.show_sidebar() && self.sidebar.is_expanded {
                        Some(self.collect_session_items())
                    } else {
                        None
                    };

                if let (Some(state), Some(fonts), Some(ui_fonts)) = (
                    self.state.as_mut(),
                    self.fonts.as_ref(),
                    self.ui_fonts.as_ref(),
                ) {
                    let blocks: Vec<&Block> = block_ids
                        .into_iter()
                        .filter_map(|id| self.blocks.get(&id))
                        .collect();

                    let conflicts = self.config.keybindings.find_conflicts();
                    state.render(
                        fonts,
                        ui_fonts,
                        &self.theme,
                        &self.config,
                        self.active_settings_tab,
                        &self.settings_search_query,
                        &conflicts,
                        &self.header,
                        &blocks,
                        &self.sidebar,
                        &self.tab_manager,
                        self.config.tab_layout,
                        &self.split_trees,
                        self.split_manager.active_pane_id,
                        self.active_tooltip.as_ref(),
                        self.hovered_setting_action,
                        self.settings_layout_dropdown_open,
                        self.prompt_confirm_exit,
                        drop_indicator,
                        session_items.as_deref(),
                        self.context_menu.as_ref(),
                    );
                    self.last_render_time = Some(std::time::Instant::now());
                }

                let _ = any_output;
                if let Some(window) = self.window.as_ref() {
                    window.request_redraw();
                }
            }

            _ => {}
        }
    }
}
