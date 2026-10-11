use akssora_terminal_parser::TerminalParser;
use akssora_terminal_pty::{PtyEvent, PtySession};
use akssora_terminal_screen::{Cell, TerminalGrid};

// INFO: Central terminal subsystem coordinator linking pseudo-terminal I/O, escape parser, and visual grid.
pub struct Terminal {
    parser: TerminalParser,
    grid: TerminalGrid,
    pty: PtySession,
    closed: bool,
}

impl Terminal {
    pub fn new(rows: usize, cols: usize) -> anyhow::Result<Self> {
        let pty = PtySession::spawn(rows as u16, cols as u16)?;
        Ok(Self {
            parser: TerminalParser::new(),
            grid: TerminalGrid::new(rows, cols),
            pty,
            closed: false,
        })
    }

    pub fn spawn_command(command: &str, rows: usize, cols: usize) -> anyhow::Result<Self> {
        let pty = PtySession::spawn_command(command, rows as u16, cols as u16)?;
        Ok(Self {
            parser: TerminalParser::new(),
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

    pub fn grid_mut(&mut self) -> &mut TerminalGrid {
        &mut self.grid
    }

    pub fn cursor(&self) -> (usize, usize) {
        self.grid.cursor()
    }

    pub fn content_rows(&self) -> usize {
        self.grid.content_rows()
    }

    pub fn visible_lines(&self) -> Vec<&Vec<Cell>> {
        self.grid.visible_lines()
    }

    pub fn scroll_offset(&self) -> usize {
        self.grid.scroll_offset()
    }

    pub fn scroll_up(&mut self, n: usize) {
        self.grid.scroll_up_view(n);
    }

    pub fn scroll_down(&mut self, n: usize) {
        self.grid.scroll_down_view(n);
    }

    pub fn is_closed(&self) -> bool {
        self.closed
    }

    pub fn kill(&mut self) {
        self.pty.kill();
        self.closed = true;
    }
}
