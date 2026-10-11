use std::fs;
use std::path::{Path, PathBuf};

use crate::entry::{EntryKind, FileEntry};

// INFO: Recursive directory scanner constructing workspace file tree models.
#[derive(Clone, Debug)]
pub struct FileTree {
    pub root: FileEntry,
}

impl FileTree {
    pub fn scan(root_path: impl AsRef<Path>) -> std::io::Result<Self> {
        let path = root_path.as_ref().to_path_buf();
        let root = Self::scan_entry(&path)?;
        Ok(Self { root })
    }

    fn scan_entry(path: &PathBuf) -> std::io::Result<FileEntry> {
        let meta = fs::metadata(path)?;
        let kind = if meta.is_dir() {
            EntryKind::Directory
        } else if meta.is_symlink() {
            EntryKind::Symlink
        } else {
            EntryKind::File
        };

        let mut entry = FileEntry::new(path.clone(), kind);
        if kind == EntryKind::Directory
            && let Ok(entries) = fs::read_dir(path)
        {
            let mut children = Vec::new();
            for res in entries.flatten() {
                let p = res.path();
                let child_meta = res.metadata().ok();
                let child_kind = if child_meta.as_ref().map(|m| m.is_dir()).unwrap_or(false) {
                    EntryKind::Directory
                } else {
                    EntryKind::File
                };
                children.push(FileEntry::new(p, child_kind));
            }
            children.sort_by(|a, b| match (a.kind, b.kind) {
                (EntryKind::Directory, EntryKind::File) => std::cmp::Ordering::Less,
                (EntryKind::File, EntryKind::Directory) => std::cmp::Ordering::Greater,
                _ => a.name.cmp(&b.name),
            });
            entry.children = children;
        }
        Ok(entry)
    }
}
