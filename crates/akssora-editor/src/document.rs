use std::path::PathBuf;

use crate::buffer::TextBuffer;

// INFO: Editor document model encapsulating file metadata, buffer content, and modification state.
#[derive(Clone, Debug, Default)]
pub struct Document {
    pub path: Option<PathBuf>,
    pub buffer: TextBuffer,
    pub cursor: (usize, usize),
    pub dirty: bool,
}

impl Document {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn open(path: PathBuf, content: &str) -> Self {
        Self {
            path: Some(path),
            buffer: TextBuffer::from_text(content),
            cursor: (0, 0),
            dirty: false,
        }
    }
}
