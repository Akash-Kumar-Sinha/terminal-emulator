use akssora_terminal_config::{Action, LineNumberMode, TabLayoutPreference};

// INFO: Category tabs for the Settings view.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SettingsTab {
    General,
    Appearance,
    Editor,
    CodeReview,
    Keybindings,
}

impl SettingsTab {
    pub const ALL: &'static [SettingsTab] = &[
        SettingsTab::General,
        SettingsTab::Appearance,
        SettingsTab::Editor,
        SettingsTab::CodeReview,
        SettingsTab::Keybindings,
    ];

    pub fn label(&self) -> &'static str {
        match self {
            Self::General => "General",
            Self::Appearance => "Appearance",
            Self::Editor => "Editor",
            Self::CodeReview => "Code Review",
            Self::Keybindings => "Keybindings",
        }
    }
}

// INFO: Interactive actions available on the Settings interface.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SettingsAction {
    SelectTab(SettingsTab),
    ToggleLayoutDropdown,
    SelectLayout(TabLayoutPreference),
    DecreaseFontSize,
    IncreaseFontSize,
    ResetFontSize,
    DecreaseUiFontSize,
    IncreaseUiFontSize,
    ResetUiFontSize,
    // Appearance settings
    ToggleCursorBlink,
    CycleCursorStyle,
    // Editor settings
    ToggleLineNumberDropdown,
    SelectLineNumberMode(LineNumberMode),
    CycleTabSize,
    CycleLineHeight,
    ToggleVimMode,
    ToggleVimClipboard,
    ToggleVimStatusBar,
    ToggleAutocompleteSymbols,
    ToggleFormatOnSave,
    ToggleLineWrap,
    ToggleAutoSave,
    // Code Review settings
    ToggleCodeReviewDiffMode,
    ToggleCodeReviewFindBar,
    // Keybindings settings
    KeybindingSearchClick,
    KeybindingResetAll,
    KeybindingRebind(Action),
}
