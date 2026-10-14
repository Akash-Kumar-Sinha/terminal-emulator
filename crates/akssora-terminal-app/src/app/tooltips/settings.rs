use crate::app::App;
use akssora_terminal_config::Action;
use akssora_terminal_ui::{SettingsAction, SettingsView, Tooltip};

// INFO: Tooltip resolution for settings view actions, font controls, toggles, and keybindings.
pub(crate) fn check_settings_tooltip(
    app: &App,
    mx: f32,
    my: f32,
    sb_w: f32,
    top_h: f32,
    window_w: f32,
) -> Option<Tooltip> {
    let cell_h = app.ui_fonts.as_ref().map(|f| f.cell_h).unwrap_or(18.0);
    let ws_w = (window_w - sb_w).max(0.0);
    let minus_rect = SettingsView::font_minus_button_rect(sb_w, top_h, ws_w, cell_h);
    let plus_rect = SettingsView::font_plus_button_rect(sb_w, top_h, ws_w, cell_h);
    let reset_rect = SettingsView::font_reset_button_rect(sb_w, top_h, ws_w, cell_h);
    let dropdown_rect = SettingsView::layout_dropdown_rect(sb_w, top_h, ws_w, cell_h);

    let contains = |(rx, ry, rw, rh): (f32, f32, f32, f32)| {
        mx >= rx && mx < rx + rw && my >= ry && my < ry + rh
    };

    if contains(minus_rect) {
        let sc = app
            .keybindings()
            .shortcut_for(Action::DecreaseFontSize)
            .map(String::from);
        Some(Tooltip::for_target("Decrease Font Size", sc, minus_rect))
    } else if contains(plus_rect) {
        let sc = app
            .keybindings()
            .shortcut_for(Action::IncreaseFontSize)
            .map(String::from);
        Some(Tooltip::for_target("Increase Font Size", sc, plus_rect))
    } else if contains(reset_rect) {
        let sc = app
            .keybindings()
            .shortcut_for(Action::ResetFontSize)
            .map(String::from);
        Some(Tooltip::for_target("Reset Font Size", sc, reset_rect))
    } else if contains(dropdown_rect) {
        Some(Tooltip::for_target(
            "Select Tab Layout",
            None,
            dropdown_rect,
        ))
    } else if let Some(action) = app.hovered_setting_action {
        let target_rect = (mx - 20.0, my - 10.0, 40.0, 20.0);
        match action {
            SettingsAction::SelectTab(tab) => Some(Tooltip::for_target(
                format!("View {} Settings", tab.label()),
                None,
                target_rect,
            )),
            SettingsAction::ToggleLayoutDropdown => None,
            SettingsAction::SelectLayout(_) => None,
            SettingsAction::DecreaseFontSize => {
                let sc = app
                    .keybindings()
                    .shortcut_for(Action::DecreaseFontSize)
                    .map(String::from);
                Some(Tooltip::for_target("Decrease Font Size", sc, target_rect))
            }
            SettingsAction::IncreaseFontSize => {
                let sc = app
                    .keybindings()
                    .shortcut_for(Action::IncreaseFontSize)
                    .map(String::from);
                Some(Tooltip::for_target("Increase Font Size", sc, target_rect))
            }
            SettingsAction::ResetFontSize => {
                let sc = app
                    .keybindings()
                    .shortcut_for(Action::ResetFontSize)
                    .map(String::from);
                Some(Tooltip::for_target("Reset Font Size", sc, target_rect))
            }
            SettingsAction::DecreaseUiFontSize => Some(Tooltip::for_target(
                "Decrease UI Font Size",
                None,
                target_rect,
            )),
            SettingsAction::IncreaseUiFontSize => Some(Tooltip::for_target(
                "Increase UI Font Size",
                None,
                target_rect,
            )),
            SettingsAction::ResetUiFontSize => Some(Tooltip::for_target(
                "Reset UI Font Size (Default 14pt)",
                None,
                target_rect,
            )),
            SettingsAction::ToggleCursorBlink => Some(Tooltip::for_target(
                "Toggle Cursor Blinking",
                None,
                target_rect,
            )),
            SettingsAction::CycleCursorStyle => Some(Tooltip::for_target(
                "Cycle Cursor Style (Block / Beam / Underline)",
                None,
                target_rect,
            )),
            SettingsAction::ToggleLineNumberDropdown => None,
            SettingsAction::SelectLineNumberMode(_) => None,
            SettingsAction::CycleTabSize => Some(Tooltip::for_target(
                "Cycle Indentation Size (2, 4, 8 spaces)",
                None,
                target_rect,
            )),
            SettingsAction::CycleLineHeight => Some(Tooltip::for_target(
                "Cycle Editor Line Height (default 1.2)",
                None,
                target_rect,
            )),
            SettingsAction::ToggleVimMode => Some(Tooltip::for_target(
                "Toggle Vim Modal Editing",
                None,
                target_rect,
            )),
            SettingsAction::ToggleVimClipboard => Some(Tooltip::for_target(
                "Toggle Vim Clipboard Integration",
                None,
                target_rect,
            )),
            SettingsAction::ToggleVimStatusBar => Some(Tooltip::for_target(
                "Toggle Vim Mode Status Bar",
                None,
                target_rect,
            )),
            SettingsAction::ToggleAutocompleteSymbols => Some(Tooltip::for_target(
                "Toggle Symbol Autocompletion",
                None,
                target_rect,
            )),
            SettingsAction::ToggleFormatOnSave => Some(Tooltip::for_target(
                "Toggle Format on Save",
                None,
                target_rect,
            )),
            SettingsAction::ToggleLineWrap => Some(Tooltip::for_target(
                "Toggle Soft Line Wrapping",
                None,
                target_rect,
            )),
            SettingsAction::ToggleAutoSave => Some(Tooltip::for_target(
                "Automatically Save Files on Edit",
                None,
                target_rect,
            )),
            SettingsAction::ToggleCodeReviewDiffMode => Some(Tooltip::for_target(
                "Toggle Unified / Split Diff View",
                None,
                target_rect,
            )),
            SettingsAction::ToggleCodeReviewFindBar => Some(Tooltip::for_target(
                "Toggle Find in Files in Code Review",
                None,
                target_rect,
            )),
            SettingsAction::KeybindingSearchClick => {
                Some(Tooltip::for_target("Search Keybindings", None, target_rect))
            }
            SettingsAction::KeybindingResetAll => Some(Tooltip::for_target(
                "Reset All Keybindings to Defaults",
                None,
                target_rect,
            )),
            SettingsAction::KeybindingRebind(act) => Some(Tooltip::for_target(
                format!("Click to Rebind: {}", act.description()),
                None,
                target_rect,
            )),
        }
    } else {
        None
    }
}
