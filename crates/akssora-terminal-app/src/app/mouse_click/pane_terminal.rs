use crate::app::App;

impl App {
    pub(crate) fn handle_pane_terminal_pressed(
        &mut self,
        mx: f32,
        my: f32,
        w: f32,
        h: f32,
        top_h: f32,
        sb_w: f32,
    ) {
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
                self.split_manager.drag_state = Some(crate::app::split_state::DividerDragState {
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
                && let Some((row, col)) =
                    crate::app::mouse_selection::pixel_to_grid_cell(block, mx, my, cell_w, cell_h)
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
    }

    pub(crate) fn handle_drag_release(&mut self, mx: f32, my: f32, w: f32, sb_w: f32) {
        if let Some(from_idx) = self.editor_dragging_tab.take() {
            let exp_w = self.explorer.effective_width();
            let ws_x = sb_w + exp_w;
            let ws_w = (w - ws_x).max(0.0);
            if let Some(target_idx) = akssora_terminal_ui::EditorTabBar::tab_at_x(
                mx,
                ws_x,
                ws_w,
                self.editor_open_files.len(),
            ) && target_idx != from_idx
            {
                self.reorder_editor_file_tab(from_idx, target_idx);
            }
        }

        if !self.explorer.tree.dragged_paths.is_empty() {
            let _ = self.explorer.tree.finish_drag();
            if let Some(win) = self.window.as_ref() {
                win.request_redraw();
            }
        }

        if let Some(doc) = self.active_editor_mut()
            && doc.selection_anchor == Some(doc.cursor)
        {
            doc.selection_anchor = None;
        }

        self.code_review.is_resizing_sidebar = false;
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

        if self.explorer.is_resizing {
            self.explorer.is_resizing = false;
            self.layout_blocks();
            if let Some(win) = self.window.as_ref() {
                win.request_redraw();
            }
        }

        // Handle selection completion vs click
        let press_pos = self.mouse_press_pos.take();
        let press_pane = self.mouse_press_pane.take();
        if self.is_selecting_text {
            self.is_selecting_text = false;
            self.selection_start_pane = None;
            self.selection_start_pt = None;
        } else if let Some(pane) = press_pane
            && let Some((px, py)) = press_pos
            && (mx - px).hypot(my - py) < 5.0
        {
            self.handle_terminal_click_in_pane(pane, mx, my);
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
        if self.editor_v_scrollbar.is_dragging || self.editor_h_scrollbar.is_dragging {
            self.editor_v_scrollbar.is_dragging = false;
            self.editor_h_scrollbar.is_dragging = false;
            if let Some(window) = self.window.as_ref() {
                window.request_redraw();
            }
        }
        if self.code_review.is_dragging_diff_scrollbar {
            self.code_review.is_dragging_diff_scrollbar = false;
            if let Some(window) = self.window.as_ref() {
                window.request_redraw();
            }
        }
    }
}
