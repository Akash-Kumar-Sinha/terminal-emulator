use std::collections::HashMap;
use std::path::{Path, PathBuf};

use rayon::prelude::*;

use crate::repo_diff::{FileDiff, RepoDiff};

// INFO: Pre-indexed hunk with precalculated visual line offsets for O(log H) binary search lookup.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IndexedHunk {
    pub hunk_idx: usize,
    /// Visual line offset where this hunk begins in the file diff view (including the hunk header).
    pub start_visual_line: usize,
    /// Total visual lines occupied by this hunk (1 header line + lines.len()).
    pub line_count: usize,
}

// INFO: Pre-indexed file diff structure enabling O(1) layout metrics and O(log H) visible line window resolution.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IndexedFileDiff {
    pub file_idx: usize,
    pub path: PathBuf,
    pub hunks: Vec<IndexedHunk>,
    /// Total visual lines across all hunks in this file.
    pub total_visual_lines: usize,
}

impl IndexedFileDiff {
    // INFO: Creates an indexed representation of a single file's hunks.
    pub fn new(file_idx: usize, file: &FileDiff) -> Self {
        let mut hunks = Vec::with_capacity(file.hunks.len());
        let mut visual_line = 0;

        for (hunk_idx, hunk) in file.hunks.iter().enumerate() {
            let hunk_line_count = 1 + hunk.lines.len(); // 1 line for header + content lines
            hunks.push(IndexedHunk {
                hunk_idx,
                start_visual_line: visual_line,
                line_count: hunk_line_count,
            });
            visual_line += hunk_line_count;
        }

        Self {
            file_idx,
            path: file.path.clone(),
            hunks,
            total_visual_lines: visual_line,
        }
    }

    // INFO: Binary searches for the hunk index containing the given visual line offset in O(log H) time.
    pub fn find_hunk_for_visual_line(&self, visual_line: usize) -> Option<usize> {
        if self.hunks.is_empty() || visual_line >= self.total_visual_lines {
            return None;
        }
        let idx = self
            .hunks
            .partition_point(|h| h.start_visual_line + h.line_count <= visual_line);
        if idx < self.hunks.len() {
            Some(idx)
        } else {
            None
        }
    }

    // INFO: Resolves the slice of hunks overlapping the requested visual line span [start_line..=end_line].
    pub fn hunks_for_visual_range(
        &self,
        start_line: usize,
        end_line: usize,
    ) -> Option<(usize, usize)> {
        if self.hunks.is_empty() || start_line > end_line {
            return None;
        }
        let first_hunk = self.find_hunk_for_visual_line(start_line).unwrap_or(0);
        let last_hunk = self
            .find_hunk_for_visual_line(end_line)
            .unwrap_or_else(|| self.hunks.len().saturating_sub(1));
        Some((
            first_hunk,
            last_hunk.min(self.hunks.len().saturating_sub(1)),
        ))
    }
}

// INFO: Immutable, thread-safe indexed repository diff model constructed via Rayon parallel preprocessing.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct IndexedRepoDiff {
    pub repo_diff: RepoDiff,
    pub indexed_files: Vec<IndexedFileDiff>,
    pub files_by_path: HashMap<PathBuf, usize>,
}

impl IndexedRepoDiff {
    // INFO: Builds an immutable indexed model using Rayon parallel preprocessing across all files.
    pub fn from_repo_diff(repo_diff: RepoDiff) -> Self {
        let indexed_files: Vec<IndexedFileDiff> = repo_diff
            .files
            .par_iter()
            .enumerate()
            .map(|(idx, file)| IndexedFileDiff::new(idx, file))
            .collect();

        let mut files_by_path = HashMap::with_capacity(indexed_files.len());
        for (idx, file) in indexed_files.iter().enumerate() {
            files_by_path.insert(file.path.clone(), idx);
        }

        Self {
            repo_diff,
            indexed_files,
            files_by_path,
        }
    }

    // INFO: Looks up pre-indexed file diff by repository-relative path in O(1) time.
    pub fn file_by_path(&self, path: &Path) -> Option<(&FileDiff, &IndexedFileDiff)> {
        let idx = *self.files_by_path.get(path)?;
        let file = self.repo_diff.files.get(idx)?;
        let indexed = self.indexed_files.get(idx)?;
        Some((file, indexed))
    }

