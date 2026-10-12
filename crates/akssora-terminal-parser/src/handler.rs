use akssora_terminal_screen::TerminalGrid;
use vte::{Params, Parser, Perform};

use crate::params::{first_param, flatten_params, nth_param};

// INFO: Adapts TerminalGrid to the vte::Perform trait to execute parsed ANSI sequences.
pub struct GridHandler<'a> {
    pub grid: &'a mut TerminalGrid,
    pub title: &'a mut Option<String>,
    pub cwd: &'a mut Option<String>,
    pub bracketed_paste: &'a mut bool,
    pub mouse_tracking: &'a mut bool,
}

impl Perform for GridHandler<'_> {
    fn print(&mut self, c: char) {
        self.grid.put_char(c);
    }

    fn execute(&mut self, byte: u8) {
        match byte {
            b'\n' | 0x0b | 0x0c => {
                self.grid.newline();
                self.grid.set_wrap_pending(false);
            }
            b'\r' => {
                let row = self.grid.cursor().0;
                self.grid.move_cursor(row, 0);
            }
            0x08 => {
                let (row, col) = self.grid.cursor();
                self.grid.move_cursor(row, col.saturating_sub(1));
            }
            b'\t' => {
                let (row, col) = self.grid.cursor();
                let next = ((col / 8) + 1) * 8;
                let target = next.min(self.grid.cols() - 1);
                self.grid.move_cursor(row, target);
            }
            _ => {}
        }
    }

    fn csi_dispatch(&mut self, params: &Params, intermediates: &[u8], _ignore: bool, action: char) {
        let private = intermediates.first() == Some(&b'?');
        let (row, col) = self.grid.cursor();

        match action {
            'A' => {
                let n = first_param(params, 1) as usize;
                self.grid.move_cursor(row.saturating_sub(n), col);
            }
            'B' => {
                let n = first_param(params, 1) as usize;
                self.grid.move_cursor(row + n, col);
            }
            'C' => {
                let n = first_param(params, 1) as usize;
                self.grid.move_cursor(row, col + n);
            }
            'D' => {
                let n = first_param(params, 1) as usize;
                self.grid.move_cursor(row, col.saturating_sub(n));
            }
            'G' => {
                let new_col = first_param(params, 1) as usize;
                self.grid.move_cursor(row, new_col.saturating_sub(1));
            }
            'd' => {
                let new_row = first_param(params, 1) as usize;
                self.grid.move_cursor(new_row.saturating_sub(1), col);
            }
            'H' | 'f' => {
                let new_row = first_param(params, 1) as usize;
                let new_col = nth_param(params, 1, 1) as usize;
                self.grid
                    .move_cursor(new_row.saturating_sub(1), new_col.saturating_sub(1));
            }
            'J' if !private => {
                self.grid.erase_in_display(first_param(params, 0));
            }
            'K' if !private => {
                self.grid.erase_in_line(first_param(params, 0));
            }
            'm' => {
                self.grid.apply_sgr(&flatten_params(params));
            }
            'h' if private => {
                for val in flatten_params(params) {
                    match val {
                        2004 => *self.bracketed_paste = true,
                        1000 | 1002 | 1003 | 1006 => *self.mouse_tracking = true,
                        _ => {}
                    }
                }
            }
            'l' if private => {
                for val in flatten_params(params) {
                    match val {
                        2004 => *self.bracketed_paste = false,
                        1000 | 1002 | 1003 | 1006 => *self.mouse_tracking = false,
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }
    fn osc_dispatch(&mut self, params: &[&[u8]], _bell_terminated: bool) {
        if params.is_empty() {
            return;
        }
        match params[0] {
            b"0" | b"2" => {
                if let Some(raw) = params.get(1)
                    && let Ok(t) = std::str::from_utf8(raw)
                {
                    let trimmed = t.trim();
                    if !trimmed.is_empty() {
                        *self.title = Some(trimmed.to_string());
                    }
                }
            }
            b"7" => {
                if let Some(raw) = params.get(1)
                    && let Ok(url) = std::str::from_utf8(raw)
                {
                    let path = if let Some(stripped) = url.strip_prefix("file://") {
                        if let Some(slash_idx) = stripped.find('/') {
                            &stripped[slash_idx..]
                        } else {
                            stripped
                        }
                    } else {
                        url
                    };
                    let trimmed = path.trim();
                    if !trimmed.is_empty() {
                        *self.cwd = Some(trimmed.to_string());
                    }
                }
            }
            _ => {}
        }
    }
}

// INFO: High-level terminal state machine parsing ANSI byte streams into grid mutations.
pub struct TerminalParser {
    parser: Parser,
    pub title: Option<String>,
    pub cwd: Option<String>,
    pub bracketed_paste: bool,
    pub mouse_tracking: bool,
}

impl Default for TerminalParser {
    fn default() -> Self {
        Self::new()
    }
}

impl TerminalParser {
    pub fn new() -> Self {
        Self {
            parser: Parser::new(),
            title: None,
            cwd: None,
            bracketed_paste: false,
            mouse_tracking: false,
        }
    }

    pub fn advance(&mut self, grid: &mut TerminalGrid, bytes: &[u8]) {
        let mut handler = GridHandler {
            grid,
            title: &mut self.title,
            cwd: &mut self.cwd,
            bracketed_paste: &mut self.bracketed_paste,
            mouse_tracking: &mut self.mouse_tracking,
        };
        self.parser.advance(&mut handler, bytes);
    }
}
