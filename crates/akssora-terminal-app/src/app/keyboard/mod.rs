use winit::event::{ElementState, KeyEvent};
use winit::event_loop::ActiveEventLoop;

use super::App;

mod code_review;
mod modals;
mod shell;
mod shortcuts;

// INFO: Keyboard input dispatching, global application shortcuts, and shell key encoding.
impl App {
    pub(crate) fn handle_keyboard_input(&mut self, event_loop: &ActiveEventLoop, event: &KeyEvent) {
        if event.state != ElementState::Pressed {
            return;
        }

        let ctrl = self.modifiers.control_key();
        let shift = self.modifiers.shift_key();

        // 0. Modals, search overlay, and explorer inputs
        if self.handle_overlay_and_modal_keys(event_loop, event, ctrl) {
            return;
        }

        // 1. Global application shortcuts
        if self.handle_global_shortcuts(event, ctrl, shift) {
            return;
        }

        // 2. Code Review panel keyboard routing
        let is_code_review = self.is_code_review_open()
            && (self.code_review.is_full_view
                || self.header.is_code_review_focused
                || self
                    .tab_manager
                    .active_tab()
                    .map(|t| matches!(t.kind, akssora_terminal_ui::TabKind::CodeReview { .. }))
                    .unwrap_or(false));

        if is_code_review && self.code_review.active_editing_file.is_none() {
            if let Some(idx) = self
                .code_review
                .diff
                .files
                .iter()
                .position(|f| !self.code_review.is_file_collapsed(&f.path))
            {
                self.prepare_code_review_editor_file(idx);
            } else if !self.code_review.diff.files.is_empty() {
                let idx = self
                    .code_review
                    .selected_file_idx
                    .min(self.code_review.diff.files.len() - 1);
                self.prepare_code_review_editor_file(idx);
            }
        }

        if is_code_review && self.handle_code_review_keyboard(event, ctrl) {
            return;
        }

        // 3. Integrated code editor keyboard routing
        let is_editor = self
            .tab_manager
            .active_tab()
            .map(|t| matches!(t.kind, akssora_terminal_ui::TabKind::Editor { .. }))
            .unwrap_or(false)
            || (is_code_review && self.code_review.active_editing_file.is_some());

        if is_editor {
            self.handle_editor_keyboard(event, is_code_review);
            return;
        }

        // 4. Shell input tracking and terminal byte dispatch
        self.handle_shell_input(event, ctrl);
    }
}
