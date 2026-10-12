use akssora_terminal_config::Color;

use crate::cell::Cell;
use crate::cursor::Cursor;
use crate::pen::Pen;
use crate::reflow::reflow_terminal_content;
use crate::selection::SelectionRange;

// INFO: Terminal 2D character matrix managing active display lines, scrollback history, selection, and ANSI grid operations.
#[derive(Clone, Debug)]
pub struct TerminalGrid {
    rows: usize,
    cols: usize,
    lines: Vec<Vec<Cell>>,
    line_wrapped: Vec<bool>,
    cursor: Cursor,
    pen: Pen,
    wrap_pending: bool,
    scrollback: Vec<Vec<Cell>>,
    scrollback_wrapped: Vec<bool>,
    scroll_offset: usize,
    selection: Option<SelectionRange>,
}

impl TerminalGrid {
    pub fn new(rows: usize, cols: usize) -> Self {
        let rows = rows.max(1);
        let cols = cols.max(1);
        Self {
            rows,
            cols,
            lines: vec![vec![Cell::default(); cols]; rows],
            line_wrapped: vec![false; rows],
            cursor: Cursor::new(0, 0),
            pen: Pen::default(),
            wrap_pending: false,
            scrollback: Vec::new(),
            scrollback_wrapped: Vec::new(),
            scroll_offset: 0,
            selection: None,
        }
    }

    pub fn lines(&self) -> &[Vec<Cell>] {
        &self.lines
    }

    pub fn line_wrapped(&self) -> &[bool] {
        &self.line_wrapped
    }

    pub fn is_line_wrapped(&self, row: usize) -> bool {
        self.line_wrapped.get(row).copied().unwrap_or(false)
    }

    pub fn cursor(&self) -> (usize, usize) {
        self.cursor.pos()
    }

    pub fn cursor_ref(&self) -> &Cursor {
        &self.cursor
    }

    pub fn cursor_mut(&mut self) -> &mut Cursor {
        &mut self.cursor
    }

    pub fn pen(&self) -> &Pen {
        &self.pen
    }

    pub fn pen_mut(&mut self) -> &mut Pen {
        &mut self.pen
    }

    pub fn rows(&self) -> usize {
        self.rows
    }

    pub fn cols(&self) -> usize {
        self.cols
    }

    pub fn size(&self) -> (usize, usize) {
        (self.rows, self.cols)
    }

    pub fn scroll_offset(&self) -> usize {
        self.scroll_offset
    }

    pub fn scrollback_len(&self) -> usize {
        self.scrollback.len()
    }

    pub fn scroll_up_view(&mut self, n: usize) {
        self.scroll_offset = (self.scroll_offset + n).min(self.scrollback.len());
    }

    pub fn scroll_down_view(&mut self, n: usize) {
        self.scroll_offset = self.scroll_offset.saturating_sub(n);
    }

    pub fn reset_scroll(&mut self) {
        self.scroll_offset = 0;
    }

    pub fn scrollback(&self) -> &[Vec<Cell>] {
        &self.scrollback
    }

    pub fn scrollback_wrapped(&self) -> &[bool] {
        &self.scrollback_wrapped
    }

    pub fn set_scroll_offset(&mut self, offset: usize) {
        self.scroll_offset = offset.min(self.scrollback.len());
    }

    pub fn selection(&self) -> Option<SelectionRange> {
        self.selection
    }

    pub fn set_selection(&mut self, sel: Option<SelectionRange>) {
        self.selection = sel;
    }

    pub fn clear_selection(&mut self) {
        self.selection = None;
    }

    pub fn scroll_up(&mut self) {
        let line = self.lines.remove(0);
        let wrapped = if !self.line_wrapped.is_empty() {
            self.line_wrapped.remove(0)
        } else {
            false
        };
        self.scrollback.push(line);
        self.scrollback_wrapped.push(wrapped);
        if self.scrollback.len() > 10_000 {
            self.scrollback.remove(0);
            self.scrollback_wrapped.remove(0);
        }
        self.lines.push(vec![Cell::default(); self.cols]);
        self.line_wrapped.push(false);
        if self.scroll_offset > 0 {
            self.scroll_offset = (self.scroll_offset + 1).min(self.scrollback.len());
        }
    }

