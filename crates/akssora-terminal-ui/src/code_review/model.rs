use std::cell::{Cell, RefCell};
use std::path::{Path, PathBuf};

use akssora_git::{FileDiff, GitInspector, RepoDiff};

use super::layout::CodeReviewLayout;
use super::types::{
    CodeReviewGutterHover, CodeReviewSidebarMode, CodeReviewTreeItem, DiffViewMode, DiscardTarget,
    ReviewComment,
};

#[derive(Clone, Debug, PartialEq)]
pub struct CodeReviewModel {
    pub repo_root: PathBuf,
    pub branch: String,
    pub diff: RepoDiff,
    pub staged_diff: RepoDiff,
    pub unstaged_diff: RepoDiff,
    pub is_staged: bool,
    pub selected_file_idx: usize,
    pub collapsed_files: std::collections::HashSet<PathBuf>,
    pub full_file_files: std::collections::HashSet<PathBuf>,
    pub expanded_context: std::collections::HashSet<(PathBuf, usize, bool)>,
    pub sidebar_expanded: bool,
    pub sidebar_width: f32,
    pub sidebar_mode: CodeReviewSidebarMode,
    pub collapsed_tree_dirs: std::collections::HashSet<PathBuf>,
    pub staged_collapsed: bool,
    pub unstaged_collapsed: bool,
    pub confirm_discard: Option<DiscardTarget>,
    pub is_resizing_sidebar: bool,
    pub hovered_sidebar_edge: bool,
    pub active_file_idx: Option<usize>,
    pub active_editing_file: Option<PathBuf>,
    pub diff_mode: DiffViewMode,
    pub filter_query: String,
    pub comments: Vec<ReviewComment>,
    pub next_comment_id: u64,
    pub active_comment_input: Option<(usize, usize, String)>, // (file_idx, line_idx, draft)
    pub commit_dialog_open: bool,
    pub commit_message: String,
    pub status_banner: Option<String>,
    pub diff_scroll_y: f32,
    pub sidebar_scroll_y: f32,
    pub is_dragging_diff_scrollbar: bool,
    pub diff_scrollbar_drag_start_y: f32,
    pub diff_scrollbar_drag_start_scroll: f32,
    pub indexed_diff: Option<std::sync::Arc<akssora_git::IndexedRepoDiff>>,
    pub is_loading: bool,
    pub diff_generation: u64,
    pub hovered_gutter: Option<CodeReviewGutterHover>,
    pub is_open: bool,
    pub is_full_view: bool,
    pub split_ratio: f32,
    pub is_resizing_split: bool,
    pub hovered_split_divider: bool,
    pub(crate) cached_total_diff_h: Cell<Option<f32>>,
    pub(crate) cached_tree_items: RefCell<Option<Vec<CodeReviewTreeItem>>>,
}

impl Default for CodeReviewModel {
    fn default() -> Self {
        Self {
            repo_root: PathBuf::from("."),
            branch: "main".to_string(),
            diff: RepoDiff::default(),
            staged_diff: RepoDiff::default(),
            unstaged_diff: RepoDiff::default(),
            is_staged: false,
            selected_file_idx: 0,
            collapsed_files: std::collections::HashSet::new(),
            full_file_files: std::collections::HashSet::new(),
            expanded_context: std::collections::HashSet::new(),
            sidebar_expanded: true,
            sidebar_width: 260.0,
            sidebar_mode: CodeReviewSidebarMode::Tree,
            collapsed_tree_dirs: std::collections::HashSet::new(),
            staged_collapsed: false,
            unstaged_collapsed: false,
            confirm_discard: None,
            is_resizing_sidebar: false,
            hovered_sidebar_edge: false,
            active_file_idx: None,
            active_editing_file: None,
            diff_mode: DiffViewMode::Unified,
            filter_query: String::new(),
            comments: Vec::new(),
            next_comment_id: 1,
            active_comment_input: None,
            commit_dialog_open: false,
            commit_message: String::new(),
            status_banner: None,
            diff_scroll_y: 0.0,
            sidebar_scroll_y: 0.0,
            is_dragging_diff_scrollbar: false,
            diff_scrollbar_drag_start_y: 0.0,
            diff_scrollbar_drag_start_scroll: 0.0,
            indexed_diff: None,
            is_loading: false,
            diff_generation: 0,
            hovered_gutter: None,
            is_open: false,
            is_full_view: false,
            split_ratio: 0.5,
            is_resizing_split: false,
            hovered_split_divider: false,
            cached_total_diff_h: Cell::new(None),
            cached_tree_items: RefCell::new(None),
        }
    }
}

