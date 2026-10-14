use akssora_terminal_config::LineNumberMode;

use crate::buffer::TextBuffer;
use crate::paragraph_iterator::{next_paragraph, prev_paragraph};
use crate::word_iterator::{
    WordType, next_word_end, next_word_start, prev_word_start, word_text_object_range,
};

use super::Document;

impl Document {
    // ── Vim Line Actions ──────────────────────────────────────────────────────

    pub fn delete_current_line(&mut self) {
        self.record_undo();
        let row = self.cursor.0;
        let line_count = self.buffer.line_count();
        let start_idx = self.buffer.pos_to_char_idx(row, 0);
        let end_idx = if row + 1 < line_count {
            self.buffer.pos_to_char_idx(row + 1, 0)
        } else {
            self.buffer.rope().len_chars()
        };
        self.vim.register = self.buffer.rope().slice(start_idx..end_idx).to_string();
        self.vim.status_message = Some("1 line deleted".to_string());
        self.buffer.delete_range(start_idx, end_idx);
        let max_r = self.buffer.line_count().saturating_sub(1);
        self.cursor = (row.min(max_r), 0);
        self.selection_anchor = None;
    }

    pub fn yank_current_line(&mut self) {
        let row = self.cursor.0;
        let line_count = self.buffer.line_count();
        let start_idx = self.buffer.pos_to_char_idx(row, 0);
        let end_idx = if row + 1 < line_count {
            self.buffer.pos_to_char_idx(row + 1, 0)
        } else {
            self.buffer.rope().len_chars()
        };
        self.vim.register = self.buffer.rope().slice(start_idx..end_idx).to_string();
        self.vim.status_message = Some("1 line yanked".to_string());
    }

    // ── Search Navigation (find_all, find_next, find_prev) ────────────────────

    pub fn find_all(&mut self, query: &str) {
        self.search.set_query(query, &self.buffer);
        if let Some(m) = self.search.current_match() {
            self.cursor = (m.row, m.col_start);
        }
    }

    pub fn find_next(&mut self) -> Option<(usize, usize)> {
        self.search.refresh_matches(&self.buffer);
        if let Some(m) = self.search.next_match() {
            self.cursor = (m.row, m.col_start);
            Some(self.cursor)
        } else {
            None
        }
    }

    pub fn find_prev(&mut self) -> Option<(usize, usize)> {
        self.search.refresh_matches(&self.buffer);
        if let Some(m) = self.search.prev_match() {
            self.cursor = (m.row, m.col_start);
            Some(self.cursor)
        } else {
            None
        }
    }

    // ── In-line Character Find Motions (f, F, t, T, ;, ,) ────────────────────

    pub fn find_char_inline(&mut self, target: char, forward: bool, till: bool) {
        let (row, col) = self.cursor;
        let Some(line) = self.buffer.get_line(row) else {
            return;
        };
        let chars: Vec<char> = line.chars().collect();

        if forward {
            for (idx, &ch) in chars.iter().enumerate().skip(col + 1) {
                if ch == target {
                    let target_col = if till { idx.saturating_sub(1) } else { idx };
                    self.cursor = (row, target_col);
                    self.vim.last_find_motion = Some(crate::vim::FindCharMotion {
                        target,
                        forward,
                        till,
                    });
                    return;
                }
            }
        } else {
            for idx in (0..col).rev() {
                if chars[idx] == target {
                    let target_col = if till {
                        (idx + 1).min(chars.len().saturating_sub(1))
                    } else {
                        idx
                    };
                    self.cursor = (row, target_col);
                    self.vim.last_find_motion = Some(crate::vim::FindCharMotion {
                        target,
                        forward,
                        till,
                    });
                    return;
                }
            }
        }
    }

    pub fn repeat_find_char(&mut self, reverse: bool) {
        if let Some(motion) = self.vim.last_find_motion {
            let forward = if reverse {
                !motion.forward
            } else {
                motion.forward
            };
            self.find_char_inline(motion.target, forward, motion.till);
        }
    }

    // ── Vim Word & Paragraph Motions ─────────────────────────────────────────

    pub fn move_word_start_forward_vim(&mut self, select: bool, word_type: WordType) {
        if select && self.selection_anchor.is_none() {
            self.selection_anchor = Some(self.cursor);
        } else if !select {
            self.selection_anchor = None;
        }
        self.cursor = next_word_start(&self.buffer, self.cursor.0, self.cursor.1, word_type);
    }

    pub fn move_word_end_forward_vim(&mut self, select: bool, word_type: WordType) {
        if select && self.selection_anchor.is_none() {
            self.selection_anchor = Some(self.cursor);
        } else if !select {
            self.selection_anchor = None;
        }
        self.cursor = next_word_end(&self.buffer, self.cursor.0, self.cursor.1, word_type);
    }

    pub fn move_word_start_backward_vim(&mut self, select: bool, word_type: WordType) {
        if select && self.selection_anchor.is_none() {
            self.selection_anchor = Some(self.cursor);
        } else if !select {
            self.selection_anchor = None;
        }
        self.cursor = prev_word_start(&self.buffer, self.cursor.0, self.cursor.1, word_type);
    }

    pub fn move_paragraph_forward_vim(&mut self, select: bool) {
        if select && self.selection_anchor.is_none() {
            self.selection_anchor = Some(self.cursor);
        } else if !select {
            self.selection_anchor = None;
        }
        self.cursor = next_paragraph(&self.buffer, self.cursor.0);
    }

    pub fn move_paragraph_backward_vim(&mut self, select: bool) {
        if select && self.selection_anchor.is_none() {
            self.selection_anchor = Some(self.cursor);
        } else if !select {
            self.selection_anchor = None;
        }
        self.cursor = prev_paragraph(&self.buffer, self.cursor.0);
    }

