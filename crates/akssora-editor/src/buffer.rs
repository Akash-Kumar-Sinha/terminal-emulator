use ropey::Rope;

// INFO: High-performance text buffer backed by Ropey for O(log N) inserts, edits, and Unicode line slicing.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TextBuffer {
    rope: Rope,
}

impl TextBuffer {
    pub fn new() -> Self {
        Self { rope: Rope::new() }
    }

    pub fn from_text(text: &str) -> Self {
        Self {
            rope: Rope::from_str(text),
        }
    }

    pub fn rope(&self) -> &Rope {
        &self.rope
    }

    pub fn rope_mut(&mut self) -> &mut Rope {
        &mut self.rope
    }

    pub fn line_count(&self) -> usize {
        let count = self.rope.len_lines();
        if count == 0 { 1 } else { count }
    }

    pub fn line_len(&self, row: usize) -> usize {
        if row >= self.rope.len_lines() {
            return 0;
        }
        let slice = self.rope.line(row);
        let s = slice.to_string();
        s.trim_end_matches(['\r', '\n']).chars().count()
    }

    pub fn get_line(&self, idx: usize) -> Option<String> {
        if idx >= self.rope.len_lines() {
            return None;
        }
        let slice = self.rope.line(idx);
        let s = slice.to_string();
        Some(s.trim_end_matches(['\r', '\n']).to_string())
    }

    pub fn get_line_raw(&self, idx: usize) -> Option<String> {
        if idx >= self.rope.len_lines() {
            return None;
        }
        Some(self.rope.line(idx).to_string())
    }

    pub fn pos_to_char_idx(&self, row: usize, col: usize) -> usize {
        if row >= self.rope.len_lines() {
            return self.rope.len_chars();
        }
        let line_start = self.rope.line_to_char(row);
        let line_len = self.line_len(row);
        let actual_col = col.min(line_len);
        (line_start + actual_col).min(self.rope.len_chars())
    }

    pub fn char_idx_to_pos(&self, char_idx: usize) -> (usize, usize) {
        let clamped = char_idx.min(self.rope.len_chars());
        let row = self.rope.char_to_line(clamped);
        let line_start = self.rope.line_to_char(row);
        let col = clamped.saturating_sub(line_start);
        (row, col)
    }

    pub fn insert_char(&mut self, row: usize, col: usize, ch: char) {
        let idx = self.pos_to_char_idx(row, col);
        self.rope.insert_char(idx, ch);
    }

    pub fn insert_str(&mut self, row: usize, col: usize, text: &str) {
        let idx = self.pos_to_char_idx(row, col);
        self.rope.insert(idx, text);
    }

    pub fn insert_newline(&mut self, row: usize, col: usize) {
        let idx = self.pos_to_char_idx(row, col);
        self.rope.insert_char(idx, '\n');
    }

    pub fn delete_char(&mut self, row: usize, col: usize) {
        let idx = self.pos_to_char_idx(row, col);
        if idx < self.rope.len_chars() {
            self.rope.remove(idx..idx + 1);
        }
    }

    pub fn delete_backspace(&mut self, row: usize, col: usize) -> (usize, usize) {
        let idx = self.pos_to_char_idx(row, col);
        if idx > 0 {
            self.rope.remove(idx - 1..idx);
            self.char_idx_to_pos(idx - 1)
        } else {
            (0, 0)
        }
    }

    pub fn delete_under_cursor(&mut self, row: usize, col: usize) {
        let idx = self.pos_to_char_idx(row, col);
        if idx < self.rope.len_chars() {
            self.rope.remove(idx..idx + 1);
        }
    }

    pub fn delete_range(&mut self, start_char: usize, end_char: usize) {
        let start = start_char.min(self.rope.len_chars());
        let end = end_char.min(self.rope.len_chars());
        if start < end {
            self.rope.remove(start..end);
        }
    }

    pub fn to_string_content(&self) -> String {
        self.rope.to_string()
    }

    pub fn clear(&mut self) {
        self.rope = Rope::new();
    }
}

impl From<&str> for TextBuffer {
    fn from(text: &str) -> Self {
        Self::from_text(text)
    }
}