impl CodeReviewModel {
    pub fn toggle_full_view(&mut self) {
        self.is_full_view = !self.is_full_view;
        self.invalidate_layout();
    }

    pub fn set_split_ratio(&mut self, ratio: f32) {
        self.split_ratio = ratio.clamp(0.15, 0.85);
        self.invalidate_layout();
    }

    pub fn invalidate_layout(&self) {
        self.cached_total_diff_h.set(None);
        self.cached_tree_items.borrow_mut().take();
    }

    pub fn set_indexed_diff(
        &mut self,
        indexed: std::sync::Arc<akssora_git::IndexedRepoDiff>,
        generation: u64,
    ) {
        if generation >= self.diff_generation {
            self.diff_generation = generation;
            self.diff = indexed.repo_diff.clone();
            self.indexed_diff = Some(indexed);
            self.is_loading = false;
            self.invalidate_layout();
            if self.selected_file_idx >= self.diff.files.len() {
                self.selected_file_idx = self.diff.files.len().saturating_sub(1);
            }
            if let Some(ref path) = self.active_editing_file {
                self.active_file_idx = self
                    .diff
                    .files
                    .iter()
                    .position(|f| self.repo_root.join(&f.path) == *path);
            }
            if self.active_file_idx.is_none() && !self.diff.files.is_empty() {
                self.select_file(self.selected_file_idx.min(self.diff.files.len() - 1));
            }
        }
    }

    pub fn new(repo_path: impl AsRef<Path>) -> Self {
        let repo_root =
            GitInspector::repo_root(&repo_path).unwrap_or_else(|| repo_path.as_ref().to_path_buf());
        let branch =
            GitInspector::current_branch(&repo_root).unwrap_or_else(|| "detached".to_string());
        let staged_indexed = akssora_git::load_repo_diff_indexed(&repo_root, true);
        let unstaged_indexed = akssora_git::load_repo_diff_indexed(&repo_root, false);
        let staged_diff = staged_indexed.repo_diff.clone();
        let unstaged_diff = unstaged_indexed.repo_diff.clone();
        let diff = unstaged_diff.clone();

        let mut model = Self {
            repo_root,
            branch,
            diff,
            staged_diff,
            unstaged_diff,
            is_staged: false,
            selected_file_idx: 0,
            collapsed_files: std::collections::HashSet::new(),
            full_file_files: std::collections::HashSet::new(),
            expanded_context: std::collections::HashSet::new(),
            sidebar_expanded: true,
            sidebar_width: 260.0,
            sidebar_mode: CodeReviewSidebarMode::Tree,
            collapsed_tree_dirs: std::collections::HashSet::new(),
            staged_collapsed: false,
            unstaged_collapsed: false,
            confirm_discard: None,
            is_resizing_sidebar: false,
            hovered_sidebar_edge: false,
            active_file_idx: None,
            active_editing_file: None,
            diff_mode: DiffViewMode::Unified,
            filter_query: String::new(),
            comments: Vec::new(),
            next_comment_id: 1,
            active_comment_input: None,
            commit_dialog_open: false,
            commit_message: String::new(),
            status_banner: None,
            diff_scroll_y: 0.0,
            sidebar_scroll_y: 0.0,
            is_dragging_diff_scrollbar: false,
            diff_scrollbar_drag_start_y: 0.0,
            diff_scrollbar_drag_start_scroll: 0.0,
            indexed_diff: Some(std::sync::Arc::new(unstaged_indexed)),
            is_loading: false,
            diff_generation: 1,
            hovered_gutter: None,
            is_open: false,
            is_full_view: false,
            split_ratio: 0.5,
            is_resizing_split: false,
            hovered_split_divider: false,
            cached_total_diff_h: Cell::new(None),
            cached_tree_items: RefCell::new(None),
        };
        if !model.diff.files.is_empty() {
            model.select_file(0);
        }
        model
    }

