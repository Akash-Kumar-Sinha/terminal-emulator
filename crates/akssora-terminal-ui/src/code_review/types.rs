use std::path::PathBuf;

// INFO: Tracks hovered changed line in Code Review gutter with Add/Remove button interaction state.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CodeReviewGutterHover {
    pub file_idx: usize,
    pub row: usize,
    pub is_deletion: bool,
    pub action: Option<akssora_editor::GutterActionKind>,
}

// INFO: Diff presentation mode supporting unified and side-by-side split modes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DiffViewMode {
    Unified,
    Split,
}

impl DiffViewMode {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Unified => "Unified",
            Self::Split => "Split",
        }
    }
}

// INFO: Inline review comment attached to a specific file and line in the diff.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReviewComment {
    pub id: u64,
    pub file_path: PathBuf,
    pub line_number: usize,
    pub author: String,
    pub body: String,
    pub created_at: String,
    pub resolved: bool,
}

// INFO: View mode for changed files sidebar in Code Review (Tree hierarchy vs Flat list).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum CodeReviewSidebarMode {
    #[default]
    Tree,
    FlatList,
}

// INFO: Item in the changed files tree hierarchy.
#[derive(Clone, Debug, PartialEq)]
pub enum CodeReviewTreeItem {
    SectionHeader {
        is_staged: bool,
        count: usize,
        is_collapsed: bool,
    },
    Directory {
        is_staged: bool,
        path: PathBuf,
        display_name: String,
        depth: usize,
        is_collapsed: bool,
    },
    File {
        is_staged: bool,
        orig_idx: usize,
        path: PathBuf,
        display_name: String,
        depth: usize,
        status: akssora_git::FileGitStatus,
        additions: usize,
        deletions: usize,
    },
}

// INFO: Target entity for destructive Git discard operations.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DiscardTarget {
    File(PathBuf),
    Directory(PathBuf),
    All,
}

// INFO: Target interactable elements on the Code Review interface.
#[derive(Clone, Debug, PartialEq)]
pub enum CodeReviewAction {
    SelectFile(usize),
    SelectFileForEdit(usize),
    SelectTreeFile {
        is_staged: bool,
        orig_idx: usize,
        path: PathBuf,
    },
    ClickEditor {
        orig_idx: usize,
        click_x: f32,
        click_y: f32,
        body_x: f32,
        body_y: f32,
        body_w: f32,
    },
    SaveFile(usize),
    ToggleSidebarMode,
    ToggleSectionCollapse(bool),
    ToggleTreeDir(PathBuf),
    ToggleSidebar,
    ToggleDiffMode,
    ToggleStaged,
    Refresh,
    StageAll,
    UnstageAll,
    PromptDiscardAll,
    StageDirectoryByPath(PathBuf),
    UnstageDirectoryByPath(PathBuf),
    PromptDiscardDirectory(PathBuf),
    StageFile(usize),
    UnstageFile(usize),
    DiscardFile(usize),
    StageFileByPath(PathBuf),
    UnstageFileByPath(PathBuf),
    DiscardFileByPath(PathBuf),
    PromptDiscardFile(PathBuf),
    ConfirmDiscardOk,
    ConfirmDiscardCancel,
    OpenCommitDialog,
    CloseCommitDialog,
    SubmitCommit,
    PushToRemote,
    ToggleFileCollapse(usize),
    ToggleFullFile(usize),
    ToggleCollapseAll,
    CollapseAll,
    ExpandAll,
    ExpandContextTop {
        file_idx: usize,
        hunk_idx: usize,
    },
    ExpandContextBottom {
        file_idx: usize,
        hunk_idx: usize,
    },
    CopyFilePath(usize),
    OpenFileInEditor(usize),
    OpenFileInEditorByPath(PathBuf),
    ClosePanel,
    ToggleExpandView,
    OpenCommentInput {
        file_idx: usize,
        line_idx: usize,
    },
    CancelCommentInput,
    SubmitComment,
    ToggleCommentResolved(u64),
    DeleteComment(u64),
    ScrollDiff(f32),
    ScrollSidebar(f32),
    SearchChangedFiles(String),
}

// INFO: Comprehensive Code Review panel state model managing repo diffs, review comments, and Git actions.
