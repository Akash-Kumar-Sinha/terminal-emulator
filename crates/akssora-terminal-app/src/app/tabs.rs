use akssora_terminal_core::Terminal;
use akssora_terminal_renderer::FontManager;
use akssora_terminal_ui::{Block, TabKind};

use super::App;

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

// INFO: Tab lifecycle, session instantiation, and block geometry layout management.
impl App {
    pub fn tab_count(&self) -> usize {
        self.tab_manager.len()
    }

    pub fn active_tab_id(&self) -> Option<u64> {
        self.tab_manager.active_id()
    }

    pub fn is_exit_prompt_active(&self) -> bool {
        self.prompt_confirm_exit
    }

    pub fn dismiss_exit_prompt(&mut self) {
        self.prompt_confirm_exit = false;
        if let Some(win) = self.window.as_ref() {
            win.request_redraw();
        }
    }

    pub fn sync_active_tab_state(&mut self) {
        if let Some(tab) = self.tab_manager.active_tab() {
            if matches!(tab.kind, TabKind::Settings) {
                self.header.is_settings_active = true;
                if self.explorer.is_expanded {
                    self.explorer.is_expanded = false;
                    self.header.is_explorer_expanded = false;
                }
            } else {
                self.header.is_settings_active = false;
            }

            if matches!(tab.kind, TabKind::CodeReview { .. }) {
                self.header.is_code_review_active = true;
                self.ensure_code_review_documents();
            } else {
                self.header.is_code_review_active = false;
            }
        }
    }

    pub fn next_tab(&mut self) {
        self.tab_manager.next_tab();
        self.sync_active_tab_state();
        self.layout_blocks();
        if let Some(win) = self.window.as_ref() {
            win.request_redraw();
        }
    }

    pub fn prev_tab(&mut self) {
        self.tab_manager.prev_tab();
        self.sync_active_tab_state();
        self.layout_blocks();
        if let Some(win) = self.window.as_ref() {
            win.request_redraw();
        }
    }

    pub fn close_active_tab_or_prompt(&mut self) {
        if self.tab_manager.len() <= 1 {
            self.prompt_confirm_exit = true;
            if let Some(win) = self.window.as_ref() {
                win.request_redraw();
            }
        } else {
            let active = self.tab_manager.active_index();
            self.close_tab(active);
            if let Some(win) = self.window.as_ref() {
                win.request_redraw();
            }
        }
    }

    // INFO: Prompts the user before closing all tabs and exiting the application.
    pub fn close_all_tabs_or_prompt(&mut self) {
        self.prompt_confirm_exit = true;
        if let Some(win) = self.window.as_ref() {
            win.request_redraw();
        }
    }

    pub fn close_all_tabs(&mut self) {
        for (_, mut block) in self.blocks.drain() {
            block.shell.kill();
        }
        self.editors.clear();
        self.command_buffers.clear();
        self.last_executed_commands.clear();
        self.pane_custom_names.clear();
        self.tab_manager.clear();
        self.split_trees.clear();
        self.split_manager.active_pane_id = None;
        self.save_session_state();
    }

    pub fn close_other_tabs(&mut self) {
        if self.tab_manager.len() <= 1 {
            self.prompt_confirm_exit = true;
            if let Some(win) = self.window.as_ref() {
                win.request_redraw();
            }
            return;
        }

        let active_id = self.tab_manager.active_id();
        let tabs_to_close: Vec<usize> = self
            .tab_manager
            .tabs()
            .iter()
            .enumerate()
            .filter(|(_, t)| Some(t.id) != active_id)
            .map(|(i, _)| i)
            .collect();

        for idx in tabs_to_close.into_iter().rev() {
            if let Some(removed) = self.tab_manager.close(idx) {
                self.command_buffers.remove(&removed.id);
                if let Some(tree) = self.split_trees.remove(&removed.id) {
                    for pane_id in tree.all_leaf_ids() {
                        self.command_buffers.remove(&pane_id);
                        if let Some(mut block) = self.blocks.remove(&pane_id) {
                            block.shell.kill();
                        }
                    }
                }
                if let Some(mut block) = self.blocks.remove(&removed.id) {
                    block.shell.kill();
                }
            }
        }
        self.layout_blocks();
        self.save_session_state();
        if let Some(win) = self.window.as_ref() {
            win.request_redraw();
        }
    }

    pub fn rename_active_tab(&mut self, new_name: impl Into<String>) -> bool {
        if let Some(id) = self.tab_manager.active_id() {
            self.tab_manager.set_custom_title(id, new_name)
        } else {
            false
        }
    }

