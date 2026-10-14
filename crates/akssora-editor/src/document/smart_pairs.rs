use crate::matching_brackets::find_matching_bracket;
use crate::symbols::{autocomplete_pair, is_closing_symbol};

use super::Document;

impl Document {
    // ── Matching Bracket Navigation ──────────────────────────────────────────

    pub fn jump_to_matching_bracket(&mut self) {
        let (row, col) = self.cursor;
        if let Some((target_row, target_col)) = find_matching_bracket(&self.buffer, row, col) {
            self.cursor = (target_row, target_col);
        }
    }

    // ── Smart Autocomplete Pair Insertion ────────────────────────────────────

    pub fn insert_char_smart(&mut self, ch: char) {
        if !self.autocomplete_symbols {
            self.insert_char(ch);
            return;
        }

        let (row, col) = self.cursor;
        let line = self.buffer.get_line(row).unwrap_or_default();
        let chars: Vec<char> = line.chars().collect();

        // Step over closing bracket if typed and matches next char
        if col < chars.len() && is_closing_symbol(ch) && chars[col] == ch {
            self.cursor = (row, col + 1);
            return;
        }

        // If user has a selection, wrap the selection in the symbol pair!
        if self.has_selection()
            && let Some(closer) = autocomplete_pair(ch)
        {
            let Some(text) = self.selected_text() else {
                self.insert_char(ch);
                return;
            };
            self.record_undo();
            self.delete_selection();
            let wrapped = format!("{ch}{text}{closer}");
            let cur = self.cursor;
            self.buffer.insert_str(cur.0, cur.1, &wrapped);
            self.cursor = (cur.0, cur.1 + wrapped.len());
            return;
        }

        // Insert opening symbol and its pair
        if let Some(closer) = autocomplete_pair(ch) {
            self.record_undo();
            self.delete_selection();
            let (r, c) = self.cursor;
            self.buffer.insert_char(r, c, ch);
            self.buffer.insert_char(r, c + 1, closer);
            self.cursor = (r, c + 1);
        } else {
            self.insert_char(ch);
        }
    }
}