    pub fn visible_lines(&self) -> Vec<&Vec<Cell>> {
        let total = self.scrollback.len() + self.lines.len();
        let end = total.saturating_sub(self.scroll_offset);
        let start = end.saturating_sub(self.rows);
        (start..end)
            .map(|i| {
                if i < self.scrollback.len() {
                    &self.scrollback[i]
                } else {
                    &self.lines[i - self.scrollback.len()]
                }
            })
            .collect()
    }

    pub fn visible_line_wrapped(&self) -> Vec<bool> {
        let total = self.scrollback.len() + self.lines.len();
        let end = total.saturating_sub(self.scroll_offset);
        let start = end.saturating_sub(self.rows);
        (start..end)
            .map(|i| {
                if i < self.scrollback.len() {
                    self.scrollback_wrapped.get(i).copied().unwrap_or(false)
                } else {
                    let idx = i - self.scrollback.len();
                    self.line_wrapped.get(idx).copied().unwrap_or(false)
                }
            })
            .collect()
    }

    pub fn content_rows(&self) -> usize {
        let mut last = 0;
        for (i, line) in self.lines.iter().enumerate() {
            if line.iter().any(|c| !c.is_empty()) {
                last = i + 1;
            }
        }
        last.max(1)
    }

    // INFO: Dynamically reflows existing cell lines and scrollback, wrapping or unwrapping text to fit new dimensions while tracking cursor position.
    pub fn resize(&mut self, rows: usize, cols: usize) {
        let new_rows = rows.max(1);
        let new_cols = cols.max(1);
        if new_rows == self.rows && new_cols == self.cols {
            return;
        }

        let reflow_res = reflow_terminal_content(
            &self.scrollback,
            &self.scrollback_wrapped,
            &self.lines,
            &self.line_wrapped,
            self.cursor.row,
            self.cursor.col,
            new_rows,
            new_cols,
        );

        self.scrollback = reflow_res.scrollback;
        self.scrollback_wrapped = reflow_res.scrollback_wrapped;
        self.lines = reflow_res.lines;
        self.line_wrapped = reflow_res.line_wrapped;
        self.rows = new_rows;
        self.cols = new_cols;
        self.cursor.row = reflow_res.cursor_row;
        self.cursor.col = reflow_res.cursor_col;
        self.wrap_pending = false;
        self.scroll_offset = self.scroll_offset.min(self.scrollback.len());
        self.selection = None;
    }

    pub fn newline(&mut self) {
        if self.cursor.row + 1 >= self.rows {
            self.scroll_up();
        } else {
            self.cursor.row += 1;
        }
    }

    pub fn put_char(&mut self, ch: char) {
        if self.wrap_pending {
            let row = self.cursor.row.min(self.rows - 1);
            if row < self.line_wrapped.len() {
                self.line_wrapped[row] = true;
            }
            self.cursor.col = 0;
            self.newline();
            self.wrap_pending = false;
        }

        let (fg, bg) = self.pen.cell_style();
        let row = self.cursor.row.min(self.rows - 1);
        let col = self.cursor.col.min(self.cols - 1);
        self.lines[row][col] = Cell {
            ch,
            fg,
            bg,
            bold: self.pen.bold,
            italic: self.pen.italic,
        };

        if self.cursor.col + 1 >= self.cols {
            self.wrap_pending = true;
        } else {
            self.cursor.col += 1;
        }
    }

    pub fn set_wrap_pending(&mut self, pending: bool) {
        self.wrap_pending = pending;
    }

    pub fn clear_line_range(&mut self, row: usize, from: usize, to: usize) {
        if row >= self.rows {
            return;
        }
        let end = to.min(self.cols);
        for col in from..end {
            self.lines[row][col] = Cell::default();
        }
        if to >= self.cols && row < self.line_wrapped.len() {
            self.line_wrapped[row] = false;
        }
    }