    pub fn rename_tab(&mut self, tab_id: u64, new_name: impl Into<String>) -> bool {
        self.tab_manager.set_custom_title(tab_id, new_name)
    }

    pub fn open_terminal_tab(&mut self) -> u64 {
        let active_cwd = self.active_working_directory();
        self.open_terminal_tab_in(active_cwd.as_deref())
    }

    pub fn open_terminal_tab_in(&mut self, cwd: Option<&std::path::Path>) -> u64 {
        let count = self.tab_manager.len() + 1;
        let title = format!("Terminal {count}");
        let (rows, cols) = self.default_grid_dims();
        let terminal = match Terminal::new_in(rows, cols, cwd) {
            Ok(t) => t,
            Err(e) => {
                eprintln!("failed to start terminal: {e:?}");
                return 0;
            }
        };

        let tab_id = self.tab_manager.add_tab(title, TabKind::Terminal);
        let mut block = Block::new(
            None, None, None, None, None, None, None, None, None, None, None, terminal,
        );
        block.pane_id = Some(tab_id);
        self.blocks.insert(tab_id, block);
        self.layout_blocks();
        self.save_session_state();
        tab_id
    }

    pub fn open_editor_tab(&mut self, path: impl Into<String>) -> u64 {
        let p = path.into();
        if let Some(pos) = self.tab_manager.tabs().iter().position(|t| match &t.kind {
            TabKind::Editor { path: open_p } => open_p == &p,
            _ => false,
        }) {
            self.tab_manager.select(pos);
            self.split_manager.active_pane_id = Some(self.tab_manager.tabs()[pos].id);
            self.layout_blocks();
            return self.tab_manager.tabs()[pos].id;
        }

        let filename = std::path::Path::new(&p)
            .file_name()
            .and_then(|f| f.to_str())
            .unwrap_or("document.txt")
            .to_string();
        let (rows, cols) = self.default_grid_dims();
        let terminal = Terminal::new(rows, cols).expect("terminal shell fallback");
        let tab_id = self
            .tab_manager
            .add_tab(filename.clone(), TabKind::Editor { path: p });
        let mut block = Block::new(
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            Some(filename),
            terminal,
        );
        block.pane_id = Some(tab_id);
        self.blocks.insert(tab_id, block);
        self.split_manager.active_pane_id = Some(tab_id);
        self.layout_blocks();
        tab_id
    }

    pub fn open_api_tab(&mut self, workspace: impl Into<String>) -> u64 {
        let ws = workspace.into();
        if let Some(pos) = self.tab_manager.tabs().iter().position(|t| match &t.kind {
            TabKind::ApiTesting { workspace: open_ws } => open_ws == &ws,
            _ => false,
        }) {
            self.tab_manager.select(pos);
            self.split_manager.active_pane_id = Some(self.tab_manager.tabs()[pos].id);
            self.layout_blocks();
            return self.tab_manager.tabs()[pos].id;
        }

        let (rows, cols) = self.default_grid_dims();
        let terminal = Terminal::new(rows, cols).expect("terminal shell fallback");
        let title = format!("API: {ws}");
        let tab_id = self
            .tab_manager
            .add_tab(title.clone(), TabKind::ApiTesting { workspace: ws });
        let mut block = Block::new(
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            Some(title),
            terminal,
        );
        block.pane_id = Some(tab_id);
        self.blocks.insert(tab_id, block);
        self.split_manager.active_pane_id = Some(tab_id);
        self.layout_blocks();
        tab_id
    }

    pub fn open_plugin_management_tab(&mut self) -> u64 {
        if let Some(pos) = self
            .tab_manager
            .tabs()
            .iter()
            .position(|t| matches!(t.kind, TabKind::PluginManagement))
        {
            self.tab_manager.select(pos);
            self.split_manager.active_pane_id = Some(self.tab_manager.tabs()[pos].id);
            self.layout_blocks();
            return self.tab_manager.tabs()[pos].id;
        }

        let (rows, cols) = self.default_grid_dims();
        let terminal = Terminal::new(rows, cols).expect("terminal shell fallback");
        let tab_id = self
            .tab_manager
            .add_tab("Plugin Management", TabKind::PluginManagement);
        let mut block = Block::new(
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            Some("Plugin Management".to_string()),
            terminal,
        );
        block.pane_id = Some(tab_id);
        self.blocks.insert(tab_id, block);
        self.split_manager.active_pane_id = Some(tab_id);
        self.layout_blocks();
        tab_id
    }

