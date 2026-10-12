use std::collections::{HashMap, HashSet};

use crate::sidebar::{SidebarRow, SidebarRowKind, SidebarTarget};

// INFO: Snapshot representation of an active terminal session for sidebar display.
#[derive(Clone, Debug, PartialEq)]
pub struct SessionRowData {
    pub pane_id: u64,
    pub tab_id: u64,
    pub tab_index: usize,
    pub name: String,
    pub secondary_meta: String,
    pub project_name: String,
    pub is_active: bool,
    pub icon: char,
}

pub const SESSION_ITEM_HEIGHT: f32 = 44.0;
pub const PROJECT_HEADER_HEIGHT: f32 = 28.0;

// INFO: Organizes sessions into split-grouped sidebar rows with headers for multi-pane layouts.
pub fn compute_split_session_rows(
    sessions: &[SessionRowData],
    collapsed_tabs: &HashSet<u64>,
    top_offset: f32,
) -> Vec<SidebarRow> {
    let mut rows = Vec::new();
    let mut current_y = top_offset;

    // Group sessions by parent tab while preserving tab order
    let mut tab_order = Vec::new();
    let mut tab_map: HashMap<u64, Vec<&SessionRowData>> = HashMap::new();

    for session in sessions {
        if !tab_map.contains_key(&session.tab_id) {
            tab_order.push(session.tab_id);
        }
        tab_map.entry(session.tab_id).or_default().push(session);
    }

    for tab_id in tab_order {
        let Some(tab_sessions) = tab_map.get(&tab_id) else {
            continue;
        };

        if tab_sessions.len() > 1 {
            let is_collapsed = collapsed_tabs.contains(&tab_id);
            let first_session = tab_sessions[0];

            // 1. Group header row for split-view layout
            rows.push(SidebarRow {
                tab_index: first_session.tab_index,
                tab_id,
                kind: SidebarRowKind::GroupHeader {
                    is_collapsed,
                    split_count: tab_sessions.len(),
                },
                y: current_y,
                height: PROJECT_HEADER_HEIGHT,
            });
            current_y += PROJECT_HEADER_HEIGHT;

            // 2. Split child sessions if not collapsed
            if !is_collapsed {
                for session in tab_sessions {
                    rows.push(SidebarRow {
                        tab_index: session.tab_index,
                        tab_id: session.tab_id,
                        kind: SidebarRowKind::Session {
                            pane_id: session.pane_id,
                            session_name: session.name.clone(),
                            secondary_meta: session.secondary_meta.clone(),
                            is_split: true,
                            icon: session.icon,
                        },
                        y: current_y,
                        height: SESSION_ITEM_HEIGHT,
                    });
                    current_y += SESSION_ITEM_HEIGHT;
                }
            }
        } else if let Some(session) = tab_sessions.first() {
            // Independent terminal entry outside split view: no group wrapper
            rows.push(SidebarRow {
                tab_index: session.tab_index,
                tab_id: session.tab_id,
                kind: SidebarRowKind::Session {
                    pane_id: session.pane_id,
                    session_name: session.name.clone(),
                    secondary_meta: session.secondary_meta.clone(),
                    is_split: false,
                    icon: session.icon,
                },
                y: current_y,
                height: SESSION_ITEM_HEIGHT,
            });
            current_y += SESSION_ITEM_HEIGHT;
        }
    }

    rows
}

// INFO: Compatibility wrapper routing to split-grouped sidebar organization.
pub fn compute_project_rows(
    sessions: &[SessionRowData],
    collapsed_tabs: &HashSet<String>,
    top_offset: f32,
) -> Vec<SidebarRow> {
    let numeric_collapsed: HashSet<u64> = collapsed_tabs
        .iter()
        .filter_map(|s| s.parse::<u64>().ok())
        .collect();
    compute_split_session_rows(sessions, &numeric_collapsed, top_offset)
}

// INFO: Hit tests coordinate against split-grouped and independent session rows to identify interaction targets.
pub fn hit_test_session_rows(
    rows: &[SidebarRow],
    mx: f32,
    my: f32,
    sidebar_w: f32,
) -> Option<SidebarTarget> {
    let btn_size = 18.0;

    for row in rows {
        if my >= row.y && my < row.y + row.height {
            let close_x = sidebar_w - btn_size - 8.0;
            let close_y = row.y + (row.height - btn_size) / 2.0;
            let in_close = mx >= close_x
                && mx < close_x + btn_size
                && my >= close_y
                && my < close_y + btn_size;

            match &row.kind {
                SidebarRowKind::GroupHeader { .. } => {
                    if (4.0..=28.0).contains(&mx) {
                        return Some(SidebarTarget::TabCollapse(row.tab_index));
                    }
                    if in_close {
                        return Some(SidebarTarget::TabClose(row.tab_index));
                    }
                    return Some(SidebarTarget::Tab(row.tab_index));
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
                SidebarRowKind::NormalTab => {
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
            }
        }
    }

    None
}

// INFO: Compatibility alias for hit testing session rows.
pub fn hit_test_project_rows(
    rows: &[SidebarRow],
    mx: f32,
    my: f32,
    sidebar_w: f32,
) -> Option<SidebarTarget> {
    hit_test_session_rows(rows, mx, my, sidebar_w)
}
