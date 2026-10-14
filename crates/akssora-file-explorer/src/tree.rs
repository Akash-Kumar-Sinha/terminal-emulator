use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use crate::entry::{EntryKind, FileEntry};

// INFO: Visual tree row representation used for layout rendering and hit-testing in explorer panels.
#[derive(Clone, Debug, PartialEq)]
pub struct ExplorerRow {
    pub path: PathBuf,
    pub name: String,
    pub kind: EntryKind,
    pub depth: usize,
    pub is_expanded: bool,
    pub is_selected: bool,
    pub is_hidden: bool,
}

// INFO: Hierarchical file explorer tree model with lazy loading, watching, filtering, and filesystem mutations.
#[derive(Clone, Debug)]
pub struct FileTree {
    pub root_path: PathBuf,
    pub root: FileEntry,
    pub show_hidden: bool,
    pub exclusions: Vec<String>,
    pub selected_paths: HashSet<PathBuf>,
    pub expanded_paths: HashSet<PathBuf>,
    pub focused_path: Option<PathBuf>,
    pub dragged_paths: Vec<PathBuf>,
    pub drop_target: Option<PathBuf>,
    pub last_scanned: Option<std::time::Instant>,
}

impl FileTree {
    pub const DEFAULT_EXCLUSIONS: &'static [&'static str] =
        &["target", ".git", "node_modules", ".cache"];

    pub fn new(root_path: impl AsRef<Path>) -> std::io::Result<Self> {
        let path = root_path
            .as_ref()
            .canonicalize()
            .unwrap_or_else(|_| root_path.as_ref().to_path_buf());
        let exclusions = Self::DEFAULT_EXCLUSIONS
            .iter()
            .map(|s| s.to_string())
            .collect();
        let mut tree = Self {
            root_path: path.clone(),
            root: FileEntry::new(path.clone(), EntryKind::Directory),
            show_hidden: true,
            exclusions,
            selected_paths: HashSet::new(),
            expanded_paths: HashSet::new(),
            focused_path: None,
            dragged_paths: Vec::new(),
            drop_target: None,
            last_scanned: Some(std::time::Instant::now()),
        };
        tree.expanded_paths.insert(path.clone());
        tree.refresh()?;
        Ok(tree)
    }

    pub fn scan(root_path: impl AsRef<Path>) -> std::io::Result<Self> {
        Self::new(root_path)
    }

    pub fn set_root(&mut self, new_root: impl AsRef<Path>) -> std::io::Result<()> {
        let path = new_root
            .as_ref()
            .canonicalize()
            .unwrap_or_else(|_| new_root.as_ref().to_path_buf());
        self.root_path = path.clone();
        self.selected_paths.clear();
        self.expanded_paths.clear();
        self.focused_path = None;
        self.expanded_paths.insert(path);
        self.refresh()
    }

    pub fn set_show_hidden(&mut self, show: bool) {
        self.show_hidden = show;
        let _ = self.refresh();
    }

    pub fn toggle_hidden(&mut self) {
        self.set_show_hidden(!self.show_hidden);
    }

    pub fn set_exclusions(&mut self, exclusions: Vec<String>) {
        self.exclusions = exclusions;
        let _ = self.refresh();
    }

    pub fn is_excluded(&self, name: &str) -> bool {
        self.exclusions.iter().any(|exc| exc == name)
    }

    pub fn should_include(&self, name: &str, is_hidden: bool) -> bool {
        if !self.show_hidden && is_hidden {
            return false;
        }
        if self.is_excluded(name) {
            return false;
        }
        true
    }

    pub fn is_expanded(&self, path: &Path) -> bool {
        self.expanded_paths.contains(path)
    }

    pub fn expand(&mut self, path: &Path) {
        self.expanded_paths.insert(path.to_path_buf());
        let show_hidden = self.show_hidden;
        let exclusions = self.exclusions.clone();
        let expanded = self.expanded_paths.clone();
        if let Some(entry) = self.root.find_mut(path) {
            entry.is_expanded = true;
            if !entry.is_loaded {
                let _ = Self::load_children_for_entry(entry, show_hidden, &exclusions, &expanded);
            }
        }
    }

    pub fn collapse(&mut self, path: &Path) {
        self.expanded_paths.remove(path);
        if let Some(entry) = self.root.find_mut(path) {
            entry.is_expanded = false;
        }
    }

    pub fn toggle_expand(&mut self, path: &Path) {
        if self.is_expanded(path) {
            self.collapse(path);
        } else {
            self.expand(path);
        }
    }

    // INFO: Lazy loads child items for a single directory entry.
    pub fn load_children_for_entry(
        entry: &mut FileEntry,
        show_hidden: bool,
        exclusions: &[String],
        expanded_paths: &HashSet<PathBuf>,
    ) -> std::io::Result<()> {
        if entry.kind != EntryKind::Directory {
            return Ok(());
        }

        let read_dir = fs::read_dir(&entry.path)?;
        let mut children = Vec::new();

        for res in read_dir.flatten() {
            let p = res.path();
            let name = p
                .file_name()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_default();
            let is_hidden = name.starts_with('.');

            if !show_hidden && is_hidden {
                continue;
            }
            if exclusions.iter().any(|exc| exc == &name) {
                continue;
            }

            let meta = res.metadata().ok();
            let kind = if meta.as_ref().map(|m| m.is_dir()).unwrap_or(false) {
                EntryKind::Directory
            } else if meta.as_ref().map(|m| m.is_symlink()).unwrap_or(false) {
                EntryKind::Symlink
            } else {
                EntryKind::File
            };

            let mut child = FileEntry::new(p.clone(), kind);
            child.is_hidden = is_hidden;
            child.is_expanded = expanded_paths.contains(&p);
            children.push(child);
        }

        children.sort_by(|a, b| match (a.kind, b.kind) {
            (EntryKind::Directory, EntryKind::File) => std::cmp::Ordering::Less,
            (EntryKind::File, EntryKind::Directory) => std::cmp::Ordering::Greater,
            _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
        });

        entry.children = children;
        entry.is_loaded = true;
        Ok(())
    }

    pub fn load_children(&self, entry: &mut FileEntry) -> std::io::Result<()> {
        Self::load_children_for_entry(
            entry,
            self.show_hidden,
            &self.exclusions,
            &self.expanded_paths,
        )
    }

    // INFO: Refreshes the tree structure recursively for all expanded nodes.
    pub fn refresh(&mut self) -> std::io::Result<()> {
        let meta = fs::metadata(&self.root_path)?;
        let kind = if meta.is_dir() {
            EntryKind::Directory
        } else if meta.is_symlink() {
            EntryKind::Symlink
        } else {
            EntryKind::File
        };

        let mut root = FileEntry::new(self.root_path.clone(), kind);
        root.is_expanded = self.expanded_paths.contains(&self.root_path);
        self.refresh_recursive(&mut root)?;
        self.root = root;
        self.last_scanned = Some(std::time::Instant::now());
        Ok(())
    }

    // INFO: Periodic filesystem watch polling that triggers tree refresh when watch interval elapses.
    pub fn poll_filesystem_watch(&mut self, interval: std::time::Duration) -> bool {
        if let Some(last) = self.last_scanned {
            if last.elapsed() >= interval {
                let _ = self.refresh();
                return true;
            }
        } else {
            let _ = self.refresh();
            return true;
        }
        false
    }

    fn refresh_recursive(&self, entry: &mut FileEntry) -> std::io::Result<()> {
        if entry.kind == EntryKind::Directory {
            let is_expanded = self.expanded_paths.contains(&entry.path);
            entry.is_expanded = is_expanded;
            if is_expanded {
                Self::load_children_for_entry(
                    entry,
                    self.show_hidden,
                    &self.exclusions,
                    &self.expanded_paths,
                )?;
                for child in &mut entry.children {
                    if child.kind == EntryKind::Directory
                        && self.expanded_paths.contains(&child.path)
                    {
                        let _ = self.refresh_recursive(child);
                    }
                }
            }
        }
        Ok(())
    }

    // INFO: Flattens the visible tree structure into a linear list of display rows.
    pub fn flatten_rows(&self) -> Vec<ExplorerRow> {
        let mut rows = Vec::new();
        for child in &self.root.children {
            self.collect_rows(child, 0, &mut rows);
        }
        rows
    }

    fn collect_rows(&self, entry: &FileEntry, depth: usize, rows: &mut Vec<ExplorerRow>) {
        let is_selected = self.selected_paths.contains(&entry.path);
        rows.push(ExplorerRow {
            path: entry.path.clone(),
            name: entry.name.clone(),
            kind: entry.kind,
            depth,
            is_expanded: entry.is_expanded,
            is_selected,
            is_hidden: entry.is_hidden,
        });

        if entry.is_expanded && entry.kind == EntryKind::Directory {
            for child in &entry.children {
                self.collect_rows(child, depth + 1, rows);
            }
        }
    }

    // ── Multi-File Selection ──────────────────────────────────────────────────

    pub fn select_single(&mut self, path: PathBuf) {
        self.selected_paths.clear();
        self.selected_paths.insert(path.clone());
        self.focused_path = Some(path);
    }

    pub fn toggle_select(&mut self, path: PathBuf) {
        if self.selected_paths.contains(&path) {
            self.selected_paths.remove(&path);
            if self.focused_path == Some(path) {
                self.focused_path = self.selected_paths.iter().next().cloned();
            }
        } else {
            self.selected_paths.insert(path.clone());
            self.focused_path = Some(path);
        }
    }

    pub fn select_range(&mut self, target: &Path) {
        let rows = self.flatten_rows();
        let target_idx = rows.iter().position(|r| r.path == target);
        let anchor_idx = self
            .focused_path
            .as_ref()
            .and_then(|fp| rows.iter().position(|r| &r.path == fp))
            .unwrap_or(0);

        if let Some(t_idx) = target_idx {
            let (start, end) = if anchor_idx <= t_idx {
                (anchor_idx, t_idx)
            } else {
                (t_idx, anchor_idx)
            };
            for r in &rows[start..=end] {
                self.selected_paths.insert(r.path.clone());
            }
            self.focused_path = Some(target.to_path_buf());
        } else {
            self.select_single(target.to_path_buf());
        }
    }

    pub fn clear_selection(&mut self) {
        self.selected_paths.clear();
        self.focused_path = None;
    }

    pub fn is_selected(&self, path: &Path) -> bool {
        self.selected_paths.contains(path)
    }

    // ── Drag and Drop Operations ─────────────────────────────────────────────

    pub fn start_drag(&mut self, path: PathBuf) {
        if !self.selected_paths.contains(&path) {
            self.select_single(path.clone());
        }
        self.dragged_paths = self.selected_paths.iter().cloned().collect();
    }

    pub fn update_drop_target(&mut self, target: Option<PathBuf>) {
        self.drop_target = target;
    }

    pub fn cancel_drag(&mut self) {
        self.dragged_paths.clear();
        self.drop_target = None;
    }

    pub fn finish_drag(&mut self) -> std::io::Result<Vec<PathBuf>> {
        let Some(target_dir) = self.drop_target.take() else {
            self.dragged_paths.clear();
            return Ok(Vec::new());
        };

        let target_is_dir = fs::metadata(&target_dir)
            .map(|m| m.is_dir())
            .unwrap_or(false);
        let dest_folder = if target_is_dir {
            target_dir
        } else {
            target_dir.parent().unwrap_or(&self.root_path).to_path_buf()
        };

        let dragged = std::mem::take(&mut self.dragged_paths);
        let mut moved_paths = Vec::new();

        for src in dragged {
            // Cannot drop directory into itself or child
            if dest_folder.starts_with(&src) {
                continue;
            }
            if let Ok(new_p) = self.move_entry(&src, &dest_folder) {
                moved_paths.push(new_p);
            }
        }

        let _ = self.refresh();
        Ok(moved_paths)
    }

    // ── File and Folder CRUD Operations ──────────────────────────────────────

    pub fn create_file(&mut self, parent: &Path, name: &str) -> std::io::Result<PathBuf> {
        let dir = if parent.is_dir() {
            parent
        } else {
            parent.parent().unwrap_or(&self.root_path)
        };
        let file_path = dir.join(name);
        fs::File::create(&file_path)?;
        self.expanded_paths.insert(dir.to_path_buf());
        self.refresh()?;
        self.select_single(file_path.clone());
        Ok(file_path)
    }

    pub fn create_dir(&mut self, parent: &Path, name: &str) -> std::io::Result<PathBuf> {
        let dir = if parent.is_dir() {
            parent
        } else {
            parent.parent().unwrap_or(&self.root_path)
        };
        let dir_path = dir.join(name);
        fs::create_dir_all(&dir_path)?;
        self.expanded_paths.insert(dir.to_path_buf());
        self.refresh()?;
        self.select_single(dir_path.clone());
        Ok(dir_path)
    }

    pub fn rename(&mut self, old_path: &Path, new_name: &str) -> std::io::Result<PathBuf> {
        let parent = old_path.parent().unwrap_or(&self.root_path);
        let new_path = parent.join(new_name);
        fs::rename(old_path, &new_path)?;

        if self.expanded_paths.remove(old_path) {
            self.expanded_paths.insert(new_path.clone());
        }
        if self.selected_paths.remove(old_path) {
            self.selected_paths.insert(new_path.clone());
        }
        if self.focused_path.as_deref() == Some(old_path) {
            self.focused_path = Some(new_path.clone());
        }

        self.refresh()?;
        Ok(new_path)
    }

    pub fn move_entry(&mut self, src: &Path, target_dir: &Path) -> std::io::Result<PathBuf> {
        let filename = src.file_name().ok_or_else(|| {
            std::io::Error::new(std::io::ErrorKind::InvalidInput, "Invalid file name")
        })?;
        let dest = target_dir.join(filename);
        if src == dest {
            return Ok(dest);
        }
        fs::rename(src, &dest)?;

        if self.expanded_paths.remove(src) {
            self.expanded_paths.insert(dest.clone());
        }
        if self.selected_paths.remove(src) {
            self.selected_paths.insert(dest.clone());
        }
        if self.focused_path.as_deref() == Some(src) {
            self.focused_path = Some(dest.clone());
        }

        self.refresh()?;
        Ok(dest)
    }

    pub fn copy_entry(&mut self, src: &Path, target_dir: &Path) -> std::io::Result<PathBuf> {
        let filename = src.file_name().ok_or_else(|| {
            std::io::Error::new(std::io::ErrorKind::InvalidInput, "Invalid file name")
        })?;
        let mut dest = target_dir.join(filename);

        // If destination exists, append " copy"
        if dest.exists() {
            let stem = src
                .file_stem()
                .map(|s| s.to_string_lossy())
                .unwrap_or_default();
            let ext = src
                .extension()
                .map(|e| format!(".{}", e.to_string_lossy()))
                .unwrap_or_default();
            dest = target_dir.join(format!("{stem} copy{ext}"));
        }

        if src.is_dir() {
            copy_dir_recursive(src, &dest)?;
        } else {
            fs::copy(src, &dest)?;
        }

        self.refresh()?;
        self.select_single(dest.clone());
        Ok(dest)
    }

    pub fn delete(&mut self, path: &Path) -> std::io::Result<()> {
        if path.is_dir() {
            fs::remove_dir_all(path)?;
        } else {
            fs::remove_file(path)?;
        }

        self.expanded_paths.remove(path);
        self.selected_paths.remove(path);
        if self.focused_path.as_deref() == Some(path) {
            self.focused_path = None;
        }

        self.refresh()?;
        Ok(())
    }

    pub fn delete_selected(&mut self) -> std::io::Result<Vec<PathBuf>> {
        let to_delete: Vec<PathBuf> = self.selected_paths.iter().cloned().collect();
        let mut deleted = Vec::new();
        for p in to_delete {
            if self.delete(&p).is_ok() {
                deleted.push(p);
            }
        }
        self.clear_selection();
        Ok(deleted)
    }
}

// INFO: Copies a directory tree recursively from source to target.
fn copy_dir_recursive(src: &Path, dst: &Path) -> std::io::Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)?.flatten() {
        let entry_path = entry.path();
        let dest_child = dst.join(entry.file_name());
        if entry_path.is_dir() {
            copy_dir_recursive(&entry_path, &dest_child)?;
        } else {
            fs::copy(&entry_path, &dest_child)?;
        }
    }
    Ok(())
}
