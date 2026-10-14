use winit::event::KeyEvent;
use winit::keyboard::{Key, NamedKey};

use crate::app::App;

impl App {
    pub(crate) fn handle_global_shortcuts(
        &mut self,
        event: &KeyEvent,
        ctrl: bool,
        shift: bool,
    ) -> bool {
        // 2. Global application shortcuts
        // Ctrl+B: toggle sidebar
        if ctrl
            && !shift
            && matches!(&event.logical_key, Key::Character(s) if s.eq_ignore_ascii_case("b"))
        {
            self.toggle_sidebar();
            return true;
        }

        // INFO: Ctrl+Shift+E: toggle file explorer
        if ctrl
            && shift
            && matches!(&event.logical_key, Key::Character(s) if s.eq_ignore_ascii_case("e"))
        {
            self.toggle_explorer();
            return true;
        }

        // INFO: Ctrl+P / Ctrl+E: toggle quick file search command-palette
        if ctrl
            && !shift
            && matches!(&event.logical_key, Key::Character(s) if s.eq_ignore_ascii_case("p") || s.eq_ignore_ascii_case("e"))
        {
            self.toggle_quick_search();
            return true;
        }

        // Ctrl+Shift+C: copy active terminal text selection
        if ctrl
            && shift
            && matches!(&event.logical_key, Key::Character(s) if s.eq_ignore_ascii_case("c"))
        {
            self.copy_active_selection();
            return true;
        }

        // Ctrl+Shift+V: paste clipboard text into active terminal
        if ctrl
            && shift
            && matches!(&event.logical_key, Key::Character(s) if s.eq_ignore_ascii_case("v"))
        {
            self.paste_into_active();
            return true;
        }

        // Shift+PageUp / Shift+PageDown: Terminal scrollback history navigation
        if shift
            && !ctrl
            && matches!(
                &event.logical_key,
                Key::Named(NamedKey::PageUp) | Key::Named(NamedKey::PageDown)
            )
        {
            let is_up = matches!(&event.logical_key, Key::Named(NamedKey::PageUp));
            if let Some(block) = self.active_block_mut() {
                let page_size = block.shell.grid().rows().saturating_sub(2).max(1);
                let current_offset = block.shell.grid().scroll_offset();
                let scrollback_len = block.shell.grid().scrollback_len();

                let new_offset = if is_up {
                    (current_offset + page_size).min(scrollback_len)
                } else {
                    current_offset.saturating_sub(page_size)
                };

                block.shell.grid_mut().set_scroll_offset(new_offset);
                if let Some(win) = self.window.as_ref() {
                    win.request_redraw();
                }
                return true;
            }
        }

        // Ctrl+Shift+D: create split view (vertical split)
        if ctrl
            && shift
            && matches!(&event.logical_key, Key::Character(s) if s.eq_ignore_ascii_case("d"))
        {
            self.split_active_pane(akssora_terminal_ui::SplitDirection::Vertical);
            return true;
        }

        // Ctrl+Shift+O: create split view (horizontal split)
        if ctrl
            && shift
            && matches!(&event.logical_key, Key::Character(s) if s.eq_ignore_ascii_case("o"))
        {
            self.split_active_pane(akssora_terminal_ui::SplitDirection::Horizontal);
            return true;
        }

        // Ctrl+W: close current tab (prompt if last remaining tab)
        if ctrl
            && !shift
            && matches!(&event.logical_key, Key::Character(s) if s.eq_ignore_ascii_case("w"))
        {
            self.close_active_tab_or_prompt();
            return true;
        }

        // Ctrl+D: close active terminal session or send EOF if buffer has text
        if ctrl
            && !shift
            && matches!(&event.logical_key, Key::Character(s) if s.eq_ignore_ascii_case("d"))
        {
            let buffer_has_text = self
                .active_pane_id()
                .and_then(|id| self.command_buffers.get(&id))
                .map(|b| !b.trim().is_empty())
                .unwrap_or(false);

            if buffer_has_text {
                // Buffer has typed input: send EOF byte (\x04) without closing session
                if let Some(block) = self.active_block_mut() {
                    block.shell.send(b"\x04");
                    if let Some(pane_id) = self.active_pane_id()
                        && let Some(buf) = self.command_buffers.get_mut(&pane_id)
                    {
                        buf.clear();
                    }
                    if let Some(win) = self.window.as_ref() {
                        win.request_redraw();
                    }
                }
                return true;
            }

            // Buffer is empty: close active split pane or tab
            if let Some(active_tab_id) = self.tab_manager.active_id()
                && let Some(tree) = self.split_trees.get(&active_tab_id)
                && tree.all_leaf_ids().len() > 1
                && let Some(pane_id) = self.split_manager.active_pane_id
            {
                self.close_split_pane(pane_id);
                return true;
            }

            self.close_active_tab_or_prompt();
            return true;
        }

        // Ctrl+Shift+W: close all tabs (prompt before closing application)
        if ctrl
            && shift
            && matches!(&event.logical_key, Key::Character(s) if s.eq_ignore_ascii_case("w"))
        {
            self.prompt_confirm_exit = true;
            if let Some(win) = self.window.as_ref() {
                win.request_redraw();
            }
            return true;
        }

        // Ctrl+Tab / Ctrl+Shift+Tab: switch next / previous tab
        if ctrl && matches!(&event.logical_key, Key::Named(NamedKey::Tab)) {
            if shift {
                let prev = if self.tab_manager.active_index() == 0 {
                    self.tab_manager.len().saturating_sub(1)
                } else {
                    self.tab_manager.active_index() - 1
                };
                self.tab_manager.select(prev);
            } else {
                let next = (self.tab_manager.active_index() + 1) % self.tab_manager.len().max(1);
                self.tab_manager.select(next);
            }
            self.sync_active_tab_state();
            if let Some(active_id) = self.tab_manager.active_id()
                && let Some(tree) = self.split_trees.get(&active_id)
                && !tree.contains_leaf(self.split_manager.active_pane_id.unwrap_or(0))
                && let Some(first_leaf) = tree.all_leaf_ids().first().copied()
            {
                self.split_manager.focus_pane(first_leaf);
            }
            self.layout_blocks();
            if let Some(win) = self.window.as_ref() {
                win.request_redraw();
            }
            return true;
        }

        if ctrl
            && matches!(
                &event.logical_key,
                Key::Character(s) if s == "=" || s == "+"
            )
        {
            self.increase_font_size();
            return true;
        }

        if ctrl && matches!(&event.logical_key, Key::Character(s) if s == "-") {
            self.decrease_font_size();
            return true;
        }

        if ctrl && matches!(&event.logical_key, Key::Character(s) if s == "0") {
            self.reset_font_size();
            return true;
        }

        // Global shortcut to toggle Code Review: Ctrl+Shift+R or Ctrl+Shift+G
        if ctrl
            && shift
            && matches!(&event.logical_key, Key::Character(s) if s.eq_ignore_ascii_case("r") || s.eq_ignore_ascii_case("g"))
        {
            self.toggle_code_review();
            return true;
        }

        false
    }
}
