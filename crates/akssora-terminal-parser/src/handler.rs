use akssora_terminal_screen::TerminalGrid;
use vte::{Params, Parser, Perform};

use crate::params::{first_param, flatten_params, nth_param};

// INFO: Adapts TerminalGrid to the vte::Perform trait to execute parsed ANSI sequences.
pub struct GridHandler<'a>(pub &'a mut TerminalGrid);

impl Perform for GridHandler<'_> {
    fn print(&mut self, c: char) {
        self.0.put_char(c);
    }

    fn execute(&mut self, byte: u8) {
        match byte {
            b'\n' | 0x0b | 0x0c => {
                self.0.newline();
                self.0.set_wrap_pending(false);
            }
            b'\r' => {
                let row = self.0.cursor().0;
                self.0.move_cursor(row, 0);
            }
            0x08 => {
                let (row, col) = self.0.cursor();
                self.0.move_cursor(row, col.saturating_sub(1));
            }
            b'\t' => {
                let (row, col) = self.0.cursor();
                let next = ((col / 8) + 1) * 8;
                let target = next.min(self.0.cols() - 1);
                self.0.move_cursor(row, target);
            }
            _ => {}
        }
    }

    fn csi_dispatch(&mut self, params: &Params, intermediates: &[u8], _ignore: bool, action: char) {
        let private = intermediates.first() == Some(&b'?');
        let (row, col) = self.0.cursor();

        match action {
            'A' => {
                let n = first_param(params, 1) as usize;
                self.0.move_cursor(row.saturating_sub(n), col);
            }
            'B' => {
                let n = first_param(params, 1) as usize;
                self.0.move_cursor(row + n, col);
            }
            'C' => {
                let n = first_param(params, 1) as usize;
                self.0.move_cursor(row, col + n);
            }
            'D' => {
                let n = first_param(params, 1) as usize;
                self.0.move_cursor(row, col.saturating_sub(n));
            }
            'G' => {
                let new_col = first_param(params, 1) as usize;
                self.0.move_cursor(row, new_col.saturating_sub(1));
            }
            'd' => {
                let new_row = first_param(params, 1) as usize;
                self.0.move_cursor(new_row.saturating_sub(1), col);
            }
            'H' | 'f' => {
                let new_row = first_param(params, 1) as usize;
                let new_col = nth_param(params, 1, 1) as usize;
                self.0
                    .move_cursor(new_row.saturating_sub(1), new_col.saturating_sub(1));
            }
            'J' if !private => {
                self.0.erase_in_display(first_param(params, 0));
            }
            'K' if !private => {
                self.0.erase_in_line(first_param(params, 0));
            }
            'm' => {
                self.0.apply_sgr(&flatten_params(params));
            }
            _ => {}
        }
    }
}

// INFO: High-level terminal state machine parsing ANSI byte streams into grid mutations.
pub struct TerminalParser {
    parser: Parser,
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
        }
    }

    pub fn advance(&mut self, grid: &mut TerminalGrid, bytes: &[u8]) {
        let mut handler = GridHandler(grid);
        self.parser.advance(&mut handler, bytes);
    }
}
