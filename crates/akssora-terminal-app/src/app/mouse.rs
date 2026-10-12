use akssora_terminal_input::scroll_delta_to_lines;
use akssora_terminal_ui::{HeaderItem, Scrollbar, SettingsView, Sidebar, TabKind};
use winit::dpi::PhysicalPosition;
use winit::event::MouseScrollDelta;
use winit::window::CursorIcon;

use super::App;

// INFO: Mouse event processing, cursor tracking, scrollbar dragging, and click dispatch.
impl App {
    pub(crate) fn handle_mouse_wheel(&mut self, delta: MouseScrollDelta) {
        let cell_h = self.fonts.as_ref().map(|f| f.cell_h).unwrap_or(16.0);
        let lines = scroll_delta_to_lines(delta, cell_h);
        let (mx, my) = self.cursor_pos;
        let pane_id = self
            .pane_at(mx, my)
            .map(|(id, _, _, _, _)| id)
            .or_else(|| self.active_pane_id());

        if let Some(id) = pane_id
            && let Some(block) = self.blocks.get_mut(&id)
        {
            if lines > 0.0 {
                block.shell.scroll_up(lines.ceil().max(1.0) as usize);
            } else if lines < 0.0 {
                block.shell.scroll_down((-lines).ceil().max(1.0) as usize);
            }
            if let Some(window) = self.window.as_ref() {
                window.request_redraw();
            }
        }
    }

