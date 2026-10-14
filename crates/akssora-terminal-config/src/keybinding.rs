// INFO: Scopes for categorizing and isolating keyboard shortcuts to prevent false conflicts.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum KeybindingScope {
    Global,
    Terminal,
    Editor,
    Panel,
}

impl KeybindingScope {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Global => "Global",
            Self::Terminal => "Terminal",
            Self::Editor => "Editor",
            Self::Panel => "Panel",
        }
    }
}

// INFO: Warning emitted when conflicting shortcuts are detected across compatible scopes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConflictWarning {
    pub shortcut: String,
    pub first_action: Action,
    pub second_action: Action,
    pub scope: KeybindingScope,
}

// INFO: Command action identifiers for keyboard bindings and user triggers.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Action {
    ToggleSidebar,
    NewTab,
    CloseTab,
    CloseOtherTabs,
    NextTab,
    PrevTab,
    CycleTabLayout,
    OpenSettings,
    IncreaseFontSize,
    DecreaseFontSize,
    ResetFontSize,
    CreateSplit,
    SplitHorizontal,
    Copy,
    Paste,
    ScrollPageUp,
    ScrollPageDown,
    CloseEditorTab,
    CloseOtherEditorTabs,
    CloseAllEditorTabs,
    FormatDocument,
    ToggleMarkdownPreview,
    ToggleMarkdownSplitPreview,
    ToggleInlineDiff,
}

impl Action {
    pub fn description(&self) -> &'static str {
        match self {
            Self::ToggleSidebar => "Toggle Sidebar",
            Self::NewTab => "New Terminal Tab",
            Self::CloseTab => "Close Active Tab",
            Self::CloseOtherTabs => "Close Other Tabs",
            Self::NextTab => "Next Tab",
            Self::PrevTab => "Previous Tab",
            Self::CycleTabLayout => "Switch Tab Layout",
            Self::OpenSettings => "Settings",
            Self::IncreaseFontSize => "Increase Font Size",
            Self::DecreaseFontSize => "Decrease Font Size",
            Self::ResetFontSize => "Reset Font Size",
            Self::CreateSplit => "Split Vertically",
            Self::SplitHorizontal => "Split Horizontally",
            Self::Copy => "Copy Selection",
            Self::Paste => "Paste Clipboard",
            Self::ScrollPageUp => "Scroll Page Up",
            Self::ScrollPageDown => "Scroll Page Down",
            Self::CloseEditorTab => "Close Editor Tab",
            Self::CloseOtherEditorTabs => "Close Other Editor Tabs",
            Self::CloseAllEditorTabs => "Close All Editor Tabs",
            Self::FormatDocument => "Format Document",
            Self::ToggleMarkdownPreview => "Toggle Markdown Preview",
            Self::ToggleMarkdownSplitPreview => "Toggle Split Markdown Preview",
            Self::ToggleInlineDiff => "Toggle Inline Diff",
        }
    }

    pub fn scope(&self) -> KeybindingScope {
        match self {
            Self::ToggleSidebar
            | Self::CycleTabLayout
            | Self::OpenSettings
            | Self::IncreaseFontSize
            | Self::DecreaseFontSize
            | Self::ResetFontSize => KeybindingScope::Global,

            Self::NewTab
            | Self::CloseTab
            | Self::CloseOtherTabs
            | Self::NextTab
            | Self::PrevTab => KeybindingScope::Panel,

            Self::CreateSplit
            | Self::SplitHorizontal
            | Self::Copy
            | Self::Paste
            | Self::ScrollPageUp
            | Self::ScrollPageDown => KeybindingScope::Terminal,

            Self::CloseEditorTab
            | Self::CloseOtherEditorTabs
            | Self::CloseAllEditorTabs
            | Self::FormatDocument
            | Self::ToggleMarkdownPreview
            | Self::ToggleMarkdownSplitPreview
            | Self::ToggleInlineDiff => KeybindingScope::Editor,
        }
    }

    pub const ALL: &'static [Action] = &[
        Action::ToggleSidebar,
        Action::NewTab,
        Action::CloseTab,
        Action::CloseOtherTabs,
        Action::NextTab,
        Action::PrevTab,
        Action::CycleTabLayout,
        Action::OpenSettings,
        Action::IncreaseFontSize,
        Action::DecreaseFontSize,
        Action::ResetFontSize,
        Action::CreateSplit,
        Action::SplitHorizontal,
        Action::Copy,
        Action::Paste,
        Action::ScrollPageUp,
        Action::ScrollPageDown,
        Action::CloseEditorTab,
        Action::CloseOtherEditorTabs,
        Action::CloseAllEditorTabs,
        Action::FormatDocument,
        Action::ToggleMarkdownPreview,
        Action::ToggleMarkdownSplitPreview,
        Action::ToggleInlineDiff,
    ];
}

