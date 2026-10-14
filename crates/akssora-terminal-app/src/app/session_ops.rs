use std::path::PathBuf;

use akssora_terminal_ui::{SessionRowData, TabKind};

use super::App;
use super::session_persist::{PersistedPane, PersistedState};
use super::session_tracker::{
    detect_git_info, inspect_process_state, resolve_session_name, shorten_path,
};

// INFO: Terminal session introspection, project metadata collection, and persistence operations.
impl App {
    pub fn collect_session_items(&self) -> Vec<SessionRowData> {
        let mut items = Vec::new();

        for (tab_idx, tab) in self.tab_manager.tabs().iter().enumerate() {
            let is_tab_active = self.tab_manager.active_index() == tab_idx;

            match &tab.kind {
                TabKind::Settings => {
                    items.push(SessionRowData {
                        pane_id: tab.id,
                        tab_id: tab.id,
                        tab_index: tab_idx,
                        name: tab.title.clone(),
                        secondary_meta: "System Preferences".to_string(),
                        project_name: String::new(),
                        is_active: is_tab_active,
                        icon: tab.kind.icon(),
                    });
                }
                TabKind::PluginManagement => {
                    items.push(SessionRowData {
                        pane_id: tab.id,
                        tab_id: tab.id,
                        tab_index: tab_idx,
                        name: tab.title.clone(),
                        secondary_meta: "Plugins & Extensions".to_string(),
                        project_name: String::new(),
                        is_active: is_tab_active,
                        icon: tab.kind.icon(),
                    });
                }
                TabKind::CodeReview { repo_path } => {
                    items.push(SessionRowData {
                        pane_id: tab.id,
                        tab_id: tab.id,
                        tab_index: tab_idx,
                        name: tab.title.clone(),
                        secondary_meta: format!("Code Review • {repo_path}"),
                        project_name: String::new(),
                        is_active: is_tab_active,
                        icon: tab.kind.icon(),
                    });
                }
                TabKind::Editor { path } => {
                    let short_path = shorten_path(std::path::Path::new(path));
                    items.push(SessionRowData {
                        pane_id: tab.id,
                        tab_id: tab.id,
                        tab_index: tab_idx,
                        name: tab.title.clone(),
                        secondary_meta: short_path,
                        project_name: String::new(),
                        is_active: is_tab_active,
                        icon: tab.kind.icon(),
                    });
                }
                TabKind::ApiTesting { workspace } => {
                    items.push(SessionRowData {
                        pane_id: tab.id,
                        tab_id: tab.id,
                        tab_index: tab_idx,
                        name: tab.title.clone(),
                        secondary_meta: workspace.clone(),
                        project_name: String::new(),
                        is_active: is_tab_active,
                        icon: tab.kind.icon(),
                    });
                }
                TabKind::Terminal => {
                    let leaf_ids = if let Some(tree) = self.split_trees.get(&tab.id) {
                        tree.all_leaf_ids()
                    } else {
                        vec![tab.id]
                    };

                    for (leaf_idx, &pane_id) in leaf_ids.iter().enumerate() {
                        let Some(block) = self.blocks.get(&pane_id) else {
                            continue;
                        };

                        let shell_pid = block.shell.process_id();
                        let (proc_cwd, active_cmd) = inspect_process_state(shell_pid);

                        let effective_cwd = proc_cwd
                            .or_else(|| block.shell.osc_cwd().map(PathBuf::from))
                            .unwrap_or_else(|| std::env::current_dir().unwrap_or_default());

                        let (project_name, git_branch) = detect_git_info(&effective_cwd);

                        let custom_name =
                            self.pane_custom_names.get(&pane_id).map(|s| s.as_str()).or(
                                if tab.custom_name && pane_id == tab.id {
                                    Some(tab.title.as_str())
                                } else {
                                    None
                                },
                            );

                        let last_cmd = self
                            .last_executed_commands
                            .get(&pane_id)
                            .map(|s| s.as_str());
                        let osc_title = block.shell.osc_title();

                        let resolved_name = resolve_session_name(
                            custom_name,
                            active_cmd.as_deref(),
                            osc_title,
                            last_cmd,
                            &project_name,
                            leaf_idx + 1,
                        );

                        let short_cwd = shorten_path(&effective_cwd);
                        let secondary_meta = if let Some(branch) = &git_branch {
                            format!("{short_cwd} \u{00B7} {branch}")
                        } else {
                            short_cwd
                        };

                        let is_pane_active = if leaf_ids.len() > 1 {
                            is_tab_active && self.split_manager.active_pane_id == Some(pane_id)
                        } else {
                            is_tab_active
                        };

                        items.push(SessionRowData {
                            pane_id,
                            tab_id: tab.id,
                            tab_index: tab_idx,
                            name: resolved_name,
                            secondary_meta,
                            project_name,
                            is_active: is_pane_active,
                            icon: tab.kind.icon(),
                        });
                    }
                }
            }
        }

        items
    }

    pub fn active_working_directory(&self) -> Option<PathBuf> {
        let active_pane = self.active_pane_id()?;
        let block = self.blocks.get(&active_pane)?;
        let (proc_cwd, _) = inspect_process_state(block.shell.process_id());
        proc_cwd
            .or_else(|| block.shell.osc_cwd().map(PathBuf::from))
            .or_else(|| std::env::current_dir().ok())
    }

    pub fn rename_pane(&mut self, pane_id: u64, new_name: impl Into<String>) {
        let name_str = new_name.into();
        self.pane_custom_names.insert(pane_id, name_str.clone());
        let _ = self.rename_tab(pane_id, name_str);
        self.save_session_state();
    }

