use akssora_terminal_input::encode_key;
use winit::event::{ElementState, KeyEvent};
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::{Key, NamedKey};

use super::App;

// INFO: Keyboard input dispatching, global application shortcuts, and shell key encoding.
impl App {
    pub(crate) fn handle_keyboard_input(&mut self, event_loop: &ActiveEventLoop, event: &KeyEvent) {
        if event.state != ElementState::Pressed {
            return;
        }

        let ctrl = self.modifiers.control_key();
        let shift = self.modifiers.shift_key();

        // 1. Exit confirmation modal keyboard handling
        if self.prompt_confirm_exit {
            match &event.logical_key {
                Key::Named(NamedKey::Enter) => {
                    self.close_all_tabs();
                    event_loop.exit();
                    return;
                }
                Key::Named(NamedKey::Escape) => {
                    self.prompt_confirm_exit = false;
                    if let Some(win) = self.window.as_ref() {
                        win.request_redraw();
                    }
                    return;
                }
                Key::Character(s) if s.eq_ignore_ascii_case("y") => {
                    self.close_all_tabs();
                    event_loop.exit();
                    return;
                }
                Key::Character(s) if s.eq_ignore_ascii_case("n") => {
                    self.prompt_confirm_exit = false;
                    if let Some(win) = self.window.as_ref() {
                        win.request_redraw();
                    }
                    return;
                }
                _ => return,
            }
        }

        // 2. Global application shortcuts
        if ctrl
            && !shift
            && matches!(&event.logical_key, Key::Character(s) if s.eq_ignore_ascii_case("b"))
        {
            self.toggle_sidebar();
            return;
        }

        // Context menu dismiss on Escape
        if self.context_menu.is_some() && matches!(&event.logical_key, Key::Named(NamedKey::Escape))
        {
            self.context_menu = None;
            if let Some(win) = self.window.as_ref() {
                win.request_redraw();
            }
            return;
        }

        // Ctrl+Shift+C: copy active terminal text selection
        if ctrl
            && shift
            && matches!(&event.logical_key, Key::Character(s) if s.eq_ignore_ascii_case("c"))
        {
            self.copy_active_selection();
            return;
        }

        // Ctrl+Shift+V: paste clipboard text into active terminal
        if ctrl
            && shift
            && matches!(&event.logical_key, Key::Character(s) if s.eq_ignore_ascii_case("v"))
        {
            self.paste_into_active();
            return;
        }

        // Shift+PageUp / Shift+PageDown: Terminal scrollback history navigation
        if shift && !ctrl {
            if matches!(&event.logical_key, Key::Named(NamedKey::PageUp)) {
                if let Some(block) = self.active_block_mut() {
                    let page_lines = block.shell.grid().rows().saturating_sub(2).max(1);
                    block.shell.scroll_up(page_lines);
                    if let Some(win) = self.window.as_ref() {
                        win.request_redraw();
                    }
                }
                return;
            } else if matches!(&event.logical_key, Key::Named(NamedKey::PageDown)) {
                if let Some(block) = self.active_block_mut() {
                    let page_lines = block.shell.grid().rows().saturating_sub(2).max(1);
                    block.shell.scroll_down(page_lines);
                    if let Some(win) = self.window.as_ref() {
                        win.request_redraw();
                    }
                }
                return;
            }
        }

        if ctrl
            && shift
            && matches!(&event.logical_key, Key::Character(s) if s.eq_ignore_ascii_case("t"))
        {
            self.open_terminal_tab();
            if let Some(win) = self.window.as_ref() {
                win.request_redraw();
            }
            return;
        }

        // Ctrl+Shift+D: create split view (vertical split)
        if ctrl
            && shift
            && matches!(&event.logical_key, Key::Character(s) if s.eq_ignore_ascii_case("d"))
        {
            self.split_active_pane(akssora_terminal_ui::SplitDirection::Vertical);
            return;
        }

        // Ctrl+Shift+E: create split view (horizontal split)
        if ctrl
            && shift
            && matches!(&event.logical_key, Key::Character(s) if s.eq_ignore_ascii_case("e"))
        {
            self.split_active_pane(akssora_terminal_ui::SplitDirection::Horizontal);
            return;
        }

        // Ctrl+W: close current tab (prompt if last remaining tab)
        if ctrl
            && !shift
            && matches!(&event.logical_key, Key::Character(s) if s.eq_ignore_ascii_case("w"))
        {
            self.close_active_tab_or_prompt();
            return;
        }

        // Ctrl+D: close active terminal session or send EOF if buffer has text
        if ctrl
            && !shift
            && matches!(&event.logical_key, Key::Character(s) if s.eq_ignore_ascii_case("d"))
        {
            let active_pane = self.active_pane_id();
            let has_input = active_pane
                .and_then(|id| self.command_buffers.get(&id))
                .map(|b| !b.is_empty())
                .unwrap_or(false);

            if has_input {
                // Buffer has typed input: send EOF byte (\x04) without closing session
                if let Some(block) = self.active_block_mut() {
                    block.shell.send(b"\x04");
                    if let Some(win) = self.window.as_ref() {
                        win.request_redraw();
                    }
                }
                return;
            }

            // Buffer is empty: close active split pane or tab
            let active_tab_id = self.tab_manager.active_id();
            let has_multiple_splits = active_tab_id
                .and_then(|tid| self.split_trees.get(&tid))
                .map(|tree| tree.all_leaf_ids().len() > 1)
                .unwrap_or(false);

            if has_multiple_splits {
                if let Some(pane_id) = active_pane {
                    self.close_split_pane(pane_id);
                }
            } else {
                self.close_active_tab_or_prompt();
            }
            return;
        }

        // Ctrl+Shift+W: close all tabs (prompt before closing application)
        if ctrl
            && shift
            && matches!(&event.logical_key, Key::Character(s) if s.eq_ignore_ascii_case("w"))
        {
            self.close_all_tabs_or_prompt();
            return;
        }

        // Ctrl+Tab / Ctrl+Shift+Tab: switch next / previous tab
        if ctrl && matches!(&event.logical_key, Key::Named(NamedKey::Tab)) {
            if shift {
                self.prev_tab();
            } else {
                self.next_tab();
            }
            return;
        }

        if ctrl
            && !shift
            && matches!(&event.logical_key, Key::Character(s) if s.eq_ignore_ascii_case("l"))
        {
            self.cycle_tab_layout();
            if let Some(win) = self.window.as_ref() {
                win.request_redraw();
            }
            return;
        }

        if ctrl && !shift && matches!(&event.logical_key, Key::Character(s) if s == ",") {
            self.open_settings_tab();
            if let Some(win) = self.window.as_ref() {
                win.request_redraw();
            }
            return;
        }

        if ctrl && matches!(&event.logical_key, Key::Character(s) if s == "+" || s == "=") {
            self.increase_font_size();
            return;
        }

        if ctrl && matches!(&event.logical_key, Key::Character(s) if s == "-") {
            self.decrease_font_size();
            return;
        }

        if ctrl && matches!(&event.logical_key, Key::Character(s) if s == "0") {
            self.reset_font_size();
            return;
        }

        // 3. Track command buffer for automatic session naming and EOF detection
        if let Some(target_pane_id) = self.active_pane_id() {
            let active_tab_id = self.tab_manager.active_id();
            match &event.logical_key {
                Key::Named(NamedKey::Enter) => {
                    if let Some(buf) = self.command_buffers.get_mut(&target_pane_id) {
                        let cmd = buf.trim().to_string();
                        if !cmd.is_empty() {
                            self.last_executed_commands
                                .insert(target_pane_id, cmd.clone());
                            if let Some(tab_id) = active_tab_id {
                                self.tab_manager.set_auto_title(tab_id, &cmd);
                            }
                        }
                        buf.clear();
                    }
                }
                Key::Named(NamedKey::Backspace) => {
                    if let Some(buf) = self.command_buffers.get_mut(&target_pane_id) {
                        buf.pop();
                    }
                }
                Key::Character(s) if !ctrl && !self.modifiers.alt_key() => {
                    let buf = self.command_buffers.entry(target_pane_id).or_default();
                    buf.push_str(s);
                }
                _ => {}
            }
        }

        // 4. Forward encoded keyboard sequence to active tab shell
        if let Some(bytes) = encode_key(event, self.modifiers)
            && let Some(block) = self.active_block_mut()
        {
            block.shell.send(&bytes);
            if let Some(win) = self.window.as_ref() {
                win.request_redraw();
            }
        }
    }
}