    // INFO: Looks up pre-indexed file diff by index in O(1) time.
    pub fn file_at(&self, idx: usize) -> Option<(&FileDiff, &IndexedFileDiff)> {
        let file = self.repo_diff.files.get(idx)?;
        let indexed = self.indexed_files.get(idx)?;
        Some((file, indexed))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repo_diff::{DiffHunk, DiffLine, DiffLineType};
    use crate::status::FileGitStatus;

    #[test]
    fn test_indexed_file_diff_binary_search_and_ranges() {
        let hunk0 = DiffHunk {
            old_start: 1,
            old_lines: 5,
            new_start: 1,
            new_lines: 5,
            header: "@@ -1,5 +1,5 @@".to_string(),
            lines: (0..10)
                .map(|i| DiffLine {
                    line_type: DiffLineType::Context,
                    content: format!("line {i}"),
                    old_line_num: Some(i + 1),
                    new_line_num: Some(i + 1),
                })
                .collect(),
        };

        let hunk1 = DiffHunk {
            old_start: 20,
            old_lines: 3,
            new_start: 20,
            new_lines: 4,
            header: "@@ -20,3 +20,4 @@".to_string(),
            lines: (0..5)
                .map(|i| DiffLine {
                    line_type: DiffLineType::Add,
                    content: format!("new line {i}"),
                    old_line_num: None,
                    new_line_num: Some(20 + i),
                })
                .collect(),
        };

        let file = FileDiff {
            path: PathBuf::from("src/test.rs"),
            old_path: None,
            status: FileGitStatus::Modified,
            hunks: vec![hunk0, hunk1],
            additions: 5,
            deletions: 0,
            is_binary: false,
            line_count: 15,
        };

        let indexed = IndexedFileDiff::new(0, &file);
        // Hunk 0: 1 (header) + 10 (lines) = 11 visual lines [0..11]
        // Hunk 1: 1 (header) + 5 (lines) = 6 visual lines [11..17]
        assert_eq!(indexed.total_visual_lines, 17);
        assert_eq!(indexed.hunks.len(), 2);
        assert_eq!(indexed.hunks[0].start_visual_line, 0);
        assert_eq!(indexed.hunks[0].line_count, 11);
        assert_eq!(indexed.hunks[1].start_visual_line, 11);
        assert_eq!(indexed.hunks[1].line_count, 6);

        // Binary search lookup
        assert_eq!(indexed.find_hunk_for_visual_line(0), Some(0));
        assert_eq!(indexed.find_hunk_for_visual_line(10), Some(0));
        assert_eq!(indexed.find_hunk_for_visual_line(11), Some(1));
        assert_eq!(indexed.find_hunk_for_visual_line(16), Some(1));
        assert_eq!(indexed.find_hunk_for_visual_line(17), None);

        // Range lookup
        assert_eq!(indexed.hunks_for_visual_range(0, 5), Some((0, 0)));
        assert_eq!(indexed.hunks_for_visual_range(5, 12), Some((0, 1)));
        assert_eq!(indexed.hunks_for_visual_range(11, 16), Some((1, 1)));
    }

    #[test]
    fn test_rayon_parallel_preprocessing_indexed_repo_diff() {
        let mut files = Vec::new();
        for f_idx in 0..50 {
            let hunk = DiffHunk {
                old_start: 1,
                old_lines: 10,
                new_start: 1,
                new_lines: 10,
                header: format!("@@ -1,10 +1,10 @@ file_{f_idx}"),
                lines: (0..20)
                    .map(|l| DiffLine {
                        line_type: DiffLineType::Context,
                        content: format!("file {f_idx} line {l}"),
                        old_line_num: Some(l + 1),
                        new_line_num: Some(l + 1),
                    })
                    .collect(),
            };
            files.push(FileDiff {
                path: PathBuf::from(format!("crates/module_{f_idx}.rs")),
                old_path: None,
                status: FileGitStatus::Modified,
                hunks: vec![hunk],
                additions: 10,
                deletions: 2,
                is_binary: false,
                line_count: 20,
            });
        }

        let repo_diff = RepoDiff {
            files,
            total_additions: 500,
            total_deletions: 100,
            files_changed: 50,
        };

        // Rayon parallel preprocessing
        let indexed_repo = IndexedRepoDiff::from_repo_diff(repo_diff);
        assert_eq!(indexed_repo.indexed_files.len(), 50);
        assert_eq!(indexed_repo.files_by_path.len(), 50);

        // Fast O(1) file lookup
        let (file, indexed_file) = indexed_repo
            .file_by_path(Path::new("crates/module_25.rs"))
            .expect("File 25 must exist");
        assert_eq!(file.path, PathBuf::from("crates/module_25.rs"));
        assert_eq!(indexed_file.total_visual_lines, 21); // 1 header + 20 lines
    }
}
