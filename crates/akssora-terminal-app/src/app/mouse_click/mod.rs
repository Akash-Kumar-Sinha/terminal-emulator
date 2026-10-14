use winit::event::{ElementState, MouseButton};
use winit::event_loop::ActiveEventLoop;

use super::App;

mod editor;
mod header;
mod modals;
mod pane_terminal;
mod settings;
mod sidebar_explorer;

// INFO: Mouse click dispatching across modals, window controls, sidebar tree items, and panes.
impl App {
    pub(crate) fn handle_mouse_input(
        &mut self,
        event_loop: &ActiveEventLoop,
        state: ElementState,
        button: MouseButton,
    ) {
        let (mx, my) = self.cursor_pos;
        let (win_w, win_h) = self.state.as_ref().map(|s| s.size()).unwrap_or((0, 0));
        let w = win_w as f32;
        let h = win_h as f32;
        let show_tabs = self.config.tab_layout.includes_title_bar();
        let show_toggle = self.config.tab_layout.show_sidebar_toggle();
        let show_sidebar = self.config.tab_layout.show_sidebar();
        let top_h = self.header.total_height(show_tabs);
        let content_start_y = top_h + 8.0;
        let sb_w = self.sidebar.effective_width(show_sidebar);

        // Right-click: trigger explorer context menu or terminal context menu
        if button == MouseButton::Right {
            if state == ElementState::Pressed {
                let _ = self.handle_right_click(mx, my, top_h, sb_w);
            }
            return;
        }

        if button != MouseButton::Left {
            return;
        }

        // Drag release
        if state == ElementState::Released {
            self.handle_drag_release(mx, my, w, sb_w);
            return;
        }

        // Pressed interactions
        if state == ElementState::Pressed {
            // 0. Modals, search overlay, and context menus
            if self.handle_modal_and_overlay_clicks(event_loop, mx, my, w, h, top_h) {
                return;
            }

            // 1. Header and window controls
            if self.handle_header_clicks(event_loop, mx, my, w, show_tabs, show_toggle) {
                return;
            }

            // 2. Sidebar and Explorer clicks
            if self.handle_sidebar_explorer_clicks(
                mx,
                my,
                top_h,
                content_start_y,
                sb_w,
                show_sidebar,
            ) {
                return;
            }

            // 3. Editor workspace, file tabs, scrollbars, and content clicks
            if self.handle_editor_workspace_clicks(mx, my, w, h, top_h, sb_w) {
                return;
            }

            // 4. Settings tab options
            if self.handle_settings_clicks(mx, my, w, top_h, sb_w) {
                return;
            }

            // 5. Pane splitting, text selection, and terminal scrollbars
            self.handle_pane_terminal_pressed(mx, my, w, h, top_h, sb_w);
        }
    }
}
