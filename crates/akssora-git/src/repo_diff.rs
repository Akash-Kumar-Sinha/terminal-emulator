use std::path::{Path, PathBuf};
use std::process::Command;

use crate::status::FileGitStatus;

// INFO: Diff line classification for unified and split diff representation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DiffLineType {
    Context,
    Add,
    Delete,
}

// INFO: Individual diff line with content and optional original/modified line numbers.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DiffLine {
    pub line_type: DiffLineType,
    pub content: String,
    pub old_line_num: Option<usize>,
    pub new_line_num: Option<usize>,
}

// INFO: Hunk of changes within a file diff, including hunk range header and line records.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DiffHunk {
    pub old_start: usize,
    pub old_lines: usize,
    pub new_start: usize,
    pub new_lines: usize,
    pub header: String,
    pub lines: Vec<DiffLine>,
}

impl DiffHunk {
    /// Trims outer unmodified context lines so the hunk starts at the first changed line
    /// and ends at the last changed line. If the hunk contains no changes, lines are unchanged.
    pub fn trim_to_changed_range(&mut self) {
        let first_change = self
            .lines
            .iter()
            .position(|l| l.line_type != DiffLineType::Context);
        let last_change = self
            .lines
            .iter()
            .rposition(|l| l.line_type != DiffLineType::Context);
        if let (Some(first), Some(last)) = (first_change, last_change)
            && first <= last
        {
            self.lines = self.lines[first..=last].to_vec();
        }
    }

    /// Ensures that up to `max_context` lines of unmodified context exist before the first change
    /// and after the last change in the hunk, preserving any changes and already expanded lines.
    pub fn ensure_five_lines_context(&mut self) {
        self.ensure_context_lines(5);
    }

    pub fn ensure_context_lines(&mut self, max_context: usize) {
        let first_change = self
            .lines
            .iter()
            .position(|l| l.line_type != DiffLineType::Context);
        let last_change = self
            .lines
            .iter()
            .rposition(|l| l.line_type != DiffLineType::Context);
        if let (Some(first), Some(last)) = (first_change, last_change) {
            let start = first.saturating_sub(max_context);
            let end = (last + 1 + max_context).min(self.lines.len());
            self.lines = self.lines[start..end].to_vec();
        }
    }

    /// Line number of the first line in this hunk (new_line_num or old_line_num).
    pub fn first_line_num(&self) -> usize {
        self.lines
            .iter()
            .find_map(|l| l.new_line_num.or(l.old_line_num))
            .unwrap_or(self.new_start.max(1))
    }

    /// Line number of the last line in this hunk (new_line_num or old_line_num).
    pub fn last_line_num(&self) -> usize {
        self.lines
            .iter()
            .rev()
            .find_map(|l| l.new_line_num.or(l.old_line_num))
            .unwrap_or(self.new_start + self.new_lines)
    }

    /// Checks if additional unchanged lines exist before the currently displayed hunk range.
    pub fn can_expand_top(&self) -> bool {
        self.first_line_num() > 1
    }

    /// Checks if additional unchanged lines exist after the currently displayed hunk range.
    pub fn can_expand_bottom(&self, total_file_lines: usize) -> bool {
        if total_file_lines == 0 {
            return false;
        }
        self.last_line_num() < total_file_lines
    }
}

// INFO: File diff representing all hunks, additions, deletions, and Git status for a changed file.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FileDiff {
    pub path: PathBuf,
    pub old_path: Option<PathBuf>,
    pub status: FileGitStatus,
    pub hunks: Vec<DiffHunk>,
    pub additions: usize,
    pub deletions: usize,
    pub is_binary: bool,
    pub line_count: usize,
}

impl FileDiff {
    pub fn is_empty(&self) -> bool {
        self.additions == 0 && self.deletions == 0
    }

    // INFO: Reconstructs original text content from diff hunks for a deleted file.
    pub fn deleted_content(&self) -> Option<String> {
        if self.status == FileGitStatus::Deleted {
            if self.hunks.is_empty() && self.deletions == 0 {
                return Some(String::new());
            }
            let mut out = String::new();
            for hunk in &self.hunks {
                for line in &hunk.lines {
                    if line.line_type == DiffLineType::Delete
                        || line.line_type == DiffLineType::Context
                    {
                        let trimmed = line
                            .content
                            .strip_suffix("\r\n")
                            .or_else(|| line.content.strip_suffix('\n'))
                            .unwrap_or(&line.content);
                        out.push_str(trimmed);
                        out.push('\n');
                    }
                }
            }
            Some(out)
        } else {
            None
        }
    }
}

