use std::path::PathBuf;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FileGitStatus {
    Modified,
    Added,
    Deleted,
    Untracked,
    Renamed,
    Conflict,
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
