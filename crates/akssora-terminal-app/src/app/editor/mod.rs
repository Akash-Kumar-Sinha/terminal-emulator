use super::{App, UnsavedCloseAction};

// INFO: Integrated editor document management, tabs, formatting, preview modes, and scrollbars.
impl App {
    // INFO: Updates open editor documents and tab metadata when a file is renamed.
    pub fn handle_file_renamed(&mut self, old_path: &std::path::Path, new_path: &std::path::Path) {
        if let Some(pos) = self.editor_open_files.iter().position(|p| p == old_path) {
            self.editor_open_files[pos] = new_path.to_path_buf();
        }
        if self.editor_active_file.as_deref() == Some(old_path) {
            self.editor_active_file = Some(new_path.to_path_buf());
        }
        if let Some(mut doc) = self.editor_documents.remove(old_path) {
            doc.path = Some(new_path.to_path_buf());
            self.editor_documents.insert(new_path.to_path_buf(), doc);
        }
        let old_str = old_path.to_string_lossy().to_string();
        let new_str = new_path.to_string_lossy().to_string();
        let title = new_path
            .file_name()
            .map(|f| f.to_string_lossy().to_string())
            .unwrap_or_else(|| "Untitled".to_string());
        for tab in self.tab_manager.tabs_mut() {
            if let akssora_terminal_ui::TabKind::Editor { path } = &mut tab.kind
                && path.as_str() == old_str.as_str()
            {
                *path = new_str.clone();
                tab.title = title.clone();
            }
        }
    }

    // INFO: Removes deleted files from editor file-tab bar and tabs.
    pub fn handle_file_deleted(&mut self, deleted_path: &std::path::Path) {
        if let Some(pos) = self
            .editor_open_files
            .iter()
            .position(|p| p == deleted_path)
        {
            self.close_editor_file_tab(pos);
        }
    }

    // INFO: Selects an opened file tab by index in the dedicated editor file-tab bar.
    pub fn select_editor_file_tab(&mut self, index: usize) {
        if index < self.editor_open_files.len() {
            let path = self.editor_open_files[index].clone();
            self.editor_active_file = Some(path.clone());
            let path_str = path.to_string_lossy().to_string();
            let title = path
                .file_name()
                .map(|f| f.to_string_lossy().to_string())
                .unwrap_or_else(|| "Untitled".to_string());
            if let Some(pos) = self
                .tab_manager
                .tabs()
                .iter()
                .position(|t| matches!(&t.kind, akssora_terminal_ui::TabKind::Editor { .. }))
            {
                self.tab_manager.select(pos);
                if let Some(tab) = self.tab_manager.active_tab_mut() {
                    tab.title = title;
                    tab.kind = akssora_terminal_ui::TabKind::Editor { path: path_str };
                }
            }
            if let Some(win) = self.window.as_ref() {
                win.request_redraw();
            }
        }
    }

    // INFO: Toggles pin state for an open editor tab file. Pinned tabs stay at the left and are protected.
    pub fn toggle_pin_editor_file(&mut self, path: &std::path::Path) {
        if self.editor_pinned_files.contains(path) {
            self.editor_pinned_files.remove(path);
        } else {
            self.editor_pinned_files.insert(path.to_path_buf());
            // Pinning promotes a preview tab to persistent
            if self.editor_preview_file.as_deref() == Some(path) {
                self.editor_preview_file = None;
            }
        }
        self.reorder_pinned_tabs_to_front();
        if let Some(win) = self.window.as_ref() {
            win.request_redraw();
        }
    }

    // INFO: Keeps pinned tabs at the beginning of editor_open_files.
    pub fn reorder_pinned_tabs_to_front(&mut self) {
        let mut pinned = Vec::new();
        let mut unpinned = Vec::new();
        for p in self.editor_open_files.drain(..) {
            if self.editor_pinned_files.contains(&p) {
                pinned.push(p);
            } else {
                unpinned.push(p);
            }
        }
        self.editor_open_files = pinned;
        self.editor_open_files.extend(unpinned);
    }

