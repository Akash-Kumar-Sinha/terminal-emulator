use akssora_terminal_config::Action;
use akssora_terminal_ui::{HeaderButton, HeaderItem, SettingsView, Sidebar, TabKind, Tooltip};

use super::App;

// INFO: Tooltip target detection, action label lookup, and dynamic shortcut binding resolution.
impl App {
    pub(crate) fn update_tooltips(&mut self, mx: f32, my: f32, window_w: f32, _window_h: f32) {
        let show_tabs = self.config.tab_layout.includes_title_bar();
        let show_toggle = self.config.tab_layout.show_sidebar_toggle();

        let hovered_item = self.header.hit_test_item_ex(
            mx,
            my,
            window_w,
            self.tab_manager.len(),
            show_tabs,
            show_toggle,
            true,
        );

        let show_sidebar = self.config.tab_layout.show_sidebar();
        let in_sidebar_range = show_sidebar && self.sidebar.is_expanded && mx <= self.sidebar.width;
        let top_h = self.header.total_height(show_tabs);
        let content_start_y = top_h + 8.0;
        let sb_w = self.sidebar.effective_width(show_sidebar);

        let is_settings = self
            .tab_manager
            .active_tab()
            .map(|t| matches!(t.kind, TabKind::Settings))
            .unwrap_or(false);

        self.active_tooltip = match hovered_item {
            Some(HeaderItem::SidebarToggle) => {
                let rect = self.header.sidebar_toggle_rect();
                let sc = self
                    .keybindings()
                    .shortcut_for(Action::ToggleSidebar)
                    .map(String::from);
                Some(Tooltip::for_target(
                    Action::ToggleSidebar.description(),
                    sc,
                    rect,
                ))
            }
            Some(HeaderItem::NewTab) => {
                let rect = self.header.new_tab_rect(self.tab_manager.len(), window_w);
                let sc = self
                    .keybindings()
                    .shortcut_for(Action::NewTab)
                    .map(String::from);
                Some(Tooltip::for_target(Action::NewTab.description(), sc, rect))
            }
            Some(HeaderItem::Settings) => {
                let rect = self.header.settings_button_rect(window_w);
                let sc = self
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
                let rect = self.header.button_rect(HeaderButton::Minimize, window_w);
                Some(Tooltip::for_target("Minimize", None, rect))
            }
            Some(HeaderItem::WindowButton(HeaderButton::Maximize)) => {
                let rect = self.header.button_rect(HeaderButton::Maximize, window_w);
                let label = if self.header.is_maximized {
                    "Restore"
                } else {
                    "Maximize"
                };
                Some(Tooltip::for_target(label, None, rect))
            }
            Some(HeaderItem::WindowButton(HeaderButton::Close)) => {
                let rect = self.header.button_rect(HeaderButton::Close, window_w);
                Some(Tooltip::for_target("Close", None, rect))
            }
            Some(HeaderItem::TabClose(idx)) => {
                let rect = self
                    .header
                    .tab_close_rect(idx, self.tab_manager.len(), window_w)
                    .unwrap_or((0.0, 0.0, 0.0, 0.0));
                let sc = self
                    .keybindings()
                    .shortcut_for(Action::CloseTab)
                    .map(String::from);
                Some(Tooltip::for_target("Close Tab", sc, rect))
            }
            _ => {
                if in_sidebar_range {
                    let sb_target = self.sidebar.hit_test(
                        mx,
                        my,
                        self.tab_manager.tabs(),
                        &self.split_trees,
                        content_start_y,
                    );

                    match sb_target {
                        Some(akssora_terminal_ui::SidebarTarget::TabClose(close_idx)) => {
                            let rect = self
                                .sidebar
                                .close_button_rect(close_idx, content_start_y)
                                .unwrap_or((0.0, 0.0, 0.0, 0.0));
                            let sc = self
                                .keybindings()
                                .shortcut_for(Action::CloseTab)
                                .map(String::from);
                            Some(Tooltip::for_target("Close Tab", sc, rect))
                        }
                        Some(akssora_terminal_ui::SidebarTarget::SplitClose {
                            tab_idx, ..
                        }) => {
                            let rect = self
                                .sidebar
                                .close_button_rect(tab_idx, content_start_y)
                                .unwrap_or((0.0, 0.0, 0.0, 0.0));
                            let sc = self
                                .keybindings()
                                .shortcut_for(Action::CloseTab)
                                .map(String::from);
                            Some(Tooltip::for_target("Close Pane", sc, rect))
                        }
                        Some(akssora_terminal_ui::SidebarTarget::NewTab) => {
                            let rect = self
                                .sidebar
                                .new_tab_button_rect(self.tab_manager.len(), content_start_y);
                            let sc = self
                                .keybindings()
                                .shortcut_for(Action::NewTab)
                                .map(String::from);
                            Some(Tooltip::for_target(Action::NewTab.description(), sc, rect))
                        }
                        Some(akssora_terminal_ui::SidebarTarget::Tab(tab_idx)) => {
                            let rect = (
                                0.0,
                                content_start_y + tab_idx as f32 * Sidebar::ITEM_HEIGHT,
                                self.sidebar.width,
                                Sidebar::ITEM_HEIGHT,
                            );
                            let title = self
                                .tab_manager
                                .tabs()
                                .get(tab_idx)
                                .map(|t| t.title.as_str())
                                .unwrap_or("Terminal");
                            Some(Tooltip::for_target(title, None, rect))
                        }
                        Some(akssora_terminal_ui::SidebarTarget::SplitPane { pane_id, .. }) => {
                            let rect = (0.0, my - 10.0, self.sidebar.width, Sidebar::ITEM_HEIGHT);
                            let title = self
                                .blocks
                                .get(&pane_id)
                                .and_then(|b| b.text.as_deref())
                                .unwrap_or("Terminal Split");
                            Some(Tooltip::for_target(title, None, rect))
                        }
                        Some(akssora_terminal_ui::SidebarTarget::Session { tab_idx, .. }) => {
                            let rect = (
                                0.0,
                                my - 10.0,
                                self.sidebar.width,
                                akssora_terminal_ui::SESSION_ITEM_HEIGHT,
                            );
                            let title = self
                                .tab_manager
                                .tabs()
                                .get(tab_idx)
                                .map(|t| t.title.as_str())
                                .unwrap_or("Session");
                            Some(Tooltip::for_target(title, None, rect))
                        }
                        _ => None,
                    }
                } else if is_settings {
                    let cell_h = self.ui_fonts.as_ref().map(|f| f.cell_h).unwrap_or(18.0);
                    let ws_w = (window_w - sb_w).max(0.0);
                    let minus_rect =
                        SettingsView::font_minus_button_rect(sb_w, top_h, ws_w, cell_h);
                    let plus_rect = SettingsView::font_plus_button_rect(sb_w, top_h, ws_w, cell_h);
                    let reset_rect =
                        SettingsView::font_reset_button_rect(sb_w, top_h, ws_w, cell_h);
                    let dropdown_rect =
                        SettingsView::layout_dropdown_rect(sb_w, top_h, ws_w, cell_h);

                    let contains = |(rx, ry, rw, rh): (f32, f32, f32, f32)| {
                        mx >= rx && mx < rx + rw && my >= ry && my < ry + rh
                    };

                    if contains(minus_rect) {
                        let sc = self
                            .keybindings()
                            .shortcut_for(Action::DecreaseFontSize)
                            .map(String::from);
                        Some(Tooltip::for_target("Decrease Font Size", sc, minus_rect))
                    } else if contains(plus_rect) {
                        let sc = self
                            .keybindings()
                            .shortcut_for(Action::IncreaseFontSize)
                            .map(String::from);
                        Some(Tooltip::for_target("Increase Font Size", sc, plus_rect))
                    } else if contains(reset_rect) {
                        let sc = self
                            .keybindings()
                            .shortcut_for(Action::ResetFontSize)
                            .map(String::from);
                        Some(Tooltip::for_target("Reset Font Size", sc, reset_rect))
                    } else if contains(dropdown_rect) {
                        Some(Tooltip::for_target(
                            "Select Tab Layout",
                            None,
                            dropdown_rect,
                        ))
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
        };
    }
}
