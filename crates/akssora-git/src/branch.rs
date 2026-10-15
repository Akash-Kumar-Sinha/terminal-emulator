use std::path::Path;
use std::process::Command;

use crate::status::GitRepoStatus;

// INFO: Queries git repository status, branch information, and worktree modifications.
pub struct GitInspector;

impl GitInspector {
    pub fn repo_root(path: impl AsRef<Path>) -> Option<std::path::PathBuf> {
        let output = Command::new("git")
            .arg("rev-parse")
            .arg("--show-toplevel")
            .current_dir(&path)
            .output()
            .ok()?;

        if output.status.success() {
            let root_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
            Some(std::path::PathBuf::from(root_str))
        } else {
            None
        }
    }

    pub fn current_branch(repo_path: impl AsRef<Path>) -> Option<String> {
        let output = Command::new("git")
            .arg("rev-parse")
            .arg("--abbrev-ref")
            .arg("HEAD")
            .current_dir(repo_path)
            .output()
            .ok()?;

        if output.status.success() {
            let branch = String::from_utf8_lossy(&output.stdout).trim().to_string();
            Some(branch)
        } else {
            None
        }
    }

    pub fn status(repo_path: impl AsRef<Path>) -> Option<GitRepoStatus> {
        let target_path = repo_path.as_ref();
        let repo_dir = Self::repo_root(target_path).unwrap_or_else(|| target_path.to_path_buf());
        let branch = Self::current_branch(&repo_dir).unwrap_or_else(|| "detached".to_string());
        let output = Command::new("git")
            .arg("status")
            .arg("--porcelain=v1")
            .current_dir(&repo_dir)
            .output()
            .ok()?;

        let mut status = GitRepoStatus {
            branch,
            clean: true,
            ..Default::default()
        };

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            status.clean = stdout.trim().is_empty();
            for line in stdout.lines() {
                if line.len() < 3 {
                    continue;
                }
                let code = &line[0..2];
                let file_str = line[3..].trim();
                let file_str = if let Some((_, new_p)) = file_str.split_once(" -> ") {
                    new_p.trim()
                } else {
                    file_str
                };
                let file_path = repo_dir.join(file_str);
                let st = if code.contains('?') {
                    crate::status::FileGitStatus::Untracked
                } else if code.contains('U') {
                    crate::status::FileGitStatus::Conflict
                } else if code.contains('A') {
                    crate::status::FileGitStatus::Added
                } else if code.contains('D') {
                    crate::status::FileGitStatus::Deleted
                } else if code.contains('R') {
                    crate::status::FileGitStatus::Renamed
                } else {
                    crate::status::FileGitStatus::Modified
                };
                status.changes.push(crate::status::GitFileChange {
                    path: file_path,
                    status: st,
                });
            }
        }

