use crate::cell::Cell;

// INFO: Terminal 2D coordinate referencing a specific row and column on the visible grid.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct TerminalPoint {
    pub row: usize,
    pub col: usize,
}

impl TerminalPoint {
    pub fn new(row: usize, col: usize) -> Self {
        Self { row, col }
    }
}

// INFO: Text selection span across rows and columns in the terminal screen or scrollback.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SelectionRange {
    pub start: TerminalPoint,
    pub end: TerminalPoint,
}

impl SelectionRange {
    pub fn new(start: TerminalPoint, end: TerminalPoint) -> Self {
        Self { start, end }
    }

    pub fn normalized(&self) -> (TerminalPoint, TerminalPoint) {
        if self.start <= self.end {
            (self.start, self.end)
        } else {
            (self.end, self.start)
        }
    }

    pub fn is_empty(&self) -> bool {
        self.start == self.end
    }

    pub fn contains(&self, row: usize, col: usize) -> bool {
        let (s, e) = self.normalized();
        if row < s.row || row > e.row {
            return false;
        }
        if s.row == e.row {
            col >= s.col && col <= e.col
        } else if row == s.row {
            col >= s.col
        } else if row == e.row {
            col <= e.col
        } else {
            true
        }
    }

    // INFO: Extracts formatted text from selected cells, preserving line wraps and trimming trailing space.
    pub fn extract_text(&self, lines: &[Vec<Cell>], wrapped_flags: &[bool]) -> String {
        let (s, e) = self.normalized();
        if lines.is_empty() {
            return String::new();
        }

        let start_row = s.row.min(lines.len() - 1);
        let end_row = e.row.min(lines.len() - 1);

        let mut result = String::new();

        for (row_idx, row) in lines.iter().enumerate().take(end_row + 1).skip(start_row) {
            if row.is_empty() {
                if row_idx < end_row {
                    result.push('\n');
                }
                continue;
            }

            let col_start = if row_idx == s.row {
                s.col.min(row.len())
            } else {
                0
            };

            let col_end = if row_idx == e.row {
                (e.col + 1).min(row.len())
            } else {
                row.len()
            };

            if col_start < col_end {
                let slice = &row[col_start..col_end];
                let line_str: String = slice.iter().map(|c| c.ch).collect();
                let trimmed = line_str.trim_end_matches([' ', '\0']);
                result.push_str(trimmed);
            }

            if row_idx < end_row {
                let is_wrapped = wrapped_flags.get(row_idx).copied().unwrap_or(false);
                if !is_wrapped {
                    result.push('\n');
                }
            }
        }

        result
    }
}