// INFO: Repository-wide diff snapshot aggregating all changed files and summary metrics.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RepoDiff {
    pub files: Vec<FileDiff>,
    pub total_additions: usize,
    pub total_deletions: usize,
    pub files_changed: usize,
}

impl RepoDiff {
    pub fn is_dirty(&self) -> bool {
        self.files_changed > 0
    }
}

// INFO: Parses standard unified diff text (as generated by `git diff`) into structured hunks and lines.
pub fn parse_unified_diff(diff_text: &str) -> Vec<FileDiff> {
    let mut files = Vec::new();
    let mut current_file: Option<FileDiff> = None;
    let mut current_hunk: Option<DiffHunk> = None;
    let mut old_line_cursor = 0;
    let mut new_line_cursor = 0;

    for line in diff_text.lines() {
        if line.starts_with("diff --git ") {
            if let Some(mut hunk) = current_hunk.take()
                && let Some(file) = current_file.as_mut()
            {
                file.additions += hunk
                    .lines
                    .iter()
                    .filter(|l| l.line_type == DiffLineType::Add)
                    .count();
                file.deletions += hunk
                    .lines
                    .iter()
                    .filter(|l| l.line_type == DiffLineType::Delete)
                    .count();
                hunk.ensure_five_lines_context();
                file.hunks.push(hunk);
            }
            if let Some(file) = current_file.take() {
                files.push(file);
            }

            // Extract file path from "diff --git a/path b/path"
            let parts: Vec<&str> = line.split_whitespace().collect();
            let file_path = if parts.len() >= 4 {
                let b_part = parts[3].strip_prefix("b/").unwrap_or(parts[3]);
                PathBuf::from(b_part)
            } else {
                PathBuf::from("unknown")
            };

            current_file = Some(FileDiff {
                path: file_path,
                old_path: None,
                status: FileGitStatus::Modified,
                hunks: Vec::new(),
                additions: 0,
                deletions: 0,
                is_binary: false,
                line_count: 0,
            });
            continue;
        }

        let Some(file) = current_file.as_mut() else {
            continue;
        };

        if line.starts_with("new file mode ") {
            file.status = FileGitStatus::Added;
        } else if line.starts_with("deleted file mode ") {
            file.status = FileGitStatus::Deleted;
        } else if line.starts_with("similarity index ") {
            file.status = FileGitStatus::Renamed;
        } else if line.starts_with("Binary files ") {
            file.is_binary = true;
        } else if line.starts_with("@@ ") {
            // New hunk header: @@ -old_start,old_lines +new_start,new_lines @@
            if let Some(mut hunk) = current_hunk.take() {
                file.additions += hunk
                    .lines
                    .iter()
                    .filter(|l| l.line_type == DiffLineType::Add)
                    .count();
                file.deletions += hunk
                    .lines
                    .iter()
                    .filter(|l| l.line_type == DiffLineType::Delete)
                    .count();
                hunk.ensure_five_lines_context();
                file.hunks.push(hunk);
            }

            let (old_s, old_c, new_s, new_c) = parse_hunk_header(line);
            old_line_cursor = old_s;
            new_line_cursor = new_s;

            current_hunk = Some(DiffHunk {
                old_start: old_s,
                old_lines: old_c,
                new_start: new_s,
                new_lines: new_c,
                header: line.to_string(),
                lines: Vec::new(),
            });
        } else if let Some(hunk) = current_hunk.as_mut() {
            if let Some(stripped) = line.strip_prefix('+') {
                hunk.lines.push(DiffLine {
                    line_type: DiffLineType::Add,
                    content: stripped.to_string(),
                    old_line_num: None,
                    new_line_num: Some(new_line_cursor),
                });
                new_line_cursor += 1;
            } else if let Some(stripped) = line.strip_prefix('-') {
                hunk.lines.push(DiffLine {
                    line_type: DiffLineType::Delete,
                    content: stripped.to_string(),
                    old_line_num: Some(old_line_cursor),
                    new_line_num: None,
                });
                old_line_cursor += 1;
            } else if let Some(stripped) = line.strip_prefix(' ') {
                hunk.lines.push(DiffLine {
                    line_type: DiffLineType::Context,
                    content: stripped.to_string(),
                    old_line_num: Some(old_line_cursor),
                    new_line_num: Some(new_line_cursor),
                });
                old_line_cursor += 1;
                new_line_cursor += 1;
            } else if line == "\\ No newline at end of file" {
                // Ignore newline warning marker
            } else if !line.starts_with("index ")
                && !line.starts_with("--- ")
                && !line.starts_with("+++ ")
            {
                // Fallback for lines without leading space
                hunk.lines.push(DiffLine {
                    line_type: DiffLineType::Context,
                    content: line.to_string(),
                    old_line_num: Some(old_line_cursor),
                    new_line_num: Some(new_line_cursor),
                });
                old_line_cursor += 1;
                new_line_cursor += 1;
            }
        }
    }

    if let Some(mut hunk) = current_hunk.take()
        && let Some(file) = current_file.as_mut()
    {
        file.additions += hunk
            .lines
            .iter()
            .filter(|l| l.line_type == DiffLineType::Add)
            .count();
        file.deletions += hunk
            .lines
            .iter()
            .filter(|l| l.line_type == DiffLineType::Delete)
            .count();
        hunk.ensure_five_lines_context();
        file.hunks.push(hunk);
    }
    if let Some(file) = current_file.take() {
        files.push(file);
    }

    files
}