    pub fn open_plugin_tab(&mut self) -> u64 {
        self.open_plugin_management_tab()
    }

    pub fn open_settings_tab(&mut self) -> u64 {
        if self.explorer.is_expanded {
            self.explorer.is_expanded = false;
            self.header.is_explorer_expanded = false;
        }
        self.header.is_settings_active = true;

        if let Some(pos) = self
            .tab_manager
            .tabs()
            .iter()
            .position(|t| matches!(t.kind, TabKind::Settings))
        {
            self.tab_manager.select(pos);
            self.split_manager.active_pane_id = Some(self.tab_manager.tabs()[pos].id);
            self.layout_blocks();
            return self.tab_manager.tabs()[pos].id;
        }

        let (rows, cols) = self.default_grid_dims();
        let terminal = Terminal::new(rows, cols).expect("terminal shell fallback");
        let tab_id = self.tab_manager.add_tab("Settings", TabKind::Settings);
        let mut block = Block::new(
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            Some("Settings".to_string()),
            terminal,
        );
        block.pane_id = Some(tab_id);
        self.blocks.insert(tab_id, block);
        self.split_manager.active_pane_id = Some(tab_id);
        self.layout_blocks();
        tab_id
    }

    // INFO: Opens a file in the integrated code editor, adding to dedicated file-tab bar and selecting it.
    pub fn open_editor_file(&mut self, path: &std::path::Path) -> u64 {
        self.open_editor_file_with_preview(path, false)
    }

    // INFO: Opens a file in preview mode (replaces previous untouched preview tab) or persistent mode.
    pub fn open_editor_file_with_preview(
        &mut self,
        path: &std::path::Path,
        is_preview: bool,
    ) -> u64 {
        let path_buf = path.to_path_buf();
        let path_str = path_buf.to_string_lossy().to_string();

        if is_preview {
            if let Some(prev) = self.editor_preview_file.clone()
                && prev != path_buf
                && !self.editor_pinned_files.contains(&prev)
            {
                let prev_dirty = self
                    .editor_documents
                    .get(&prev)
                    .map(|d| d.dirty)
                    .unwrap_or(false);
                if !prev_dirty
                    && let Some(pos) = self.editor_open_files.iter().position(|p| p == &prev)
                {
                    self.editor_open_files.remove(pos);
                    self.editor_documents.remove(&prev);
                }
            }
            self.editor_preview_file = Some(path_buf.clone());
        } else {
            // Opening persistently clears preview on this file
            if self.editor_preview_file.as_deref() == Some(&path_buf) {
                self.editor_preview_file = None;
            }
        }

        if !self.editor_open_files.contains(&path_buf) {
            self.editor_open_files.push(path_buf.clone());
        }
        self.editor_active_file = Some(path_buf.clone());
        self.explorer.active_editor_file = Some(path_buf.clone());
        self.refresh_git_status();

        if !self.editor_documents.contains_key(&path_buf) {
            let mut doc = if !path_buf.exists() {
                let repo_root = self.code_review.repo_root.clone();
                let matching_diff = self
                    .code_review
                    .diff
                    .files
                    .iter()
                    .find(|f| repo_root.join(&f.path) == path_buf);
                if let Ok(content) = akssora_git::GitInspector::file_deleted_content(
                    &repo_root,
                    &path_buf,
                    matching_diff,
                ) {
                    let mut d = akssora_editor::Document::open(path_buf.clone(), &content);
                    d.set_deleted(true);
                    d
                } else {
                    akssora_editor::Document::open(path_buf.clone(), "")
                }
            } else {
                akssora_editor::Document::open_file(path)
                    .unwrap_or_else(|_| akssora_editor::Document::open(path_buf.clone(), ""))
            };
            doc.line_wrap = self.config.line_wrap;
            doc.line_numbers = self.config.line_numbers;
            doc.line_number_mode = self.config.line_number_mode;
            doc.tab_size = self.config.tab_size;
            doc.line_height = self.config.line_height;
            self.editor_documents.insert(path_buf.clone(), doc);
        }

        let doc = self
            .editor_documents
            .get(&path_buf)
            .expect("document exists");
        let title = doc.display_title();

        // Check if an editor tab already exists in tab_manager
        if let Some(pos) = self
            .tab_manager
            .tabs()
            .iter()
            .position(|t| matches!(&t.kind, TabKind::Editor { .. }))
        {
            self.tab_manager.select(pos);
            let tab_id = self.tab_manager.tabs()[pos].id;
            if let Some(tab) = self.tab_manager.active_tab_mut() {
                tab.title = title;
                tab.kind = TabKind::Editor { path: path_str };
            }
            if let Some(doc_clone) = self.editor_documents.get(&path_buf).cloned() {
                self.editors.insert(tab_id, doc_clone);
            }
            self.split_manager.active_pane_id = Some(tab_id);
            self.header.is_code_review_active = false;
            self.layout_blocks();
            if let Some(win) = self.window.as_ref() {
                win.request_redraw();
            }
            return tab_id;
        }

        // Otherwise create new editor tab in tab_manager
        let tab_id = self
            .tab_manager
            .add_tab(title, TabKind::Editor { path: path_str });
        if let Some(doc_clone) = self.editor_documents.get(&path_buf).cloned() {
            self.editors.insert(tab_id, doc_clone);
        }
        self.split_manager.active_pane_id = Some(tab_id);
        self.header.is_code_review_active = false;
        self.layout_blocks();
        if let Some(win) = self.window.as_ref() {
            win.request_redraw();
        }
        tab_id
    }

