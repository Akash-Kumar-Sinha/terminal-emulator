use super::Document;

impl Document {
    // ── Undo / Redo Tracking ──────────────────────────────────────────────────

    pub fn record_undo(&mut self) {
        if self.undo_stack.len() >= 100 {
            self.undo_stack.remove(0);
        }
        self.undo_stack.push((self.buffer.clone(), self.cursor));
        self.redo_stack.clear();
        self.dirty = true;
        self.cached_max_line_len.set(None);
        self.invalidate_wrap_cache();
        self.invalidate_markdown_cache();
    }

    pub fn undo(&mut self) {
        if let Some((prev_buf, prev_cursor)) = self.undo_stack.pop() {
            self.redo_stack.push((self.buffer.clone(), self.cursor));
            self.buffer = prev_buf;
            self.cursor = prev_cursor;
            self.dirty = true;
            self.cached_max_line_len.set(None);
            self.invalidate_wrap_cache();
            self.invalidate_markdown_cache();
            self.clamp_cursor();
        }
    }

    pub fn redo(&mut self) {
        if let Some((next_buf, next_cursor)) = self.redo_stack.pop() {
            self.undo_stack.push((self.buffer.clone(), self.cursor));
            self.buffer = next_buf;
            self.cursor = next_cursor;
            self.dirty = true;
            self.cached_max_line_len.set(None);
            self.invalidate_wrap_cache();
            self.invalidate_markdown_cache();
            self.clamp_cursor();
        }
    }

    // ── Text Insertion and Deletion ───────────────────────────────────────────

    pub fn insert_char(&mut self, ch: char) {
        self.record_undo();
        self.delete_selection();
        let (row, col) = self.cursor;
        self.buffer.insert_char(row, col, ch);
        self.cursor = (row, col + 1);
        self.selection_anchor = None;
    }

    pub fn insert_str(&mut self, text: &str) {
        self.record_undo();
        self.delete_selection();
        let (row, col) = self.cursor;
        self.buffer.insert_str(row, col, text);
        if text.contains('\n') {
            let added_rows = text.matches('\n').count();
            let last_line = text.lines().last().unwrap_or("");
            self.cursor = (row + added_rows, last_line.len());
        } else {
            self.cursor = (row, col + text.len());
        }
        self.selection_anchor = None;
    }

    pub fn insert_tab(&mut self) {
        let spaces = " ".repeat(self.tab_size);
        self.insert_str(&spaces);
    }

    pub fn insert_newline(&mut self) {
        self.record_undo();
        self.delete_selection();
        let (row, col) = self.cursor;
        self.buffer.insert_newline(row, col);
        self.cursor = (row + 1, 0);
        self.selection_anchor = None;
    }

    pub fn backspace(&mut self) {
        if self.has_selection() {
            self.record_undo();
            self.delete_selection();
            return;
        }
        let (row, col) = self.cursor;
        if row == 0 && col == 0 {
            return;
        }
        self.record_undo();
        self.cursor = self.buffer.delete_backspace(row, col);
        self.selection_anchor = None;
    }

    pub fn delete(&mut self) {
        if self.has_selection() {
            self.record_undo();
            self.delete_selection();
            return;
        }
        let (row, col) = self.cursor;
        if row >= self.buffer.line_count() {
            return;
        }
        self.record_undo();
        self.buffer.delete_under_cursor(row, col);
        self.selection_anchor = None;
    }

    pub fn paste(&mut self) {
        if self.vim.register.is_empty() {
            return;
        }
        self.record_undo();
        let reg = self.vim.register.clone();
        if reg.ends_with('\n') {
            // Line paste below
            let row = self.cursor.0;
            let insert_row = row + 1;
            self.buffer.insert_newline(row, self.buffer.line_len(row));
            self.buffer
                .insert_str(insert_row, 0, reg.trim_end_matches('\n'));
            self.cursor = (insert_row, 0);
        } else {
            let (row, col) = self.cursor;
            self.buffer.insert_str(row, col + 1, &reg);
            self.cursor = (row, col + reg.len());
        }
    }

    pub fn paste_before(&mut self) {
        if self.vim.register.is_empty() {
            return;
        }
        self.record_undo();
        let reg = self.vim.register.clone();
        if reg.ends_with('\n') {
            let row = self.cursor.0;
            self.buffer.insert_str(row, 0, &reg);
            self.cursor = (row, 0);
        } else {
            let (row, col) = self.cursor;
            self.buffer.insert_str(row, col, &reg);
            self.cursor = (row, col + reg.len().saturating_sub(1));
        }
    }

    // ── Search / Find & Replace ──────────────────────────────────────────────

    pub fn replace_current(&mut self) -> bool {
        self.search.refresh_matches(&self.buffer);
        let Some(m) = self.search.current_match() else {
            return false;
        };
        self.record_undo();
        let start_idx = self.buffer.pos_to_char_idx(m.row, m.col_start);
        let end_idx = self.buffer.pos_to_char_idx(m.row, m.col_end);
        self.buffer.delete_range(start_idx, end_idx);
        let repl = self.search.replace_text.clone();
        self.buffer.insert_str(m.row, m.col_start, &repl);
        self.cursor = (m.row, m.col_start + repl.len());
        self.search.refresh_matches(&self.buffer);
        true
    }

    pub fn replace_all(&mut self) -> usize {
        self.search.refresh_matches(&self.buffer);
        if self.search.matches.is_empty() {
            return 0;
        }
        self.record_undo();
        let mut count = 0;
        while self.replace_current() {
            count += 1;
        }
        count
    }
}