    // INFO: Moves an open editor tab from source index to target index.
    pub fn reorder_editor_file_tab(&mut self, from: usize, to: usize) {
        if from < self.editor_open_files.len() && to < self.editor_open_files.len() && from != to {
            let item = self.editor_open_files.remove(from);
            self.editor_open_files.insert(to, item);
            if let Some(win) = self.window.as_ref() {
                win.request_redraw();
            }
        }
    }

    // INFO: Requests closing an editor tab by index. If dirty and auto_save disabled, opens save confirmation prompt.
    pub fn request_close_editor_tab(&mut self, index: usize) {
        if index < self.editor_open_files.len() {
            if self.config.auto_save {
                let path = &self.editor_open_files[index];
                if let Some(doc) = self.editor_documents.get_mut(path)
                    && doc.dirty
                {
                    let _ = doc.save();
                }
                self.close_editor_file_tab(index);
                return;
            }

            let path = &self.editor_open_files[index];
            let is_dirty = self
                .editor_documents
                .get(path)
                .map(|d| d.dirty)
                .unwrap_or(false);
            if is_dirty {
                self.unsaved_prompt_target = Some(UnsavedCloseAction::CloseSingle(index));
                if let Some(win) = self.window.as_ref() {
                    win.request_redraw();
                }
            } else {
                self.close_editor_file_tab(index);
            }
        }
    }

    // INFO: Requests closing all other editor tabs except the active/specified one.
    pub fn request_close_other_editor_tabs(&mut self) {
        let Some(active) = self.editor_active_file.clone() else {
            return;
        };
        if self.config.auto_save {
            for (path, doc) in self.editor_documents.iter_mut() {
                if path != &active && doc.dirty {
                    let _ = doc.save();
                }
            }
            self.editor_open_files
                .retain(|p| p == &active || self.editor_pinned_files.contains(p));
            if let Some(win) = self.window.as_ref() {
                win.request_redraw();
            }
            return;
        }

        // Check if any other tab has unsaved changes
        let has_dirty_other = self.editor_open_files.iter().any(|p| {
            p != &active
                && self
                    .editor_documents
                    .get(p)
                    .map(|d| d.dirty)
                    .unwrap_or(false)
        });
        if has_dirty_other {
            self.unsaved_prompt_target = Some(UnsavedCloseAction::CloseOthers(active));
            if let Some(win) = self.window.as_ref() {
                win.request_redraw();
            }
        } else {
            self.editor_open_files
                .retain(|p| p == &active || self.editor_pinned_files.contains(p));
            if let Some(win) = self.window.as_ref() {
                win.request_redraw();
            }
        }
    }

    // INFO: Requests closing all editor tabs. Prompts if any file is dirty and auto_save is disabled.
    pub fn request_close_all_editor_tabs(&mut self) {
        if self.config.auto_save {
            for doc in self.editor_documents.values_mut() {
                if doc.dirty {
                    let _ = doc.save();
                }
            }
            while !self.editor_open_files.is_empty() {
                self.close_editor_file_tab(0);
            }
            if let Some(win) = self.window.as_ref() {
                win.request_redraw();
            }
            return;
        }

        let has_dirty = self.editor_open_files.iter().any(|p| {
            self.editor_documents
                .get(p)
                .map(|d| d.dirty)
                .unwrap_or(false)
        });
        if has_dirty {
            self.unsaved_prompt_target = Some(UnsavedCloseAction::CloseAll);
            if let Some(win) = self.window.as_ref() {
                win.request_redraw();
            }
        } else {
            while !self.editor_open_files.is_empty() {
                self.close_editor_file_tab(0);
            }
        }
    }

