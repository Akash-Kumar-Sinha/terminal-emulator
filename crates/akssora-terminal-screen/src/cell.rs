use akssora_terminal_config::Color;

// INFO: Represents character content and visual styling attributes for a single terminal display cell.
#[derive(Clone, Copy, PartialEq, Debug)]
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

impl Cell {
    pub fn new(ch: char) -> Self {
        Self {
            ch,
            ..Default::default()
        }
    }

    pub fn with_style(ch: char, fg: Color, bg: Color, bold: bool, italic: bool) -> Self {
        Self {
            ch,
            fg,
            bg,
            bold,
            italic,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.ch == ' ' || self.ch == '\0'
    }
}
