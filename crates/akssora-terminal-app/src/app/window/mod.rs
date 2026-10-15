use super::{App, UnsavedCloseAction};
use crate::state::WgpuState;
use akssora_terminal_renderer::{FontManager, GlyphRenderer};
use akssora_terminal_ui::Block;
use std::sync::Arc;
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::{Modifiers, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::window::{Window, WindowId};

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

                // Step F: UI thread non-blocking poll of background diff worker
                self.poll_diff_worker();

                // Pump all terminal sessions with bounded batches
                let mut any_output = false;
                for block in self.blocks.values_mut() {
                    if block.shell.pump() {
                        any_output = true;
                    }
                }
                if any_output {
                    self.sync_code_review_with_active_terminal();
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

                self.sync_explorer_with_active_terminal();
                if self
                    .explorer
                    .tree
                    .poll_filesystem_watch(std::time::Duration::from_millis(1500))
                    || self.explorer.git_status.is_none()
                {
                    self.refresh_git_status();
                }
                let file_tabs = self.collect_editor_file_tabs();
                let is_code_review = self.is_code_review_open();
                let is_editor = self
                    .tab_manager
                    .active_tab()
                    .map(|t| matches!(t.kind, akssora_terminal_ui::TabKind::Editor { .. }))
                    .unwrap_or(false);
                let active_editor = if is_editor {
                    if let Some(p) = &self.editor_active_file {
                        self.editor_documents.get(p)
                    } else if let Some(active_id) = self.tab_manager.active_id() {
                        self.editors.get(&active_id)
                    } else {
                        None
                    }
                } else if is_code_review {
                    self.code_review
                        .active_editing_file
                        .as_ref()
                        .and_then(|p| self.editor_documents.get(p))
                        .or_else(|| {
                            self.code_review
                                .diff
                                .files
                                .get(self.code_review.selected_file_idx)
                                .map(|f| self.code_review.repo_root.join(&f.path))
                                .and_then(|p| self.editor_documents.get(&p))
                        })
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
                        self.settings_line_number_dropdown_open,
                        self.prompt_confirm_exit,
                        drop_indicator,
                        session_items.as_deref(),
                        self.context_menu.as_ref(),
                        Some(&self.explorer),
                        &self.editor_documents,
                        active_editor,
                        &file_tabs,
                        self.editor_hovered_tab,
                        self.editor_hovered_close,
                        self.editor_hovered_toolbar,
                        Some(&self.quick_search),
                        Some((&self.editor_v_scrollbar, &self.editor_h_scrollbar)),
                        self.editor_hovered_gutter,
                        self.unsaved_prompt_target
                            .as_ref()
                            .map(|target| match target {
                                UnsavedCloseAction::CloseSingle(idx) => self
                                    .editor_open_files
                                    .get(*idx)
                                    .and_then(|p| p.file_name())
                                    .map(|f| f.to_str().unwrap_or("Untitled"))
                                    .unwrap_or("Untitled"),
                                UnsavedCloseAction::CloseOthers(p) => {
                                    p.file_name().and_then(|f| f.to_str()).unwrap_or("file")
                                }
                                UnsavedCloseAction::CloseAll => "all modified files",
                            }),
                        Some(&self.code_review),
                        self.hovered_code_review_action.as_ref(),
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