    // INFO: Saves the currently prompted dirty document and proceeds with closure.
    pub fn save_prompt_action(&mut self) {
        if let Some(target) = self.unsaved_prompt_target.take() {
            match target {
                UnsavedCloseAction::CloseSingle(idx) => {
                    if idx < self.editor_open_files.len() {
                        let path = &self.editor_open_files[idx];
                        if let Some(doc) = self.editor_documents.get_mut(path) {
                            let _ = doc.save();
                        }
                        self.close_editor_file_tab(idx);
                    }
                }
                UnsavedCloseAction::CloseOthers(keep_path) => {
                    for (path, doc) in self.editor_documents.iter_mut() {
                        if path != &keep_path && doc.dirty {
                            let _ = doc.save();
                        }
                    }
                    self.editor_open_files
                        .retain(|p| p == &keep_path || self.editor_pinned_files.contains(p));
                }
                UnsavedCloseAction::CloseAll => {
                    for doc in self.editor_documents.values_mut() {
                        if doc.dirty {
                            let _ = doc.save();
                        }
                    }
                    while !self.editor_open_files.is_empty() {
                        self.close_editor_file_tab(0);
                    }
                }
            }
            self.refresh_git_status();
            if let Some(win) = self.window.as_ref() {
                win.request_redraw();
            }
        }
    }

    // INFO: Discards changes on the currently prompted dirty document and proceeds with closure.
    pub fn discard_prompt_action(&mut self) {
        if let Some(target) = self.unsaved_prompt_target.take() {
            match target {
                UnsavedCloseAction::CloseSingle(idx) => {
                    self.close_editor_file_tab(idx);
                }
                UnsavedCloseAction::CloseOthers(keep_path) => {
                    self.editor_open_files
                        .retain(|p| p == &keep_path || self.editor_pinned_files.contains(p));
                }
                UnsavedCloseAction::CloseAll => {
                    while !self.editor_open_files.is_empty() {
                        self.close_editor_file_tab(0);
                    }
                }
            }
            if let Some(win) = self.window.as_ref() {
                win.request_redraw();
            }
        }
    }

    // INFO: Cancels closure without losing unsaved changes.
    pub fn cancel_prompt_action(&mut self) {
        self.unsaved_prompt_target = None;
        if let Some(win) = self.window.as_ref() {
            win.request_redraw();
        }
    }

    // INFO: Refreshes Git status immediately across Explorer tree and editor tabs.
    pub fn refresh_git_status(&mut self) {
        let root = &self.explorer.tree.root_path;
        self.explorer.git_status = akssora_git::GitInspector::status(root);
        for doc in self.editor_documents.values_mut() {
            doc.refresh_git_original_content(root);
        }
        for doc in self.editors.values_mut() {
            doc.refresh_git_original_content(root);
        }
        self.sync_code_review_with_active_terminal();
    }

    // INFO: Closes an individual file tab in the dedicated editor file-tab bar.
    pub fn close_editor_file_tab(&mut self, index: usize) {
        if index < self.editor_open_files.len() {
            let removed = self.editor_open_files.remove(index);
            self.editor_pinned_files.remove(&removed);
            if self.editor_preview_file.as_deref() == Some(&removed) {
                self.editor_preview_file = None;
            }
            self.editor_documents.remove(&removed);
            if self.editor_active_file.as_deref() == Some(&removed) {
                if !self.editor_open_files.is_empty() {
                    let new_idx = index.min(self.editor_open_files.len() - 1);
                    self.select_editor_file_tab(new_idx);
                } else {
                    self.editor_active_file = None;
                    if let Some(pos) = self.tab_manager.tabs().iter().position(|t| {
                        matches!(&t.kind, akssora_terminal_ui::TabKind::Editor { .. })
                    }) {
                        let tab_id = self.tab_manager.tabs()[pos].id;
                        self.editors.remove(&tab_id);
                        if self.tab_manager.len() > 1 {
                            self.close_tab(pos);
                        } else {
                            self.tab_manager.clear();
                            self.open_terminal_tab();
                        }
                    }
                }
            }
            self.explorer.active_editor_file = self.editor_active_file.clone();
            if let Some(win) = self.window.as_ref() {
                win.request_redraw();
            }
        }
    }

