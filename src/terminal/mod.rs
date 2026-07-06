use vte::{Params, Parser, Perform};

use crate::pty::{PtyEvent, PtySession};
use crate::theme::Color;

#[derive(Clone, Copy, PartialEq)]
pub struct Cell {
    pub ch: char,
    pub fg: Color,
    pub bg: Color,
    pub bold: bool,
    pub italic: bool,
}

impl Default for Cell {
    fn default() -> Self {
        Self {
            ch: ' ',
            fg: Color::Default,
            bg: Color::Default,
            bold: false,
            italic: false,
        }
    }
}

#[derive(Clone, Copy)]
struct Pen {
    fg: Color,
    bg: Color,
    bold: bool,
    italic: bool,
    inverse: bool,
}

impl Default for Pen {
    fn default() -> Self {
        Self {
            fg: Color::Default,
            bg: Color::Default,
            bold: false,
            italic: false,
            inverse: false,
        }
    }
}

impl Pen {
    fn cell_style(&self) -> (Color, Color) {
        if self.inverse {
            let fg = match self.bg {
                Color::Default => Color::Indexed(0),
                other => other,
            };
            let bg = match self.fg {
                Color::Default => Color::Indexed(7),
                other => other,
            };
            (fg, bg)
        } else {
            (self.fg, self.bg)
        }
    }
}

pub struct TerminalGrid {
    rows: usize,
    cols: usize,
    lines: Vec<Vec<Cell>>,
    cursor_row: usize,
    cursor_col: usize,
    pen: Pen,
    wrap_pending: bool,
}

impl TerminalGrid {
    pub fn new(rows: usize, cols: usize) -> Self {
        let rows = rows.max(1);
        let cols = cols.max(1);
        Self {
            rows,
            cols,
            lines: vec![vec![Cell::default(); cols]; rows],
            cursor_row: 0,
            cursor_col: 0,
            pen: Pen::default(),
            wrap_pending: false,
        }
    }

    pub fn lines(&self) -> &[Vec<Cell>] {
        &self.lines
    }

    pub fn cursor(&self) -> (usize, usize) {
        (self.cursor_row, self.cursor_col)
    }

    pub fn size(&self) -> (usize, usize) {
        (self.rows, self.cols)
    }

    pub fn resize(&mut self, rows: usize, cols: usize) {
        let rows = rows.max(1);
        let cols = cols.max(1);
        if rows == self.rows && cols == self.cols {
            return;
        }
        for line in &mut self.lines {
            line.resize(cols, Cell::default());
        }
        if rows > self.lines.len() {
            let extra = rows - self.lines.len();
            for _ in 0..extra {
                self.lines.push(vec![Cell::default(); cols]);
            }
        } else {
            let drop = self.lines.len() - rows;
            self.lines.drain(0..drop);
            self.cursor_row = self.cursor_row.saturating_sub(drop);
        }
        self.rows = rows;
        self.cols = cols;
        self.cursor_row = self.cursor_row.min(rows - 1);
        self.cursor_col = self.cursor_col.min(cols - 1);
        self.wrap_pending = false;
    }

    fn scroll_up(&mut self) {
        self.lines.remove(0);
        self.lines.push(vec![Cell::default(); self.cols]);
    }

    fn newline(&mut self) {
        if self.cursor_row + 1 >= self.rows {
            self.scroll_up();
        } else {
            self.cursor_row += 1;
        }
    }

    fn put_char(&mut self, ch: char) {
        if self.wrap_pending {
            self.cursor_col = 0;
            self.newline();
            self.wrap_pending = false;
        }
        let (fg, bg) = self.pen.cell_style();
        let row = self.cursor_row.min(self.rows - 1);
        let col = self.cursor_col.min(self.cols - 1);
        self.lines[row][col] = Cell {
            ch,
            fg,
            bg,
            bold: self.pen.bold,
            italic: self.pen.italic,
        };
        if self.cursor_col + 1 >= self.cols {
            self.wrap_pending = true;
        } else {
            self.cursor_col += 1;
        }
    }

    fn clear_line_range(&mut self, row: usize, from: usize, to: usize) {
        if row >= self.rows {
            return;
        }
        let end = to.min(self.cols);
        for col in from..end {
            self.lines[row][col] = Cell::default();
        }
    }

    fn erase_in_line(&mut self, mode: u16) {
        let row = self.cursor_row;
        match mode {
            0 => self.clear_line_range(row, self.cursor_col, self.cols),
            1 => self.clear_line_range(row, 0, self.cursor_col + 1),
            2 => self.clear_line_range(row, 0, self.cols),
            _ => {}
        }
    }

    fn erase_in_display(&mut self, mode: u16) {
        match mode {
            0 => {
                self.clear_line_range(self.cursor_row, self.cursor_col, self.cols);
                for row in (self.cursor_row + 1)..self.rows {
                    self.clear_line_range(row, 0, self.cols);
                }
            }
            1 => {
                for row in 0..self.cursor_row {
                    self.clear_line_range(row, 0, self.cols);
                }
                self.clear_line_range(self.cursor_row, 0, self.cursor_col + 1);
            }
            2 | 3 => {
                for row in 0..self.rows {
                    self.clear_line_range(row, 0, self.cols);
                }
            }
            _ => {}
        }
    }

