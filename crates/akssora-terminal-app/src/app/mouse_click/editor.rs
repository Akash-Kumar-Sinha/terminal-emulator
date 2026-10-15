use crate::app::App;

impl App {
    pub(crate) fn handle_editor_workspace_clicks(
        &mut self,
        mx: f32,
        my: f32,
        w: f32,
        h: f32,
        top_h: f32,
        sb_w: f32,
    ) -> bool {
        // INFO: Editor workspace and file-tab bar interaction
        let is_code_review = self.is_code_review_open();
        let exp_w = self.explorer.effective_width();
        let ws_x = sb_w + exp_w;
        let ws_w = (w - ws_x).max(0.0);
        let cell_w = self.fonts.as_ref().map(|f| f.cell_w).unwrap_or(9.0);
        let cell_h = self.fonts.as_ref().map(|f| f.cell_h).unwrap_or(18.0);
        let tab_bar_h = if !self.editor_open_files.is_empty() {
            akssora_terminal_ui::EditorTabBar::HEIGHT
        } else {
            0.0
        };

        // INFO: Prioritize Code Review panel clicks when Code Review tab is active.
        if is_code_review && self.handle_code_review_click(mx, my, w, h, top_h, ws_x) {
            return true;
        }

        // INFO: Handle dedicated file-tab bar clicks.
        if !is_code_review
            && tab_bar_h > 0.0
            && mx >= ws_x
            && my >= top_h
            && my < top_h + tab_bar_h
            && let Some(target) = akssora_terminal_ui::EditorTabBar::hit_test(
                mx,
                my,
                ws_x,
                top_h,
                ws_w,
                self.editor_open_files.len(),
            )
        {
            match target {
                akssora_terminal_ui::EditorTabTarget::Tab(idx) => {
                    self.select_editor_file_tab(idx);
                    self.editor_dragging_tab = Some(idx);
                }
                akssora_terminal_ui::EditorTabTarget::Pin(idx) => {
                    if idx < self.editor_open_files.len() {
                        let p = self.editor_open_files[idx].clone();
                        self.toggle_pin_editor_file(&p);
                    }
                }
                akssora_terminal_ui::EditorTabTarget::Close(idx) => {
                    self.request_close_editor_tab(idx);
                }
                akssora_terminal_ui::EditorTabTarget::FormatDocument => {
                    self.format_active_editor();
                }
                akssora_terminal_ui::EditorTabTarget::TogglePreview => {
                    self.toggle_active_markdown_preview();
                }
                akssora_terminal_ui::EditorTabTarget::ToggleSplitPreview => {
                    self.toggle_active_markdown_split_preview();
                }
                akssora_terminal_ui::EditorTabTarget::ToggleInlineDiff => {
                    if let Some(doc) = self.active_editor_mut() {
                        doc.inline_diff = !doc.inline_diff;
                    }
                }
            }
            if let Some(win) = self.window.as_ref() {
                win.request_redraw();
            }
            return true;
        }

        // 3.2.2 Editor scrollbar clicks (track jump or thumb dragging)
        if !is_code_review
            && let Some((
                total_lines,
                visible_rows,
                max_line_len,
                visible_cols,
                v_overflow,
                h_overflow,
            )) = self.update_editor_scrollbar_geometries()
        {
            if v_overflow && self.editor_v_scrollbar.hit_test(mx, my) {
                let current_scroll = self.active_editor().map(|d| d.scroll_row).unwrap_or(0);
                if self.editor_v_scrollbar.hit_test_thumb_top_down(
                    mx,
                    my,
                    total_lines,
                    visible_rows,
                    current_scroll,
                ) {
                    self.editor_v_scrollbar.is_dragging = true;
                    self.editor_v_scrollbar.drag_start_pos = my;
                    self.editor_v_scrollbar.drag_start_offset = current_scroll as f32;
                } else {
                    let target = self.editor_v_scrollbar.offset_from_pos_top_down(
                        my,
                        total_lines,
                        visible_rows,
                    );
                    let max_scroll = total_lines.saturating_sub(visible_rows);
                    let target = target.min(max_scroll);
                    if let Some(doc) = self.active_editor_mut() {
                        doc.scroll_row = target;
                    }
                    self.editor_v_scrollbar.is_dragging = true;
                    self.editor_v_scrollbar.drag_start_pos = my;
                    self.editor_v_scrollbar.drag_start_offset = target as f32;
                }
                if let Some(win) = self.window.as_ref() {
                    win.request_redraw();
                }
                return true;
            }

            if h_overflow && self.editor_h_scrollbar.hit_test(mx, my) {
                let current_scroll = self.active_editor().map(|d| d.scroll_col).unwrap_or(0);
                if self.editor_h_scrollbar.hit_test_thumb_top_down(
                    mx,
                    my,
                    max_line_len,
                    visible_cols,
                    current_scroll,
                ) {
                    self.editor_h_scrollbar.is_dragging = true;
                    self.editor_h_scrollbar.drag_start_pos = mx;
                    self.editor_h_scrollbar.drag_start_offset = current_scroll as f32;
                } else {
                    let target = self.editor_h_scrollbar.offset_from_pos_top_down(
                        mx,
                        max_line_len,
                        visible_cols,
                    );
                    let max_scroll = max_line_len.saturating_sub(visible_cols);
                    let target = target.min(max_scroll);
                    if let Some(doc) = self.active_editor_mut() {
                        doc.scroll_col = target;
                    }
                    self.editor_h_scrollbar.is_dragging = true;
                    self.editor_h_scrollbar.drag_start_pos = mx;
                    self.editor_h_scrollbar.drag_start_offset = target as f32;
                }
                if let Some(win) = self.window.as_ref() {
                    win.request_redraw();
                }
                return true;
            }
        }

        // 3.2.3 Editor text content clicks
        let content_y = top_h + tab_bar_h;
        if !is_code_review
            && let Some(doc) = self.active_editor_mut()
            && mx >= ws_x
            && my >= content_y
            && my < h - 22.0
            && doc.markdown_mode != akssora_editor::MarkdownViewMode::Preview
        {
            let split_limit = if doc.markdown_mode == akssora_editor::MarkdownViewMode::Split {
                ws_x + ((ws_w - 1.0) * 0.5).floor()
            } else {
                ws_x + ws_w
            };

            if mx < split_limit {
                if let Some((target_line, action)) = doc.hit_test_gutter_click(
                    mx,
                    my,
                    ws_x,
                    content_y,
                    split_limit - ws_x,
                    cell_w,
                    cell_h,
                ) && let Some(path) = doc.path.clone()
                {
                    match action {
                        akssora_editor::GutterActionKind::StageLine => {
                            if let Some(repo_root) = akssora_git::GitInspector::repo_root(&path) {
                                let rel_path = path.strip_prefix(&repo_root).unwrap_or(&path);
                                let _ = akssora_git::GitInspector::stage_or_unstage_line_at(
                                    &repo_root,
                                    rel_path,
                                    target_line,
                                    false,
                                );
                                doc.refresh_git_original_content(&repo_root);
                            }
                        }
                        akssora_editor::GutterActionKind::RevertLine => {
                            let _ = doc.revert_line_at(target_line);
                            if let Some(repo_root) = akssora_git::GitInspector::repo_root(&path) {
                                doc.refresh_git_original_content(&repo_root);
                            }
                        }
                    }
                    self.refresh_git_status();
                    if let Some(win) = self.window.as_ref() {
                        win.request_redraw();
                    }
                    return true;
                }

                doc.handle_content_click(
                    mx,
                    my,
                    ws_x,
                    content_y,
                    split_limit - ws_x,
                    cell_w,
                    cell_h,
                );
                self.mouse_press_pos = Some((mx, my));
                if let Some(win) = self.window.as_ref() {
                    win.request_redraw();
                }
                return true;
            }
        }

        false
    }
}
