use crate::app::App;
use akssora_terminal_config::Action;
use akssora_terminal_ui::Tooltip;

// INFO: Tooltip resolution for markdown/source editor toolbar actions and tab strip.
pub(crate) fn check_editor_tooltip(
    app: &App,
    mx: f32,
    my: f32,
    sb_w: f32,
    exp_w: f32,
    top_h: f32,
    window_w: f32,
) -> Option<Tooltip> {
    let ws_x = sb_w + exp_w;
    let ws_w = (window_w - ws_x).max(0.0);

    if let Some(action) = app.editor_hovered_toolbar {
        let rect = akssora_terminal_ui::EditorTabBar::toolbar_rect(action, ws_x, top_h, ws_w);
        match action {
            akssora_terminal_ui::EditorToolbarAction::FormatDocument => {
                let sc = app
                    .keybindings()
                    .shortcut_for(Action::FormatDocument)
                    .map(String::from);
                Some(Tooltip::for_target("Format Document", sc, rect))
            }
            akssora_terminal_ui::EditorToolbarAction::TogglePreview => {
                let sc = app
                    .keybindings()
                    .shortcut_for(Action::ToggleMarkdownPreview)
                    .map(String::from);
                Some(Tooltip::for_target("Toggle Markdown Preview", sc, rect))
            }
            akssora_terminal_ui::EditorToolbarAction::ToggleSplitPreview => {
                let sc = app
                    .keybindings()
                    .shortcut_for(Action::ToggleMarkdownSplitPreview)
                    .map(String::from);
                Some(Tooltip::for_target(
                    "Toggle Split Markdown Preview",
                    sc,
                    rect,
                ))
            }
            akssora_terminal_ui::EditorToolbarAction::ToggleInlineDiff => {
                let sc = app
                    .keybindings()
                    .shortcut_for(Action::ToggleInlineDiff)
                    .map(String::from);
                Some(Tooltip::for_target("Toggle Inline Diff", sc, rect))
            }
        }
    } else if let Some(_close_idx) = app.editor_hovered_close {
        let rect = (mx - 10.0, my - 10.0, 20.0, 20.0);
        let sc = app
            .keybindings()
            .shortcut_for(Action::CloseTab)
            .map(String::from);
        Some(Tooltip::for_target("Close File", sc, rect))
    } else if let Some(tab_idx) = app.editor_hovered_tab
        && let Some(path) = app.editor_open_files.get(tab_idx)
    {
        let rect = (mx - 20.0, my - 10.0, 40.0, 20.0);
        Some(Tooltip::for_target(path.display().to_string(), None, rect))
    } else if let Some((_row, Some(action))) = app.editor_hovered_gutter {
        let rect = (mx - 10.0, my - 10.0, 20.0, 20.0);
        let label = match action {
            akssora_editor::GutterActionKind::StageLine => "Add This Line (+)",
            akssora_editor::GutterActionKind::RevertLine => "Revert This Line (↶)",
        };
        Some(Tooltip::for_target(label, None, rect))
    } else {
        None
    }
}
