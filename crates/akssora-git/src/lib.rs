pub mod branch;
pub mod diff;
pub mod indexed_diff;
pub mod repo_diff;
pub mod status;

pub use branch::GitInspector;
pub use diff::{
    EditOp, GIT_DELETED_COLOR, LineGitChange, LineGitStatus, compute_line_diff, myers_diff,
};
pub use indexed_diff::{IndexedFileDiff, IndexedHunk, IndexedRepoDiff};
pub use repo_diff::{
    DiffHunk, DiffLine, DiffLineType, FileDiff, RepoDiff, load_repo_diff, load_repo_diff_indexed,
    parse_unified_diff,
};
pub use status::{FileGitStatus, GitFileChange, GitRepoStatus};