// INFO: Keybinding configuration mapping actions to human-readable shortcuts.
#[derive(Clone, Debug, PartialEq)]
pub struct Keybindings {
    pub toggle_sidebar: Option<String>,
    pub new_tab: Option<String>,
    pub close_tab: Option<String>,
    pub close_other_tabs: Option<String>,
    pub next_tab: Option<String>,
    pub prev_tab: Option<String>,
    pub cycle_tab_layout: Option<String>,
    pub open_settings: Option<String>,
    pub increase_font_size: Option<String>,
    pub decrease_font_size: Option<String>,
    pub reset_font_size: Option<String>,
    pub create_split: Option<String>,
    pub split_horizontal: Option<String>,
    pub copy: Option<String>,
    pub paste: Option<String>,
    pub scroll_page_up: Option<String>,
    pub scroll_page_down: Option<String>,
    pub close_editor_tab: Option<String>,
    pub close_other_editor_tabs: Option<String>,
    pub close_all_editor_tabs: Option<String>,
    pub format_document: Option<String>,
    pub toggle_markdown_preview: Option<String>,
    pub toggle_markdown_split_preview: Option<String>,
    pub toggle_inline_diff: Option<String>,
}

impl Default for Keybindings {
    fn default() -> Self {
        Self {
            toggle_sidebar: Some("Ctrl+B".to_string()),
            new_tab: Some("Ctrl+Shift+T".to_string()),
            close_tab: Some("Ctrl+W".to_string()),
            close_other_tabs: Some("Ctrl+Shift+W".to_string()),
            next_tab: Some("Ctrl+Tab".to_string()),
            prev_tab: Some("Ctrl+Shift+Tab".to_string()),
            cycle_tab_layout: Some("Ctrl+L".to_string()),
            open_settings: Some("Ctrl+,".to_string()),
            increase_font_size: Some("Ctrl++".to_string()),
            decrease_font_size: Some("Ctrl+-".to_string()),
            reset_font_size: Some("Ctrl+0".to_string()),
            create_split: Some("Ctrl+Shift+D".to_string()),
            split_horizontal: Some("Ctrl+Shift+E".to_string()),
            copy: Some("Ctrl+Shift+C".to_string()),
            paste: Some("Ctrl+Shift+V".to_string()),
            scroll_page_up: Some("Shift+PageUp".to_string()),
            scroll_page_down: Some("Shift+PageDown".to_string()),
            close_editor_tab: Some("Ctrl+W".to_string()),
            close_other_editor_tabs: Some("Ctrl+Shift+W".to_string()),
            close_all_editor_tabs: Some("Ctrl+K, Ctrl+W".to_string()),
            format_document: Some("Alt+Shift+F".to_string()),
            toggle_markdown_preview: Some("Ctrl+Shift+V".to_string()),
            toggle_markdown_split_preview: Some("Ctrl+K V".to_string()),
            toggle_inline_diff: Some("Ctrl+Alt+D".to_string()),
        }
    }
}

impl Keybindings {
    pub fn shortcut_for(&self, action: Action) -> Option<&str> {
        match action {
            Action::ToggleSidebar => self.toggle_sidebar.as_deref(),
            Action::NewTab => self.new_tab.as_deref(),
            Action::CloseTab => self.close_tab.as_deref(),
            Action::CloseOtherTabs => self.close_other_tabs.as_deref(),
            Action::NextTab => self.next_tab.as_deref(),
            Action::PrevTab => self.prev_tab.as_deref(),
            Action::CycleTabLayout => self.cycle_tab_layout.as_deref(),
            Action::OpenSettings => self.open_settings.as_deref(),
            Action::IncreaseFontSize => self.increase_font_size.as_deref(),
            Action::DecreaseFontSize => self.decrease_font_size.as_deref(),
            Action::ResetFontSize => self.reset_font_size.as_deref(),
            Action::CreateSplit => self.create_split.as_deref(),
            Action::SplitHorizontal => self.split_horizontal.as_deref(),
            Action::Copy => self.copy.as_deref(),
            Action::Paste => self.paste.as_deref(),
            Action::ScrollPageUp => self.scroll_page_up.as_deref(),
            Action::ScrollPageDown => self.scroll_page_down.as_deref(),
            Action::CloseEditorTab => self.close_editor_tab.as_deref(),
            Action::CloseOtherEditorTabs => self.close_other_editor_tabs.as_deref(),
            Action::CloseAllEditorTabs => self.close_all_editor_tabs.as_deref(),
            Action::FormatDocument => self.format_document.as_deref(),
            Action::ToggleMarkdownPreview => self.toggle_markdown_preview.as_deref(),
            Action::ToggleMarkdownSplitPreview => self.toggle_markdown_split_preview.as_deref(),
            Action::ToggleInlineDiff => self.toggle_inline_diff.as_deref(),
        }
    }