    pub fn refresh(&mut self) {
        self.invalidate_layout();
        self.branch =
            GitInspector::current_branch(&self.repo_root).unwrap_or_else(|| "detached".to_string());
        let staged_indexed = akssora_git::load_repo_diff_indexed(&self.repo_root, true);
        let unstaged_indexed = akssora_git::load_repo_diff_indexed(&self.repo_root, false);
        self.staged_diff = staged_indexed.repo_diff.clone();
        self.unstaged_diff = unstaged_indexed.repo_diff.clone();
        let next_gen = self.diff_generation + 1;
        let active = if self.is_staged {
            staged_indexed
        } else {
            unstaged_indexed
        };
        self.set_indexed_diff(std::sync::Arc::new(active), next_gen);
        if self.selected_file_idx >= self.diff.files.len() {
            self.selected_file_idx = self.diff.files.len().saturating_sub(1);
        }
        if let Some(ref path) = self.active_editing_file {
            self.active_file_idx = self
                .diff
                .files
                .iter()
                .position(|f| self.repo_root.join(&f.path) == *path);
        } else if !self.diff.files.is_empty() && self.active_file_idx.is_none() {
            self.select_file(0);
        }
    }

    pub fn toggle_sidebar_mode(&mut self) {
        self.cached_tree_items.borrow_mut().take();
        self.sidebar_mode = match self.sidebar_mode {
            CodeReviewSidebarMode::Tree => CodeReviewSidebarMode::FlatList,
            CodeReviewSidebarMode::FlatList => CodeReviewSidebarMode::Tree,
        };
    }

    pub fn toggle_section_collapse(&mut self, is_staged: bool) {
        self.cached_tree_items.borrow_mut().take();
        if is_staged {
            self.staged_collapsed = !self.staged_collapsed;
        } else {
            self.unstaged_collapsed = !self.unstaged_collapsed;
        }
    }

    pub fn toggle_tree_dir(&mut self, dir: PathBuf) {
        self.cached_tree_items.borrow_mut().take();
        if self.collapsed_tree_dirs.contains(&dir) {
            self.collapsed_tree_dirs.remove(&dir);
        } else {
            self.collapsed_tree_dirs.insert(dir);
        }
    }

