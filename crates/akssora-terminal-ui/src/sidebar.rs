use std::collections::{HashMap, HashSet};

use akssora_terminal_config::Theme;

use crate::{PaneTree, Tab};

// INFO: Target interactable elements inside the sidebar navigation panel.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SidebarTarget {
    Tab(usize),
    TabCollapse(usize),
    TabClose(usize),
    SplitPane { tab_idx: usize, pane_id: u64 },
    SplitClose { tab_idx: usize, pane_id: u64 },
    NewTab,
    ProjectCollapse(String),
    Session { tab_idx: usize, pane_id: u64 },
    SessionClose { tab_idx: usize, pane_id: u64 },
}

// INFO: Structured row item metadata for rendering sidebar tree nodes.
#[derive(Clone, Debug, PartialEq)]
pub struct SidebarRow {
    pub tab_index: usize,
    pub tab_id: u64,
    pub kind: SidebarRowKind,
    pub y: f32,
    pub height: f32,
}

#[derive(Clone, Debug, PartialEq)]
pub enum SidebarRowKind {
    NormalTab,
    GroupHeader {
        is_collapsed: bool,
        split_count: usize,
    },
    SplitChild {
        pane_id: u64,
        child_index: usize,
        is_last: bool,
    },
    ProjectHeader {
        project_name: String,
        session_count: usize,
        is_collapsed: bool,
    },
    Session {
        pane_id: u64,
        session_name: String,
        secondary_meta: String,
        is_split: bool,
        icon: char,
    },
}

// INFO: Sidebar view component rendering vertical tab list, toggle controls, and resource navigation.
#[derive(Clone, Debug)]
pub struct Sidebar {
    pub width: f32,
    pub saved_width: f32,
    pub is_expanded: bool,
    pub is_resizing: bool,
    pub hovered_edge: bool,
    pub hovered_toggle: bool,
    pub hovered_tab: Option<usize>,
    pub hovered_close: Option<usize>,
    pub hovered_new_tab: bool,
    pub hovered_pane: Option<u64>,
    pub hovered_pane_close: Option<u64>,
    pub hovered_collapse_tab: Option<usize>,
    pub collapsed_tabs: HashSet<u64>,
    pub collapsed_projects: HashSet<String>,
}

impl Default for Sidebar {
    fn default() -> Self {
        Self::new()
    }
}

impl Sidebar {
    pub const DEFAULT_WIDTH: f32 = 240.0;
    pub const MIN_WIDTH: f32 = 160.0;
    pub const MAX_WIDTH: f32 = 480.0;
    pub const EXPANDED_WIDTH: f32 = Self::DEFAULT_WIDTH;
    pub const ITEM_HEIGHT: f32 = 32.0;
    pub const TOP_HEADER_HEIGHT: f32 = 38.0;

    pub fn new() -> Self {
        Self {
            width: Self::DEFAULT_WIDTH,
            saved_width: Self::DEFAULT_WIDTH,
            is_expanded: true,
            is_resizing: false,
            hovered_edge: false,
            hovered_toggle: false,
            hovered_tab: None,
            hovered_close: None,
            hovered_new_tab: false,
            hovered_pane: None,
            hovered_pane_close: None,
            hovered_collapse_tab: None,
            collapsed_tabs: HashSet::new(),
            collapsed_projects: HashSet::new(),
        }
    }

    pub fn toggle(&mut self) {
        self.is_expanded = !self.is_expanded;
        if self.is_expanded {
            self.width = if self.saved_width >= Self::MIN_WIDTH {
                self.saved_width
            } else {
                Self::DEFAULT_WIDTH
            };
        } else {
            if self.width >= Self::MIN_WIDTH {
                self.saved_width = self.width;
            }
            self.width = 0.0;
        }
    }

    pub fn toggle_tab_collapse(&mut self, tab_id: u64) {
        if self.collapsed_tabs.contains(&tab_id) {
            self.collapsed_tabs.remove(&tab_id);
        } else {
            self.collapsed_tabs.insert(tab_id);
        }
    }

