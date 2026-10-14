// INFO: Centralized icon repository, 2D vector graphics, and glyph symbol definitions for Akssora.

pub mod sink;

pub mod add;
pub mod api_testing_session;
pub mod app_logo;
pub mod arrow_left_right;
pub mod arrow_revert;
pub mod arrow_right;
pub mod arrow_undo;
pub mod arrow_up_right;
pub mod checkbox_checked;
pub mod checkbox_unchecked;
pub mod checkmark;
pub mod chevron_down;
pub mod chevron_right;
pub mod chevron_up;
pub mod close;
pub mod code_review_diff;
pub mod code_review_session;
pub mod commit;
pub mod copy;
pub mod dirty_circle;
pub mod editor_session;
pub mod explorer_toggle;
pub mod file_config;
pub mod file_doc;
pub mod file_general;
pub mod file_git;
pub mod file_image;
pub mod file_lock;
pub mod file_python;
pub mod file_rust;
pub mod file_script;
pub mod file_shell;
pub mod file_web;
pub mod folder;
pub mod format_document;
pub mod full_file;
pub mod git_branch;
pub mod git_status_added;
pub mod git_status_conflict;
pub mod git_status_deleted;
pub mod git_status_modified;
pub mod git_status_renamed;
pub mod git_status_untracked;
pub mod inline_diff;
pub mod panel_collapsed;
pub mod panel_expanded;
pub mod pin;
pub mod plugin_session;
pub mod preview;
pub mod remove;
pub mod search;
pub mod settings_gear;
pub mod settings_session;
pub mod sidebar_list;
pub mod sidebar_toggle;
pub mod sidebar_tree;
pub mod split_preview;
pub mod symlink;
pub mod terminal_session;
pub mod tree_branch;
pub mod tree_branch_last;
pub mod window_close;
pub mod window_maximize;
pub mod window_minimize;
pub mod window_restore;

pub use sink::{IconSink, MockIconSink};

pub use add::AddIcon;
pub use api_testing_session::ApiTestingSessionIcon;
pub use app_logo::AppLogoIcon;
pub use arrow_left_right::ArrowLeftRightIcon;
pub use arrow_revert::ArrowRevertIcon;
pub use arrow_right::ArrowRightIcon;
pub use arrow_undo::ArrowUndoIcon;
pub use arrow_up_right::ArrowUpRightIcon;
pub use checkbox_checked::CheckboxCheckedIcon;
pub use checkbox_unchecked::CheckboxUncheckedIcon;
pub use checkmark::CheckmarkIcon;
pub use chevron_down::ChevronDownIcon;
pub use chevron_right::ChevronRightIcon;
pub use chevron_up::ChevronUpIcon;
pub use close::CloseIcon;
pub use code_review_diff::CodeReviewDiffIcon;
pub use code_review_session::CodeReviewSessionIcon;
pub use commit::CommitIcon;
pub use copy::CopyIcon;
pub use dirty_circle::DirtyCircleIcon;
pub use editor_session::EditorSessionIcon;
pub use explorer_toggle::ExplorerToggleIcon;
pub use file_config::FileConfigIcon;
pub use file_doc::FileDocIcon;
pub use file_general::FileGeneralIcon;
pub use file_git::FileGitIcon;
pub use file_image::FileImageIcon;
pub use file_lock::FileLockIcon;
pub use file_python::FilePythonIcon;
pub use file_rust::FileRustIcon;
pub use file_script::FileScriptIcon;
pub use file_shell::FileShellIcon;
pub use file_web::FileWebIcon;
pub use folder::FolderIcon;
pub use format_document::FormatDocumentIcon;
pub use full_file::FullFileIcon;
pub use git_branch::GitBranchIcon;
pub use git_status_added::GitStatusAddedIcon;
pub use git_status_conflict::GitStatusConflictIcon;
pub use git_status_deleted::GitStatusDeletedIcon;
pub use git_status_modified::GitStatusModifiedIcon;
pub use git_status_renamed::GitStatusRenamedIcon;
pub use git_status_untracked::GitStatusUntrackedIcon;
pub use inline_diff::InlineDiffIcon;
pub use panel_collapsed::PanelCollapsedIcon;
pub use panel_expanded::PanelExpandedIcon;
pub use pin::PinIcon;
pub use plugin_session::PluginSessionIcon;
pub use preview::PreviewIcon;
pub use remove::RemoveIcon;
pub use search::SearchIcon;
pub use settings_gear::SettingsGearIcon;
pub use settings_session::SettingsSessionIcon;
pub use sidebar_list::SidebarListIcon;
pub use sidebar_toggle::SidebarToggleIcon;
pub use sidebar_tree::SidebarTreeIcon;
pub use split_preview::SplitPreviewIcon;
pub use symlink::SymlinkIcon;
pub use terminal_session::TerminalSessionIcon;
pub use tree_branch::TreeBranchIcon;
pub use tree_branch_last::TreeBranchLastIcon;
pub use window_close::WindowCloseIcon;
pub use window_maximize::WindowMaximizeIcon;
pub use window_minimize::WindowMinimizeIcon;
pub use window_restore::WindowRestoreIcon;

// INFO: Resolves an expressive Unicode icon for a given file name or extension.
pub fn file_type_icon(name: &str) -> &'static str {
    let lower = name.to_lowercase();
    if FileConfigIcon::matches(&lower) {
        FileConfigIcon::GLYPH
    } else if lower.ends_with(FileRustIcon::EXTENSION) {
        FileRustIcon::GLYPH
    } else if FileDocIcon::matches(&lower) {
        FileDocIcon::GLYPH
    } else if FilePythonIcon::matches(&lower) {
        FilePythonIcon::GLYPH
    } else if FileScriptIcon::matches(&lower) {
        FileScriptIcon::GLYPH
    } else if FileWebIcon::matches(&lower) {
        FileWebIcon::GLYPH
    } else if FileShellIcon::matches(&lower) {
        FileShellIcon::GLYPH
    } else if FileGitIcon::matches(&lower) {
        FileGitIcon::GLYPH
    } else if FileImageIcon::matches(&lower) {
        FileImageIcon::GLYPH
    } else if FileLockIcon::matches(&lower) {
        FileLockIcon::GLYPH
    } else {
        FileGeneralIcon::GLYPH
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_centralized_file_type_icons() {
        assert_eq!(file_type_icon("main.rs"), "\u{2699}");
        assert_eq!(file_type_icon("package.json"), "\u{2756}");
        assert_eq!(file_type_icon("index.html"), "\u{25C8}");
        assert_eq!(file_type_icon("README.md"), "\u{2261}");
        assert_eq!(file_type_icon("script.py"), "\u{03C0}");
        assert_eq!(file_type_icon("app.tsx"), "\u{2737}");
        assert_eq!(file_type_icon("deploy.sh"), "\u{203A}");
        assert_eq!(file_type_icon(".gitignore"), "\u{2605}");
        assert_eq!(file_type_icon("image.png"), "\u{25A3}");
        assert_eq!(file_type_icon("Cargo.lock"), "\u{25C6}");
        assert_eq!(file_type_icon("unknown.xyz"), "\u{229F}");
    }
}