    // INFO: Flattens hierarchical changed files into visible tree rows with separate Staged/Changes sections and compact folders.
    pub fn visible_tree_items(&self) -> Vec<CodeReviewTreeItem> {
        if let Some(items) = self.cached_tree_items.borrow().as_ref() {
            return items.clone();
        }

        // Determine staged and unstaged file collections
        let (raw_staged, raw_unstaged) = if self.staged_diff.files.is_empty()
            && self.unstaged_diff.files.is_empty()
            && !self.diff.files.is_empty()
        {
            if self.is_staged {
                (&self.diff.files, &self.unstaged_diff.files)
            } else {
                (&self.staged_diff.files, &self.diff.files)
            }
        } else {
            (&self.staged_diff.files, &self.unstaged_diff.files)
        };

        let filter = self.filter_query.to_lowercase();
        let staged_filtered: Vec<(usize, &FileDiff)> = raw_staged
            .iter()
            .enumerate()
            .filter(|(_, f)| {
                filter.is_empty() || f.path.to_string_lossy().to_lowercase().contains(&filter)
            })
            .collect();

        let unstaged_filtered: Vec<(usize, &FileDiff)> = raw_unstaged
            .iter()
            .enumerate()
            .filter(|(_, f)| {
                filter.is_empty() || f.path.to_string_lossy().to_lowercase().contains(&filter)
            })
            .collect();

        if !akssora_git::GitInspector::is_git_repo(&self.repo_root) {
            *self.cached_tree_items.borrow_mut() = Some(Vec::new());
            return Vec::new();
        }

        let mut items = Vec::new();

        // 1. Staged Changes Section Header
        items.push(CodeReviewTreeItem::SectionHeader {
            is_staged: true,
            count: staged_filtered.len(),
            is_collapsed: self.staged_collapsed,
        });

        if !self.staged_collapsed {
            if self.sidebar_mode == CodeReviewSidebarMode::FlatList {
                for (orig_idx, file) in &staged_filtered {
                    items.push(CodeReviewTreeItem::File {
                        is_staged: true,
                        orig_idx: *orig_idx,
                        path: file.path.clone(),
                        display_name: file.path.to_string_lossy().to_string(),
                        depth: 1,
                        status: file.status,
                        additions: file.additions,
                        deletions: file.deletions,
                    });
                }
            } else {
                build_section_tree_items(
                    true,
                    &staged_filtered,
                    &self.collapsed_tree_dirs,
                    &mut items,
                );
            }
        }

        // 2. Changes (Unstaged) Section Header
        items.push(CodeReviewTreeItem::SectionHeader {
            is_staged: false,
            count: unstaged_filtered.len(),
            is_collapsed: self.unstaged_collapsed,
        });

        if !self.unstaged_collapsed {
            if self.sidebar_mode == CodeReviewSidebarMode::FlatList {
                for (orig_idx, file) in &unstaged_filtered {
                    items.push(CodeReviewTreeItem::File {
                        is_staged: false,
                        orig_idx: *orig_idx,
                        path: file.path.clone(),
                        display_name: file.path.to_string_lossy().to_string(),
                        depth: 1,
                        status: file.status,
                        additions: file.additions,
                        deletions: file.deletions,
                    });
                }
            } else {
                build_section_tree_items(
                    false,
                    &unstaged_filtered,
                    &self.collapsed_tree_dirs,
                    &mut items,
                );
            }
        }

        *self.cached_tree_items.borrow_mut() = Some(items.clone());
        items
    }

    pub fn toggle_staged(&mut self) {
        self.is_staged = !self.is_staged;
        self.refresh();
    }

    pub fn toggle_diff_mode(&mut self) {
        self.diff_mode = match self.diff_mode {
            DiffViewMode::Unified => DiffViewMode::Split,
            DiffViewMode::Split => DiffViewMode::Unified,
        };
    }

    pub fn is_file_collapsed(&self, path: &Path) -> bool {
        self.collapsed_files.contains(path)
    }

    // INFO: Toggles file collapse state.
    pub fn toggle_file_collapsed(&mut self, path: &Path) {
        self.cached_total_diff_h.set(None);
        if self.collapsed_files.contains(path) {
            self.collapsed_files.remove(path);
        } else {
            self.collapsed_files.insert(path.to_path_buf());
        }
    }

    // INFO: Programmatically sets collapsed state for tests.
    pub fn set_file_collapsed(&mut self, path: &Path, collapsed: bool) {
        self.cached_total_diff_h.set(None);
        if collapsed {
            self.collapsed_files.insert(path.to_path_buf());
        } else {
            self.collapsed_files.remove(path);
        }
    }

    pub fn is_full_file(&self, path: &Path) -> bool {
        self.full_file_files.contains(path)
    }

    pub fn toggle_full_file(&mut self, path: &Path) {
        self.cached_total_diff_h.set(None);
        if self.full_file_files.contains(path) {
            self.full_file_files.remove(path);
        } else {
            self.full_file_files.insert(path.to_path_buf());
        }
    }

