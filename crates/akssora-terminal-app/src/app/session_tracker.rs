use std::path::{Path, PathBuf};

// INFO: Replaces home directory prefix with ~ and truncates deep path components if necessary.
pub fn shorten_path(path: &Path) -> String {
    let home = std::env::var("HOME").ok().map(PathBuf::from);
    let mut path_str = if let Some(home_dir) = home {
        if let Ok(rel) = path.strip_prefix(&home_dir) {
            if rel.as_os_str().is_empty() {
                "~".to_string()
            } else {
                format!("~/{}", rel.display())
            }
        } else {
            path.display().to_string()
        }
    } else {
        path.display().to_string()
    };

    // If string is longer than 32 chars, keep ~/.../last_dir
    if path_str.len() > 32
        && let Some(name) = path.file_name().and_then(|n| n.to_str())
    {
        if let Some(parent) = path
            .parent()
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str())
        {
            path_str = format!("~/.../{parent}/{name}");
        } else {
            path_str = format!("~/.../{name}");
        }
    }
    path_str
}

// INFO: Finds git repository root and current branch without executing git commands.
pub fn detect_git_info(cwd: &Path) -> (String, Option<String>) {
    let mut curr = Some(cwd);
    let mut git_root = None;

    while let Some(dir) = curr {
        let git_dir = dir.join(".git");
        if git_dir.exists() {
            git_root = Some((dir.to_path_buf(), git_dir));
            break;
        }
        curr = dir.parent();
    }

    if let Some((root_dir, git_path)) = git_root {
        let project_name = root_dir
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("project")
            .to_string();

        let branch = read_git_branch(&git_path);
        (project_name, branch)
    } else {
        let fallback_name = cwd
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("terminal")
            .to_string();
        (fallback_name, None)
    }
}

fn read_git_branch(git_path: &Path) -> Option<String> {
    let head_file = if git_path.is_dir() {
        git_path.join("HEAD")
    } else {
        // .git is a file (e.g. worktree / submodule), format: "gitdir: <path>"
        let content = std::fs::read_to_string(git_path).ok()?;
        let gitdir_line = content.lines().find(|l| l.starts_with("gitdir:"))?;
        let rel_path = gitdir_line.trim_start_matches("gitdir:").trim();
        let parent = git_path.parent()?;
        parent.join(rel_path).join("HEAD")
    };

    let content = std::fs::read_to_string(head_file).ok()?;
    let trimmed = content.trim();
    if let Some(branch_ref) = trimmed.strip_prefix("ref: refs/heads/") {
        Some(branch_ref.to_string())
    } else if let Some(branch_ref) = trimmed.strip_prefix("ref: ") {
        Some(branch_ref.to_string())
    } else if trimmed.len() >= 7 {
        // Detached HEAD: display 7-character commit hash
        Some(trimmed[..7].to_string())
    } else {
        None
    }
}

// INFO: Inspects Linux /proc filesystem to locate foreground process command and current working directory.
pub fn inspect_process_state(shell_pid: Option<u32>) -> (Option<PathBuf>, Option<String>) {
    let Some(pid) = shell_pid else {
        return (None, None);
    };

    #[cfg(target_os = "linux")]
    {
        let shell_cwd = std::fs::read_link(format!("/proc/{pid}/cwd")).ok();

        // Check foreground child processes
        if let Ok(children_content) =
            std::fs::read_to_string(format!("/proc/{pid}/task/{pid}/children"))
        {
            let child_pids: Vec<&str> = children_content.split_whitespace().collect();
            if let Some(&last_child) = child_pids.last() {
                let child_cwd = std::fs::read_link(format!("/proc/{last_child}/cwd")).ok();
                let effective_cwd = child_cwd.or(shell_cwd);

                if let Ok(cmdline_bytes) = std::fs::read(format!("/proc/{last_child}/cmdline")) {
                    let parts: Vec<&str> = cmdline_bytes
                        .split(|&b| b == 0)
                        .filter_map(|s| std::str::from_utf8(s).ok())
                        .filter(|s| !s.is_empty())
                        .collect();

                    if !parts.is_empty() {
                        let cmd_str = clean_command_name(&parts);
                        return (effective_cwd, Some(cmd_str));
                    }
                }

                if let Ok(comm) = std::fs::read_to_string(format!("/proc/{last_child}/comm")) {
                    let trimmed = comm.trim();
                    if !trimmed.is_empty() {
                        return (effective_cwd, Some(trimmed.to_string()));
                    }
                }

                return (effective_cwd, None);
            }
        }

        (shell_cwd, None)
    }

    #[cfg(not(target_os = "linux"))]
    {
        let _ = pid;
        (std::env::current_dir().ok(), None)
    }
}

#[cfg(target_os = "linux")]
fn clean_command_name(parts: &[&str]) -> String {
    let first = parts[0];
    let base = Path::new(first)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(first);

    if parts.len() > 1 && !parts[1].starts_with('-') {
        let second = Path::new(parts[1])
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(parts[1]);
        format!("{base} {second}")
    } else {
        base.to_string()
    }
}

// INFO: Determines meaningful terminal session title from custom setting, command, OSC title, or project fallback.
pub fn resolve_session_name(
    custom_name: Option<&str>,
    active_cmd: Option<&str>,
    osc_title: Option<&str>,
    last_executed_cmd: Option<&str>,
    project_name: &str,
    fallback_index: usize,
) -> String {
    if let Some(custom) = custom_name {
        let trimmed = custom.trim();
        if !trimmed.is_empty() {
            return trimmed.to_string();
        }
    }

    if let Some(cmd) = active_cmd {
        let trimmed = cmd.trim();
        if !trimmed.is_empty() && trimmed != "bash" && trimmed != "sh" && trimmed != "zsh" {
            return trimmed.to_string();
        }
    }

    if let Some(last_cmd) = last_executed_cmd {
        let trimmed = last_cmd.trim();
        if !trimmed.is_empty() {
            return trimmed.to_string();
        }
    }

    if let Some(title) = osc_title {
        let trimmed = title.trim();
        if !trimmed.is_empty() && !trimmed.contains("Terminal") {
            return trimmed.to_string();
        }
    }

    if !project_name.is_empty() && project_name != "terminal" && project_name != "project" {
        project_name.to_string()
    } else {
        format!("Terminal {fallback_index}")
    }
}
