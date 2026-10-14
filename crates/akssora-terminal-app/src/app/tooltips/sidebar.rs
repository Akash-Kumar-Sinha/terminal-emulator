use crate::app::App;
use akssora_terminal_config::Action;
use akssora_terminal_ui::{Sidebar, Tooltip};

// INFO: Tooltip resolution for sidebar tabs, split panes, sessions, and collapse buttons.
pub(crate) fn check_sidebar_tooltip(
    app: &App,
    mx: f32,
    my: f32,
    content_start_y: f32,
) -> Option<Tooltip> {
    let sb_target = app.sidebar.hit_test(
        mx,
        my,
        app.tab_manager.tabs(),
        &app.split_trees,
        content_start_y,
    );

    match sb_target {
        Some(akssora_terminal_ui::SidebarTarget::TabClose(close_idx)) => {
            let rect = app
                .sidebar
                .close_button_rect(close_idx, content_start_y)
                .unwrap_or((0.0, 0.0, 0.0, 0.0));
            let sc = app
                .keybindings()
                .shortcut_for(Action::CloseTab)
                .map(String::from);
            Some(Tooltip::for_target("Close Tab", sc, rect))
        }
        Some(akssora_terminal_ui::SidebarTarget::SplitClose { tab_idx, .. }) => {
            let rect = app
                .sidebar
                .close_button_rect(tab_idx, content_start_y)
                .unwrap_or((0.0, 0.0, 0.0, 0.0));
            let sc = app
                .keybindings()
                .shortcut_for(Action::CloseTab)
                .map(String::from);
            Some(Tooltip::for_target("Close Pane", sc, rect))
        }
        Some(akssora_terminal_ui::SidebarTarget::SessionClose { tab_idx, .. }) => {
            let rect = app
                .sidebar
                .close_button_rect(tab_idx, content_start_y)
                .unwrap_or((0.0, 0.0, 0.0, 0.0));
            let sc = app
                .keybindings()
                .shortcut_for(Action::CloseTab)
                .map(String::from);
            Some(Tooltip::for_target("Close Session", sc, rect))
        }
        Some(akssora_terminal_ui::SidebarTarget::NewTab) => {
            let rect = app
                .sidebar
                .new_tab_button_rect(app.tab_manager.len(), content_start_y);
            let sc = app
                .keybindings()
                .shortcut_for(Action::NewTab)
                .map(String::from);
            Some(Tooltip::for_target(Action::NewTab.description(), sc, rect))
        }
        Some(akssora_terminal_ui::SidebarTarget::Tab(tab_idx)) => {
            let rect = (
                0.0,
                content_start_y + tab_idx as f32 * Sidebar::ITEM_HEIGHT,
                app.sidebar.width,
                Sidebar::ITEM_HEIGHT,
            );
            let title = app
                .tab_manager
                .tabs()
                .get(tab_idx)
                .map(|t| t.title.as_str())
                .unwrap_or("Terminal");
            let sc = if tab_idx < 9 {
                Some(format!("Ctrl+{}", tab_idx + 1))
            } else {
                None
            };
            Some(Tooltip::for_target(format!("Tab: {}", title), sc, rect))
        }
        Some(akssora_terminal_ui::SidebarTarget::SplitPane { pane_id, .. }) => {
            let rect = (0.0, my - 10.0, app.sidebar.width, Sidebar::ITEM_HEIGHT);
            let title = app
                .blocks
                .get(&pane_id)
                .and_then(|b| b.text.as_deref())
                .unwrap_or("Terminal Split");
            Some(Tooltip::for_target(format!("Pane: {}", title), None, rect))
        }
        Some(akssora_terminal_ui::SidebarTarget::Session { tab_idx, .. }) => {
            let rect = (
                0.0,
                my - 10.0,
                app.sidebar.width,
                akssora_terminal_ui::SESSION_ITEM_HEIGHT,
            );
            let title = app
                .tab_manager
                .tabs()
                .get(tab_idx)
                .map(|t| t.title.as_str())
                .unwrap_or("Session");
            Some(Tooltip::for_target(
                format!("Session: {}", title),
                None,
                rect,
            ))
        }
        Some(akssora_terminal_ui::SidebarTarget::TabCollapse(_)) => {
            let rect = (0.0, my - 10.0, app.sidebar.width, Sidebar::ITEM_HEIGHT);
            Some(Tooltip::for_target("Toggle Collapse Tab", None, rect))
        }
        Some(akssora_terminal_ui::SidebarTarget::ProjectCollapse(_)) => {
            let rect = (0.0, my - 10.0, app.sidebar.width, Sidebar::ITEM_HEIGHT);
            Some(Tooltip::for_target("Toggle Collapse Project", None, rect))
        }
        _ => {
            if app.sidebar.hovered_toggle {
                let rect = app.sidebar.toggle_button_rect();
                let sc = app
                    .keybindings()
                    .shortcut_for(Action::ToggleSidebar)
                    .map(String::from);
                Some(Tooltip::for_target("Toggle Sidebar", sc, rect))
            } else {
                None
            }
        }
    }
}