    pub fn is_context_expanded(&self, path: &Path, hunk_idx: usize, is_top: bool) -> bool {
        self.expanded_context
            .contains(&(path.to_path_buf(), hunk_idx, is_top))
    }

    pub fn set_context_expanded(&mut self, path: &Path, hunk_idx: usize, is_top: bool) {
        self.cached_total_diff_h.set(None);
        self.expanded_context
            .insert((path.to_path_buf(), hunk_idx, is_top));
    }

    // INFO: Collapses all file diff cards.
    pub fn collapse_all(&mut self) {
        self.cached_total_diff_h.set(None);
        for file in &self.diff.files {
            self.collapsed_files.insert(file.path.clone());
        }
    }

    // INFO: Expands all file diff cards.
    pub fn expand_all(&mut self) {
        self.cached_total_diff_h.set(None);
        self.collapsed_files.clear();
    }

    // INFO: Toggles collapsing or expanding all file diff cards simultaneously.
    pub fn toggle_collapse_all(&mut self) {
        self.cached_total_diff_h.set(None);
        if !self.diff.files.is_empty() && self.collapsed_files.len() >= self.diff.files.len() {
            self.expand_all();
        } else {
            self.collapse_all();
        }
    }

    pub fn filtered_files(&self) -> Vec<(usize, &FileDiff)> {
        if self.filter_query.is_empty() {
            return self.diff.files.iter().enumerate().collect();
        }
        let lower = self.filter_query.to_lowercase();
        self.diff
            .files
            .iter()
            .enumerate()
            .filter(|(_, f)| f.path.to_string_lossy().to_lowercase().contains(&lower))
            .collect()
    }

    pub fn set_filter_query(&mut self, query: String) {
        if self.filter_query != query {
            self.filter_query = query;
            self.invalidate_layout();
        }
    }

    pub fn selected_file(&self) -> Option<&FileDiff> {
        self.diff.files.get(self.selected_file_idx)
    }

    pub fn select_file(&mut self, idx: usize) {
        if idx < self.diff.files.len() {
            self.selected_file_idx = idx;
            if self.active_file_idx != Some(idx) {
                self.active_file_idx = Some(idx);
                let full_path = self.repo_root.join(&self.diff.files[idx].path);
                self.active_editing_file = Some(full_path);
                self.cached_total_diff_h.set(None);
            }
        }
    }

    pub fn stage_file_by_path(&mut self, path: &Path) {
        let _ = GitInspector::stage_file(&self.repo_root, path);
        self.refresh();
    }

    pub fn unstage_file_by_path(&mut self, path: &Path) {
        let _ = GitInspector::unstage_file(&self.repo_root, path);
        self.refresh();
    }

    pub fn discard_file_by_path(&mut self, path: &Path) {
        let _ = GitInspector::discard_file_changes(&self.repo_root, path);
        self.refresh();
    }

    pub fn stage_directory_by_path(&mut self, dir: &Path) {
        let _ = GitInspector::stage_directory(&self.repo_root, dir);
        self.refresh();
    }

    pub fn unstage_directory_by_path(&mut self, dir: &Path) {
        let _ = GitInspector::unstage_directory(&self.repo_root, dir);
        self.refresh();
    }

    pub fn discard_directory_by_path(&mut self, dir: &Path) {
        let _ = GitInspector::discard_directory_changes(&self.repo_root, dir);
        self.refresh();
    }

    pub fn stage_all(&mut self) {
        let _ = GitInspector::stage_all(&self.repo_root);
        self.refresh();
    }

    pub fn unstage_all(&mut self) {
        let _ = GitInspector::unstage_all(&self.repo_root);
        self.refresh();
    }

    pub fn discard_all_changes(&mut self) {
        let _ = GitInspector::discard_all_changes(&self.repo_root);
        self.refresh();
    }

    pub fn stage_file_at(&mut self, idx: usize) {
        if let Some(file) = self.diff.files.get(idx) {
            let path = file.path.clone();
            self.stage_file_by_path(&path);
        }
    }

