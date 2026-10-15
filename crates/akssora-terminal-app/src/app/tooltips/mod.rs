pub mod code_review;
pub(crate) mod editor;
pub(crate) mod explorer;
pub(crate) mod header;
pub(crate) mod modals;
pub(crate) mod settings;
pub(crate) mod sidebar;

use super::App;
use akssora_terminal_ui::TabKind;

// INFO: Tooltip target detection, action label lookup, and dynamic shortcut binding resolution.
impl App {
    pub(crate) fn update_tooltips(&mut self, mx: f32, my: f32, window_w: f32, window_h: f32) {
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
        let exp_w = self.explorer.effective_width();

        let is_settings = self
            .tab_manager
            .active_tab()
            .map(|t| matches!(t.kind, TabKind::Settings))
            .unwrap_or(false);

        let is_editor = self
            .tab_manager
            .active_tab()
            .map(|t| matches!(t.kind, TabKind::Editor { .. }))
            .unwrap_or(false);

        let is_code_review = self.is_code_review_open();

        // 1. Header Buttons & Tabs
        self.active_tooltip = header::check_header_tooltip(self, hovered_item, window_w);

        // 2. Sidebar buttons and items
        if self.active_tooltip.is_none() && in_sidebar_range {
            self.active_tooltip = sidebar::check_sidebar_tooltip(self, mx, my, content_start_y);
        }

        // 3. Settings View buttons
        if self.active_tooltip.is_none() && is_settings {
            self.active_tooltip =
                settings::check_settings_tooltip(self, mx, my, sb_w, top_h, window_w);
        }

        // 4. File Explorer buttons & rows
        if self.active_tooltip.is_none() {
            self.active_tooltip = explorer::check_explorer_tooltip(self, mx, my, sb_w, exp_w);
        }

        // 5. Markdown Editor Toolbar & Tabs
        if self.active_tooltip.is_none() && is_editor {
            self.active_tooltip =
                editor::check_editor_tooltip(self, mx, my, sb_w, exp_w, top_h, window_w);
        }

        // 6. Code Review Panel buttons
        if self.active_tooltip.is_none() && is_code_review {
            self.active_tooltip = code_review::check_code_review_tooltip(
                self, mx, my, sb_w, exp_w, window_w, window_h,
            );
        }

        // 7. Modals (Confirm Exit & Unsaved Changes)
        if let Some(modal_tip) = modals::check_modal_tooltip(self, mx, my, window_w, window_h) {
            self.active_tooltip = Some(modal_tip);
        }
    }
}
