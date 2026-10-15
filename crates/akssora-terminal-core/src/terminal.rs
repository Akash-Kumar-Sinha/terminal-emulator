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
        Self::new_in(rows, cols, None)
    }

    pub fn new_in(rows: usize, cols: usize, cwd: Option<&std::path::Path>) -> anyhow::Result<Self> {
        let pty = PtySession::spawn_in(rows as u16, cols as u16, cwd)?;
        Ok(Self {
            parser: TerminalParser::new(),
            grid: TerminalGrid::new(rows, cols),
            pty,
            closed: false,
        })
    }

    pub fn process_id(&self) -> Option<u32> {
        self.pty.process_id()
    }

    pub fn osc_title(&self) -> Option<&str> {
        self.parser.title.as_deref()
    }

    pub fn osc_cwd(&self) -> Option<&str> {
        self.parser.cwd.as_deref()
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

    // INFO: Pumps queued PTY events in bounded batches (max 64 chunks = 256KB per pump cycle) to prevent UI thread starvation.
    pub fn pump(&mut self) -> bool {
        let mut changed = false;
        let mut batch_count = 0;
        loop {
            if batch_count >= 64 {
                break;
            }
            match self.pty.poll() {
                Some(PtyEvent::Data(bytes)) => {
                    self.parser.advance(&mut self.grid, &bytes);
                    changed = true;
                    batch_count += 1;
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

    // INFO: Resizes terminal grid and updates pseudo-terminal dimensions if dimensions changed.
    pub fn resize(&mut self, rows: usize, cols: usize) {
        if rows == self.grid.rows() && cols == self.grid.cols() {
            return;
        }
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

    pub fn reset_scroll(&mut self) {
        self.grid.reset_scroll();
    }

    pub fn is_closed(&self) -> bool {
        self.closed
    }

    pub fn kill(&mut self) {
        self.pty.kill();
        self.closed = true;
    }

    pub fn bracketed_paste(&self) -> bool {
        self.parser.bracketed_paste
    }

    pub fn mouse_tracking(&self) -> bool {
        self.parser.mouse_tracking
    }

    pub fn selection(&self) -> Option<akssora_terminal_screen::SelectionRange> {
        self.grid.selection()
    }

    pub fn set_selection(&mut self, sel: Option<akssora_terminal_screen::SelectionRange>) {
        self.grid.set_selection(sel);
    }

    pub fn clear_selection(&mut self) {
        self.grid.clear_selection();
    }

    pub fn selected_text(&self) -> Option<String> {
        self.grid.selected_text()
    }

    // INFO: Safely pastes clipboard text, using bracketed paste mode if active or normalized newline escaping.
    pub fn paste_text(&mut self, text: &str) {
        self.reset_scroll();
        if self.parser.bracketed_paste {
            let mut payload = Vec::with_capacity(text.len() + 16);
            payload.extend_from_slice(b"\x1b[200~");
            payload.extend_from_slice(text.as_bytes());
            payload.extend_from_slice(b"\x1b[201~");
            self.send(&payload);
        } else {
            // When not bracketed, normalize line endings to prevent accidental instant execution of last command
            let normalized = text.replace("\r\n", "\r").replace('\n', "\r");
            self.send(normalized.as_bytes());
        }
    }
}