    fn move_cursor(&mut self, row: usize, col: usize) {
        self.cursor_row = row.min(self.rows - 1);
        self.cursor_col = col.min(self.cols - 1);
        self.wrap_pending = false;
    }

    fn apply_sgr(&mut self, codes: &[u16]) {
        if codes.is_empty() {
            self.pen = Pen::default();
            return;
        }
        let mut i = 0;
        while i < codes.len() {
            let code = codes[i];
            match code {
                0 => self.pen = Pen::default(),
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
}

fn flatten_params(params: &Params) -> Vec<u16> {
    let mut out = Vec::new();
    for group in params.iter() {
        if group.is_empty() {
            out.push(0);
        } else {
            out.extend_from_slice(group);
        }
    }
    out
}

fn first_param(params: &Params, default: u16) -> u16 {
    match params.iter().next() {
        Some(g) if !g.is_empty() && g[0] != 0 => g[0],
        _ => default,
    }
}

fn nth_param(params: &Params, n: usize, default: u16) -> u16 {
    match params.iter().nth(n) {
        Some(g) if !g.is_empty() && g[0] != 0 => g[0],
        _ => default,
    }
}

impl Perform for TerminalGrid {
    fn print(&mut self, c: char) {
        self.put_char(c);
    }

    fn execute(&mut self, byte: u8) {
        match byte {
            b'\n' | 0x0b | 0x0c => {
                self.newline();
                self.wrap_pending = false;
            }
            b'\r' => {
                self.cursor_col = 0;
                self.wrap_pending = false;
            }
            0x08 => {
                self.cursor_col = self.cursor_col.saturating_sub(1);
                self.wrap_pending = false;
            }
            b'\t' => {
                let next = ((self.cursor_col / 8) + 1) * 8;
                self.cursor_col = next.min(self.cols - 1);
                self.wrap_pending = false;
            }
            _ => {}
        }
    }

    fn csi_dispatch(&mut self, params: &Params, intermediates: &[u8], _ignore: bool, action: char) {
        let private = intermediates.first() == Some(&b'?');
        match action {
            'A' => {
                let n = first_param(params, 1) as usize;
                self.move_cursor(self.cursor_row.saturating_sub(n), self.cursor_col);
            }
            'B' => {
                let n = first_param(params, 1) as usize;
                self.move_cursor(self.cursor_row + n, self.cursor_col);
            }
            'C' => {
                let n = first_param(params, 1) as usize;
                self.move_cursor(self.cursor_row, self.cursor_col + n);
            }
            'D' => {
                let n = first_param(params, 1) as usize;
                self.move_cursor(self.cursor_row, self.cursor_col.saturating_sub(n));
            }
            'G' => {
                let col = first_param(params, 1) as usize;
                self.move_cursor(self.cursor_row, col.saturating_sub(1));
            }
            'd' => {
                let row = first_param(params, 1) as usize;
                self.move_cursor(row.saturating_sub(1), self.cursor_col);
            }
            'H' | 'f' => {
                let row = first_param(params, 1) as usize;
                let col = nth_param(params, 1, 1) as usize;
                self.move_cursor(row.saturating_sub(1), col.saturating_sub(1));
            }
            'J' if !private => self.erase_in_display(first_param(params, 0)),
            'K' if !private => self.erase_in_line(first_param(params, 0)),
            'm' => self.apply_sgr(&flatten_params(params)),
            _ => {}
        }
    }
}

pub struct Terminal {
    parser: Parser,
    grid: TerminalGrid,
    pty: PtySession,
    closed: bool,
}

impl Terminal {
    pub fn new(rows: usize, cols: usize) -> anyhow::Result<Self> {
        let pty = PtySession::spawn(rows as u16, cols as u16)?;
        Ok(Self {
            parser: Parser::new(),
            grid: TerminalGrid::new(rows, cols),
            pty,
            closed: false,
        })
    }

    pub fn pump(&mut self) -> bool {
        let mut changed = false;
        loop {
            match self.pty.poll() {
                Some(PtyEvent::Data(bytes)) => {
                    self.parser.advance(&mut self.grid, &bytes);
                    changed = true;
                }
                Some(PtyEvent::Closed) => {
                    self.closed = true;
                    break;
                }
                None => break,
            }
        }
        changed
    }

    pub fn send(&mut self, bytes: &[u8]) {
        let _ = self.pty.write_input(bytes);
    }

    pub fn resize(&mut self, rows: usize, cols: usize) {
        self.grid.resize(rows, cols);
        let _ = self.pty.resize(rows as u16, cols as u16);
    }

    pub fn grid(&self) -> &TerminalGrid {
        &self.grid
    }

    pub fn cursor(&self) -> (usize, usize) {
        self.grid.cursor()
    }

    pub fn is_closed(&self) -> bool {
        self.closed
    }
}
