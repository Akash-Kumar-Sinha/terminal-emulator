use winit::event::KeyEvent;
use winit::keyboard::{Key, NamedKey};

use crate::app::App;

impl App {
    pub(crate) fn handle_code_review_keyboard(&mut self, event: &KeyEvent, ctrl: bool) -> bool {
        if let Some(target) = self.code_review.confirm_discard.clone() {
            match &event.logical_key {
                Key::Named(NamedKey::Enter) => {
                    match target {
                        akssora_terminal_ui::DiscardTarget::File(path) => {
                            let full_path = self.code_review.repo_root.join(&path);
                            self.editor_documents.remove(&full_path);
                            self.code_review.discard_file_by_path(&path);
                            self.code_review.confirm_discard = None;
                            self.invalidate_code_review_data(Some(&path));
                        }
                        akssora_terminal_ui::DiscardTarget::Directory(dir) => {
                            let full_dir = self.code_review.repo_root.join(&dir);
                            self.editor_documents
                                .retain(|p, _| !p.starts_with(&full_dir));
                            self.code_review.discard_directory_by_path(&dir);
                            self.code_review.confirm_discard = None;
                            self.invalidate_code_review_data(None);
                        }
                        akssora_terminal_ui::DiscardTarget::All => {
                            self.editor_documents
                                .retain(|p, _| !p.starts_with(&self.code_review.repo_root));
                            self.code_review.discard_all_changes();
                            self.code_review.confirm_discard = None;
                            self.invalidate_code_review_data(None);
                        }
                    }
                    if let Some(win) = self.window.as_ref() {
                        win.request_redraw();
                    }
                    return true;
                }
                Key::Named(NamedKey::Escape) => {
                    self.code_review.confirm_discard = None;
                    if let Some(win) = self.window.as_ref() {
                        win.request_redraw();
                    }
                    return true;
                }
                _ => return true,
            }
        }

        if self.code_review.commit_dialog_open {
            match &event.logical_key {
                Key::Named(NamedKey::Enter) => {
                    let msg = self.code_review.commit_message.clone();
                    let _ = self.code_review.commit_staged(&msg);
                    if let Some(win) = self.window.as_ref() {
                        win.request_redraw();
                    }
                    return true;
                }
                Key::Named(NamedKey::Escape) => {
                    self.code_review.commit_dialog_open = false;
                    if let Some(win) = self.window.as_ref() {
                        win.request_redraw();
                    }
                    return true;
                }
                Key::Named(NamedKey::Backspace) => {
                    self.code_review.commit_message.pop();
                    if let Some(win) = self.window.as_ref() {
                        win.request_redraw();
                    }
                    return true;
                }
                Key::Named(NamedKey::Space) if !ctrl => {
                    self.code_review.commit_message.push(' ');
                    if let Some(win) = self.window.as_ref() {
                        win.request_redraw();
                    }
                    return true;
                }
                Key::Character(s) if !ctrl => {
                    self.code_review.commit_message.push_str(s);
                    if let Some(win) = self.window.as_ref() {
                        win.request_redraw();
                    }
                    return true;
                }
                _ => return true,
            }
        } else if self.code_review.active_editing_file.is_none() {
            match &event.logical_key {
                Key::Named(NamedKey::Enter) | Key::Named(NamedKey::Space) => {
                    let sel = self.code_review.selected_file_idx;
                    if let Some(file) = self.code_review.diff.files.get(sel) {
                        self.code_review.collapsed_files.remove(&file.path);
                        self.prepare_code_review_editor_file(sel);
                    }
                }
                Key::Named(NamedKey::ArrowUp) => {
                    if self.code_review.selected_file_idx > 0 {
                        self.code_review.selected_file_idx -= 1;
                    }
                }
                Key::Named(NamedKey::ArrowDown) => {
                    if self.code_review.selected_file_idx + 1 < self.code_review.diff.files.len() {
                        self.code_review.selected_file_idx += 1;
                    }
                }
                Key::Character(s) if !ctrl => match s.as_str() {
                    "k" => {
                        if self.code_review.selected_file_idx > 0 {
                            self.code_review.selected_file_idx -= 1;
                        }
                    }
                    "j" => {
                        if self.code_review.selected_file_idx + 1
                            < self.code_review.diff.files.len()
                        {
                            self.code_review.selected_file_idx += 1;
                        }
                    }
                    "r" => self.code_review.refresh(),
                    "s" => self.code_review.toggle_staged(),
                    "m" => self.code_review.toggle_diff_mode(),
                    "c" => self.code_review.commit_dialog_open = true,
                    "p" => {
                        let _ = self.code_review.push();
                    }
                    "f" => {
                        self.code_review.sidebar_expanded = !self.code_review.sidebar_expanded;
                    }
                    _ => {}
                },
                Key::Named(NamedKey::Escape) => {
                    self.toggle_code_review();
                }
                _ => {}
            }
            if let Some(win) = self.window.as_ref() {
                win.request_redraw();
            }
            return true;
        }

        false
    }
}