    pub(crate) fn handle_cursor_moved(&mut self, position: PhysicalPosition<f64>) {
        self.cursor_pos = (position.x as f32, position.y as f32);
        let (mx, my) = self.cursor_pos;
        let (win_w, win_h) = self.state.as_ref().map(|s| s.size()).unwrap_or((0, 0));
        let w = win_w as f32;
        let h = win_h as f32;

        let show_tabs = self.config.tab_layout.includes_title_bar();
        let show_toggle = self.config.tab_layout.show_sidebar_toggle();

        let item = self.header.hit_test_item_ex(
            mx,
            my,
            w,
            self.tab_manager.len(),
            show_tabs,
            show_toggle,
            true,
        );
        self.header.hovered_item = item;
        self.header.hovered = match item {
            Some(HeaderItem::WindowButton(b)) => Some(b),
            _ => None,
        };

        let show_sidebar = self.config.tab_layout.show_sidebar();
        let top_h = self.header.total_height(show_tabs);
        let content_start_y = top_h + 8.0;
        let sb_w = self.sidebar.effective_width(show_sidebar);

        // Update context menu hover
        if let Some(menu) = self.context_menu.as_mut() {
            menu.update_hover(mx, my);
            if let Some(win) = self.window.as_ref() {
                win.request_redraw();
            }
        }

        // Sidebar edge hover and resizing (debounced by cell width)
        if show_sidebar && self.sidebar.is_expanded {
            let on_edge = (mx - sb_w).abs() <= 5.0 && my >= top_h;
            self.sidebar.hovered_edge = on_edge;

            if self.sidebar.is_resizing {
                let clamped_w = mx.clamp(Sidebar::MIN_WIDTH, Sidebar::MAX_WIDTH);
                let diff = (clamped_w - self.sidebar.width).abs();
                self.sidebar.width = clamped_w;
                self.sidebar.saved_width = clamped_w;
                self.config.sidebar_width = clamped_w;
                let cell_w = self.fonts.as_ref().map(|f| f.cell_w).unwrap_or(9.0);
                let now = std::time::Instant::now();
                let time_ok = self
                    .last_layout_time
                    .map(|t| now.duration_since(t).as_millis() >= 16)
                    .unwrap_or(true);
                if diff >= cell_w && time_ok {
                    self.last_layout_time = Some(now);
                    self.layout_blocks();
                }
            }
        } else {
            self.sidebar.hovered_edge = false;
        }

        if show_sidebar && self.sidebar.is_expanded && mx <= sb_w && !self.sidebar.hovered_edge {
            self.sidebar.hovered_toggle = false;
            let items = self.collect_session_items();
            let target = self
                .sidebar
                .hit_test_sessions(mx, my, &items, content_start_y)
                .or_else(|| {
                    self.sidebar.hit_test(
                        mx,
                        my,
                        self.tab_manager.tabs(),
                        &self.split_trees,
                        content_start_y,
                    )
                });
            match target {
                Some(akssora_terminal_ui::SidebarTarget::Tab(idx)) => {
                    self.sidebar.hovered_tab = Some(idx);
                    self.sidebar.hovered_close = None;
                    self.sidebar.hovered_pane = None;
                    self.sidebar.hovered_pane_close = None;
                    self.sidebar.hovered_collapse_tab = None;
                    self.sidebar.hovered_new_tab = false;
                }
                Some(akssora_terminal_ui::SidebarTarget::TabCollapse(idx)) => {
                    self.sidebar.hovered_tab = Some(idx);
                    self.sidebar.hovered_close = None;
                    self.sidebar.hovered_pane = None;
                    self.sidebar.hovered_pane_close = None;
                    self.sidebar.hovered_collapse_tab = Some(idx);
                    self.sidebar.hovered_new_tab = false;
                }
                Some(akssora_terminal_ui::SidebarTarget::TabClose(idx)) => {
                    self.sidebar.hovered_tab = Some(idx);
                    self.sidebar.hovered_close = Some(idx);
                    self.sidebar.hovered_pane = None;
                    self.sidebar.hovered_pane_close = None;
                    self.sidebar.hovered_collapse_tab = None;
                    self.sidebar.hovered_new_tab = false;
                }
                Some(akssora_terminal_ui::SidebarTarget::SplitPane { tab_idx, pane_id }) => {
                    self.sidebar.hovered_tab = Some(tab_idx);
                    self.sidebar.hovered_close = None;
                    self.sidebar.hovered_pane = Some(pane_id);
                    self.sidebar.hovered_pane_close = None;
                    self.sidebar.hovered_collapse_tab = None;
                    self.sidebar.hovered_new_tab = false;
                }
                Some(akssora_terminal_ui::SidebarTarget::SplitClose { tab_idx, pane_id }) => {
                    self.sidebar.hovered_tab = Some(tab_idx);
                    self.sidebar.hovered_close = None;
                    self.sidebar.hovered_pane = Some(pane_id);
                    self.sidebar.hovered_pane_close = Some(pane_id);
                    self.sidebar.hovered_collapse_tab = None;
                    self.sidebar.hovered_new_tab = false;
                }
                Some(akssora_terminal_ui::SidebarTarget::Session { tab_idx, pane_id }) => {
                    self.sidebar.hovered_tab = Some(tab_idx);
                    self.sidebar.hovered_close = None;
                    self.sidebar.hovered_pane = Some(pane_id);
                    self.sidebar.hovered_pane_close = None;
                    self.sidebar.hovered_collapse_tab = None;
                    self.sidebar.hovered_new_tab = false;
                }
                Some(akssora_terminal_ui::SidebarTarget::SessionClose { tab_idx, pane_id }) => {
                    self.sidebar.hovered_tab = Some(tab_idx);
                    self.sidebar.hovered_close = None;
                    self.sidebar.hovered_pane = Some(pane_id);
                    self.sidebar.hovered_pane_close = Some(pane_id);
                    self.sidebar.hovered_collapse_tab = None;
                    self.sidebar.hovered_new_tab = false;
                }
                Some(akssora_terminal_ui::SidebarTarget::ProjectCollapse(_)) => {
                    self.sidebar.hovered_tab = None;
                    self.sidebar.hovered_close = None;
                    self.sidebar.hovered_pane = None;
                    self.sidebar.hovered_pane_close = None;
                    self.sidebar.hovered_collapse_tab = None;
                    self.sidebar.hovered_new_tab = false;
                }
                Some(akssora_terminal_ui::SidebarTarget::NewTab) => {
                    self.sidebar.hovered_tab = None;
                    self.sidebar.hovered_close = None;
                    self.sidebar.hovered_pane = None;
                    self.sidebar.hovered_pane_close = None;
                    self.sidebar.hovered_collapse_tab = None;
                    self.sidebar.hovered_new_tab = true;
                }
                None => {
                    self.sidebar.hovered_tab = None;
                    self.sidebar.hovered_close = None;
                    self.sidebar.hovered_pane = None;
                    self.sidebar.hovered_pane_close = None;
                    self.sidebar.hovered_collapse_tab = None;
                    self.sidebar.hovered_new_tab = false;
                }
            }
        } else {
            self.sidebar.hovered_toggle = false;
            self.sidebar.hovered_tab = None;
            self.sidebar.hovered_close = None;
            self.sidebar.hovered_pane = None;
            self.sidebar.hovered_pane_close = None;
            self.sidebar.hovered_collapse_tab = None;
            self.sidebar.hovered_new_tab = false;
        }

        // Hit test settings actions if settings tab is active
        let is_settings = self
            .tab_manager
            .active_tab()
            .map(|t| matches!(t.kind, TabKind::Settings))
            .unwrap_or(false);
        if is_settings {
            let cell_h = self.fonts.as_ref().map(|f| f.cell_h).unwrap_or(18.0);
            self.hovered_setting_action = SettingsView::hit_test_action_tab(
                mx,
                my,
                sb_w,
                top_h,
                (w - sb_w).max(0.0),
                cell_h,
                self.active_settings_tab,
                akssora_terminal_config::Action::ALL,
            );
        } else {
            self.hovered_setting_action = None;
        }

        // Update active tooltip
        self.update_tooltips(mx, my, w, h);

        // Handle divider dragging
        if let Some(drag) = self.split_manager.drag_state.clone() {
            let active_tab_id = drag.tab_id;
            if let Some(tree) = self.split_trees.get_mut(&active_tab_id) {
                let delta = match drag.direction {
                    akssora_terminal_ui::SplitDirection::Vertical => {
                        (mx - drag.start_x) / drag.container_w.max(1.0)
                    }
                    akssora_terminal_ui::SplitDirection::Horizontal => {
                        (my - drag.start_y) / drag.container_h.max(1.0)
                    }
                };
                tree.adjust_ratio(&drag.path, delta);
                self.split_manager.drag_state = Some(crate::app::split_state::DividerDragState {
                    tab_id: drag.tab_id,
                    path: drag.path,
                    direction: drag.direction,
                    start_x: mx,
                    start_y: my,
                    container_w: drag.container_w,
                    container_h: drag.container_h,
                });
                self.layout_blocks();
            }
        }

        // Handle scrollbar dragging and hover
        if let Some(block) = self.active_block_mut() {
            let total_lines = block.shell.grid().scrollback_len() + block.shell.grid().rows();
            let visible_lines = block.shell.grid().rows();

            if block.scrollbar.is_dragging {
                let max_scroll = block.shell.grid().scrollback_len() as f32;
                let ratio = (visible_lines as f32 / total_lines.max(1) as f32).clamp(0.05, 1.0);
                let thumb_h = (block.scrollbar.height * ratio)
                    .max(Scrollbar::MIN_THUMB_HEIGHT)
                    .min(block.scrollbar.height);
                let travel = block.scrollbar.height - thumb_h;
                if travel > 0.0 {
                    let delta_y = my - block.scrollbar.drag_start_y;
                    let delta_offset = -(delta_y / travel) * max_scroll;
                    let new_offset =
                        (block.scrollbar.drag_start_offset + delta_offset).clamp(0.0, max_scroll);
                    block
                        .shell
                        .grid_mut()
                        .set_scroll_offset(new_offset.round() as usize);
                }
            } else {
                block.scrollbar.is_hovered = block.scrollbar.hit_test(mx, my);
            }
        }

        // Handle mouse text selection drag
        if let Some((px, py)) = self.mouse_press_pos {
            let dist = (mx - px).hypot(my - py);
            let not_other_drag = !self.sidebar.is_resizing
                && self.split_manager.drag_state.is_none()
                && self.split_manager.pane_drag.is_none()
                && self
                    .active_block_mut()
                    .map(|b| !b.scrollbar.is_dragging)
                    .unwrap_or(true);

            if dist >= 4.0 && not_other_drag {
                self.is_selecting_text = true;
                self.update_mouse_selection_drag(mx, my);
            }
        }

        // Handle pane drag-and-drop rearrangement
        let target_hit = if self.split_manager.pane_drag.is_some() {
            self.pane_at(mx, my)
        } else {
            None
        };

        if let Some(pane_drag) = self.split_manager.pane_drag.as_mut() {
            pane_drag.current_x = mx;
            pane_drag.current_y = my;

            // Find pane under cursor (other than source)
            pane_drag.hover_target = None;
            if let Some((target_id, px, py, pw, ph)) = target_hit
                && target_id != pane_drag.source_pane_id
            {
                let zone = akssora_terminal_ui::DropZone::from_point(mx, my, px, py, pw, ph);
                pane_drag.hover_target = Some((target_id, zone, (px, py, pw, ph)));
            }
        }

        // Determine and apply appropriate cursor icon for resize edges, dividers, and pane dragging
        let mut desired_cursor = CursorIcon::Default;
        if self.sidebar.is_resizing || self.sidebar.hovered_edge {
            desired_cursor = CursorIcon::ColResize;
        } else if self.split_manager.pane_drag.is_some() {
            desired_cursor = CursorIcon::Grabbing;
        } else if let Some(drag) = &self.split_manager.drag_state {
            desired_cursor = match drag.direction {
                akssora_terminal_ui::SplitDirection::Vertical => CursorIcon::ColResize,
                akssora_terminal_ui::SplitDirection::Horizontal => CursorIcon::RowResize,
            };
        } else if let Some(active_tab_id) = self.tab_manager.active_id()
            && let Some(tree) = self.split_trees.get(&active_tab_id)
        {
            let content_x = sb_w;
            let content_w = (w - sb_w).max(0.0);
            let content_y = top_h;
            let content_h = (h - top_h).max(0.0);
            if let Some((dir, _, _, _)) = self
                .split_manager
                .hit_test_divider(tree, mx, my, content_x, content_y, content_w, content_h)
            {
                desired_cursor = match dir {
                    akssora_terminal_ui::SplitDirection::Vertical => CursorIcon::ColResize,
                    akssora_terminal_ui::SplitDirection::Horizontal => CursorIcon::RowResize,
                };
            } else if self.pane_at(mx, my).is_some() {
                desired_cursor = CursorIcon::Default;
            }
        } else if self.pane_at(mx, my).is_some() {
            desired_cursor = CursorIcon::Default;
        }

        if let Some(window) = self.window.as_ref() {
            window.set_cursor(desired_cursor);
            window.request_redraw();
        }
    }
}
