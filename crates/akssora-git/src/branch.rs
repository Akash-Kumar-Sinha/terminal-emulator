use std::path::Path;
use std::process::Command;

use crate::status::GitRepoStatus;

// INFO: Queries git repository status, branch information, and worktree modifications.
pub struct GitInspector;

impl GitInspector {
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
        let branch = Self::current_branch(&repo_path).unwrap_or_else(|| "detached".to_string());
        let output = Command::new("git")
            .arg("status")
            .arg("--porcelain")
            .current_dir(repo_path)
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
        }

        Some(status)
    }
}
