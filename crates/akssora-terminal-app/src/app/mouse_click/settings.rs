use akssora_terminal_ui::{SettingsAction, SettingsView, TabKind};

use crate::app::App;

impl App {
    pub fn handle_settings_clicks(
        &mut self,
        mx: f32,
        my: f32,
        w: f32,
        top_h: f32,
        sb_w: f32,
    ) -> bool {
        let is_settings = self
            .tab_manager
            .active_tab()
            .map(|t| matches!(t.kind, TabKind::Settings))
            .unwrap_or(false);
        if !is_settings {
            return false;
        }

        let exp_w = self.explorer.effective_width();
        let ws_x = sb_w + exp_w;
        let cell_h = self.ui_fonts.as_ref().map(|f| f.cell_h).unwrap_or(18.0);

        let action_hit = SettingsView::hit_test_action_tab(
            mx,
            my,
            ws_x,
            top_h,
            (w - ws_x).max(0.0),
            cell_h,
            self.active_settings_tab,
            akssora_terminal_config::Action::ALL,
            self.settings_layout_dropdown_open,
            self.settings_line_number_dropdown_open,
        );

        if let Some(action) = action_hit {
            match action {
                SettingsAction::SelectTab(tab) => {
                    self.active_settings_tab = tab;
                    self.settings_layout_dropdown_open = false;
                    self.settings_line_number_dropdown_open = false;
                }
                SettingsAction::ToggleLayoutDropdown => {
                    self.settings_layout_dropdown_open = !self.settings_layout_dropdown_open;
                    self.settings_line_number_dropdown_open = false;
                }
                SettingsAction::SelectLayout(opt) => {
                    self.set_tab_layout(opt);
                    self.settings_layout_dropdown_open = false;
                }
                SettingsAction::DecreaseFontSize => self.decrease_font_size(),
                SettingsAction::IncreaseFontSize => self.increase_font_size(),
                SettingsAction::ResetFontSize => self.reset_font_size(),
                SettingsAction::DecreaseUiFontSize => self.decrease_ui_font_size(),
                SettingsAction::IncreaseUiFontSize => self.increase_ui_font_size(),
                SettingsAction::ResetUiFontSize => self.reset_ui_font_size(),
                SettingsAction::ToggleCursorBlink => {
                    self.config.cursor_blink = !self.config.cursor_blink;
                    let _ = self.config.save();
                }
                SettingsAction::CycleCursorStyle => {
                    self.config.cursor_style = match self.config.cursor_style.as_str() {
                        "Block" => "Beam".to_string(),
                        "Beam" => "Underline".to_string(),
                        _ => "Block".to_string(),
                    };
                    let _ = self.config.save();
                }
                SettingsAction::CycleTabSize => {
                    self.config.tab_size = match self.config.tab_size {
                        2 => 4,
                        4 => 8,
                        _ => 2,
                    };
                    for doc in self.editor_documents.values_mut() {
                        doc.tab_size = self.config.tab_size;
                    }
                    for doc in self.editors.values_mut() {
                        doc.tab_size = self.config.tab_size;
                    }
                    let _ = self.config.save();
                }
                SettingsAction::CycleLineHeight => {
                    let next = if (self.config.line_height - 1.0).abs() < 0.05 {
                        1.1
                    } else if (self.config.line_height - 1.1).abs() < 0.05 {
                        1.2
                    } else if (self.config.line_height - 1.2).abs() < 0.05 {
                        1.3
                    } else if (self.config.line_height - 1.3).abs() < 0.05 {
                        1.4
                    } else if (self.config.line_height - 1.4).abs() < 0.05 {
                        1.5
                    } else if (self.config.line_height - 1.5).abs() < 0.05 {
                        1.6
                    } else if (self.config.line_height - 1.6).abs() < 0.05 {
                        1.8
                    } else if (self.config.line_height - 1.8).abs() < 0.05 {
                        2.0
                    } else {
                        1.0
                    };
                    self.config.line_height = next;
                    for doc in self.editor_documents.values_mut() {
                        doc.line_height = self.config.line_height;
                    }
                    for doc in self.editors.values_mut() {
                        doc.line_height = self.config.line_height;
                    }
                    let _ = self.config.save();
                }
                SettingsAction::ToggleLineNumberDropdown => {
                    self.settings_line_number_dropdown_open =
                        !self.settings_line_number_dropdown_open;
                    self.settings_layout_dropdown_open = false;
                }
                SettingsAction::SelectLineNumberMode(mode) => {
                    self.config.line_number_mode = mode;
                    self.config.line_numbers = mode != akssora_terminal_config::LineNumberMode::Off;
                    for doc in self.editor_documents.values_mut() {
                        doc.line_number_mode = mode;
                        doc.line_numbers = self.config.line_numbers;
                    }
                    for doc in self.editors.values_mut() {
                        doc.line_number_mode = mode;
                        doc.line_numbers = self.config.line_numbers;
                    }
                    self.update_editor_scrollbar_geometries();
                    let _ = self.config.save();
                    self.settings_line_number_dropdown_open = false;
                }
                SettingsAction::ToggleVimMode => {
                    self.config.vim_mode = !self.config.vim_mode;
                    for doc in self.editor_documents.values_mut() {
                        doc.vim_enabled = self.config.vim_mode;
                    }
                    for doc in self.editors.values_mut() {
                        doc.vim_enabled = self.config.vim_mode;
                    }
                    let _ = self.config.save();
                }
                SettingsAction::ToggleVimClipboard => {
                    self.config.vim_unnamed_system_clipboard =
                        !self.config.vim_unnamed_system_clipboard;
                    let _ = self.config.save();
                }
                SettingsAction::ToggleVimStatusBar => {
                    self.config.vim_status_bar = !self.config.vim_status_bar;
                    let _ = self.config.save();
                }
                SettingsAction::ToggleAutocompleteSymbols => {
                    self.config.autocomplete_symbols = !self.config.autocomplete_symbols;
                    for doc in self.editor_documents.values_mut() {
                        doc.autocomplete_symbols = self.config.autocomplete_symbols;
                    }
                    for doc in self.editors.values_mut() {
                        doc.autocomplete_symbols = self.config.autocomplete_symbols;
                    }
                    let _ = self.config.save();
                }
                SettingsAction::ToggleFormatOnSave => {
                    self.config.format_on_save = !self.config.format_on_save;
                    let _ = self.config.save();
                }
                SettingsAction::ToggleLineWrap => {
                    self.config.line_wrap = !self.config.line_wrap;
                    for doc in self.editor_documents.values_mut() {
                        doc.line_wrap = self.config.line_wrap;
                    }
                    for doc in self.editors.values_mut() {
                        doc.line_wrap = self.config.line_wrap;
                    }
                    let _ = self.config.save();
                }
                SettingsAction::ToggleAutoSave => {
                    self.config.auto_save = !self.config.auto_save;
                    let _ = self.config.save();
                }
                SettingsAction::ToggleCodeReviewDiffMode => {
                    self.config.code_review_diff_mode =
                        match self.config.code_review_diff_mode.as_str() {
                            "Unified" => "Split".to_string(),
                            _ => "Unified".to_string(),
                        };
                    let _ = self.config.save();
                }
                SettingsAction::ToggleCodeReviewFindBar => {
                    self.config.code_review_show_find_bar = !self.config.code_review_show_find_bar;
                    let _ = self.config.save();
                }
                SettingsAction::KeybindingSearchClick => {}
                SettingsAction::KeybindingResetAll => {
                    self.config.keybindings = akssora_terminal_config::Keybindings::default();
                }
                SettingsAction::KeybindingRebind(_) => {}
            }
        } else if self.settings_layout_dropdown_open || self.settings_line_number_dropdown_open {
            self.settings_layout_dropdown_open = false;
            self.settings_line_number_dropdown_open = false;
        }

        if let Some(win) = self.window.as_ref() {
            win.request_redraw();
        }
        true
    }
}
