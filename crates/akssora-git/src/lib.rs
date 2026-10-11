pub mod branch;
pub mod status;

pub use branch::GitInspector;
pub use status::{FileGitStatus, GitFileChange, GitRepoStatus};
