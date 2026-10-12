use akssora_terminal_ui::{HeaderButton, HeaderItem, SettingsAction, SettingsView, TabKind};
use winit::event::{ElementState, MouseButton};
use winit::event_loop::ActiveEventLoop;

use super::App;

// INFO: Mouse click dispatching across modals, window controls, sidebar tree items, and panes.
impl App {
    pub(crate) fn handle_mouse_input(
        &mut self,
        event_loop: &ActiveEventLoop,
        state: ElementState,
        button: MouseButton,
    ) {
        let (mx, my) = self.cursor_pos;
        let (win_w, win_h) = self.state.as_ref().map(|s| s.size()).unwrap_or((0, 0));
        let w = win_w as f32;
        let h = win_h as f32;
        let show_tabs = self.config.tab_layout.includes_title_bar();
        let show_toggle = self.config.tab_layout.show_sidebar_toggle();
        let show_sidebar = self.config.tab_layout.show_sidebar();
        let top_h = self.header.total_height(show_tabs);
        let content_start_y = top_h + 8.0;
        let sb_w = self.sidebar.effective_width(show_sidebar);

        // Right-click: trigger terminal context menu
        if button == MouseButton::Right {
            if state == ElementState::Pressed {
                if let Some(menu) = &self.context_menu {
                    if menu.contains(mx, my) {
                        return;
                    }
                    self.context_menu = None;
                }

                if self.pane_at(mx, my).is_some() {
                    self.focus_pane_at(mx, my);
                    self.open_context_menu(mx, my);
                }
            }
            return;
        }

        if button != MouseButton::Left {
            return;
        }

        // Left-click on context menu
        if let Some(menu) = &self.context_menu
            && state == ElementState::Pressed
        {
            if let Some(action) = menu.hit_test(mx, my) {
                self.handle_context_menu_action(action);
                return;
            } else {
                self.context_menu = None;
                if let Some(win) = self.window.as_ref() {
                    win.request_redraw();
                }
            }
        }

        if state == ElementState::Pressed {
            // 1. Exit confirmation modal click interaction
            if self.prompt_confirm_exit {
                let modal_w = 460.0f32.min(w - 40.0);
                let modal_h = 160.0f32;
                let mx_box = (w - modal_w) / 2.0;
                let my_box = (h - modal_h) / 2.0;
                let btn_y = my_box + modal_h - 48.0;
                let btn_h = 32.0;
                let c_x = mx_box + 24.0;
                let c_w = 170.0;
                let can_x = c_x + c_w + 16.0;
                let can_w = 150.0;

                if mx >= c_x && mx < c_x + c_w && my >= btn_y && my < btn_y + btn_h {
                    self.close_all_tabs();
                    event_loop.exit();
                    return;
                } else if (mx >= can_x && mx < can_x + can_w && my >= btn_y && my < btn_y + btn_h)
                    || mx < mx_box
                    || mx >= mx_box + modal_w
                    || my < my_box
                    || my >= my_box + modal_h
                {
                    self.prompt_confirm_exit = false;
                    if let Some(win) = self.window.as_ref() {
                        win.request_redraw();
                    }
                    return;
                }
                return;
            }

            // 2. Header and titlebar interaction targets
            match self.header.hit_test_item_ex(
                mx,
                my,
                w,
                self.tab_manager.len(),
                show_tabs,
                show_toggle,
                true,
            ) {
                Some(HeaderItem::SidebarToggle) => {
                    self.header.is_sidebar_toggle_active = true;
                    self.toggle_sidebar();
                    if let Some(win) = self.window.as_ref() {
                        win.request_redraw();
                    }
                    return;
                }
                Some(HeaderItem::WindowButton(HeaderButton::Minimize)) => {
                    if let Some(win) = self.window.as_ref() {
                        win.set_minimized(true);
                    }
                    return;
                }
                Some(HeaderItem::WindowButton(HeaderButton::Maximize)) => {
                    if let Some(win) = self.window.as_ref() {
                        let next_state = !win.is_maximized();
                        win.set_maximized(next_state);
                        self.header.is_maximized = next_state;
                        win.request_redraw();
                    }
                    return;
                }
                Some(HeaderItem::WindowButton(HeaderButton::Close)) => {
                    event_loop.exit();
                    return;
                }
                Some(HeaderItem::Settings) => {
                    self.header.is_settings_active = true;
                    self.open_settings_tab();
                    if let Some(win) = self.window.as_ref() {
                        win.request_redraw();
                    }
                    return;
                }
                Some(HeaderItem::Tab(idx)) => {
                    self.tab_manager.select(idx);
                    if let Some(active_id) = self.tab_manager.active_id()
                        && let Some(tree) = self.split_trees.get(&active_id)
                        && !tree.contains_leaf(self.split_manager.active_pane_id.unwrap_or(0))
                        && let Some(first_leaf) = tree.all_leaf_ids().first().copied()
                    {
                        self.split_manager.focus_pane(first_leaf);
                    }
                    self.layout_blocks();
                    if let Some(win) = self.window.as_ref() {
                        win.request_redraw();
                    }
                    return;
                }
                Some(HeaderItem::TabClose(idx)) => {
                    self.close_tab(idx);
                    if let Some(win) = self.window.as_ref() {
                        win.request_redraw();
                    }
                    return;
                }
                Some(HeaderItem::NewTab) => {
                    self.open_terminal_tab();
                    if let Some(win) = self.window.as_ref() {
                        win.request_redraw();
                    }
                    return;
                }
                None => {
                    if self.header.in_drag_region_with_tabs(
                        mx,
                        my,
                        w,
                        self.tab_manager.len(),
                        show_tabs,
                    ) && let Some(win) = self.window.as_ref()
                    {
                        let _ = win.drag_window();
                        return;
                    }
                }
            }

            // 3. Sidebar resize dragging and interaction targets
            if show_sidebar && self.sidebar.is_expanded {
                if self.sidebar.hovered_edge {
                    self.sidebar.is_resizing = true;
                    return;
                }
                if mx <= sb_w {
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
                    if let Some(t) = target {
                        self.handle_sidebar_target(t);
                        return;
                    }
                }
            }

            // 4. Settings tab options
            let is_settings = self
                .tab_manager
                .active_tab()
                .map(|t| matches!(t.kind, TabKind::Settings))
                .unwrap_or(false);
            let cell_h = self.fonts.as_ref().map(|f| f.cell_h).unwrap_or(18.0);
            if is_settings
                && let Some(action) = SettingsView::hit_test_action_tab(
                    mx,
                    my,
                    sb_w,
                    top_h,
                    (w - sb_w).max(0.0),
                    cell_h,
                    self.active_settings_tab,
                    akssora_terminal_config::Action::ALL,
                )
            {
                match action {
                    SettingsAction::SelectTab(tab) => {
                        self.active_settings_tab = tab;
                    }
                    SettingsAction::ToggleLayoutDropdown => {
                        self.settings_layout_dropdown_open = !self.settings_layout_dropdown_open;
                    }
                    SettingsAction::SelectLayout(opt) => {
                        self.set_tab_layout(opt);
                        self.settings_layout_dropdown_open = false;
                    }
                    SettingsAction::DecreaseFontSize => self.decrease_font_size(),
                    SettingsAction::IncreaseFontSize => self.increase_font_size(),
                    SettingsAction::ResetFontSize => self.reset_font_size(),
                    SettingsAction::SelectThemeMonochrome => {
                        self.config.theme_name = "Akssora Pure Black".to_string();
                        self.theme = akssora_terminal_config::Theme::monochrome_black();
                        self.config.theme = self.theme.clone();
                        let _ = self.config.save();
                    }
                    SettingsAction::SelectThemeSlate => {
                        self.config.theme_name = "Slate Dark".to_string();
                        self.theme = akssora_terminal_config::Theme::slate_dark();
                        self.config.theme = self.theme.clone();
                        let _ = self.config.save();
                    }
                    SettingsAction::ToggleCursorBlink => {
                        self.config.cursor_blink = !self.config.cursor_blink;
                        let _ = self.config.save();
                    }
                    SettingsAction::CycleCursorStyle => {
                        self.config.cursor_style = match self.config.cursor_style.as_str() {
                            "Block" => "Beam".to_string(),
                            "Beam" => "Underline".to_string(),
                            _ => "Block".to_string(),
                        };
                        let _ = self.config.save();
                    }
                    SettingsAction::CycleTabSize => {
                        self.config.tab_size = match self.config.tab_size {
                            2 => 4,
                            4 => 8,
                            _ => 2,
                        };
                        let _ = self.config.save();
                    }
                    SettingsAction::ToggleLineNumbers => {
                        self.config.line_numbers = !self.config.line_numbers;
                        let _ = self.config.save();
                    }
                    SettingsAction::KeybindingSearchClick => {
                        // Keybinding search box clicked
                    }
                    SettingsAction::KeybindingResetAll => {
                        self.config.keybindings = akssora_terminal_config::Keybindings::default();
                    }
                    SettingsAction::KeybindingRebind(_) => {
                        // Action clicked for remapping
                    }
                }
                if let Some(win) = self.window.as_ref() {
                    win.request_redraw();
                }
                return;
            }

            // 5. Split divider drag start check
            if let Some(active_tab_id) = self.tab_manager.active_id()
                && let Some(tree) = self.split_trees.get(&active_tab_id)
            {
                let content_x = sb_w;
                let content_w = (w - sb_w).max(0.0);
                let content_y = top_h;
                let content_h = (h - top_h).max(0.0);
                if let Some((dir, path, cw, ch)) = self
                    .split_manager
                    .hit_test_divider(tree, mx, my, content_x, content_y, content_w, content_h)
                {
                    self.split_manager.drag_state =
                        Some(crate::app::split_state::DividerDragState {
                            tab_id: active_tab_id,
                            path,
                            direction: dir,
                            start_x: mx,
                            start_y: my,
                            container_w: cw,
                            container_h: ch,
                        });
                    return;
                }
            }

            // 6. Focus pane under cursor, start selection tracking or pane drag
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
                    && let Some((row, col)) = crate::app::mouse_selection::pixel_to_grid_cell(
                        block, mx, my, cell_w, cell_h,
                    )
                {
                    self.selection_start_pt =
                        Some(akssora_terminal_screen::TerminalPoint::new(row, col));
                }

                // Only start pane drag rearrangement if Ctrl modifier is held or clicking the top pane header/drag handle zone.
                // Ordinary mouse click-and-drag inside the terminal text area is reserved for text selection.
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

            // 7. Terminal block scrollbar interaction
            if let Some(block) = self.active_block_mut() {
                let total_lines = block.shell.grid().scrollback_len() + block.shell.grid().rows();
                let visible_lines = block.shell.grid().rows();
                let current_scroll = block.shell.grid().scroll_offset() as f32;

                if block.scrollbar.hit_test(mx, my) {
                    if block.scrollbar.hit_test_thumb(
                        mx,
                        my,
                        total_lines,
                        visible_lines,
                        current_scroll,
                    ) {
                        block.scrollbar.is_dragging = true;
                        block.scrollbar.drag_start_y = my;
                        block.scrollbar.drag_start_offset = current_scroll;
                    } else {
                        let target = block
                            .scrollbar
                            .offset_from_y(my, total_lines, visible_lines);
                        block
                            .shell
                            .grid_mut()
                            .set_scroll_offset(target.round() as usize);
                        block.scrollbar.is_dragging = true;
                        block.scrollbar.drag_start_y = my;
                        block.scrollbar.drag_start_offset = target;
                    }
                    if let Some(window) = self.window.as_ref() {
                        window.request_redraw();
                    }
                }
            }
        } else if state == ElementState::Released {
            self.split_manager.drag_state = None;

            // Handle pane drag drop completion
            if let Some(pane_drag) = self.split_manager.pane_drag.take() {
                let drag_dist = (pane_drag.current_x - pane_drag.start_x)
                    .hypot(pane_drag.current_y - pane_drag.start_y);
                if drag_dist >= 10.0
                    && let Some((target_id, zone, _)) = pane_drag.hover_target
                {
                    self.reposition_pane(pane_drag.source_pane_id, target_id, zone);
                }
                if let Some(win) = self.window.as_ref() {
                    win.request_redraw();
                }
            }

            if self.sidebar.is_resizing {
                self.sidebar.is_resizing = false;
                self.layout_blocks();
                let _ = self.config.save();
                if let Some(win) = self.window.as_ref() {
                    win.request_redraw();
                }
            }

            // Handle selection completion vs click
            if self.is_selecting_text {
                self.is_selecting_text = false;
                self.mouse_press_pos = None;
                self.mouse_press_pane = None;
                self.selection_start_pane = None;
                self.selection_start_pt = None;
            } else if let Some(press_pane) = self.mouse_press_pane.take()
                && let Some((px, py)) = self.mouse_press_pos.take()
                && (mx - px).hypot(my - py) < 5.0
            {
                self.handle_terminal_click_in_pane(press_pane, mx, my);
            }

            if self.header.is_sidebar_toggle_active {
                self.header.is_sidebar_toggle_active = false;
                if let Some(window) = self.window.as_ref() {
                    window.request_redraw();
                }
            }
            if self.header.is_settings_active {
                self.header.is_settings_active = false;
                if let Some(window) = self.window.as_ref() {
                    window.request_redraw();
                }
            }
            if let Some(block) = self.active_block_mut()
                && block.scrollbar.is_dragging
            {
                block.scrollbar.is_dragging = false;
                if let Some(window) = self.window.as_ref() {
                    window.request_redraw();
                }
            }
        }
    }
}