    // INFO: Collects open file tab descriptors for rendering the dedicated editor file-tab bar.
    pub fn collect_editor_file_tabs(&self) -> Vec<akssora_terminal_ui::EditorFileTab> {
        let root = &self.explorer.tree.root_path;
        self.editor_open_files
            .iter()
            .map(|path| {
                let is_active = self.editor_active_file.as_ref() == Some(path);
                let doc = self.editor_documents.get(path);
                let is_dirty = doc.map(|d| d.dirty).unwrap_or(false);
                let is_preview = self.editor_preview_file.as_ref() == Some(path);
                let is_pinned = self.editor_pinned_files.contains(path);
                let relative_path = path
                    .strip_prefix(root)
                    .unwrap_or(path)
                    .to_string_lossy()
                    .to_string();
                let git_status = self
                    .explorer
                    .git_status
                    .as_ref()
                    .and_then(|gs| gs.file_status(root, path));
                let title = path
                    .file_name()
                    .map(|f| f.to_string_lossy().to_string())
                    .unwrap_or_else(|| "Untitled".to_string());
                akssora_terminal_ui::EditorFileTab {
                    path: path.clone(),
                    title,
                    relative_path,
                    is_active,
                    is_dirty,
                    is_preview,
                    is_pinned,
                    git_status,
                }
            })
            .collect()
    }

    // INFO: Retrieves the document model for the currently active tab if it is an editor tab or Code Review active file.
    pub fn active_editor(&self) -> Option<&akssora_editor::Document> {
        let is_code_review = self
            .tab_manager
            .active_tab()
            .map(|t| matches!(t.kind, akssora_terminal_ui::TabKind::CodeReview { .. }))
            .unwrap_or(false);
        if is_code_review && let Some(p) = &self.code_review.active_editing_file {
            return self.editor_documents.get(p);
        }

        let is_editor = self
            .tab_manager
            .active_tab()
            .map(|t| matches!(t.kind, akssora_terminal_ui::TabKind::Editor { .. }))
            .unwrap_or(false);
        if !is_editor || self.editor_open_files.is_empty() {
            return None;
        }
        if let Some(p) = &self.editor_active_file {
            return self.editor_documents.get(p);
        }
        let active_id = self.tab_manager.active_id()?;
        self.editors.get(&active_id)
    }

    // INFO: Retrieves a mutable reference to the active editor document.
    pub fn active_editor_mut(&mut self) -> Option<&mut akssora_editor::Document> {
        let is_code_review = self
            .tab_manager
            .active_tab()
            .map(|t| matches!(t.kind, akssora_terminal_ui::TabKind::CodeReview { .. }))
            .unwrap_or(false);
        if is_code_review && let Some(p) = self.code_review.active_editing_file.clone() {
            return self.editor_documents.get_mut(&p);
        }

        let is_editor = self
            .tab_manager
            .active_tab()
            .map(|t| matches!(t.kind, akssora_terminal_ui::TabKind::Editor { .. }))
            .unwrap_or(false);
        if !is_editor || self.editor_open_files.is_empty() {
            return None;
        }
        if let Some(p) = self.editor_active_file.clone() {
            return self.editor_documents.get_mut(&p);
        }
        let active_id = self.tab_manager.active_id()?;
        self.editors.get_mut(&active_id)
    }

    // INFO: Formats the active Markdown editor document using configured formatter and workspace config.
    pub fn format_active_editor(&mut self) -> bool {
        let tool = akssora_editor::FormatterTool::parse(&self.config.markdown_formatter);
        let root = self.explorer.tree.root_path.clone();
        if let Some(doc) = self.active_editor_mut()
            && doc.is_markdown()
        {
            match doc.format_markdown(tool, Some(&root)) {
                Ok(changed) => {
                    if changed {
                        self.refresh_git_status();
                    }
                    if let Some(win) = self.window.as_ref() {
                        win.request_redraw();
                    }
                    return changed;
                }
                Err(e) => {
                    eprintln!("Markdown formatting error: {e}");
                }
            }
        }
        false
    }

    // INFO: Toggles full Markdown preview mode for the active editor.
    pub fn toggle_active_markdown_preview(&mut self) {
        if let Some(doc) = self.active_editor_mut()
            && doc.is_markdown()
        {
            doc.markdown_mode = doc.markdown_mode.toggle_preview();
            if let Some(win) = self.window.as_ref() {
                win.request_redraw();
            }
        }
    }