        Some(status)
    }

    pub fn file_original_content(
        repo_root: impl AsRef<Path>,
        file_path: impl AsRef<Path>,
    ) -> Option<String> {
        let repo_root = repo_root.as_ref();
        let file_path = file_path.as_ref();
        let rel_path = if file_path.is_relative() {
            file_path
        } else {
            file_path.strip_prefix(repo_root).unwrap_or(file_path)
        };
        let rel_str = rel_path.to_str()?;

        // 1. Try Git show from index (:rel_str) for unstaged changes
        if let Ok(output) = Command::new("git")
            .arg("show")
            .arg(format!(":{}", rel_str))
            .current_dir(repo_root)
            .output()
            && output.status.success()
        {
            return Some(String::from_utf8_lossy(&output.stdout).to_string());
        }

        // 2. Try Git show from HEAD (HEAD:rel_str) for staged deletions / base commit
        if let Ok(output) = Command::new("git")
            .arg("show")
            .arg(format!("HEAD:{}", rel_str))
            .current_dir(repo_root)
            .output()
            && output.status.success()
        {
            return Some(String::from_utf8_lossy(&output.stdout).to_string());
        }

        // 3. Fallback: Check diff hunks for deleted file content
        let repo_diff = crate::repo_diff::load_repo_diff(repo_root, false);
        if let Some(file_diff) = repo_diff.files.iter().find(|f| f.path == rel_path)
            && let Some(content) = file_diff.deleted_content()
        {
            return Some(content);
        }
        let staged_diff = crate::repo_diff::load_repo_diff(repo_root, true);
        if let Some(file_diff) = staged_diff.files.iter().find(|f| f.path == rel_path)
            && let Some(content) = file_diff.deleted_content()
        {
            return Some(content);
        }

        if Self::repo_root(repo_root).is_some() {
            Some(String::new())
        } else {
            None
        }
    }

    // INFO: Retrieves original source content for a deleted file from Git base commit, index, or diff hunks.
    pub fn file_deleted_content(
        repo_root: impl AsRef<Path>,
        file_path: impl AsRef<Path>,
        file_diff: Option<&crate::repo_diff::FileDiff>,
    ) -> Result<String, String> {
        let repo_root = repo_root.as_ref();
        let file_path = file_path.as_ref();
        let rel_path = if file_path.is_relative() {
            file_path
        } else {
            file_path.strip_prefix(repo_root).unwrap_or(file_path)
        };

        // 1. Try explicit diff hunks from provided FileDiff
        if let Some(fd) = file_diff
            && let Some(content) = fd.deleted_content()
        {
            return Ok(content);
        }

        // 2. Try index or HEAD via git show
        let rel_str = rel_path.to_str().unwrap_or_default();
        if !rel_str.is_empty() {
            if let Ok(output) = Command::new("git")
                .arg("show")
                .arg(format!(":{}", rel_str))
                .current_dir(repo_root)
                .output()
                && output.status.success()
            {
                return Ok(String::from_utf8_lossy(&output.stdout).to_string());
            }

            if let Ok(output) = Command::new("git")
                .arg("show")
                .arg(format!("HEAD:{}", rel_str))
                .current_dir(repo_root)
                .output()
                && output.status.success()
            {
                return Ok(String::from_utf8_lossy(&output.stdout).to_string());
            }
        }

        // 3. Fallback: load unstaged or staged repo diff
        let unstaged = crate::repo_diff::load_repo_diff(repo_root, false);
        if let Some(fd) = unstaged.files.iter().find(|f| f.path == rel_path)
            && let Some(content) = fd.deleted_content()
        {
            return Ok(content);
        }
        let staged = crate::repo_diff::load_repo_diff(repo_root, true);
        if let Some(fd) = staged.files.iter().find(|f| f.path == rel_path)
            && let Some(content) = fd.deleted_content()
        {
            return Ok(content);
        }

        Err(format!(
            "Error: Original content cannot be retrieved for deleted file '{}'",
            file_path.display()
        ))
    }

    pub fn stage_file(
        repo_root: impl AsRef<Path>,
        file_path: impl AsRef<Path>,
    ) -> Result<(), String> {
        let output = Command::new("git")
            .arg("add")
            .arg(file_path.as_ref())
            .current_dir(repo_root)
            .output()
            .map_err(|e| e.to_string())?;
        if output.status.success() {
            Ok(())
        } else {
            Err(String::from_utf8_lossy(&output.stderr).to_string())
        }
    }

    pub fn unstage_file(
        repo_root: impl AsRef<Path>,
        file_path: impl AsRef<Path>,
    ) -> Result<(), String> {
        let output = Command::new("git")
            .arg("restore")
            .arg("--staged")
            .arg(file_path.as_ref())
            .current_dir(repo_root)
            .output()
            .map_err(|e| e.to_string())?;
        if output.status.success() {
            Ok(())
        } else {
            Err(String::from_utf8_lossy(&output.stderr).to_string())
        }
    }

    pub fn discard_file_changes(
        repo_root: impl AsRef<Path>,
        file_path: impl AsRef<Path>,
    ) -> Result<(), String> {
        let root = repo_root.as_ref();
        let path = file_path.as_ref();
        let output = Command::new("git")
            .arg("checkout")
            .arg("HEAD")
            .arg("--")
            .arg(path)
            .current_dir(root)
            .output()
            .map_err(|e| e.to_string())?;
        if output.status.success() {
            Ok(())
        } else {
            // If checkout failed, check if file is untracked and remove it
            let full_path = root.join(path);
            if full_path.exists() {
                if full_path.is_dir() {
                    let _ = std::fs::remove_dir_all(&full_path);
                } else {
                    let _ = std::fs::remove_file(&full_path);
                }
                Ok(())
            } else {
                Err(String::from_utf8_lossy(&output.stderr).to_string())
            }
        }
    }

    pub fn stage_all(repo_root: impl AsRef<Path>) -> Result<(), String> {
        let output = Command::new("git")
            .arg("add")
            .arg("-A")
            .current_dir(repo_root)
            .output()
            .map_err(|e| e.to_string())?;
        if output.status.success() {
            Ok(())
        } else {
            Err(String::from_utf8_lossy(&output.stderr).to_string())
        }
    }

    pub fn unstage_all(repo_root: impl AsRef<Path>) -> Result<(), String> {
        let output = Command::new("git")
            .arg("restore")
            .arg("--staged")
            .arg(".")
            .current_dir(repo_root)
            .output()
            .map_err(|e| e.to_string())?;
        if output.status.success() {
            Ok(())
        } else {
            Err(String::from_utf8_lossy(&output.stderr).to_string())
        }
    }

    pub fn discard_all_changes(repo_root: impl AsRef<Path>) -> Result<(), String> {
        let root = repo_root.as_ref();
        let _ = Command::new("git")
            .arg("checkout")
            .arg("HEAD")
            .arg("--")
            .arg(".")
            .current_dir(root)
            .output();
        let _ = Command::new("git")
            .arg("clean")
            .arg("-fd")
            .current_dir(root)
            .output();
        Ok(())
    }

    pub fn stage_directory(
        repo_root: impl AsRef<Path>,
        dir_path: impl AsRef<Path>,
    ) -> Result<(), String> {
        Self::stage_file(repo_root, dir_path)
    }

    pub fn unstage_directory(
        repo_root: impl AsRef<Path>,
        dir_path: impl AsRef<Path>,
    ) -> Result<(), String> {
        Self::unstage_file(repo_root, dir_path)
    }

    pub fn discard_directory_changes(
        repo_root: impl AsRef<Path>,
        dir_path: impl AsRef<Path>,
    ) -> Result<(), String> {
        let root = repo_root.as_ref();
        let path = dir_path.as_ref();
        let _ = Command::new("git")
            .arg("checkout")
            .arg("HEAD")
            .arg("--")
            .arg(path)
            .current_dir(root)
            .output();
        let _ = Command::new("git")
            .arg("clean")
            .arg("-fd")
            .arg("--")
            .arg(path)
            .current_dir(root)
            .output();
        Ok(())
    }

    pub fn commit(repo_root: impl AsRef<Path>, message: &str) -> Result<String, String> {
        let output = Command::new("git")
            .arg("commit")
            .arg("-m")
            .arg(message)
            .current_dir(repo_root)
            .output()
            .map_err(|e| e.to_string())?;
        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            Err(String::from_utf8_lossy(&output.stderr).to_string())
        }
    }

    pub fn is_git_repo(path: impl AsRef<Path>) -> bool {
        Self::repo_root(path).is_some()
    }

    pub fn list_branches(repo_path: impl AsRef<Path>) -> Vec<String> {
        let output = Command::new("git")
            .arg("branch")
            .arg("--format=%(refname:short)")
            .current_dir(repo_path)
            .output()
            .ok();

        if let Some(out) = output
            && out.status.success()
        {
            let stdout = String::from_utf8_lossy(&out.stdout);
            let branches: Vec<String> = stdout
                .lines()
                .map(|l| l.trim().to_string())
                .filter(|l| !l.is_empty())
                .collect();
            if !branches.is_empty() {
                return branches;
            }
        }
        Vec::new()
    }

    pub fn switch_branch(repo_path: impl AsRef<Path>, branch_name: &str) -> Result<String, String> {
        let output = Command::new("git")
            .arg("checkout")
            .arg(branch_name)
            .current_dir(repo_path)
            .output()
            .map_err(|e| e.to_string())?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            Err(String::from_utf8_lossy(&output.stderr).to_string())
        }
    }

    pub fn push(repo_root: impl AsRef<Path>) -> Result<String, String> {
        let output = Command::new("git")
            .arg("push")
            .current_dir(repo_root)
            .output()
            .map_err(|e| e.to_string())?;
        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            Err(String::from_utf8_lossy(&output.stderr).to_string())
        }
    }

    // INFO: Stages or unstages an individual line or smallest valid change hunk using Git diff patch application.
    // INFO: Stages or unstages strictly a single logical line in Git without affecting neighboring lines or whole hunks.
    pub fn stage_or_unstage_line_at(
        repo_root: impl AsRef<Path>,
        file_path: impl AsRef<Path>,
        target_line: usize,
        is_staged: bool,
    ) -> Result<(), String> {
        let repo_dir = repo_root.as_ref();
        let rel_path = file_path
            .as_ref()
            .strip_prefix(repo_dir)
            .unwrap_or(file_path.as_ref());
        let rel_str = rel_path.to_string_lossy();

        let repo_diff = crate::repo_diff::load_repo_diff(repo_dir, is_staged);
        let file_diff_opt = repo_diff.files.iter().find(|f| f.path == rel_path);

        // 1. Read base index content (or HEAD content)
        let base_content = if !is_staged {
            Self::file_original_content(repo_dir, rel_path)
                .or_else(|| file_diff_opt.and_then(|f| f.deleted_content()))
                .unwrap_or_default()
        } else {
            let full_disk_path = repo_dir.join(rel_path);
            if !full_disk_path.exists() {
                String::new()
            } else {
                std::fs::read_to_string(&full_disk_path).unwrap_or_default()
            }
        };

        // 2. Read working tree content (or index content if unstaging)
        let curr_content = if !is_staged {
            let full_disk_path = repo_dir.join(rel_path);
            if !full_disk_path.exists() {
                String::new()
            } else {
                std::fs::read_to_string(&full_disk_path)
                    .map_err(|e| format!("Failed to read working tree file: {e}"))?
            }
        } else {
            Self::file_original_content(repo_dir, rel_path).unwrap_or_default()
        };

        let orig_lines: Vec<&str> = if base_content.is_empty() {
            Vec::new()
        } else {
            base_content.lines().collect()
        };
        let curr_lines: Vec<&str> = if curr_content.is_empty() {
            Vec::new()
        } else {
            curr_content.lines().collect()
        };

        let statuses = crate::diff::compute_line_diff(&orig_lines, &curr_lines);
        if target_line >= statuses.len() && !curr_lines.is_empty() {
            return Err("Target line is out of range".to_string());
        }

        let target_st = statuses.get(target_line).cloned().unwrap_or_default();
        if target_st.change.is_none() && !target_st.deleted_above && !target_st.deleted_below {
            return Err("No Git change detected on target line".to_string());
        }

        // 3. Construct new index content by applying ONLY target line
        let ops = crate::diff::myers_diff(&orig_lines, &curr_lines);
        let mut new_lines = Vec::new();

        for op in ops {
            match op {
                crate::diff::EditOp::Keep(orig_idx, _) => {
                    if orig_idx < orig_lines.len() {
                        new_lines.push(orig_lines[orig_idx]);
                    }
                }
                crate::diff::EditOp::Insert(curr_idx) => {
                    if curr_idx == target_line && curr_idx < curr_lines.len() {
                        new_lines.push(curr_lines[curr_idx]);
                    }
                }
                crate::diff::EditOp::Delete(orig_idx) => {
                    let is_target_delete =
                        if target_st.change == Some(crate::diff::LineGitChange::Modified) {
                            target_st.original_line.as_deref() == orig_lines.get(orig_idx).copied()
                        } else if target_st.deleted_above || target_st.deleted_below {
                            target_st
                                .deleted_lines
                                .iter()
                                .any(|d| orig_lines.get(orig_idx).map(|s| *s == d).unwrap_or(false))
                        } else {
                            false
                        };

                    if is_target_delete {
                        // If it's a modification, we replace with target line
                        if target_st.change == Some(crate::diff::LineGitChange::Modified)
                            && target_line < curr_lines.len()
                        {
                            new_lines.push(curr_lines[target_line]);
                        }
                        // If pure deletion, omit from new_lines
                    } else if orig_idx < orig_lines.len() {
                        // Keep non-target deletion in the base index!
                        new_lines.push(orig_lines[orig_idx]);
                    }
                }
            }
        }

        // 4. Write new blob using `git hash-object -w --stdin`
        let mut new_text = new_lines.join("\n");
        if (curr_content.ends_with('\n') || base_content.ends_with('\n')) && !new_text.is_empty() {
            new_text.push('\n');
        }

        let mut hash_cmd = Command::new("git");
        hash_cmd
            .arg("hash-object")
            .arg("-w")
            .arg("--stdin")
            .current_dir(repo_dir);
        hash_cmd
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped());

        let mut child = hash_cmd
            .spawn()
            .map_err(|e| format!("Failed to spawn git hash-object: {e}"))?;
        if let Some(mut stdin) = child.stdin.take() {
            use std::io::Write;
            stdin
                .write_all(new_text.as_bytes())
                .map_err(|e| format!("Failed to write to git hash-object: {e}"))?;
        }
        let output = child
            .wait_with_output()
            .map_err(|e| format!("git hash-object wait failed: {e}"))?;
        if !output.status.success() {
            return Err("git hash-object failed".to_string());
        }
        let sha = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if sha.len() != 40 {
            return Err(format!("Invalid object sha: {sha}"));
        }

        // 5. Get current file mode
        let mut ls_cmd = Command::new("git");
        ls_cmd
            .arg("ls-files")
            .arg("--stage")
            .arg("--")
            .arg(&*rel_str)
            .current_dir(repo_dir);
        let mode = if let Ok(ls_out) = ls_cmd.output()
            && ls_out.status.success()
        {
            let s = String::from_utf8_lossy(&ls_out.stdout);
            s.split_whitespace().next().unwrap_or("100644").to_string()
        } else {
            "100644".to_string()
        };

        // 6. Update git index cacheinfo
        let mut update_cmd = Command::new("git");
        update_cmd
            .arg("update-index")
            .arg("--add")
            .arg("--cacheinfo")
            .arg(format!("{mode},{sha},{rel_str}"))
            .current_dir(repo_dir);
        let update_out = update_cmd
            .output()
            .map_err(|e| format!("Failed to run git update-index: {e}"))?;
        if !update_out.status.success() {
            return Err(format!(
                "git update-index failed: {}",
                String::from_utf8_lossy(&update_out.stderr)
            ));
        }

        Ok(())
    }
}
