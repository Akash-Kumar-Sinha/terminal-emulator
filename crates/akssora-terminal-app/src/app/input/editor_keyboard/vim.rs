use akssora_editor::Document;
use winit::event::KeyEvent;
use winit::keyboard::{Key, NamedKey};

// INFO: Result of processing a Vim mode key event.
#[derive(Debug, Default)]
pub(crate) struct VimKeyResult {
    pub should_close_active_editor: bool,
}

// INFO: Vim mode key event dispatcher and modal state machine.
pub(crate) fn handle_vim_key(
    doc: &mut Document,
    event: &KeyEvent,
    ctrl: bool,
    visible_cols: usize,
    _is_code_review: bool,
) -> VimKeyResult {
    let mut result = VimKeyResult::default();

    match doc.vim.mode {
        akssora_editor::VimMode::Command => match &event.logical_key {
            Key::Named(NamedKey::Enter) => {
                let cmd = doc.vim.command_line.clone();
                match doc.execute_vim_command(&cmd) {
                    Ok(msg) => {
                        if msg == "quit" || msg == "quit_forced" {
                            result.should_close_active_editor = true;
                        } else {
                            doc.vim.status_message = Some(msg);
                            doc.vim.set_mode(akssora_editor::VimMode::Normal);
                        }
                    }
                    Err(err) => {
                        doc.vim.status_message = Some(format!("Error: {err}"));
                        doc.vim.set_mode(akssora_editor::VimMode::Normal);
                    }
                }
            }
            Key::Named(NamedKey::Escape) => {
                doc.vim.set_mode(akssora_editor::VimMode::Normal);
            }
            Key::Named(NamedKey::Backspace) => {
                if doc.vim.command_line.is_empty() {
                    doc.vim.set_mode(akssora_editor::VimMode::Normal);
                } else {
                    doc.vim.command_line.pop();
                }
            }
            Key::Named(NamedKey::Space) if !ctrl => {
                doc.vim.command_line.push(' ');
            }
            Key::Character(s) if !ctrl => {
                doc.vim.command_line.push_str(s);
            }
            _ => {}
        },
        akssora_editor::VimMode::Replace => match &event.logical_key {
            Key::Named(NamedKey::Escape) => {
                doc.vim.set_mode(akssora_editor::VimMode::Normal);
            }
            Key::Named(NamedKey::Space) if !ctrl => {
                doc.replace_char_under_cursor(' ');
                doc.vim.set_mode(akssora_editor::VimMode::Normal);
            }
            Key::Character(s) if !ctrl => {
                if let Some(ch) = s.chars().next() {
                    doc.replace_char_under_cursor(ch);
                    doc.vim.set_mode(akssora_editor::VimMode::Normal);
                }
            }
            _ => {}
        },
        akssora_editor::VimMode::Visual | akssora_editor::VimMode::VisualLine => {
            match &event.logical_key {
                Key::Named(NamedKey::Escape) => {
                    doc.clear_selection();
                    doc.vim.set_mode(akssora_editor::VimMode::Normal);
                }
                Key::Named(NamedKey::ArrowLeft) => doc.move_cursor_left(true),
                Key::Named(NamedKey::ArrowRight) => doc.move_cursor_right(true),
                Key::Named(NamedKey::Space) => doc.move_cursor_right(true),
                Key::Named(NamedKey::ArrowUp) => doc.move_cursor_up_visual(true, visible_cols),
                Key::Named(NamedKey::ArrowDown) => doc.move_cursor_down_visual(true, visible_cols),
                Key::Character(s) if !ctrl => match s.as_str() {
                    "h" => doc.move_cursor_left(true),
                    "l" => doc.move_cursor_right(true),
                    "k" => doc.move_cursor_up_visual(true, visible_cols),
                    "j" => doc.move_cursor_down_visual(true, visible_cols),
                    "w" => doc.move_word_start_forward_vim(true, akssora_editor::WordType::Default),
                    "W" => doc.move_word_start_forward_vim(true, akssora_editor::WordType::BigWord),
                    "b" => {
                        doc.move_word_start_backward_vim(true, akssora_editor::WordType::Default)
                    }
                    "B" => {
                        doc.move_word_start_backward_vim(true, akssora_editor::WordType::BigWord)
                    }
                    "e" => doc.move_word_end_forward_vim(true, akssora_editor::WordType::Default),
                    "E" => doc.move_word_end_forward_vim(true, akssora_editor::WordType::BigWord),
                    "0" => doc.move_to_line_start_visual(true, visible_cols),
                    "$" => doc.move_to_line_end_visual(true, visible_cols),
                    "%" => doc.jump_to_matching_bracket(),
                    "d" | "x" => {
                        if let Some(text) = doc.selected_text() {
                            doc.vim.set_register_content(text, false);
                        }
                        doc.delete_selection();
                        doc.vim.set_mode(akssora_editor::VimMode::Normal);
                    }
                    "y" => {
                        if let Some(text) = doc.selected_text() {
                            doc.vim.set_register_content(text, false);
                            doc.vim.status_message = Some("yanked selection".to_string());
                        }
                        doc.clear_selection();
                        doc.vim.set_mode(akssora_editor::VimMode::Normal);
                    }
                    "c" => {
                        if let Some(text) = doc.selected_text() {
                            doc.vim.set_register_content(text, false);
                        }
                        doc.delete_selection();
                        doc.vim.set_mode(akssora_editor::VimMode::Insert);
                    }
                    "~" => {
                        doc.toggle_case_under_cursor();
                        doc.vim.set_mode(akssora_editor::VimMode::Normal);
                    }
                    _ => {}
                },
                _ => {}
            }
        }
        akssora_editor::VimMode::Insert => match &event.logical_key {
            Key::Named(NamedKey::Escape) => {
                doc.vim.set_mode(akssora_editor::VimMode::Normal);
            }
            Key::Named(NamedKey::ArrowLeft) => doc.move_cursor_left(false),
            Key::Named(NamedKey::ArrowRight) => doc.move_cursor_right(false),
            Key::Named(NamedKey::ArrowUp) => doc.move_cursor_up_visual(false, visible_cols),
            Key::Named(NamedKey::ArrowDown) => doc.move_cursor_down_visual(false, visible_cols),
            Key::Named(NamedKey::Home) => doc.move_to_line_start_visual(false, visible_cols),
            Key::Named(NamedKey::End) => doc.move_to_line_end_visual(false, visible_cols),
            Key::Named(NamedKey::Backspace) => doc.backspace(),
            Key::Named(NamedKey::Delete) => doc.delete(),
            Key::Named(NamedKey::Enter) => doc.insert_newline(),
            Key::Named(NamedKey::Tab) => doc.insert_tab(),
            Key::Named(NamedKey::Space) if !ctrl => doc.insert_char_smart(' '),
            Key::Character(s) if !ctrl => {
                for ch in s.chars() {
                    doc.insert_char_smart(ch);
                }
            }
            _ => {}
        },
        _ => {
            // VimMode::Normal
            match &event.logical_key {
                Key::Named(NamedKey::Escape) => {
                    doc.vim.clear_operator_state();
                    doc.clear_selection();
                }
                Key::Named(NamedKey::ArrowLeft) => doc.move_cursor_left(false),
                Key::Named(NamedKey::ArrowRight) => doc.move_cursor_right(false),
                Key::Named(NamedKey::Space) => doc.move_cursor_right(false),
                Key::Named(NamedKey::ArrowUp) => doc.move_cursor_up_visual(false, visible_cols),
                Key::Named(NamedKey::ArrowDown) => doc.move_cursor_down_visual(false, visible_cols),
                Key::Named(NamedKey::Home) => doc.move_to_line_start_visual(false, visible_cols),
                Key::Named(NamedKey::End) => doc.move_to_line_end_visual(false, visible_cols),
                Key::Character(s) if !ctrl => {
                    let count = doc.vim.effective_count();
                    // Handle pending operators like 'r'
                    if doc.vim.pending_operator == Some('r') {
                        if let Some(ch) = s.chars().next() {
                            for _ in 0..count {
                                doc.replace_char_under_cursor(ch);
                            }
                        }
                        doc.vim.clear_operator_state();
                    } else if doc.vim.pending_operator == Some('f')
                        || doc.vim.pending_operator == Some('F')
                        || doc.vim.pending_operator == Some('t')
                        || doc.vim.pending_operator == Some('T')
                    {
                        if let Some(ch) = s.chars().next() {
                            let op = doc.vim.pending_operator.unwrap();
                            let forward = op == 'f' || op == 't';
                            let till = op == 't' || op == 'T';
                            doc.find_char_inline(ch, forward, till);
                        }
                        doc.vim.clear_operator_state();
                    } else if doc.vim.pending_operator == Some('d') {
                        if s == "d" {
                            for _ in 0..count {
                                doc.delete_current_line();
                            }
                        } else if s == "w" {
                            doc.delete_text_object(false, 'w');
                        } else if s == "W" {
                            doc.delete_text_object(false, 'W');
                        }
                        doc.vim.clear_operator_state();
                    } else if doc.vim.pending_operator == Some('c') {
                        if s == "c" {
                            doc.delete_current_line();
                            doc.vim.set_mode(akssora_editor::VimMode::Insert);
                        } else if s == "w" {
                            doc.delete_text_object(false, 'w');
                            doc.vim.set_mode(akssora_editor::VimMode::Insert);
                        } else {
                            doc.vim.clear_operator_state();
                        }
                    } else if doc.vim.pending_operator == Some('y') {
                        if s == "y" {
                            for _ in 0..count {
                                doc.yank_current_line();
                            }
                        } else if s == "w" {
                            doc.yank_text_object(false, 'w');
                        }
                        doc.vim.clear_operator_state();
                    } else {
                        // Standard Normal mode dispatch
                        match s.as_str() {
                            "0" if doc.vim.count_prefix.is_some() => {
                                doc.vim.push_count_digit(0);
                            }
                            "0" => doc.move_to_line_start_visual(false, visible_cols),
                            "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" => {
                                if let Ok(d) = s.parse::<usize>() {
                                    doc.vim.push_count_digit(d);
                                }
                            }
                            ":" => {
                                doc.vim.set_mode(akssora_editor::VimMode::Command);
                            }
                            "i" => {
                                doc.vim.set_mode(akssora_editor::VimMode::Insert);
                            }
                            "I" => {
                                doc.move_to_line_start_visual(false, visible_cols);
                                doc.vim.set_mode(akssora_editor::VimMode::Insert);
                            }
                            "a" => {
                                doc.move_cursor_right(false);
                                doc.vim.set_mode(akssora_editor::VimMode::Insert);
                            }
                            "A" => {
                                doc.move_to_line_end_visual(false, visible_cols);
                                doc.vim.set_mode(akssora_editor::VimMode::Insert);
                            }
                            "o" => {
                                let row = doc.cursor.0;
                                let len = doc.buffer.line_len(row);
                                doc.cursor = (row, len);
                                doc.insert_newline();
                                doc.vim.set_mode(akssora_editor::VimMode::Insert);
                            }
                            "O" => {
                                let row = doc.cursor.0;
                                doc.cursor = (row, 0);
                                doc.insert_newline();
                                doc.cursor = (row, 0);
                                doc.vim.set_mode(akssora_editor::VimMode::Insert);
                            }
                            "h" => {
                                for _ in 0..count {
                                    doc.move_cursor_left(false);
                                }
                                doc.vim.clear_operator_state();
                            }
                            "l" => {
                                for _ in 0..count {
                                    doc.move_cursor_right(false);
                                }
                                doc.vim.clear_operator_state();
                            }
                            "k" => {
                                for _ in 0..count {
                                    doc.move_cursor_up_visual(false, visible_cols);
                                }
                                doc.vim.clear_operator_state();
                            }
                            "j" => {
                                for _ in 0..count {
                                    doc.move_cursor_down_visual(false, visible_cols);
                                }
                                doc.vim.clear_operator_state();
                            }
                            "w" => {
                                for _ in 0..count {
                                    doc.move_word_start_forward_vim(
                                        false,
                                        akssora_editor::WordType::Default,
                                    );
                                }
                                doc.vim.clear_operator_state();
                            }
                            "W" => {
                                for _ in 0..count {
                                    doc.move_word_start_forward_vim(
                                        false,
                                        akssora_editor::WordType::BigWord,
                                    );
                                }
                                doc.vim.clear_operator_state();
                            }
                            "b" => {
                                for _ in 0..count {
                                    doc.move_word_start_backward_vim(
                                        false,
                                        akssora_editor::WordType::Default,
                                    );
                                }
                                doc.vim.clear_operator_state();
                            }
                            "B" => {
                                for _ in 0..count {
                                    doc.move_word_start_backward_vim(
                                        false,
                                        akssora_editor::WordType::BigWord,
                                    );
                                }
                                doc.vim.clear_operator_state();
                            }
                            "e" => {
                                for _ in 0..count {
                                    doc.move_word_end_forward_vim(
                                        false,
                                        akssora_editor::WordType::Default,
                                    );
                                }
                                doc.vim.clear_operator_state();
                            }
                            "E" => {
                                for _ in 0..count {
                                    doc.move_word_end_forward_vim(
                                        false,
                                        akssora_editor::WordType::BigWord,
                                    );
                                }
                                doc.vim.clear_operator_state();
                            }
                            "{" => {
                                doc.move_paragraph_backward_vim(false);
                                doc.vim.clear_operator_state();
                            }
                            "}" => {
                                doc.move_paragraph_forward_vim(false);
                                doc.vim.clear_operator_state();
                            }
                            "$" => {
                                doc.move_to_line_end_visual(false, visible_cols);
                                doc.vim.clear_operator_state();
                            }
                            "^" => {
                                doc.move_to_line_start_visual(false, visible_cols);
                                doc.vim.clear_operator_state();
                            }
                            "%" => {
                                doc.jump_to_matching_bracket();
                                doc.vim.clear_operator_state();
                            }
                            ";" => {
                                doc.repeat_find_char(false);
                            }
                            "," => {
                                doc.repeat_find_char(true);
                            }
                            "f" | "F" | "t" | "T" => {
                                doc.vim.pending_operator = Some(s.chars().next().unwrap());
                                doc.vim.showcmd = s.to_string();
                            }
                            "r" => {
                                doc.vim.pending_operator = Some('r');
                                doc.vim.showcmd = "r".to_string();
                            }
                            "R" => {
                                doc.vim.set_mode(akssora_editor::VimMode::Replace);
                            }
                            "d" => {
                                doc.vim.pending_operator = Some('d');
                                doc.vim.showcmd = "d".to_string();
                            }
                            "c" => {
                                doc.vim.pending_operator = Some('c');
                                doc.vim.showcmd = "c".to_string();
                            }
                            "y" => {
                                doc.vim.pending_operator = Some('y');
                                doc.vim.showcmd = "y".to_string();
                            }
                            "x" => {
                                for _ in 0..count {
                                    doc.delete();
                                }
                                doc.vim.clear_operator_state();
                            }
                            "u" => {
                                doc.undo();
                                doc.vim.clear_operator_state();
                            }
                            "p" => {
                                doc.paste();
                                doc.vim.clear_operator_state();
                            }
                            "P" => {
                                doc.paste();
                                doc.vim.clear_operator_state();
                            }
                            "J" => {
                                doc.join_lines();
                                doc.vim.clear_operator_state();
                            }
                            "~" => {
                                doc.toggle_case_under_cursor();
                                doc.vim.clear_operator_state();
                            }
                            "v" => {
                                doc.selection_anchor = Some(doc.cursor);
                                doc.vim.set_mode(akssora_editor::VimMode::Visual);
                            }
                            "V" => {
                                doc.move_to_line_start(false);
                                doc.selection_anchor = Some(doc.cursor);
                                doc.move_to_line_end(true);
                                doc.vim.set_mode(akssora_editor::VimMode::VisualLine);
                            }
                            _ => {
                                doc.vim.clear_operator_state();
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    }

    result
}
