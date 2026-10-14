use winit::event::KeyEvent;
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::{Key, NamedKey};

use crate::app::App;

impl App {
    pub(crate) fn handle_overlay_and_modal_keys(
        &mut self,
        event_loop: &ActiveEventLoop,
        event: &KeyEvent,
        ctrl: bool,
    ) -> bool {
        // 0. Unsaved changes modal keyboard handling
        if self.unsaved_prompt_target.is_some() {
            match &event.logical_key {
                Key::Named(NamedKey::Enter) => {
                    self.save_prompt_action();
                    return true;
                }
                Key::Named(NamedKey::Escape) => {
                    self.cancel_prompt_action();
                    return true;
                }
                Key::Character(s) if s.eq_ignore_ascii_case("d") => {
                    self.discard_prompt_action();
                    return true;
                }
                Key::Character(s) if s.eq_ignore_ascii_case("s") => {
                    self.save_prompt_action();
                    return true;
                }
                Key::Character(s) if s.eq_ignore_ascii_case("c") => {
                    self.cancel_prompt_action();
                    return true;
                }
                _ => return true,
            }
        }

        // 1. Exit confirmation modal keyboard handling
        if self.prompt_confirm_exit {
            match &event.logical_key {
                Key::Named(NamedKey::Enter) => {
                    self.close_all_tabs();
                    event_loop.exit();
                    return true;
                }
                Key::Named(NamedKey::Escape) => {
                    self.prompt_confirm_exit = false;
                    if let Some(win) = self.window.as_ref() {
                        win.request_redraw();
                    }
                    return true;
                }
                Key::Character(s) if s.eq_ignore_ascii_case("y") => {
                    self.close_all_tabs();
                    event_loop.exit();
                    return true;
                }
                Key::Character(s) if s.eq_ignore_ascii_case("n") => {
                    self.prompt_confirm_exit = false;
                    if let Some(win) = self.window.as_ref() {
                        win.request_redraw();
                    }
                    return true;
                }
                _ => return true,
            }
        }

        // INFO: 1.1 Quick File Search overlay command-palette keyboard handling
        if self.quick_search.is_open {
            match &event.logical_key {
                Key::Named(NamedKey::Escape) => {
                    self.quick_search.close();
                    if let Some(win) = self.window.as_ref() {
                        win.request_redraw();
                    }
                    return true;
                }
                Key::Named(NamedKey::ArrowDown) => {
                    self.quick_search.select_next();
                    if let Some(win) = self.window.as_ref() {
                        win.request_redraw();
                    }
                    return true;
                }
                Key::Named(NamedKey::ArrowUp) => {
                    self.quick_search.select_prev();
                    if let Some(win) = self.window.as_ref() {
                        win.request_redraw();
                    }
                    return true;
                }
                Key::Named(NamedKey::Enter) => {
                    if let Some(res) = self.quick_search.selected_result().cloned() {
                        self.quick_search.close();
                        self.open_editor_file(&res.path);
                    }
                    if let Some(win) = self.window.as_ref() {
                        win.request_redraw();
                    }
                    return true;
                }
                Key::Named(NamedKey::Backspace) => {
                    self.quick_search.query.pop();
                    self.update_quick_search();
                    return true;
                }
                Key::Named(NamedKey::Space) if !ctrl => {
                    self.quick_search.query.push(' ');
                    self.update_quick_search();
                    return true;
                }
                Key::Character(s) => {
                    if ctrl {
                        if s.eq_ignore_ascii_case("n") {
                            self.quick_search.select_next();
                        } else if s.eq_ignore_ascii_case("p") {
                            self.quick_search.select_prev();
                        }
                    } else {
                        self.quick_search.query.push_str(s);
                        self.update_quick_search();
                    }
                    return true;
                }
                _ => return true,
            }
        }

        // Explorer confirmation modal keyboard handling
        if self.explorer.confirm_delete.is_some() {
            match &event.logical_key {
                Key::Named(NamedKey::Enter) => {
                    let deleted = self.explorer.tree.delete_selected().unwrap_or_default();
                    for p in deleted {
                        self.handle_file_deleted(&p);
                    }
                    self.explorer.confirm_delete = None;
                    if let Some(win) = self.window.as_ref() {
                        win.request_redraw();
                    }
                    return true;
                }
                Key::Character(s) if s.eq_ignore_ascii_case("y") => {
                    let deleted = self.explorer.tree.delete_selected().unwrap_or_default();
                    for p in deleted {
                        self.handle_file_deleted(&p);
                    }
                    self.explorer.confirm_delete = None;
                    if let Some(win) = self.window.as_ref() {
                        win.request_redraw();
                    }
                    return true;
                }
                Key::Named(NamedKey::Escape) => {
                    self.explorer.confirm_delete = None;
                    if let Some(win) = self.window.as_ref() {
                        win.request_redraw();
                    }
                    return true;
                }
                Key::Character(s) if s.eq_ignore_ascii_case("n") => {
                    self.explorer.confirm_delete = None;
                    if let Some(win) = self.window.as_ref() {
                        win.request_redraw();
                    }
                    return true;
                }
                _ => return true,
            }
        }

        // Explorer inline text input keyboard handling
        if self.explorer.inline_input.is_some() {
            match &event.logical_key {
                Key::Named(NamedKey::Enter) => {
                    let (text, parent, mode) = {
                        let inp = self.explorer.inline_input.as_ref().unwrap();
                        (
                            inp.text.trim().to_string(),
                            inp.parent_dir.clone(),
                            inp.mode.clone(),
                        )
                    };

                    if text.is_empty() {
                        if let Some(inp) = self.explorer.inline_input.as_mut() {
                            inp.error = Some("Name cannot be empty".to_string());
                        }
                        if let Some(win) = self.window.as_ref() {
                            win.request_redraw();
                        }
                        return true;
                    }

                    match mode {
                        akssora_terminal_ui::InlineInputMode::NewFile => {
                            match self.explorer.tree.create_file(&parent, &text) {
                                Ok(new_path) => {
                                    self.explorer.inline_input = None;
                                    self.open_editor_file(&new_path);
                                }
                                Err(e) => {
                                    if let Some(inp) = self.explorer.inline_input.as_mut() {
                                        inp.error = Some(e.to_string());
                                    }
                                }
                            }
                        }
                        akssora_terminal_ui::InlineInputMode::NewFolder => {
                            match self.explorer.tree.create_dir(&parent, &text) {
                                Ok(_) => {
                                    self.explorer.inline_input = None;
                                }
                                Err(e) => {
                                    if let Some(inp) = self.explorer.inline_input.as_mut() {
                                        inp.error = Some(e.to_string());
                                    }
                                }
                            }
                        }
                        akssora_terminal_ui::InlineInputMode::Rename { target } => {
                            match self.explorer.tree.rename(&target, &text) {
                                Ok(new_path) => {
                                    self.explorer.inline_input = None;
                                    self.handle_file_renamed(&target, &new_path);
                                }
                                Err(e) => {
                                    if let Some(inp) = self.explorer.inline_input.as_mut() {
                                        inp.error = Some(e.to_string());
                                    }
                                }
                            }
                        }
                        akssora_terminal_ui::InlineInputMode::ChangeRoot => {
                            let path = std::path::PathBuf::from(&text);
                            if path.is_dir() {
                                let _ = self.set_explorer_root(path);
                                self.explorer.inline_input = None;
                            } else if let Some(inp) = self.explorer.inline_input.as_mut() {
                                inp.error = Some("Not a valid directory".to_string());
                            }
                        }
                    }
                    if let Some(win) = self.window.as_ref() {
                        win.request_redraw();
                    }
                    return true;
                }
                Key::Named(NamedKey::Escape) => {
                    self.explorer.inline_input = None;
                    if let Some(win) = self.window.as_ref() {
                        win.request_redraw();
                    }
                    return true;
                }
                Key::Named(NamedKey::Backspace) => {
                    if let Some(inp) = self.explorer.inline_input.as_mut() {
                        inp.text.pop();
                        inp.error = None;
                    }
                    if let Some(win) = self.window.as_ref() {
                        win.request_redraw();
                    }
                    return true;
                }
                Key::Named(NamedKey::Space) if !ctrl => {
                    if let Some(inp) = self.explorer.inline_input.as_mut() {
                        inp.text.push(' ');
                        inp.error = None;
                    }
                    if let Some(win) = self.window.as_ref() {
                        win.request_redraw();
                    }
                    return true;
                }
                Key::Character(s) if !ctrl => {
                    if let Some(inp) = self.explorer.inline_input.as_mut() {
                        inp.text.push_str(s);
                        inp.error = None;
                    }
                    if let Some(win) = self.window.as_ref() {
                        win.request_redraw();
                    }
                    return true;
                }
                _ => return true,
            }
        }

        // Context menu dismiss on Escape
        if (self.context_menu.is_some() || self.explorer.context_menu.is_some())
            && matches!(&event.logical_key, Key::Named(NamedKey::Escape))
        {
            self.context_menu = None;
            self.explorer.context_menu = None;
            if let Some(win) = self.window.as_ref() {
                win.request_redraw();
            }
            return true;
        }

        // INFO: File Explorer direct keyboard shortcuts: % creates file, F2 renames
        if self.explorer.is_expanded && !ctrl {
            match &event.logical_key {
                Key::Character(s) if s == "%" => {
                    let parent = self.selected_explorer_directory();
                    self.explorer.inline_input = Some(akssora_terminal_ui::InlineInputState {
                        mode: akssora_terminal_ui::InlineInputMode::NewFile,
                        text: String::new(),
                        parent_dir: parent,
                        error: None,
                    });
                    if let Some(win) = self.window.as_ref() {
                        win.request_redraw();
                    }
                    return true;
                }
                Key::Named(NamedKey::F2) => {
                    if let Some(focused) = self.explorer.tree.focused_path.clone() {
                        let name = focused
                            .file_name()
                            .map(|n| n.to_string_lossy().to_string())
                            .unwrap_or_default();
                        self.explorer.inline_input = Some(akssora_terminal_ui::InlineInputState {
                            mode: akssora_terminal_ui::InlineInputMode::Rename {
                                target: focused.clone(),
                            },
                            text: name,
                            parent_dir: focused
                                .parent()
                                .unwrap_or(&self.explorer.tree.root_path)
                                .to_path_buf(),
                            error: None,
                        });
                        if let Some(win) = self.window.as_ref() {
                            win.request_redraw();
                        }
                        return true;
                    }
                }
                _ => {}
            }
        }

        false
    }
}