    // ── Vim Text Objects & Extended Operators ────────────────────────────────

    pub fn delete_text_object(&mut self, around: bool, kind: char) {
        if (kind == 'w' || kind == 'W')
            && let Some(((sr, sc), (er, ec))) =
                word_text_object_range(&self.buffer, self.cursor.0, self.cursor.1, around)
        {
            self.record_undo();
            let start_idx = self.buffer.pos_to_char_idx(sr, sc);
            let end_idx = self.buffer.pos_to_char_idx(er, ec);
            self.vim.set_register_content(
                self.buffer.rope().slice(start_idx..end_idx).to_string(),
                false,
            );
            self.buffer.delete_range(start_idx, end_idx);
            self.cursor = (sr, sc);
        }
    }

    pub fn yank_text_object(&mut self, around: bool, kind: char) {
        if (kind == 'w' || kind == 'W')
            && let Some(((sr, sc), (er, ec))) =
                word_text_object_range(&self.buffer, self.cursor.0, self.cursor.1, around)
        {
            let start_idx = self.buffer.pos_to_char_idx(sr, sc);
            let end_idx = self.buffer.pos_to_char_idx(er, ec);
            let content = self.buffer.rope().slice(start_idx..end_idx).to_string();
            self.vim.set_register_content(content, false);
            self.vim.status_message = Some("text object yanked".to_string());
        }
    }

    pub fn join_lines(&mut self) {
        let row = self.cursor.0;
        if row + 1 >= self.buffer.line_count() {
            return;
        }
        self.record_undo();
        let curr_len = self.buffer.line_len(row);
        let next_line = self.buffer.get_line(row + 1).unwrap_or_default();
        let trimmed_next = next_line.trim_start();
        let delete_start = self.buffer.pos_to_char_idx(row, curr_len);
        let delete_end = self.buffer.pos_to_char_idx(row + 2, 0);
        self.buffer.delete_range(delete_start, delete_end);
        let join_text = format!(" {trimmed_next}\n");
        self.buffer.insert_str(row, curr_len, &join_text);
        self.cursor = (row, curr_len);
    }

    pub fn toggle_case_under_cursor(&mut self) {
        let (row, col) = self.cursor;
        let line_len = self.buffer.line_len(row);
        if col >= line_len {
            return;
        }
        let Some(line) = self.buffer.get_line(row) else {
            return;
        };
        let chars: Vec<char> = line.chars().collect();
        let ch = chars[col];
        let toggled = if ch.is_uppercase() {
            ch.to_lowercase().next().unwrap_or(ch)
        } else {
            ch.to_uppercase().next().unwrap_or(ch)
        };
        self.record_undo();
        self.buffer.delete_char(row, col);
        self.buffer.insert_char(row, col, toggled);
        if col + 1 < line_len {
            self.cursor = (row, col + 1);
        }
    }

    pub fn replace_char_under_cursor(&mut self, ch: char) {
        let (row, col) = self.cursor;
        let line_len = self.buffer.line_len(row);
        if col >= line_len {
            return;
        }
        self.record_undo();
        self.buffer.delete_char(row, col);
        self.buffer.insert_char(row, col, ch);
        self.cursor = (row, col);
    }

    // ── Vim Ex Command Execution ─────────────────────────────────────────────

    pub fn execute_vim_command(&mut self, cmd: &str) -> Result<String, String> {
        let trimmed = cmd.trim();
        if trimmed == "w" {
            self.save().map_err(|e| e.to_string())?;
            Ok(format!("\"{}\" written", self.title()))
        } else if trimmed == "q" {
            if self.dirty {
                Err("No write since last change (add ! to override)".to_string())
            } else {
                Ok("quit".to_string())
            }
        } else if trimmed == "q!" {
            Ok("quit_forced".to_string())
        } else if trimmed == "wq" || trimmed == "x" {
            self.save().map_err(|e| e.to_string())?;
            Ok("quit".to_string())
        } else if let Ok(line_num) = trimmed.parse::<usize>() {
            let max_r = self.buffer.line_count().saturating_sub(1);
            let target = line_num.saturating_sub(1).min(max_r);
            self.cursor = (target, 0);
            Ok(format!("line {line_num}"))
        } else if trimmed.starts_with("%s/") {
            let parts: Vec<&str> = trimmed.split('/').collect();
            if parts.len() >= 3 {
                let old = parts[1];
                let new = parts[2];
                let content = self.buffer.to_string_content();
                let replaced = content.replace(old, new);
                if replaced != content {
                    self.record_undo();
                    self.buffer = TextBuffer::from_text(&replaced);
                    self.clamp_cursor();
                    Ok("substitutions complete".to_string())
                } else {
                    Ok("pattern not found".to_string())
                }
            } else {
                Err("invalid substitution syntax".to_string())
            }
        } else if trimmed == "set number" || trimmed == "set nu" {
            self.line_numbers = true;
            self.line_number_mode = LineNumberMode::Absolute;
            Ok("line numbers enabled".to_string())
        } else if trimmed == "set nonumber" || trimmed == "set nonu" {
            self.line_numbers = false;
            self.line_number_mode = LineNumberMode::Off;
            Ok("line numbers disabled".to_string())
        } else if trimmed == "set wrap" {
            self.line_wrap = true;
            Ok("line wrapping enabled".to_string())
        } else if trimmed == "set nowrap" {
            self.line_wrap = false;
            Ok("line wrapping disabled".to_string())
        } else {
            Err(format!("Not an editor command: {trimmed}"))
        }
    }
}