    pub fn unstage_file_at(&mut self, idx: usize) {
        if let Some(file) = self.diff.files.get(idx) {
            let path = file.path.clone();
            self.unstage_file_by_path(&path);
        }
    }

    pub fn discard_file_at(&mut self, idx: usize) {
        if let Some(file) = self.diff.files.get(idx) {
            let path = file.path.clone();
            self.discard_file_by_path(&path);
        }
    }

    pub fn commit_staged(&mut self, message: &str) -> Result<String, String> {
        let trimmed = message.trim();
        if trimmed.is_empty() {
            return Err("Commit message cannot be empty".to_string());
        }
        let res = GitInspector::commit(&self.repo_root, trimmed)?;
        self.commit_message.clear();
        self.commit_dialog_open = false;
        self.refresh();
        Ok(res)
    }

    pub fn push(&mut self) -> Result<String, String> {
        let res = GitInspector::push(&self.repo_root)?;
        self.refresh();
        Ok(res)
    }

    pub fn add_comment(
        &mut self,
        file_path: PathBuf,
        line_number: usize,
        author: &str,
        body: &str,
    ) {
        let comment = ReviewComment {
            id: self.next_comment_id,
            file_path,
            line_number,
            author: author.to_string(),
            body: body.to_string(),
            created_at: "Just now".to_string(),
            resolved: false,
        };
        self.next_comment_id += 1;
        self.comments.push(comment);
        self.active_comment_input = None;
    }

    pub fn toggle_comment_resolved(&mut self, id: u64) {
        if let Some(c) = self.comments.iter_mut().find(|c| c.id == id) {
            c.resolved = !c.resolved;
        }
    }

    pub fn delete_comment(&mut self, id: u64) {
        self.comments.retain(|c| c.id != id);
    }

    pub fn comments_for_file_and_line(
        &self,
        file_path: &Path,
        line_number: usize,
    ) -> Vec<&ReviewComment> {
        self.comments
            .iter()
            .filter(|c| c.file_path == file_path && c.line_number == line_number)
            .collect()
    }

    pub fn comments_count_for_file(&self, file_path: &Path) -> usize {
        self.comments
            .iter()
            .filter(|c| c.file_path == file_path)
            .count()
    }

    // INFO: Computes total scrollable diff content height bounded by viewport sizing.
    pub fn total_diff_content_height(&self, viewport_h: f32) -> f32 {
        if let Some(h) = self.cached_total_diff_h.get() {
            return h;
        }
        let mut total: f32 = 12.0;
        for (idx, file) in self.diff.files.iter().enumerate() {
            let is_collapsed = self.is_file_collapsed(&file.path);
            let is_editing = self.active_file_idx == Some(idx);
            let is_full = self.is_full_file(&file.path);
            total += CodeReviewLayout::card_height_for_file_state_ex(
                file,
                is_collapsed,
                is_editing,
                is_full,
                viewport_h,
            ) + CodeReviewLayout::CARD_MARGIN_BOTTOM;
        }
        self.cached_total_diff_h.set(Some(total));
        total
    }

    pub fn card_y_offset_for_file(&self, target_idx: usize, viewport_h: f32) -> f32 {
        CodeReviewLayout::card_y_offset_for_file(self, target_idx, viewport_h)
    }

    pub fn max_diff_scroll(&self, viewport_h: f32) -> f32 {
        (self.total_diff_content_height(viewport_h) - viewport_h).max(0.0)
    }

    pub fn max_sidebar_scroll(&self, viewport_h: f32) -> f32 {
        let sidebar_total_h = self.visible_tree_items().len() as f32
            * CodeReviewLayout::FILE_ROW_HEIGHT
            + CodeReviewLayout::SIDEBAR_TOOLBAR_HEIGHT
            + 16.0;
        (sidebar_total_h - viewport_h).max(0.0)
    }

    pub fn scroll_diff(&mut self, dy: f32, viewport_h: f32) {
        let max_scroll = self.max_diff_scroll(viewport_h);
        self.diff_scroll_y = (self.diff_scroll_y + dy).clamp(0.0, max_scroll);
    }