    pub fn close_tab(&mut self, idx: usize) {
        if self.tab_manager.len() <= 1 {
            self.prompt_confirm_exit = true;
            if let Some(win) = self.window.as_ref() {
                win.request_redraw();
            }
            return;
        }

        if let Some(removed) = self.tab_manager.close(idx) {
            if matches!(removed.kind, TabKind::Editor { .. }) {
                self.editor_open_files.clear();
                self.editor_active_file = None;
                self.editor_documents.clear();
            }
            self.editors.remove(&removed.id);
            self.command_buffers.remove(&removed.id);
            self.last_executed_commands.remove(&removed.id);
            self.pane_custom_names.remove(&removed.id);
            if let Some(tree) = self.split_trees.remove(&removed.id) {
                for pane_id in tree.all_leaf_ids() {
                    self.editors.remove(&pane_id);
                    self.command_buffers.remove(&pane_id);
                    self.last_executed_commands.remove(&pane_id);
                    self.pane_custom_names.remove(&pane_id);
                    if let Some(mut block) = self.blocks.remove(&pane_id) {
                        block.shell.kill();
                    }
                }
            }
            if let Some(mut block) = self.blocks.remove(&removed.id) {
                block.shell.kill();
            }
            if self.tab_manager.is_empty() {
                self.open_terminal_tab();
            } else {
                if let Some(active_id) = self.tab_manager.active_id() {
                    if let Some(tree) = self.split_trees.get(&active_id) {
                        if !tree.contains_leaf(self.split_manager.active_pane_id.unwrap_or(0)) {
                            self.split_manager.active_pane_id =
                                tree.all_leaf_ids().first().copied();
                        }
                    } else {
                        self.split_manager.active_pane_id = Some(active_id);
                    }
                }
                self.layout_blocks();
            }
            self.save_session_state();
        }
    }

    pub(crate) fn default_grid_dims(&self) -> (usize, usize) {
        let (win_w, win_h) = self
            .state
            .as_ref()
            .map(|s| s.size())
            .unwrap_or((self.config.window.width, self.config.window.height));
        let show_sidebar = self.config.tab_layout.show_sidebar();
        let sb_w = self.sidebar.effective_width(show_sidebar);
        let exp_w = self.explorer.effective_width();
        let usable_w = (win_w as f32 - sb_w - exp_w).max(MIN_BLOCK_HEIGHT) as u32;
        let top_h = self
            .header
            .total_height(self.config.tab_layout.includes_title_bar());
        let max_h = max_block_height(win_h, top_h);
        let fonts = self.fonts.as_ref();
        if let Some(f) = fonts {
            grid_dims(usable_w, max_h as u32, f)
        } else {
            (24, 80)
        }
    }

    // INFO: Recalculates geometries and resizes terminal grids across all tabs and split panes.
    pub fn layout_blocks(&mut self) {
        self.layout_blocks_with_splits();
    }

    pub(crate) fn active_block_mut(&mut self) -> Option<&mut Block> {
        let active_tab_id = self.tab_manager.active_id()?;
        let target_id = if let Some(focused_pane) = self.split_manager.active_pane_id {
            if let Some(tree) = self.split_trees.get(&active_tab_id) {
                if tree.contains_leaf(focused_pane) {
                    focused_pane
                } else {
                    active_tab_id
                }
            } else {
                active_tab_id
            }
        } else {
            active_tab_id
        };
        self.blocks.get_mut(&target_id)
    }
}
