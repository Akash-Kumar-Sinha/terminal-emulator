// INFO: In-memory text buffer structure supporting line-based editing operations.
#[derive(Clone, Debug, Default)]
pub struct TextBuffer {
    lines: Vec<String>,
}

impl TextBuffer {
    pub fn new() -> Self {
        Self {
            lines: vec![String::new()],
        }
    }

    pub fn from_text(text: &str) -> Self {
        let lines: Vec<String> = if text.is_empty() {
            vec![String::new()]
        } else {
            text.lines().map(|s| s.to_string()).collect()
        };
        Self { lines }
    }

    pub fn lines(&self) -> &[String] {
        &self.lines
    }

    pub fn line_count(&self) -> usize {
        self.lines.len()
    }

    pub fn get_line(&self, idx: usize) -> Option<&str> {
        self.lines.get(idx).map(|s| s.as_str())
    }

    pub fn insert_char(&mut self, row: usize, col: usize, ch: char) {
        if row >= self.lines.len() {
            self.lines.resize(row + 1, String::new());
        }
        let line = &mut self.lines[row];
        if col >= line.len() {
            line.push(ch);
        } else {
            line.insert(col, ch);
        }
    }

    pub fn insert_newline(&mut self, row: usize, col: usize) {
        if row >= self.lines.len() {
            self.lines.resize(row + 1, String::new());
        }
        let line = &mut self.lines[row];
        let rest = if col < line.len() {
            line.split_off(col)
        } else {
            String::new()
        };
        self.lines.insert(row + 1, rest);
    }

    pub fn delete_char(&mut self, row: usize, col: usize) {
        if row < self.lines.len() {
            let line = &mut self.lines[row];
            if col < line.len() {
                line.remove(col);
            }
        }
    }
}

impl From<&str> for TextBuffer {
    fn from(text: &str) -> Self {
        Self::from_text(text)
    }
}