fn parse_hunk_header(header: &str) -> (usize, usize, usize, usize) {
    let mut old_start = 1;
    let mut old_count = 1;
    let mut new_start = 1;
    let mut new_count = 1;

    let parts: Vec<&str> = header.split("@@").collect();
    if parts.len() >= 2 {
        let coords = parts[1].trim();
        for item in coords.split_whitespace() {
            if let Some(stripped) = item.strip_prefix('-') {
                if let Some((s, c)) = stripped.split_once(',') {
                    old_start = s.parse().unwrap_or(1);
                    old_count = c.parse().unwrap_or(1);
                } else {
                    old_start = stripped.parse().unwrap_or(1);
                    old_count = 1;
                }
            } else if let Some(stripped) = item.strip_prefix('+') {
                if let Some((s, c)) = stripped.split_once(',') {
                    new_start = s.parse().unwrap_or(1);
                    new_count = c.parse().unwrap_or(1);
                } else {
                    new_start = stripped.parse().unwrap_or(1);
                    new_count = 1;
                }
            }
        }
    }

    (old_start, old_count, new_start, new_count)
}

// INFO: Queries git diff commands directly against working tree or staging index.
pub fn load_repo_diff(repo_root: impl AsRef<Path>, staged: bool) -> RepoDiff {
    let repo_dir = repo_root.as_ref();
    let mut cmd = Command::new("git");
    cmd.arg("diff").arg("-U5").arg("--no-color");
    if staged {
        cmd.arg("--cached");
    }
    cmd.current_dir(repo_dir);

    let output = cmd.output().ok();
    let diff_text = output
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).to_string())
        .unwrap_or_default();

    let mut files = parse_unified_diff(&diff_text);

    // If unstaged, also check for untracked files
    if !staged {
        let untracked_cmd = Command::new("git")
            .arg("ls-files")
            .arg("--others")
            .arg("--exclude-standard")
            .current_dir(repo_dir)
            .output()
            .ok();

        if let Some(out) = untracked_cmd
            && out.status.success()
        {
            let stdout = String::from_utf8_lossy(&out.stdout);
            for file_line in stdout.lines() {
                let trimmed = file_line.trim();
                if trimmed.is_empty() {
                    continue;
                }
                let path = PathBuf::from(trimmed);
                if !files.iter().any(|f| f.path == path) {
                    let full_path = repo_dir.join(&path);
                    let content = std::fs::read_to_string(&full_path).unwrap_or_default();
                    let lines: Vec<DiffLine> = content
                        .lines()
                        .enumerate()
                        .map(|(idx, l)| DiffLine {
                            line_type: DiffLineType::Add,
                            content: l.to_string(),
                            old_line_num: None,
                            new_line_num: Some(idx + 1),
                        })
                        .collect();
                    let additions = lines.len();
                    files.push(FileDiff {
                        path,
                        old_path: None,
                        status: FileGitStatus::Untracked,
                        hunks: vec![DiffHunk {
                            old_start: 0,
                            old_lines: 0,
                            new_start: 1,
                            new_lines: additions,
                            header: format!("@@ -0,0 +1,{additions} @@"),
                            lines,
                        }],
                        additions,
                        deletions: 0,
                        is_binary: false,
                        line_count: additions,
                    });
                }
            }
        }
    }

    use rayon::prelude::*;

    files.par_iter_mut().for_each(|file| {
        let full_path = repo_dir.join(&file.path);
        if full_path.exists()
            && let Ok(meta) = full_path.metadata()
            && meta.len() < 5_000_000
            && let Ok(content) = std::fs::read_to_string(&full_path)
        {
            file.line_count = content.lines().count();
        }
        if file.line_count == 0 {
            let hunk_lines: usize = file.hunks.iter().map(|h| h.lines.len() + 1).sum();
            file.line_count = hunk_lines.max(file.additions + file.deletions).max(1);
        }
    });

    let total_additions = files.iter().map(|f| f.additions).sum();
    let total_deletions = files.iter().map(|f| f.deletions).sum();
    let files_changed = files.len();

    RepoDiff {
        files,
        total_additions,
        total_deletions,
        files_changed,
    }
}