    pub fn toggle_project_collapse(&mut self, project: &str) {
        if self.collapsed_projects.contains(project) {
            self.collapsed_projects.remove(project);
        } else {
            self.collapsed_projects.insert(project.to_string());
        }
    }

    pub fn toggle_button_rect(&self) -> (f32, f32, f32, f32) {
        let btn_w = if self.is_expanded { 36.0 } else { 0.0 };
        (0.0, 0.0, btn_w, Self::TOP_HEADER_HEIGHT)
    }

    pub fn hit_test_toggle(&self, mx: f32, my: f32) -> bool {
        if !self.is_expanded {
            return false;
        }
        let (bx, by, bw, bh) = self.toggle_button_rect();
        mx >= bx && mx < bx + bw && my >= by && my < by + bh
    }

    pub fn effective_width(&self, enabled: bool) -> f32 {
        if enabled && self.is_expanded {
            self.width
        } else {
            0.0
        }
    }

    // INFO: Computes layout rows for all tabs and their nested split pane groups.
    pub fn compute_rows(
        &self,
        tabs: &[Tab],
        split_trees: &HashMap<u64, PaneTree>,
        top_offset: f32,
    ) -> Vec<SidebarRow> {
        let mut rows = Vec::new();
        let mut current_y = top_offset;

        for (tab_idx, tab) in tabs.iter().enumerate() {
            let split_leaves = split_trees
                .get(&tab.id)
                .map(|t| t.all_leaf_ids())
                .unwrap_or_default();

            if split_leaves.len() > 1 {
                let is_collapsed = self.collapsed_tabs.contains(&tab.id);
                rows.push(SidebarRow {
                    tab_index: tab_idx,
                    tab_id: tab.id,
                    kind: SidebarRowKind::GroupHeader {
                        is_collapsed,
                        split_count: split_leaves.len(),
                    },
                    y: current_y,
                    height: Self::ITEM_HEIGHT,
                });
                current_y += Self::ITEM_HEIGHT;

                if !is_collapsed {
                    let total = split_leaves.len();
                    for (child_idx, &pane_id) in split_leaves.iter().enumerate() {
                        let is_last = child_idx == total - 1;
                        rows.push(SidebarRow {
                            tab_index: tab_idx,
                            tab_id: tab.id,
                            kind: SidebarRowKind::SplitChild {
                                pane_id,
                                child_index: child_idx,
                                is_last,
                            },
                            y: current_y,
                            height: Self::ITEM_HEIGHT,
                        });
                        current_y += Self::ITEM_HEIGHT;
                    }
                }
            } else {
                rows.push(SidebarRow {
                    tab_index: tab_idx,
                    tab_id: tab.id,
                    kind: SidebarRowKind::NormalTab,
                    y: current_y,
                    height: Self::ITEM_HEIGHT,
                });
                current_y += Self::ITEM_HEIGHT;
            }
        }

        rows
    }

    pub fn compute_session_rows(
        &self,
        sessions: &[crate::sidebar_session::SessionRowData],
        top_offset: f32,
    ) -> Vec<SidebarRow> {
        crate::sidebar_session::compute_split_session_rows(
            sessions,
            &self.collapsed_tabs,
            top_offset,
        )
    }

    pub fn new_tab_button_rect_from_rows(
        &self,
        rows: &[SidebarRow],
        top_offset: f32,
    ) -> (f32, f32, f32, f32) {
        let y = rows
            .last()
            .map(|r| r.y + r.height + 4.0)
            .unwrap_or(top_offset + 4.0);
        (0.0, y, self.width, Self::ITEM_HEIGHT)
    }

