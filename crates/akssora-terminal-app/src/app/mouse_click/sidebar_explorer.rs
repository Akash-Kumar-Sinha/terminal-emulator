use crate::app::App;

impl App {
    pub(crate) fn handle_sidebar_explorer_clicks(
        &mut self,
        mx: f32,
        my: f32,
        top_h: f32,
        content_start_y: f32,
        sb_w: f32,
        show_sidebar: bool,
    ) -> bool {
        // 3. Sidebar resize dragging and interaction targets
        if show_sidebar && self.sidebar.is_expanded {
            if self.sidebar.hovered_edge {
                self.sidebar.is_resizing = true;
                return true;
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
                    return true;
                }
            }
        }

        // INFO: Explorer resize dragging
        if self.explorer.is_expanded && self.explorer.hovered_edge {
            self.explorer.is_resizing = true;
            return true;
        }

        // INFO: File Explorer click interactions
        if self.explorer.is_expanded
            && mx >= sb_w
            && mx <= sb_w + self.explorer.width
            && my >= top_h
            && let Some(target) = self.explorer.hit_test_at(mx, my, top_h, sb_w)
        {
            match target {
                akssora_terminal_ui::ExplorerTarget::Toolbar(act) => match act {
                    akssora_terminal_ui::ExplorerToolbarAction::ToggleHidden => {
                        self.explorer.tree.toggle_hidden();
                    }
                    akssora_terminal_ui::ExplorerToolbarAction::NewFolder => {
                        let parent = self.selected_explorer_directory();
                        self.explorer.inline_input = Some(akssora_terminal_ui::InlineInputState {
                            mode: akssora_terminal_ui::InlineInputMode::NewFolder,
                            text: String::new(),
                            parent_dir: parent,
                            error: None,
                        });
                    }
                    akssora_terminal_ui::ExplorerToolbarAction::NewFile => {
                        let parent = self.selected_explorer_directory();
                        self.explorer.inline_input = Some(akssora_terminal_ui::InlineInputState {
                            mode: akssora_terminal_ui::InlineInputMode::NewFile,
                            text: String::new(),
                            parent_dir: parent,
                            error: None,
                        });
                    }
                    akssora_terminal_ui::ExplorerToolbarAction::ChangeRoot => {
                        self.explorer.inline_input = Some(akssora_terminal_ui::InlineInputState {
                            mode: akssora_terminal_ui::InlineInputMode::ChangeRoot,
                            text: self.explorer.tree.root_path.to_string_lossy().to_string(),
                            parent_dir: self.explorer.tree.root_path.clone(),
                            error: None,
                        });
                    }
                },
                akssora_terminal_ui::ExplorerTarget::RowToggle(path) => {
                    self.explorer.tree.toggle_expand(&path);
                }
                akssora_terminal_ui::ExplorerTarget::RowItem { path, kind, .. } => {
                    if self.modifiers.control_key() {
                        self.explorer.tree.toggle_select(path.clone());
                    } else if self.modifiers.shift_key() {
                        self.explorer.tree.select_range(&path);
                    } else {
                        self.explorer.tree.select_single(path.clone());
                        if kind == akssora_file_explorer::EntryKind::Directory {
                            self.explorer.tree.toggle_expand(&path);
                        } else if self.modifiers.alt_key() {
                            self.open_file_in_split_pane(&path);
                        } else {
                            self.open_editor_file_with_preview(&path, true);
                        }
                    }
                    self.explorer.tree.start_drag(path);
                }
                _ => {}
            }
            if let Some(win) = self.window.as_ref() {
                win.request_redraw();
            }
            return true;
        }

        false
    }
}
