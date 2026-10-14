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
    pub is_loaded: bool,
    pub is_hidden: bool,
    pub children: Vec<FileEntry>,
}

impl FileEntry {
    pub fn new(path: PathBuf, kind: EntryKind) -> Self {
        let name = path
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| "/".to_string());
        let is_hidden = name.starts_with('.');
        Self {
            path,
            name,
            kind,
            is_expanded: false,
            is_loaded: false,
            is_hidden,
            children: Vec::new(),
        }
    }

    pub fn is_dir(&self) -> bool {
        self.kind == EntryKind::Directory
    }

    pub fn find(&self, target: &std::path::Path) -> Option<&FileEntry> {
        if self.path == target {
            return Some(self);
        }
        for child in &self.children {
            if let Some(found) = child.find(target) {
                return Some(found);
            }
        }
        None
    }

    pub fn find_mut(&mut self, target: &std::path::Path) -> Option<&mut FileEntry> {
        if self.path == target {
            return Some(self);
        }
        for child in &mut self.children {
            if let Some(found) = child.find_mut(target) {
                return Some(found);
            }
        }
        None
    }
}