    pub fn set_shortcut(&mut self, action: Action, shortcut: Option<String>) {
        let val = shortcut.filter(|s| !s.trim().is_empty());
        match action {
            Action::ToggleSidebar => self.toggle_sidebar = val,
            Action::NewTab => self.new_tab = val,
            Action::CloseTab => self.close_tab = val,
            Action::CloseOtherTabs => self.close_other_tabs = val,
            Action::NextTab => self.next_tab = val,
            Action::PrevTab => self.prev_tab = val,
            Action::CycleTabLayout => self.cycle_tab_layout = val,
            Action::OpenSettings => self.open_settings = val,
            Action::IncreaseFontSize => self.increase_font_size = val,
            Action::DecreaseFontSize => self.decrease_font_size = val,
            Action::ResetFontSize => self.reset_font_size = val,
            Action::CreateSplit => self.create_split = val,
            Action::SplitHorizontal => self.split_horizontal = val,
            Action::Copy => self.copy = val,
            Action::Paste => self.paste = val,
            Action::ScrollPageUp => self.scroll_page_up = val,
            Action::ScrollPageDown => self.scroll_page_down = val,
            Action::CloseEditorTab => self.close_editor_tab = val,
            Action::CloseOtherEditorTabs => self.close_other_editor_tabs = val,
            Action::CloseAllEditorTabs => self.close_all_editor_tabs = val,
            Action::FormatDocument => self.format_document = val,
            Action::ToggleMarkdownPreview => self.toggle_markdown_preview = val,
            Action::ToggleMarkdownSplitPreview => self.toggle_markdown_split_preview = val,
            Action::ToggleInlineDiff => self.toggle_inline_diff = val,
        }
    }

    /// Detect conflicting keyboard shortcuts across all actions.
    /// Conflicts occur when two distinct actions have the exact same normalized shortcut
    /// and either share the same scope or one of them is in Global scope.
    pub fn find_conflicts(&self) -> Vec<ConflictWarning> {
        let mut conflicts = Vec::new();
        let all = Action::ALL;
        for i in 0..all.len() {
            let act_a = all[i];
            let Some(sh_a) = self.shortcut_for(act_a) else {
                continue;
            };
            let norm_a = normalize_shortcut(sh_a);
            if norm_a.is_empty() {
                continue;
            }
            for &act_b in all.iter().skip(i + 1) {
                let Some(sh_b) = self.shortcut_for(act_b) else {
                    continue;
                };
                let norm_b = normalize_shortcut(sh_b);
                if norm_a == norm_b {
                    let scope_a = act_a.scope();
                    let scope_b = act_b.scope();
                    // Conflict if same scope, or if either is Global
                    if scope_a == scope_b
                        || scope_a == KeybindingScope::Global
                        || scope_b == KeybindingScope::Global
                    {
                        conflicts.push(ConflictWarning {
                            shortcut: sh_a.to_string(),
                            first_action: act_a,
                            second_action: act_b,
                            scope: if scope_a == KeybindingScope::Global {
                                scope_b
                            } else {
                                scope_a
                            },
                        });
                    }
                }
            }
        }
        conflicts
    }

    /// Check if assigning `candidate` to `action` would conflict with existing shortcuts.
    pub fn check_conflict(&self, action: Action, candidate: &str) -> Option<ConflictWarning> {
        let norm_c = normalize_shortcut(candidate);
        if norm_c.is_empty() {
            return None;
        }
        let target_scope = action.scope();
        for &other_action in Action::ALL {
            if other_action == action {
                continue;
            }
            if let Some(existing_sh) = self.shortcut_for(other_action)
                && normalize_shortcut(existing_sh) == norm_c
            {
                let other_scope = other_action.scope();
                if target_scope == other_scope
                    || target_scope == KeybindingScope::Global
                    || other_scope == KeybindingScope::Global
                {
                    return Some(ConflictWarning {
                        shortcut: candidate.to_string(),
                        first_action: action,
                        second_action: other_action,
                        scope: if target_scope == KeybindingScope::Global {
                            other_scope
                        } else {
                            target_scope
                        },
                    });
                }
            }
        }
        None
    }
}

fn normalize_shortcut(s: &str) -> String {
    s.trim().to_uppercase().replace(' ', "")
}
