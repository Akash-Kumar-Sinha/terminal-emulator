use crate::app::App;
use akssora_terminal_ui::{CodeReviewAction, Tooltip};

// INFO: Tooltip resolution for Code Review panel actions (stage, unstage, collapse, diff modes).
pub fn check_code_review_tooltip(
    app: &App,
    mx: f32,
    my: f32,
    _sb_w: f32,
    _exp_w: f32,
    _window_w: f32,
    _window_h: f32,
) -> Option<Tooltip> {
    let (ws_x, ws_y, ws_w, ws_h) = app.code_review_bounds()?;

    if let Some((_row, Some(action))) = app.editor_hovered_gutter {
        let rect = (mx - 10.0, my - 10.0, 20.0, 20.0);
        let label = match action {
            akssora_editor::GutterActionKind::StageLine => "Stage Change",
            akssora_editor::GutterActionKind::RevertLine => "Revert Changes",
        };
        return Some(Tooltip::for_target(label, None, rect));
    }

    if let Some(action) = akssora_terminal_ui::CodeReviewLayout::hit_test(
        &app.code_review,
        ws_x,
        ws_y,
        ws_w,
        ws_h,
        mx,
        my,
    ) {
        let target_rect = (mx - 20.0, my - 10.0, 40.0, 20.0);
        match action {
            CodeReviewAction::ToggleExpandView => Some(Tooltip::for_target(
                if app.code_review.is_full_view {
                    "Restore Split View"
                } else {
                    "Expand Code Review"
                },
                None,
                target_rect,
            )),
            CodeReviewAction::ToggleSidebar => Some(Tooltip::for_target(
                "Toggle Files Sidebar",
                None,
                target_rect,
            )),
            CodeReviewAction::ToggleStaged => Some(Tooltip::for_target(
                if app.code_review.is_staged {
                    "Switch to Unstaged Changes"
                } else {
                    "Switch to Staged Changes"
                },
                None,
                target_rect,
            )),
            CodeReviewAction::ToggleCollapseAll => Some(Tooltip::for_target(
                "Collapse / Expand All Accordions",
                None,
                target_rect,
            )),
            CodeReviewAction::CollapseAll => Some(Tooltip::for_target(
                "Collapse All Accordions",
                None,
                target_rect,
            )),
            CodeReviewAction::ExpandAll => Some(Tooltip::for_target(
                "Expand All Accordions",
                None,
                target_rect,
            )),
            CodeReviewAction::ToggleSidebarMode => Some(Tooltip::for_target(
                "Switch Tree / List View",
                None,
                target_rect,
            )),
            CodeReviewAction::OpenCommitDialog => Some(Tooltip::for_target(
                "Commit Staged Changes",
                None,
                target_rect,
            )),
            CodeReviewAction::ClosePanel => Some(Tooltip::for_target(
                "Close Code Review Panel",
                None,
                target_rect,
            )),
            CodeReviewAction::ToggleFullFile(idx) => {
                let is_full = app
                    .code_review
                    .diff
                    .files
                    .get(idx)
                    .map(|f| app.code_review.is_full_file(&f.path))
                    .unwrap_or(false);
                Some(Tooltip::for_target(
                    if is_full {
                        "Show Diff Only"
                    } else {
                        "Show Full File"
                    },
                    None,
                    target_rect,
                ))
            }
            CodeReviewAction::OpenFileInEditor(_) | CodeReviewAction::OpenFileInEditorByPath(_) => {
                Some(Tooltip::for_target(
                    "Open File in Editor",
                    None,
                    target_rect,
                ))
            }
            CodeReviewAction::StageFile(_) | CodeReviewAction::StageFileByPath(_) => {
                Some(Tooltip::for_target("Stage File Changes", None, target_rect))
            }
            CodeReviewAction::UnstageFile(_) | CodeReviewAction::UnstageFileByPath(_) => Some(
                Tooltip::for_target("Unstage File Changes", None, target_rect),
            ),
            CodeReviewAction::DiscardFile(_)
            | CodeReviewAction::DiscardFileByPath(_)
            | CodeReviewAction::PromptDiscardFile(_) => Some(Tooltip::for_target(
                "Discard Changes to File",
                None,
                target_rect,
            )),
            CodeReviewAction::StageDirectoryByPath(_) => Some(Tooltip::for_target(
                "Stage Folder Changes",
                None,
                target_rect,
            )),
            CodeReviewAction::UnstageDirectoryByPath(_) => Some(Tooltip::for_target(
                "Unstage Folder Changes",
                None,
                target_rect,
            )),
            CodeReviewAction::PromptDiscardDirectory(_) => Some(Tooltip::for_target(
                "Discard Changes in Folder",
                None,
                target_rect,
            )),
            CodeReviewAction::StageAll => {
                Some(Tooltip::for_target("Stage All Changes", None, target_rect))
            }
            CodeReviewAction::UnstageAll => Some(Tooltip::for_target(
                "Unstage All Changes",
                None,
                target_rect,
            )),
            CodeReviewAction::PromptDiscardAll => Some(Tooltip::for_target(
                "Discard All Changes",
                None,
                target_rect,
            )),
            CodeReviewAction::ToggleSectionCollapse(is_staged) => Some(Tooltip::for_target(
                if is_staged {
                    "Toggle Staged Changes"
                } else {
                    "Toggle Changes"
                },
                None,
                target_rect,
            )),
            CodeReviewAction::CopyFilePath(_) => Some(Tooltip::for_target(
                "Copy Relative File Path",
                None,
                target_rect,
            )),
            CodeReviewAction::ToggleFileCollapse(_) => Some(Tooltip::for_target(
                "Toggle Collapse File Diff",
                None,
                target_rect,
            )),
            CodeReviewAction::SelectFile(_) | CodeReviewAction::SelectTreeFile { .. } => None,
            _ => None,
        }
    } else {
        None
    }
}
