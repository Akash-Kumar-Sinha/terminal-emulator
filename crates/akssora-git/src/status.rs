use std::path::{Path, PathBuf};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FileGitStatus {
    Modified,
    Added,
    Deleted,
    Untracked,
    Renamed,
    Conflict,
}

impl FileGitStatus {
    pub fn badge_label(&self) -> &'static str {
        match self {
            Self::Modified => "M",
            Self::Added => "A",
            Self::Deleted => "D",
            Self::Untracked => "U",
            Self::Renamed => "R",
            Self::Conflict => "!",
        }
    }

    pub fn status_icon(&self) -> &'static str {
        match self {
            Self::Modified => akssora_icons::GitStatusModifiedIcon::GLYPH,
            Self::Added => akssora_icons::GitStatusAddedIcon::GLYPH,
            Self::Deleted => akssora_icons::GitStatusDeletedIcon::GLYPH,
            Self::Untracked => akssora_icons::GitStatusUntrackedIcon::GLYPH,
            Self::Renamed => akssora_icons::GitStatusRenamedIcon::GLYPH,
            Self::Conflict => akssora_icons::GitStatusConflictIcon::GLYPH,
        }
    }

    pub fn status_description(&self) -> &'static str {
        match self {
            Self::Modified => "Modified",
            Self::Added => "Added",
            Self::Deleted => "Deleted",
            Self::Untracked => "Untracked",
            Self::Renamed => "Renamed",
            Self::Conflict => "Conflict",
        }
    }

    pub fn badge_color(&self) -> [f32; 4] {
        match self {
            Self::Modified => [0.88, 0.68, 0.28, 0.95],
            Self::Added | Self::Untracked => [0.45, 0.78, 0.45, 0.95],
            Self::Deleted | Self::Conflict => [0.85, 0.30, 0.30, 0.95],
            Self::Renamed => [0.45, 0.70, 0.95, 0.95],
        }
    }

    pub fn text_color(&self) -> [f32; 4] {
        match self {
            Self::Modified => [0.88, 0.72, 0.38, 0.95],
            Self::Added | Self::Untracked => [0.55, 0.82, 0.55, 0.95],
            Self::Deleted => [0.75, 0.40, 0.40, 0.90],
            Self::Renamed => [0.55, 0.75, 0.95, 0.95],
            Self::Conflict => [0.90, 0.35, 0.35, 0.95],
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct GitFileChange {
    pub path: PathBuf,
    pub status: FileGitStatus,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct GitRepoStatus {
    pub branch: String,
    pub ahead: usize,
    pub behind: usize,
    pub changes: Vec<GitFileChange>,
    pub clean: bool,
}

impl GitRepoStatus {
    pub fn file_status(&self, repo_root: &Path, file_path: &Path) -> Option<FileGitStatus> {
        let rel_path = file_path.strip_prefix(repo_root).unwrap_or(file_path);
        let mut dir_status = None;

        for change in &self.changes {
            let change_rel = change.path.strip_prefix(repo_root).unwrap_or(&change.path);
            if change_rel == rel_path || change.path == file_path {
                return Some(change.status);
            }

            // If file_path is a directory containing changes, bubble up status to directory
            if change_rel.starts_with(rel_path) || change.path.starts_with(file_path) {
                match change.status {
                    FileGitStatus::Conflict => return Some(FileGitStatus::Conflict),
                    FileGitStatus::Modified => dir_status = Some(FileGitStatus::Modified),
                    FileGitStatus::Added => {
                        if dir_status != Some(FileGitStatus::Modified) {
                            dir_status = Some(FileGitStatus::Added);
                        }
                    }
                    FileGitStatus::Deleted => {
                        if dir_status != Some(FileGitStatus::Modified) {
                            dir_status = Some(FileGitStatus::Deleted);
                        }
                    }
                    FileGitStatus::Renamed => {
                        if dir_status != Some(FileGitStatus::Modified) {
                            dir_status = Some(FileGitStatus::Renamed);
                        }
                    }
                    FileGitStatus::Untracked => {
                        if dir_status.is_none() {
                            dir_status = Some(FileGitStatus::Untracked);
                        }
                    }
                }
            }
        }
        dir_status
    }
}
