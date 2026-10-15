pub(crate) mod shortcuts;
pub(crate) mod standard;
pub(crate) mod vim;

use super::super::App;
use winit::event::KeyEvent;

// INFO: Keyboard navigation, Vim bindings, and editing dispatch for active code editor document.
impl App {
    pub(crate) fn handle_editor_keyboard(&mut self, event: &KeyEvent, is_code_review: bool) {
        let (win_w, win_h) = self.state.as_ref().map(|s| s.size()).unwrap_or((1000, 700));
        let (cell_w, cell_h) = self
            .fonts
            .as_ref()
            .map(|f| (f.cell_w, f.cell_h))
            .unwrap_or((8.0, 16.0));
        let show_sidebar = self.config.tab_layout.show_sidebar();
        let sb_w = self.sidebar.effective_width(show_sidebar);
        let exp_w = self.explorer.effective_width();
        let top_h = self.header.height;
        let tab_bar_h = if !self.editor_open_files.is_empty() {
            28.0
        } else {
            0.0
        };
        let (ws_w, ws_h) =
            if is_code_review && let Some((_, _, cr_w, cr_h)) = self.code_review_bounds() {
                (cr_w, cr_h)
            } else {
                let ws_w = (win_w as f32 - sb_w - exp_w).max(100.0);
                let ws_h = (win_h as f32 - top_h - tab_bar_h - 30.0).max(cell_h);
                (ws_w, ws_h)
            };
        let visible_rows = (ws_h / cell_h).floor() as usize;

        let gutter_w = if let Some(doc) = self.active_editor() {
            if doc.line_numbers
                && doc.line_number_mode != akssora_terminal_config::LineNumberMode::Off
            {
                48.0
            } else {
                0.0
            }
        } else {
            0.0
        };
        let content_w = (ws_w - gutter_w - 24.0).max(cell_w);
        let visible_cols = (content_w / cell_w).floor().max(1.0) as usize;

        // Check global shortcuts first
        if shortcuts::handle_editor_shortcuts(self, event, visible_rows, visible_cols) {
            return;
        }

        let ctrl = self.modifiers.control_key();
        let shift = self.modifiers.shift_key();
        let auto_save = self.config.auto_save;
        let mut should_close_active_editor = false;

        if let Some(doc) = self.active_editor_mut() {
            if doc.vim_enabled {
                let res = vim::handle_vim_key(doc, event, ctrl, visible_cols, is_code_review);
                if res.should_close_active_editor {
                    should_close_active_editor = true;
                }
            } else {
                let _res = standard::handle_standard_key(
                    doc,
                    event,
                    shift,
                    ctrl,
                    visible_rows,
                    visible_cols,
                    is_code_review,
                );
            }

            doc.scroll_to_cursor(visible_rows.max(1), visible_cols.max(1));
            if auto_save && doc.dirty {
                let _ = doc.save();
            }
        }

        if should_close_active_editor
            && let Some(pos) = self
                .editor_active_file
                .as_ref()
                .and_then(|p| self.editor_open_files.iter().position(|f| f == p))
        {
            self.request_close_editor_tab(pos);
        }

        if let Some(win) = self.window.as_ref() {
            win.request_redraw();
        }
    }
}