    pub fn scroll_sidebar(&mut self, dy: f32, viewport_h: f32) {
        let max_scroll = self.max_sidebar_scroll(viewport_h);
        self.sidebar_scroll_y = (self.sidebar_scroll_y + dy).clamp(0.0, max_scroll);
    }

    pub fn clamp_scrolls(&mut self, diff_viewport_h: f32, sidebar_viewport_h: f32) {
        let max_diff = self.max_diff_scroll(diff_viewport_h);
        self.diff_scroll_y = self.diff_scroll_y.clamp(0.0, max_diff);
        let max_sb = self.max_sidebar_scroll(sidebar_viewport_h);
        self.sidebar_scroll_y = self.sidebar_scroll_y.clamp(0.0, max_sb);
    }
}

#[derive(Default)]
struct DirTreeNode {
    name: String,
    path: PathBuf,
    subdirs: std::collections::BTreeMap<String, DirTreeNode>,
    files: Vec<(
        usize,
        PathBuf,
        String,
        akssora_git::FileGitStatus,
        usize,
        usize,
    )>,
}

impl DirTreeNode {
    fn compact(&mut self) {
        for child in self.subdirs.values_mut() {
            child.compact();
        }
        for child in self.subdirs.values_mut() {
            while child.files.is_empty() && child.subdirs.len() == 1 {
                let (_grandchild_key, grandchild) = child.subdirs.pop_first().unwrap();
                child.name = format!("{}/{}", child.name, grandchild.name);
                child.path = grandchild.path;
                child.files = grandchild.files;
                child.subdirs = grandchild.subdirs;
            }
        }
    }

    fn flatten(
        &self,
        is_staged: bool,
        depth: usize,
        collapsed_dirs: &std::collections::HashSet<PathBuf>,
        items: &mut Vec<CodeReviewTreeItem>,
    ) {
        for child in self.subdirs.values() {
            let is_collapsed = collapsed_dirs.contains(&child.path);
            items.push(CodeReviewTreeItem::Directory {
                is_staged,
                path: child.path.clone(),
                display_name: child.name.clone(),
                depth,
                is_collapsed,
            });
            if !is_collapsed {
                child.flatten(is_staged, depth + 1, collapsed_dirs, items);
            }
        }
        for (orig_idx, path, display_name, status, additions, deletions) in &self.files {
            items.push(CodeReviewTreeItem::File {
                is_staged,
                orig_idx: *orig_idx,
                path: path.clone(),
                display_name: display_name.clone(),
                depth,
                status: *status,
                additions: *additions,
                deletions: *deletions,
            });
        }
    }
}

fn sort_node_files_recursive(node: &mut DirTreeNode) {
    node.files.sort_by(|a, b| a.2.cmp(&b.2));
    for child in node.subdirs.values_mut() {
        sort_node_files_recursive(child);
    }
}

fn build_section_tree_items(
    is_staged: bool,
    files: &[(usize, &FileDiff)],
    collapsed_dirs: &std::collections::HashSet<PathBuf>,
    items: &mut Vec<CodeReviewTreeItem>,
) {
    let mut root = DirTreeNode::default();
    for (orig_idx, file) in files {
        let path = &file.path;
        let components: Vec<_> = path
            .components()
            .filter(|c| c.as_os_str() != ".")
            .map(|c| c.as_os_str().to_string_lossy().to_string())
            .collect();
        if components.is_empty() {
            continue;
        }
        let file_name = components.last().unwrap().clone();
        let dir_parts = &components[..components.len() - 1];

        let mut curr = &mut root;
        let mut curr_path = PathBuf::new();
        for part in dir_parts {
            curr_path.push(part);
            curr = curr
                .subdirs
                .entry(part.clone())
                .or_insert_with(|| DirTreeNode {
                    name: part.clone(),
                    path: curr_path.clone(),
                    subdirs: std::collections::BTreeMap::new(),
                    files: Vec::new(),
                });
        }
        curr.files.push((
            *orig_idx,
            path.clone(),
            file_name,
            file.status,
            file.additions,
            file.deletions,
        ));
    }
    sort_node_files_recursive(&mut root);

    root.compact();
    root.flatten(is_staged, 1, collapsed_dirs, items);
}

