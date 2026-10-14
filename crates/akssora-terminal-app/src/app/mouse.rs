use akssora_terminal_ui::TabKind;
use winit::dpi::PhysicalPosition;

use super::App;

// INFO: Mouse event processing coordinator, cursor tracking, and frame redraw requests.
impl App {
    pub(crate) fn handle_cursor_moved(&mut self, position: PhysicalPosition<f64>) {
        self.cursor_pos = (position.x as f32, position.y as f32);
        let (mx, my) = self.cursor_pos;
        let (win_w, win_h) = self.state.as_ref().map(|s| s.size()).unwrap_or((0, 0));
        let w = win_w as f32;
        let h = win_h as f32;

        let show_tabs = self.config.tab_layout.includes_title_bar();
        let show_toggle = self.config.tab_layout.show_sidebar_toggle();
        let show_sidebar = self.config.tab_layout.show_sidebar();
        let top_h = self.header.total_height(show_tabs);
        let sb_w = self.sidebar.effective_width(show_sidebar);
        let exp_w = self.explorer.effective_width();
        let ws_x = sb_w + exp_w;

        let is_code_review = self
            .tab_manager
            .active_tab()
            .map(|t| matches!(t.kind, TabKind::CodeReview { .. }))
            .unwrap_or(false);
        let is_editor = self
            .tab_manager
            .active_tab()
            .map(|t| matches!(t.kind, TabKind::Editor { .. }))
            .unwrap_or(false)
            || (is_code_review && self.code_review.active_editing_file.is_some());
        let tab_bar_h = if !self.editor_open_files.is_empty() {
            akssora_terminal_ui::EditorTabBar::HEIGHT
        } else {
            0.0
        };

        // 1. Update all hover states, tooltips, and resize edges
        self.update_hover_states(
            mx,
            my,
            w,
            h,
            top_h,
            sb_w,
            exp_w,
            show_sidebar,
            show_tabs,
            show_toggle,
        );

        // 2. Handle active dragging actions (split dividers, scrollbars, selections, pane rearrangement)
        self.handle_mouse_drags(mx, my, w, h, is_editor, top_h, tab_bar_h, ws_x);

        // 3. Determine and apply appropriate cursor icon
        let desired_cursor = self.determine_cursor_icon(mx, my, w, h, top_h, sb_w);

        if let Some(window) = self.window.as_ref() {
            window.set_cursor(desired_cursor);
            window.request_redraw();
        }
    }
}
