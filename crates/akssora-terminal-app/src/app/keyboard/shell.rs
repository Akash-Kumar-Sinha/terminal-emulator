use akssora_terminal_input::encode_key;
use winit::event::KeyEvent;
use winit::keyboard::{Key, NamedKey};

use crate::app::App;

impl App {
    pub(crate) fn handle_shell_input(&mut self, event: &KeyEvent, ctrl: bool) {
        // 4. Track command buffer for automatic session naming and EOF detection
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
                Key::Named(NamedKey::Space) if !ctrl && !self.modifiers.alt_key() => {
                    let buf = self.command_buffers.entry(target_pane_id).or_default();
                    buf.push(' ');
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