    // INFO: Toggles side-by-side split Markdown preview mode for the active editor.
    pub fn toggle_active_markdown_split_preview(&mut self) {
        if let Some(doc) = self.active_editor_mut()
            && doc.is_markdown()
        {
            doc.markdown_mode = doc.markdown_mode.toggle_split();
            if let Some(win) = self.window.as_ref() {
                win.request_redraw();
            }
        }
    }

    // INFO: Updates geometries of the vertical and horizontal editor scrollbars based on current viewport and document dimensions.
    pub fn update_editor_scrollbar_geometries(
        &mut self,
    ) -> Option<(usize, usize, usize, usize, bool, bool)> {
        let (win_w, win_h) = self.state.as_ref().map(|s| s.size())?;
        let (cell_w, cell_h) = self.fonts.as_ref().map(|f| (f.cell_w, f.cell_h))?;
        let doc = self.active_editor()?;

        let max_line_len = doc.max_line_len();

        let show_sidebar = self.config.tab_layout.show_sidebar();
        let sb_w = self.sidebar.effective_width(show_sidebar);
        let exp_w = self.explorer.effective_width();
        let ws_x = sb_w + exp_w;
        let ws_y = self.header.height;
        let ws_w = (win_w as f32 - ws_x).max(0.0);
        let ws_h = (win_h as f32 - ws_y).max(0.0);

        let tab_bar_h = if !self.editor_open_files.is_empty() {
            28.0
        } else {
            0.0
        };

        let is_code_review = self
            .tab_manager
            .active_tab()
            .map(|t| matches!(t.kind, akssora_terminal_ui::TabKind::CodeReview { .. }))
            .unwrap_or(false);

        let (src_x, src_y, src_w, editor_content_h) = if is_code_review {
            let show_tabs = self.config.tab_layout.includes_title_bar();
            let top_h = self.header.total_height(show_tabs);
            if let Some((bx, by, bw, bh)) =
                akssora_terminal_ui::CodeReviewLayout::active_card_body_rect(
                    &self.code_review,
                    ws_x,
                    top_h,
                    ws_w,
                    ws_h,
                )
            {
                (bx, by, bw, bh)
            } else {
                (
                    ws_x,
                    ws_y + tab_bar_h,
                    ws_w,
                    (ws_h - tab_bar_h - 22.0).max(0.0),
                )
            }
        } else {
            let show_tabs = self.config.tab_layout.includes_title_bar();
            let top_h = self.header.total_height(show_tabs);
            (
                ws_x,
                top_h + tab_bar_h,
                ws_w,
                (ws_h - tab_bar_h - 22.0).max(0.0),
            )
        };
        let gutter_w = if doc.line_numbers
            && doc.line_number_mode != akssora_terminal_config::LineNumberMode::Off
        {
            48.0
        } else {
            0.0
        };

        let rough_content_w = (src_w - gutter_w - 16.0).max(0.0);
        let wrap_cols = (rough_content_w / cell_w).floor().max(1.0) as usize;

        let total_lines = if doc.line_wrap {
            doc.total_visual_rows(wrap_cols)
        } else {
            doc.buffer.line_count()
        };

        let line_h = (cell_h * doc.line_height).max(cell_h);
        let visible_rows = (editor_content_h / line_h).floor() as usize;

        let content_w = (src_w - gutter_w - 24.0).max(cell_w);
        let visible_cols = (content_w / cell_w).floor().max(1.0) as usize;

        let v_overflow = total_lines > visible_rows;
        let h_overflow = !doc.line_wrap && max_line_len > visible_cols;

        self.editor_v_scrollbar
            .update_geometry(src_x, src_y, src_w, editor_content_h);

        let h_sb_track_x = src_x + gutter_w;
        let h_sb_track_w = content_w;
        self.editor_h_scrollbar.update_geometry_horizontal(
            h_sb_track_x,
            src_y,
            h_sb_track_w,
            editor_content_h,
        );

        Some((
            total_lines,
            visible_rows,
            max_line_len,
            visible_cols,
            v_overflow,
            h_overflow,
        ))
    }
}
