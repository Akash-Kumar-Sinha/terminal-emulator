use std::path::PathBuf;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EntryKind {
    File,
    Directory,
    Symlink,
}

// INFO: Filesystem node representing either a file or a folder hierarchy node in the explorer.
#[derive(Clone, Debug, PartialEq)]
pub struct FileEntry {
    pub path: PathBuf,
    pub name: String,
    pub kind: EntryKind,
    pub is_expanded: bool,
    pub children: Vec<FileEntry>,
}

impl FileEntry {
    pub fn new(path: PathBuf, kind: EntryKind) -> Self {
        let name = path
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| "/".to_string());
        Self {
            path,
            name,
            kind,
            is_expanded: false,
            children: Vec::new(),
        }
    }
}
