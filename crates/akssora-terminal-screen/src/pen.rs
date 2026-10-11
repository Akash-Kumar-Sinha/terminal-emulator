use akssora_terminal_config::Color;

// INFO: Tracks active text attributes and color states applied to newly rendered cells.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Pen {
    pub fg: Color,
    pub bg: Color,
    pub bold: bool,
    pub italic: bool,
    pub inverse: bool,
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
    pub fn cell_style(&self) -> (Color, Color) {
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

    pub fn reset(&mut self) {
        *self = Self::default();
    }
}