#[cfg(test)]
mod tests {
    use super::*;
    use akssora_git::{DiffHunk, DiffLine, DiffLineType, FileDiff, FileGitStatus};

    #[test]
    fn test_accordion_toggle_collapse_expand() {
        let mut model = CodeReviewModel::default();
        let path = PathBuf::from("src/main.rs");

        // Initially not in collapsed_files (expanded)
        assert!(!model.is_file_collapsed(&path));

        // Toggle collapse: should now be collapsed
        model.toggle_file_collapsed(&path);
        assert!(model.is_file_collapsed(&path));

        // Toggle again: should now be expanded
        model.toggle_file_collapsed(&path);
        assert!(!model.is_file_collapsed(&path));
    }

    #[test]
    fn test_card_header_sticky_positioning() {
        // card at y = 100, height = 300, viewport top = 50
        // Has not scrolled past top yet: stays at curr_y
        let y1 = CodeReviewLayout::card_header_y(100.0, 300.0, false, 50.0);
        assert_eq!(y1, 100.0);

        // Card scrolled above viewport top: curr_y = -50, card_h = 300, vy = 50
        // Should stick at vy (50.0)
        let y2 = CodeReviewLayout::card_header_y(-50.0, 300.0, false, 50.0);
        assert_eq!(y2, 50.0);

        // Card almost completely scrolled off: curr_y = -280, card_h = 300 (bottom is at +20)
        // max_y is curr_y + card_h - CARD_HEADER_HEIGHT = -280 + 300 - 36 = -16.0
        // Should be bounded by max_y, never escaping the card!
        let y3 = CodeReviewLayout::card_header_y(-280.0, 300.0, false, 50.0);
        assert_eq!(y3, -16.0);
    }

    #[test]
    fn test_dynamic_card_height_short_diff() {
        let file = FileDiff {
            path: PathBuf::from("test.rs"),
            old_path: None,
            status: FileGitStatus::Modified,
            hunks: vec![DiffHunk {
                old_start: 1,
                old_lines: 3,
                new_start: 1,
                new_lines: 3,
                header: "@@ -1,3 +1,3 @@".to_string(),
                lines: vec![
                    DiffLine {
                        line_type: DiffLineType::Context,
                        content: "line 1\n".to_string(),
                        old_line_num: Some(1),
                        new_line_num: Some(1),
                    },
                    DiffLine {
                        line_type: DiffLineType::Add,
                        content: "line 2\n".to_string(),
                        old_line_num: None,
                        new_line_num: Some(2),
                    },
                    DiffLine {
                        line_type: DiffLineType::Context,
                        content: "line 3\n".to_string(),
                        old_line_num: Some(2),
                        new_line_num: Some(3),
                    },
                ],
            }],
            additions: 1,
            deletions: 0,
            is_binary: false,
            line_count: 3,
        };

        // Collapsed height
        let h_collapsed =
            CodeReviewLayout::card_height_for_file_state_ex(&file, true, false, false, 800.0);
        assert_eq!(h_collapsed, CodeReviewLayout::CARD_HEADER_HEIGHT);

        // Expanded short diff height: CARD_HEADER_HEIGHT (36) + 3 * 20 (60) + 1 * 24 (hunk header) + 16 = 136.0
        let h_expanded =
            CodeReviewLayout::card_height_for_file_state_ex(&file, false, false, false, 800.0);
        assert_eq!(h_expanded, 36.0 + 60.0 + 24.0 + 16.0);
        // Short diff should NOT stretch to viewport height (800)
        assert!(h_expanded < 200.0);
    }
}
