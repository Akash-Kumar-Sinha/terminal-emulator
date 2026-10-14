use super::App;

// INFO: File Explorer coordination methods including toggles, path resolution, and terminal split opener.
impl App {
    // INFO: Toggles the left hierarchical file explorer panel visibility.
    pub fn toggle_explorer(&mut self) {
        self.explorer.toggle();
        self.header.is_explorer_expanded = self.explorer.is_expanded;
        self.layout_blocks();
        if let Some(win) = self.window.as_ref() {
            win.request_redraw();
        }
    }

    // INFO: Updates the active workspace root directory for the file explorer.
    pub fn set_explorer_root(&mut self, root: impl AsRef<std::path::Path>) -> std::io::Result<()> {
        self.explorer_has_custom_root = true;
        self.explorer.tree.set_root(root)?;
        self.refresh_git_status();
        Ok(())
    }

    // INFO: Syncs file explorer root with the active terminal's current directory when custom root is not locked.
    pub fn sync_explorer_with_active_terminal(&mut self) {
        if !self.explorer_has_custom_root
            && let Some(cwd) = self.active_working_directory()
            && self.explorer.tree.root_path != cwd
            && self.explorer.tree.set_root(&cwd).is_ok()
        {
            self.refresh_git_status();
        }
    }

    // INFO: Opens a file directly in a vertical split pane of the active terminal session.
    pub fn open_file_in_split_pane(&mut self, path: &std::path::Path) -> Option<u64> {
        let parent_dir = path
            .parent()
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| self.explorer.tree.root_path.clone());
        let split_id = self
            .split_active_pane_in(
                akssora_terminal_ui::SplitDirection::Vertical,
                Some(&parent_dir),
            )
            .or_else(|| Some(self.open_terminal_tab_in(Some(&parent_dir))))?;
        if let Some(block) = self.blocks.get_mut(&split_id) {
            let editor = std::env::var("EDITOR").unwrap_or_else(|_| "nano".to_string());
            let cmd = format!("{} \"{}\"\n", editor, path.to_string_lossy());
            block.shell.paste_text(&cmd);
        }
        self.layout_blocks();
        if let Some(win) = self.window.as_ref() {
            win.request_redraw();
        }
        Some(split_id)
    }

    // INFO: Toggles the global quick file opener command-palette overlay.
    pub fn toggle_quick_search(&mut self) {
        if self.quick_search.is_open {
            self.quick_search.close();
        } else {
            let root = self.explorer.tree.root_path.clone();
            let exclusions = self.explorer.tree.exclusions.clone();
            let show_hidden = self.explorer.tree.show_hidden;
            self.quick_search.open(&root, &exclusions, show_hidden);
        }
        if let Some(win) = self.window.as_ref() {
            win.request_redraw();
        }
    }

    // INFO: Updates search query and recalculates fuzzy match candidate list.
    pub fn update_quick_search(&mut self) {
        let root = self.explorer.tree.root_path.clone();
        let exclusions = self.explorer.tree.exclusions.clone();
        let show_hidden = self.explorer.tree.show_hidden;
        self.quick_search
            .update_search(&root, &exclusions, show_hidden);
        if let Some(win) = self.window.as_ref() {
            win.request_redraw();
        }
    }

    // INFO: Resolves the effective working directory of the current terminal or active context.
    pub fn current_working_directory(&self) -> std::path::PathBuf {
        if let Some(cwd) = self.active_working_directory() {
            return cwd;
        }
        if let Some(active_file) = &self.editor_active_file
            && let Some(parent) = active_file.parent()
        {
            return parent.to_path_buf();
        }
        self.explorer.tree.root_path.clone()
    }

    // INFO: Resolves target directory for file/folder creation based on current explorer selection or workspace root.
    pub fn selected_explorer_directory(&self) -> std::path::PathBuf {
        if let Some(focused) = &self.explorer.tree.focused_path {
            if focused.is_dir() {
                return focused.clone();
            } else if let Some(parent) = focused.parent() {
                return parent.to_path_buf();
            }
        }
        if let Some(sel) = self.explorer.tree.selected_paths.iter().next() {
            if sel.is_dir() {
                return sel.clone();
            } else if let Some(parent) = sel.parent() {
                return parent.to_path_buf();
            }
        }
        self.explorer.tree.root_path.clone()
    }
}
