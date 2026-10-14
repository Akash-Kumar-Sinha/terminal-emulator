use crate::app::App;
use akssora_terminal_ui::{ExplorerToolbarAction, Tooltip};

// INFO: Tooltip resolution for file explorer toolbar icons (new file, folder, refresh, hidden files).
pub(crate) fn check_explorer_tooltip(
    app: &App,
    mx: f32,
    my: f32,
    sb_w: f32,
    exp_w: f32,
) -> Option<Tooltip> {
    if app.explorer.is_expanded
        && mx >= sb_w
        && mx <= sb_w + exp_w
        && let Some(action) = app.explorer.hovered_toolbar
    {
        let target_rect = (mx - 20.0, my - 10.0, 40.0, 20.0);
        match action {
            ExplorerToolbarAction::ChangeRoot => Some(Tooltip::for_target(
                "Change Workspace Root Folder",
                None,
                target_rect,
            )),
            ExplorerToolbarAction::NewFile => {
                Some(Tooltip::for_target("New File", None, target_rect))
            }
            ExplorerToolbarAction::NewFolder => {
                Some(Tooltip::for_target("New Folder", None, target_rect))
            }
            ExplorerToolbarAction::ToggleHidden => Some(Tooltip::for_target(
                "Toggle Hidden Files",
                None,
                target_rect,
            )),
        }
    } else {
        None
    }
}