    pub fn erase_in_line(&mut self, mode: u16) {
        let row = self.cursor.row;
        match mode {
            0 => self.clear_line_range(row, self.cursor.col, self.cols),
            1 => self.clear_line_range(row, 0, self.cursor.col + 1),
            2 => self.clear_line_range(row, 0, self.cols),
            _ => {}
        }
    }

    pub fn erase_in_display(&mut self, mode: u16) {
        match mode {
            0 => {
                self.clear_line_range(self.cursor.row, self.cursor.col, self.cols);
                for row in (self.cursor.row + 1)..self.rows {
                    self.clear_line_range(row, 0, self.cols);
                }
            }
            1 => {
                for row in 0..self.cursor.row {
                    self.clear_line_range(row, 0, self.cols);
                }
                self.clear_line_range(self.cursor.row, 0, self.cursor.col + 1);
            }
            2 | 3 => {
                for row in 0..self.rows {
                    self.clear_line_range(row, 0, self.cols);
                }
            }
            _ => {}
        }
    }

    pub fn move_cursor(&mut self, row: usize, col: usize) {
        self.cursor.row = row.min(self.rows - 1);
        self.cursor.col = col.min(self.cols - 1);
        self.wrap_pending = false;
    }

    pub fn apply_sgr(&mut self, codes: &[u16]) {
        if codes.is_empty() {
            self.pen.reset();
            return;
        }

        let mut i = 0;
        while i < codes.len() {
            let code = codes[i];
            match code {
                0 => self.pen.reset(),
                1 => self.pen.bold = true,
                3 => self.pen.italic = true,
                7 => self.pen.inverse = true,
                22 => self.pen.bold = false,
                23 => self.pen.italic = false,
                27 => self.pen.inverse = false,
                30..=37 => self.pen.fg = Color::Indexed((code - 30) as u8),
                39 => self.pen.fg = Color::Default,
                40..=47 => self.pen.bg = Color::Indexed((code - 40) as u8),
                49 => self.pen.bg = Color::Default,
                90..=97 => self.pen.fg = Color::Indexed((code - 90 + 8) as u8),
                100..=107 => self.pen.bg = Color::Indexed((code - 100 + 8) as u8),
                38 | 48 => {
                    let is_fg = code == 38;
                    match codes.get(i + 1).copied() {
                        Some(2) => {
                            let r = codes.get(i + 2).copied().unwrap_or(0) as u8;
                            let g = codes.get(i + 3).copied().unwrap_or(0) as u8;
                            let b = codes.get(i + 4).copied().unwrap_or(0) as u8;
                            let c = Color::Rgb(r, g, b);
                            if is_fg {
                                self.pen.fg = c;
                            } else {
                                self.pen.bg = c;
                            }
                            i += 4;
                        }
                        Some(5) => {
                            let n = codes.get(i + 2).copied().unwrap_or(0) as u8;
                            let c = Color::Indexed(n);
                            if is_fg {
                                self.pen.fg = c;
                            } else {
                                self.pen.bg = c;
                            }
                            i += 2;
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
            i += 1;
        }
    }

    // INFO: Returns currently selected text if an active selection exists.
    pub fn selected_text(&self) -> Option<String> {
        let sel = self.selection?;
        if sel.is_empty() {
            return None;
        }
        let vis = self.visible_lines();
        let owned_lines: Vec<Vec<Cell>> = vis.into_iter().cloned().collect();
        let wrapped = self.visible_line_wrapped();
        let text = sel.extract_text(&owned_lines, &wrapped);
        if text.is_empty() { None } else { Some(text) }
    }

    // INFO: Returns all text on the visible grid.
    pub fn all_text(&self) -> String {
        let mut result = String::new();
        let vis = self.visible_lines();
        let wrapped = self.visible_line_wrapped();
        for (i, row) in vis.iter().enumerate() {
            let s: String = row.iter().map(|c| c.ch).collect();
            let trimmed = s.trim_end_matches([' ', '\0']);
            result.push_str(trimmed);
            let is_wrapped = wrapped.get(i).copied().unwrap_or(false);
            if !is_wrapped && i + 1 < vis.len() {
                result.push('\n');
            }
        }
        result
    }
}