    pub fn hit_test(
        &self,
        mx: f32,
        my: f32,
        tabs: &[Tab],
        split_trees: &HashMap<u64, PaneTree>,
        top_offset: f32,
    ) -> Option<SidebarTarget> {
        if !self.is_expanded || mx < 0.0 || mx > self.width {
            return None;
        }

        let rows = self.compute_rows(tabs, split_trees, top_offset);
        let btn_size = 18.0;

        for row in &rows {
            if my >= row.y && my < row.y + row.height {
                let close_x = self.width - btn_size - 8.0;
                let close_y = row.y + (row.height - btn_size) / 2.0;
                let in_close = mx >= close_x
                    && mx < close_x + btn_size
                    && my >= close_y
                    && my < close_y + btn_size;

                match &row.kind {
                    SidebarRowKind::NormalTab => {
                        if in_close {
                            return Some(SidebarTarget::TabClose(row.tab_index));
                        }
                        return Some(SidebarTarget::Tab(row.tab_index));
                    }
                    SidebarRowKind::GroupHeader { .. } => {
                        if (4.0..=24.0).contains(&mx) {
                            return Some(SidebarTarget::TabCollapse(row.tab_index));
                        }
                        if in_close {
                            return Some(SidebarTarget::TabClose(row.tab_index));
                        }
                        return Some(SidebarTarget::Tab(row.tab_index));
                    }
                    SidebarRowKind::SplitChild { pane_id, .. } => {
                        if in_close {
                            return Some(SidebarTarget::SplitClose {
                                tab_idx: row.tab_index,
                                pane_id: *pane_id,
                            });
                        }
                        return Some(SidebarTarget::SplitPane {
                            tab_idx: row.tab_index,
                            pane_id: *pane_id,
                        });
                    }
                    SidebarRowKind::ProjectHeader { project_name, .. } => {
                        return Some(SidebarTarget::ProjectCollapse(project_name.clone()));
                    }
                    SidebarRowKind::Session { pane_id, .. } => {
                        if in_close {
                            return Some(SidebarTarget::SessionClose {
                                tab_idx: row.tab_index,
                                pane_id: *pane_id,
                            });
                        }
                        return Some(SidebarTarget::Session {
                            tab_idx: row.tab_index,
                            pane_id: *pane_id,
                        });
                    }
                }
            }
        }

        let (nx, ny, nw, nh) = self.new_tab_button_rect_from_rows(&rows, top_offset);
        if mx >= nx && mx < nx + nw && my >= ny && my < ny + nh {
            return Some(SidebarTarget::NewTab);
        }

        None
    }

    pub fn tab_rect(&self, idx: usize, top_offset: f32) -> (f32, f32, f32, f32) {
        let y = top_offset + idx as f32 * Self::ITEM_HEIGHT;
        (0.0, y, self.width, Self::ITEM_HEIGHT)
    }

    pub fn close_button_rect(&self, idx: usize, top_offset: f32) -> Option<(f32, f32, f32, f32)> {
        if !self.is_expanded {
            return None;
        }
        let (x, y, w, h) = self.tab_rect(idx, top_offset);
        let btn_size = 18.0;
        let bx = x + w - btn_size - 8.0;
        let by = y + (h - btn_size) / 2.0;
        Some((bx, by, btn_size, btn_size))
    }

    pub fn new_tab_button_rect(&self, tab_count: usize, top_offset: f32) -> (f32, f32, f32, f32) {
        let y = top_offset + tab_count as f32 * Self::ITEM_HEIGHT + 4.0;
        (0.0, y, self.width, Self::ITEM_HEIGHT)
    }

    pub fn hit_test_sessions(
        &self,
        mx: f32,
        my: f32,
        sessions: &[crate::sidebar_session::SessionRowData],
        top_offset: f32,
    ) -> Option<SidebarTarget> {
        if !self.is_expanded || mx < 0.0 || mx > self.width {
            return None;
        }
        let rows = self.compute_session_rows(sessions, top_offset);
        let target = crate::sidebar_session::hit_test_project_rows(&rows, mx, my, self.width);
        if target.is_some() {
            return target;
        }
        let (nx, ny, nw, nh) = self.new_tab_button_rect_from_rows(&rows, top_offset);
        if mx >= nx && mx < nx + nw && my >= ny && my < ny + nh {
            return Some(SidebarTarget::NewTab);
        }
        None
    }

    pub fn bg_color(&self, theme: &Theme) -> [f32; 4] {
        theme.sidebar_background
    }

    pub fn border_color(&self, theme: &Theme) -> [f32; 4] {
        theme.border_subtle
    }
}
