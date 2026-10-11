// INFO: Manages terminal cursor coordinates and visibility status.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Cursor {
    pub row: usize,
    pub col: usize,
    pub visible: bool,
}

impl Cursor {
    pub fn new(row: usize, col: usize) -> Self {
        Self {
            row,
            col,
            visible: true,
        }
    }

    pub fn pos(&self) -> (usize, usize) {
        (self.row, self.col)
    }

    pub fn set_pos(&mut self, row: usize, col: usize) {
        self.row = row;
        self.col = col;
    }
}