    pub fn save_session_state(&self) {
        let mut panes = Vec::new();
        for tab in self.tab_manager.tabs().iter() {
            if !matches!(tab.kind, TabKind::Terminal) {
                continue;
            }
            let leaf_ids = if let Some(tree) = self.split_trees.get(&tab.id) {
                tree.all_leaf_ids()
            } else {
                vec![tab.id]
            };
            for &pane_id in &leaf_ids {
                if let Some(block) = self.blocks.get(&pane_id) {
                    let (proc_cwd, _) = inspect_process_state(block.shell.process_id());
                    let cwd = proc_cwd
                        .or_else(|| block.shell.osc_cwd().map(PathBuf::from))
                        .unwrap_or_else(|| std::env::current_dir().unwrap_or_default());

                    let custom_name = self.pane_custom_names.contains_key(&pane_id)
                        || (tab.custom_name && pane_id == tab.id);

                    let name = self
                        .pane_custom_names
                        .get(&pane_id)
                        .cloned()
                        .unwrap_or_else(|| tab.title.clone());

                    panes.push(PersistedPane {
                        pane_id,
                        tab_id: tab.id,
                        name,
                        custom_name,
                        cwd,
                    });
                }
            }
        }

        let state = PersistedState {
            panes,
            active_tab_id: self.tab_manager.active_id(),
        };
        let path = self
            .session_file
            .clone()
            .unwrap_or_else(PersistedState::default_path);
        let _ = state.save_to_file(&path);
    }

    pub fn restore_session_state(&mut self) -> bool {
        let path = self
            .session_file
            .clone()
            .unwrap_or_else(PersistedState::default_path);
        let Some(saved) = PersistedState::load_from_file(&path) else {
            return false;
        };

        if saved.panes.is_empty() {
            return false;
        }

        // Group panes by tab_id
        let mut tab_groups: Vec<(u64, Vec<PersistedPane>)> = Vec::new();
        for pane in saved.panes {
            if let Some(group) = tab_groups.iter_mut().find(|(t, _)| *t == pane.tab_id) {
                group.1.push(pane);
            } else {
                tab_groups.push((pane.tab_id, vec![pane]));
            }
        }

        for (_, panes) in tab_groups {
            let first = &panes[0];
            let tab_id = self.open_terminal_tab_in(Some(&first.cwd));
            if first.custom_name {
                self.rename_pane(tab_id, &first.name);
            }

            for split_pane in panes.iter().skip(1) {
                let split_id = self.split_active_pane_in(
                    akssora_terminal_ui::SplitDirection::Vertical,
                    Some(&split_pane.cwd),
                );
                if let Some(sid) = split_id
                    && split_pane.custom_name
                {
                    self.rename_pane(sid, &split_pane.name);
                }
            }
        }

        if let Some(target_tab) = saved.active_tab_id {
            self.tab_manager.select_by_id(target_tab);
        }
        self.layout_blocks();
        true
    }

    pub fn handle_sidebar_target(&mut self, target: akssora_terminal_ui::SidebarTarget) {
        use akssora_terminal_ui::SidebarTarget;
        match target {
            SidebarTarget::Tab(tab_idx) => {
                self.tab_manager.select(tab_idx);
                self.sync_active_tab_state();
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
            }
            SidebarTarget::TabCollapse(tab_idx) => {
                if let Some(tab) = self.tab_manager.tabs().get(tab_idx) {
                    self.sidebar.toggle_tab_collapse(tab.id);
                    if let Some(win) = self.window.as_ref() {
                        win.request_redraw();
                    }
                }
            }
            SidebarTarget::TabClose(close_idx) => {
                self.close_tab(close_idx);
                if let Some(win) = self.window.as_ref() {
                    win.request_redraw();
                }
            }
            SidebarTarget::SplitPane { tab_idx, pane_id }
            | SidebarTarget::Session { tab_idx, pane_id } => {
                self.tab_manager.select(tab_idx);
                self.sync_active_tab_state();
                self.split_manager.focus_pane(pane_id);
                self.layout_blocks();
                if let Some(win) = self.window.as_ref() {
                    win.request_redraw();
                }
            }
            SidebarTarget::SplitClose { pane_id, .. } => {
                self.close_split_pane(pane_id);
            }
            SidebarTarget::SessionClose { tab_idx, pane_id } => {
                let tab_id = self
                    .tab_manager
                    .tabs()
                    .get(tab_idx)
                    .map(|t| t.id)
                    .unwrap_or(0);
                let is_split = self
                    .split_trees
                    .get(&tab_id)
                    .map(|t| t.contains_leaf(pane_id) && t.all_leaf_ids().len() > 1)
                    .unwrap_or(false);
                if is_split {
                    self.close_split_pane(pane_id);
                } else {
                    self.close_tab(tab_idx);
                }
                if let Some(win) = self.window.as_ref() {
                    win.request_redraw();
                }
            }
            SidebarTarget::ProjectCollapse(project_name) => {
                self.sidebar.toggle_project_collapse(&project_name);
                if let Some(win) = self.window.as_ref() {
                    win.request_redraw();
                }
            }
            SidebarTarget::NewTab => {
                let active_cwd = self.active_working_directory();
                self.open_terminal_tab_in(active_cwd.as_deref());
                if let Some(win) = self.window.as_ref() {
                    win.request_redraw();
                }
            }
        }
    }
}