// INFO: Loads repository diff and constructs an immutable indexed diff model using Rayon parallel preprocessing.
pub fn load_repo_diff_indexed(
    repo_dir: &Path,
    staged: bool,
) -> crate::indexed_diff::IndexedRepoDiff {
    let repo_diff = load_repo_diff(repo_dir, staged);
    crate::indexed_diff::IndexedRepoDiff::from_repo_diff(repo_diff)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diff_hunk_five_lines_context_trimming() {
        // 10 context lines before, 1 add, 10 context lines after
        let mut lines = Vec::new();
        for i in 1..=10 {
            lines.push(DiffLine {
                line_type: DiffLineType::Context,
                content: format!("context before {i}\n"),
                old_line_num: Some(i),
                new_line_num: Some(i),
            });
        }
        lines.push(DiffLine {
            line_type: DiffLineType::Add,
            content: "added line\n".to_string(),
            old_line_num: None,
            new_line_num: Some(11),
        });
        for i in 11..=20 {
            lines.push(DiffLine {
                line_type: DiffLineType::Context,
                content: format!("context after {i}\n"),
                old_line_num: Some(i),
                new_line_num: Some(i + 1),
            });
        }

        let mut hunk = DiffHunk {
            old_start: 1,
            old_lines: 20,
            new_start: 1,
            new_lines: 21,
            header: "@@ -1,20 +1,21 @@".to_string(),
            lines,
        };

        hunk.ensure_five_lines_context();

        // Should have 5 before + 1 added + 5 after = 11 lines
        assert_eq!(hunk.lines.len(), 11);
        assert_eq!(hunk.lines[0].content, "context before 6\n");
        assert_eq!(hunk.lines[5].line_type, DiffLineType::Add);
        assert_eq!(hunk.lines[10].content, "context after 15\n");
    }

    #[test]
    fn test_diff_hunk_fewer_than_five_lines_context() {
        let mut lines = Vec::new();
        for i in 1..=2 {
            lines.push(DiffLine {
                line_type: DiffLineType::Context,
                content: format!("ctx {i}\n"),
                old_line_num: Some(i),
                new_line_num: Some(i),
            });
        }
        lines.push(DiffLine {
            line_type: DiffLineType::Add,
            content: "new\n".to_string(),
            old_line_num: None,
            new_line_num: Some(3),
        });
        lines.push(DiffLine {
            line_type: DiffLineType::Context,
            content: "after\n".to_string(),
            old_line_num: Some(3),
            new_line_num: Some(4),
        });

        let mut hunk = DiffHunk {
            old_start: 1,
            old_lines: 3,
            new_start: 1,
            new_lines: 4,
            header: "@@ -1,3 +1,4 @@".to_string(),
            lines,
        };

        hunk.ensure_five_lines_context();
        // Should keep all 4 lines
        assert_eq!(hunk.lines.len(), 4);
    }

    #[test]
    fn test_diff_hunk_can_expand_top_and_bottom() {
        let hunk = DiffHunk {
            old_start: 10,
            old_lines: 5,
            new_start: 10,
            new_lines: 6,
            header: "@@ -10,5 +10,6 @@".to_string(),
            lines: vec![
                DiffLine {
                    line_type: DiffLineType::Context,
                    content: "line 10\n".to_string(),
                    old_line_num: Some(10),
                    new_line_num: Some(10),
                },
                DiffLine {
                    line_type: DiffLineType::Add,
                    content: "line 11 added\n".to_string(),
                    old_line_num: None,
                    new_line_num: Some(11),
                },
                DiffLine {
                    line_type: DiffLineType::Context,
                    content: "line 12\n".to_string(),
                    old_line_num: Some(11),
                    new_line_num: Some(12),
                },
            ],
        };

        assert!(hunk.can_expand_top()); // starts at line 10 > 1
        assert!(hunk.can_expand_bottom(20)); // ends at 12 < 20
        assert!(!hunk.can_expand_bottom(12)); // ends at 12 == 12
    }
}
