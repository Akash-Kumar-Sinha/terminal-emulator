use crate::app::App;
use akssora_terminal_ui::{CodeReviewAction, Tooltip};

// INFO: Tooltip resolution for Code Review panel actions (stage, unstage, collapse, diff modes).
pub fn check_code_review_tooltip(
    app: &App,
    mx: f32,
    my: f32,
    sb_w: f32,
    exp_w: f32,
    window_w: f32,
    window_h: f32,
) -> Option<Tooltip> {
    let ws_x = sb_w + exp_w;
    let ws_y = app.header.height;
    let ws_w = (window_w - ws_x).max(100.0);
    let ws_h = (window_h - ws_y).max(100.0);

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
            CodeReviewAction::OpenFileInEditor(_) => Some(Tooltip::for_target(
                "Open File in Editor",
                None,
                target_rect,
            )),
            CodeReviewAction::StageFile(_) => {
                Some(Tooltip::for_target("Stage File Changes", None, target_rect))
            }
            CodeReviewAction::UnstageFile(_) => Some(Tooltip::for_target(
                "Unstage File Changes",
                None,
                target_rect,
            )),
            CodeReviewAction::DiscardFile(_) => Some(Tooltip::for_target(
                "Discard Changes to File",
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
            CodeReviewAction::SelectFile(_) => {
                Some(Tooltip::for_target("View File Diff", None, target_rect))
            }
            _ => None,
        }
    } else {
        None
    }
}
