use crate::app::App;
use akssora_terminal_config::Action;
use akssora_terminal_ui::{HeaderButton, HeaderItem, Tooltip};

// INFO: Tooltip resolution for top title bar buttons, window controls, and tab strip items.
pub(crate) fn check_header_tooltip(
    app: &App,
    hovered_item: Option<HeaderItem>,
    window_w: f32,
) -> Option<Tooltip> {
    match hovered_item {
        Some(HeaderItem::SidebarToggle) => {
            let rect = app.header.sidebar_toggle_rect();
            let sc = app
                .keybindings()
                .shortcut_for(Action::ToggleSidebar)
                .map(String::from);
            Some(Tooltip::for_target(
                Action::ToggleSidebar.description(),
                sc,
                rect,
            ))
        }
        Some(HeaderItem::ExplorerToggle) => {
            let rect = app.header.explorer_toggle_rect();
            let label = if app.explorer.is_expanded {
                "Collapse File Explorer"
            } else {
                "Expand File Explorer"
            };
            Some(Tooltip::for_target(
                label,
                Some("Ctrl+Shift+E".to_string()),
                rect,
            ))
        }
        Some(HeaderItem::CodeReview) => {
            let rect = app.header.code_review_button_rect(window_w);
            let label = if app.header.is_code_review_active {
                "Close Code Review"
            } else {
                "Code Review Panel"
            };
            Some(Tooltip::for_target(
                label,
                Some("Ctrl+Shift+R".to_string()),
                rect,
            ))
        }
        Some(HeaderItem::NewTab) => {
            let rect = app.header.new_tab_rect(app.tab_manager.len(), window_w);
            let sc = app
                .keybindings()
                .shortcut_for(Action::NewTab)
                .map(String::from);
            Some(Tooltip::for_target(Action::NewTab.description(), sc, rect))
        }
        Some(HeaderItem::Settings) => {
            let rect = app.header.settings_button_rect(window_w);
            let sc = app
                .keybindings()
                .shortcut_for(Action::OpenSettings)
                .map(String::from);
            Some(Tooltip::for_target(
                Action::OpenSettings.description(),
                sc,
                rect,
            ))
        }
        Some(HeaderItem::WindowButton(HeaderButton::Minimize)) => {
            let rect = app.header.button_rect(HeaderButton::Minimize, window_w);
            Some(Tooltip::for_target("Minimize", None, rect))
        }
        Some(HeaderItem::WindowButton(HeaderButton::Maximize)) => {
            let rect = app.header.button_rect(HeaderButton::Maximize, window_w);
            let label = if app.header.is_maximized {
                "Restore"
            } else {
                "Maximize"
            };
            Some(Tooltip::for_target(label, None, rect))
        }
        Some(HeaderItem::WindowButton(HeaderButton::Close)) => {
            let rect = app.header.button_rect(HeaderButton::Close, window_w);
            Some(Tooltip::for_target("Close", None, rect))
        }
        Some(HeaderItem::TabClose(idx)) => {
            let rect = app
                .header
                .tab_close_rect(idx, app.tab_manager.len(), window_w)
                .unwrap_or((0.0, 0.0, 0.0, 0.0));
            let sc = app
                .keybindings()
                .shortcut_for(Action::CloseTab)
                .map(String::from);
            Some(Tooltip::for_target("Close Tab", sc, rect))
        }
        Some(HeaderItem::Tab(idx)) => {
            if let Some(tab) = app.tab_manager.tabs().get(idx) {
                let rect = app
                    .header
                    .tab_rect(idx, app.tab_manager.len(), window_w)
                    .unwrap_or((0.0, 0.0, 0.0, 0.0));
                let sc = if idx < 9 {
                    Some(format!("Ctrl+{}", idx + 1))
                } else {
                    None
                };
                Some(Tooltip::for_target(
                    format!("Tab {}: {}", idx + 1, tab.title),
                    sc,
                    rect,
                ))
            } else {
                None
            }
        }
        _ => None,
    }
}
