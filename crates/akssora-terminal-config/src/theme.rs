use crate::color::{Color, rgb, srgb_to_linear};

pub const COLOR_BLACK: [f32; 4] = rgb(0, 0, 0);
pub const COLOR_NEAR_BLACK: [f32; 4] = rgb(1, 1, 1);
pub const COLOR_WHITE: [f32; 4] = rgb(255, 255, 255);
pub const COLOR_RED: [f32; 4] = [0.74, 0.21, 0.16, 1.0];
pub const COLOR_GREEN: [f32; 4] = rgb(9, 232, 111);
pub const COLOR_YELLOW: [f32; 4] = [0.90, 0.63, 0.10, 1.0];
pub const COLOR_BLUE: [f32; 4] = [0.10, 0.67, 0.85, 1.0];
pub const COLOR_AMBER: [f32; 4] = [0.76, 0.50, 0.0, 1.0];
pub const COLOR_GRAY_MUTED: [f32; 4] = [0.65, 0.65, 0.70, 1.0];
pub const COLOR_GRAY_DARK: [f32; 4] = rgb(2, 2, 2);
pub const COLOR_GRAY_SURFACE: [f32; 4] = rgb(4, 4, 4);
pub const COLOR_GRAY_ELEVATED: [f32; 4] = rgb(1, 1, 1);
pub const COLOR_GRAY_CONTROL: [f32; 4] = rgb(5, 5, 5);
pub const COLOR_DIFF_ADD_BG: [f32; 4] = [0.008, 0.020, 0.008, 1.0];
pub const COLOR_DIFF_DELETE_BG: [f32; 4] = [0.020, 0.004, 0.004, 1.0];
pub const COLOR_DIFF_MODIFIED_BG: [f32; 4] = [0.020, 0.016, 0.004, 1.0];
pub const COLOR_DIFF_CHIP_DELETE_BG: [f32; 4] = [0.28, 0.10, 0.10, 1.0];
pub const COLOR_DIFF_CHIP_DELETE_FG: [f32; 4] = [0.95, 0.65, 0.65, 0.90];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PanelName {
    Terminal,
    Sidebar,
    FileExplorer,
    CodeReview,
    Settings,
    Editor,
    Header,
    QuickSearch,
    ModalDialog,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ButtonName {
    SidebarToggle,
    ExplorerToggle,
    CodeReview,
    NewTab,
    Settings,
    WindowMinimize,
    WindowMaximize,
    WindowClose,
    TabClose,
    FontIncrease,
    FontDecrease,
    FontReset,
    ModalSave,
    ModalDontSave,
    ModalCancel,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ButtonState {
    Normal,
    Hovered,
    Active,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Theme {
    pub name: String,

    pub red: [f32; 4],
    pub green: [f32; 4],
    pub yellow: [f32; 4],
    pub blue: [f32; 4],
    pub amber: [f32; 4],
    pub white: [f32; 4],
    pub black: [f32; 4],

    pub terminal_panel_background: [f32; 4],
    pub terminal_panel_foreground: [f32; 4],
    pub terminal_panel_border: [f32; 4],

    pub sidebar_panel_background: [f32; 4],
    pub sidebar_panel_border: [f32; 4],
    pub sidebar_panel_item_hover: [f32; 4],
    pub sidebar_panel_item_selected: [f32; 4],

    pub file_explorer_panel_background: [f32; 4],
    pub file_explorer_panel_border: [f32; 4],
    pub explorer_directory_icon: [f32; 4],
    pub explorer_file_icon: [f32; 4],
    pub explorer_item_hover: [f32; 4],
    pub explorer_item_selected: [f32; 4],

    pub code_review_panel_background: [f32; 4],
    pub settings_panel_background: [f32; 4],
    pub settings_panel_border: [f32; 4],
    pub editor_panel_background: [f32; 4],
    pub header_panel_background: [f32; 4],
    pub header_panel_border: [f32; 4],
    pub modal_dialog_background: [f32; 4],

    pub button_default_background: [f32; 4],
    pub button_foreground: [f32; 4],

    pub sidebar_toggle_button_hover: [f32; 4],
    pub sidebar_toggle_button_active: [f32; 4],
    pub explorer_toggle_button_hover: [f32; 4],
    pub explorer_toggle_button_active: [f32; 4],
    pub code_review_button_background: [f32; 4],
    pub code_review_button_hover: [f32; 4],
    pub code_review_button_active: [f32; 4],
    pub new_tab_button_hover: [f32; 4],
    pub new_tab_button_active: [f32; 4],
    pub settings_button_hover: [f32; 4],
    pub settings_button_active: [f32; 4],

    pub window_minimize_button_hover: [f32; 4],
    pub window_maximize_button_hover: [f32; 4],
    pub window_close_button_hover: [f32; 4],
    pub window_close_button_foreground: [f32; 4],

    pub tab_close_button_hover: [f32; 4],
    pub tab_close_button_foreground: [f32; 4],

    pub font_button_background: [f32; 4],
    pub font_button_hover: [f32; 4],
    pub reset_button_hover: [f32; 4],

    pub modal_confirm_button_background: [f32; 4],
    pub modal_confirm_button_hover: [f32; 4],
    pub modal_cancel_button_background: [f32; 4],
    pub modal_cancel_button_hover: [f32; 4],
    pub modal_danger_button_background: [f32; 4],
    pub modal_danger_button_hover: [f32; 4],

    pub warning_background: [f32; 4],
    pub warning_foreground: [f32; 4],
    pub danger_foreground: [f32; 4],
    pub success_foreground: [f32; 4],

    pub background: [f32; 4],
    pub sidebar_background: [f32; 4],
    pub header_background: [f32; 4],
    pub title_bar_background: [f32; 4],

    pub session_background: [f32; 4],
    pub session_selected_background: [f32; 4],
    pub session_hover_background: [f32; 4],

    pub tab_background: [f32; 4],
    pub tab_selected_background: [f32; 4],
    pub tab_hover_background: [f32; 4],

    pub button_hover_background: [f32; 4],
    pub button_active_background: [f32; 4],
    pub close_button_hover_background: [f32; 4],

    pub border: [f32; 4],
    pub border_subtle: [f32; 4],

    pub foreground: [f32; 4],
    pub text_primary: [f32; 4],
    pub text_secondary: [f32; 4],
    pub text_muted: [f32; 4],

    pub header_foreground: [f32; 4],
    pub header_icon: [f32; 4],
    pub header_button_hover_background: [f32; 4],
    pub header_button_active_background: [f32; 4],
    pub header_button_focus_ring: [f32; 4],

    pub palette_background: [f32; 4],
    pub search_bar_background: [f32; 4],
    pub search_bar_hover_background: [f32; 4],

    pub modal_backdrop: [f32; 4],
    pub modal_card_background: [f32; 4],
    pub modal_button_background: [f32; 4],
    pub danger: [f32; 4],
    pub drop_highlight: [f32; 4],
    pub current_line_background: [f32; 4],

    pub code_review_background: [f32; 4],
    pub code_review_card_background: [f32; 4],
    pub code_review_accordion_trigger_background: [f32; 4],
    pub code_review_accordion_body_background: [f32; 4],
    pub code_review_card_border: [f32; 4],
    pub code_review_header_background: [f32; 4],
    pub code_review_header_divider: [f32; 4],
    pub code_review_sidebar_background: [f32; 4],
    pub code_review_sidebar_border: [f32; 4],
    pub code_review_sidebar_item_hover: [f32; 4],
    pub code_review_sidebar_item_selected: [f32; 4],

    pub code_review_header_button_background: [f32; 4],
    pub code_review_header_button_hover: [f32; 4],
    pub code_review_action_button_hover: [f32; 4],
    pub code_review_commit_dialog_button: [f32; 4],
    pub code_review_commit_dialog_button_hover: [f32; 4],

    pub diff_add_green: [f32; 4],
    pub diff_add_green_background: [f32; 4],
    pub diff_delete_red: [f32; 4],
    pub diff_delete_red_background: [f32; 4],
    pub diff_modified_yellow: [f32; 4],
    pub diff_modified_yellow_background: [f32; 4],
    pub diff_chip_delete_background: [f32; 4],
    pub diff_chip_delete_foreground: [f32; 4],
    pub diff_gutter_line_number: [f32; 4],
    pub diff_gutter_add_badge_bg: [f32; 4],
    pub diff_gutter_add_badge_fg: [f32; 4],
    pub diff_gutter_mod_badge_bg: [f32; 4],
    pub diff_gutter_mod_badge_fg: [f32; 4],
    pub diff_gutter_del_badge_bg: [f32; 4],
    pub diff_gutter_del_badge_fg: [f32; 4],
    pub diff_gutter_badge_hover: [f32; 4],
    pub diff_unmodified_banner_foreground: [f32; 4],
    pub diff_unmodified_banner_divider: [f32; 4],
    pub diff_stats_pill_background: [f32; 4],

    pub accent: [f32; 4],
    pub accent_subtle: [f32; 4],
    pub accent_hover: [f32; 4],

    pub close_button_foreground: [f32; 4],

    pub tooltip_background: [f32; 4],
    pub tooltip_border: [f32; 4],
    pub tooltip_foreground: [f32; 4],
    pub tooltip_shortcut_foreground: [f32; 4],

    pub cursor: [f32; 4],
    pub cursor_foreground: [f32; 4],
    pub selection_background: [f32; 4],
    pub hover_background: [f32; 4],

    pub scrollbar_background: [f32; 4],
    pub scrollbar_track: [f32; 4],
    pub scrollbar_thumb: [f32; 4],

    pub ansi: [[f32; 4]; 16],
}

impl Default for Theme {
    fn default() -> Self {
        Self::monochrome_black()
    }
}

impl Theme {
    pub fn monochrome_black() -> Self {
        let black = COLOR_BLACK;
        let near_black = COLOR_NEAR_BLACK;

        let white = COLOR_WHITE;
        let text_secondary = rgb(220, 220, 225);
        let text_muted = COLOR_GRAY_MUTED;

        Self {
            name: "Akssora Pure Black".to_string(),

            red: COLOR_RED,
            green: COLOR_GREEN,
            yellow: COLOR_YELLOW,
            blue: COLOR_BLUE,
            amber: COLOR_AMBER,
            white,
            black,

            terminal_panel_background: black,
            terminal_panel_foreground: white,
            terminal_panel_border: COLOR_GRAY_DARK,

            sidebar_panel_background: near_black,
            sidebar_panel_border: COLOR_GRAY_DARK,
            sidebar_panel_item_hover: COLOR_GRAY_DARK,
            sidebar_panel_item_selected: COLOR_GRAY_SURFACE,

            file_explorer_panel_background: near_black,
            file_explorer_panel_border: COLOR_GRAY_DARK,
            explorer_directory_icon: COLOR_BLUE,
            explorer_file_icon: [0.75, 0.75, 0.80, 0.9],
            explorer_item_hover: COLOR_GRAY_DARK,
            explorer_item_selected: COLOR_GRAY_SURFACE,

            code_review_panel_background: black,
            settings_panel_background: black,
            settings_panel_border: COLOR_GRAY_DARK,
            editor_panel_background: black,
            header_panel_background: near_black,
            header_panel_border: COLOR_GRAY_DARK,
            modal_dialog_background: COLOR_GRAY_ELEVATED,

            button_default_background: [0.0, 0.0, 0.0, 0.0],
            button_foreground: white,

            sidebar_toggle_button_hover: COLOR_GRAY_DARK,
            sidebar_toggle_button_active: COLOR_GRAY_SURFACE,
            explorer_toggle_button_hover: COLOR_GRAY_DARK,
            explorer_toggle_button_active: COLOR_GRAY_SURFACE,
            code_review_button_background: [0.0, 0.0, 0.0, 0.0],
            code_review_button_hover: COLOR_GRAY_DARK,
            code_review_button_active: COLOR_GRAY_SURFACE,
            new_tab_button_hover: COLOR_GRAY_DARK,
            new_tab_button_active: COLOR_GRAY_SURFACE,
            settings_button_hover: COLOR_GRAY_DARK,
            settings_button_active: COLOR_GRAY_SURFACE,

            window_minimize_button_hover: COLOR_GRAY_DARK,
            window_maximize_button_hover: COLOR_GRAY_DARK,
            window_close_button_hover: COLOR_RED,
            window_close_button_foreground: white,

            tab_close_button_hover: COLOR_GRAY_DARK,
            tab_close_button_foreground: white,

            font_button_background: COLOR_GRAY_DARK,
            font_button_hover: COLOR_GRAY_SURFACE,
            reset_button_hover: COLOR_GRAY_SURFACE,

            modal_confirm_button_background: COLOR_GRAY_DARK,
            modal_confirm_button_hover: COLOR_GRAY_SURFACE,
            modal_cancel_button_background: COLOR_GRAY_DARK,
            modal_cancel_button_hover: COLOR_GRAY_SURFACE,
            modal_danger_button_background: COLOR_RED,
            modal_danger_button_hover: [0.85, 0.25, 0.20, 1.0],

            warning_background: rgb(4, 4, 4),
            warning_foreground: COLOR_AMBER,
            danger_foreground: COLOR_RED,
            success_foreground: COLOR_GREEN,

            background: black,
            sidebar_background: near_black,
            header_background: near_black,
            title_bar_background: black,

            session_background: near_black,
            session_selected_background: COLOR_GRAY_SURFACE,
            session_hover_background: COLOR_GRAY_DARK,

            tab_background: COLOR_GRAY_DARK,
            tab_selected_background: COLOR_GRAY_SURFACE,
            tab_hover_background: COLOR_GRAY_CONTROL,

            button_hover_background: COLOR_GRAY_DARK,
            button_active_background: COLOR_GRAY_SURFACE,
            close_button_hover_background: COLOR_GRAY_DARK,

            border: COLOR_GRAY_DARK,
            border_subtle: COLOR_GRAY_DARK,

            foreground: white,
            text_primary: white,
            text_secondary,
            text_muted,

            header_foreground: white,
            header_icon: white,
            header_button_hover_background: COLOR_GRAY_DARK,
            header_button_active_background: COLOR_GRAY_SURFACE,
            header_button_focus_ring: [0.10, 0.67, 0.85, 0.80],

            palette_background: COLOR_GRAY_ELEVATED,
            search_bar_background: COLOR_GRAY_DARK,
            search_bar_hover_background: COLOR_GRAY_CONTROL,

            modal_backdrop: [0.0, 0.0, 0.0, 0.70],
            modal_card_background: COLOR_GRAY_ELEVATED,
            modal_button_background: COLOR_GRAY_DARK,
            danger: COLOR_RED,
            drop_highlight: rgb(8, 8, 8),
            current_line_background: rgb(2, 2, 2),

            code_review_background: black,
            code_review_card_background: near_black,
            code_review_accordion_trigger_background: rgb(4, 4, 4),
            code_review_accordion_body_background: rgb(1, 1, 1),
            code_review_card_border: COLOR_GRAY_DARK,
            code_review_header_background: near_black,
            code_review_header_divider: COLOR_GRAY_DARK,
            code_review_sidebar_background: rgb(1, 1, 1),
            code_review_sidebar_border: COLOR_GRAY_DARK,
            code_review_sidebar_item_hover: COLOR_GRAY_DARK,
            code_review_sidebar_item_selected: COLOR_GRAY_SURFACE,

            code_review_header_button_background: COLOR_GRAY_DARK,
            code_review_header_button_hover: COLOR_GRAY_SURFACE,
            code_review_action_button_hover: COLOR_GRAY_SURFACE,
            code_review_commit_dialog_button: [0.11, 0.63, 0.35, 0.95],
            code_review_commit_dialog_button_hover: [0.14, 0.75, 0.42, 1.0],

            diff_add_green: [0.45, 0.78, 0.45, 0.95],
            diff_add_green_background: COLOR_DIFF_ADD_BG,
            diff_delete_red: [0.85, 0.30, 0.30, 0.95],
            diff_delete_red_background: COLOR_DIFF_DELETE_BG,
            diff_modified_yellow: [0.88, 0.68, 0.28, 0.95],
            diff_modified_yellow_background: COLOR_DIFF_MODIFIED_BG,
            diff_chip_delete_background: COLOR_DIFF_CHIP_DELETE_BG,
            diff_chip_delete_foreground: COLOR_DIFF_CHIP_DELETE_FG,
            diff_gutter_line_number: [0.45, 0.45, 0.50, 1.0],
            diff_gutter_add_badge_bg: [0.18, 0.42, 0.22, 0.45],
            diff_gutter_add_badge_fg: [0.60, 0.92, 0.60, 0.95],
            diff_gutter_mod_badge_bg: [0.42, 0.32, 0.12, 0.45],
            diff_gutter_mod_badge_fg: [0.95, 0.75, 0.30, 0.95],
            diff_gutter_del_badge_bg: [0.45, 0.15, 0.15, 0.45],
            diff_gutter_del_badge_fg: [0.95, 0.45, 0.45, 0.95],
            diff_gutter_badge_hover: [0.55, 0.55, 0.55, 0.35],
            diff_unmodified_banner_foreground: [0.50, 0.50, 0.55, 1.0],
            diff_unmodified_banner_divider: COLOR_GRAY_DARK,
            diff_stats_pill_background: rgb(6, 6, 6),

            accent: COLOR_BLUE,
            accent_subtle: [0.10, 0.67, 0.85, 0.60],
            accent_hover: [0.20, 0.77, 0.95, 1.0],

            close_button_foreground: white,

            tooltip_background: COLOR_GRAY_ELEVATED,
            tooltip_border: COLOR_GRAY_DARK,
            tooltip_foreground: [0.95, 0.95, 0.95, 1.0],
            tooltip_shortcut_foreground: COLOR_GRAY_MUTED,

            cursor: COLOR_BLUE,
            cursor_foreground: black,
            selection_background: rgb(8, 8, 8),
            hover_background: COLOR_GRAY_DARK,

            scrollbar_background: rgb(4, 4, 4),
            scrollbar_track: [0.0, 0.0, 0.0, 0.0],
            scrollbar_thumb: [0.25, 0.25, 0.27, 0.85],

            ansi: [
                rgb(43, 43, 43),
                rgb(213, 63, 61),
                rgb(91, 189, 78),
                rgb(220, 173, 60),
                rgb(40, 110, 230),
                rgb(196, 96, 209),
                rgb(38, 178, 178),
                rgb(200, 200, 205),
                rgb(108, 112, 122),
                rgb(232, 84, 74),
                rgb(110, 208, 92),
                rgb(233, 190, 90),
                rgb(58, 138, 247),
                rgb(214, 128, 226),
                rgb(58, 196, 196),
                rgb(255, 255, 255),
            ],
        }
    }

    pub fn from_name(_name: &str) -> Self {
        Self::monochrome_black()
    }

    pub fn clear_color_linear(&self) -> (f64, f64, f64, f64) {
        let [r, g, b, a] = self.background;

        (
            srgb_to_linear(r) as f64,
            srgb_to_linear(g) as f64,
            srgb_to_linear(b) as f64,
            a as f64,
        )
    }

    pub fn ansi_256_rgba(&self, idx: u8) -> [f32; 4] {
        match idx {
            0..=15 => self.ansi[idx as usize],

            16..=231 => {
                let i = idx - 16;

                let level = |c: u8| -> u8 { if c == 0 { 0 } else { 55 + c * 40 } };

                rgb(level(i / 36), level((i % 36) / 6), level(i % 6))
            }

            232..=255 => {
                let value = 8 + (idx - 232) * 10;
                rgb(value, value, value)
            }
        }
    }

    pub fn resolve_fg(&self, color: Color, bold: bool) -> [f32; 4] {
        match color {
            Color::Default => self.foreground,

            Color::Indexed(idx) => {
                if bold && idx < 8 {
                    self.ansi[idx as usize + 8]
                } else {
                    self.ansi_256_rgba(idx)
                }
            }

            Color::Rgb(r, g, b) => rgb(r, g, b),
        }
    }

    pub fn resolve_bg(&self, color: Color) -> Option<[f32; 4]> {
        match color {
            Color::Default => None,
            Color::Indexed(idx) => Some(self.ansi_256_rgba(idx)),
            Color::Rgb(r, g, b) => Some(rgb(r, g, b)),
        }
    }

    pub fn color_for_panel(&self, panel: PanelName) -> [f32; 4] {
        match panel {
            PanelName::Terminal => self.terminal_panel_background,
            PanelName::Sidebar => self.sidebar_panel_background,
            PanelName::FileExplorer => self.file_explorer_panel_background,
            PanelName::CodeReview => self.code_review_panel_background,
            PanelName::Settings => self.settings_panel_background,
            PanelName::Editor => self.editor_panel_background,
            PanelName::Header => self.header_panel_background,
            PanelName::QuickSearch => self.palette_background,
            PanelName::ModalDialog => self.modal_dialog_background,
        }
    }

    pub fn color_for_button(&self, button: ButtonName, state: ButtonState) -> [f32; 4] {
        match state {
            ButtonState::Normal => match button {
                ButtonName::CodeReview => self.code_review_button_background,
                _ => self.header_background,
            },
            ButtonState::Hovered => match button {
                ButtonName::SidebarToggle => self.sidebar_toggle_button_hover,
                ButtonName::ExplorerToggle => self.explorer_toggle_button_hover,
                ButtonName::CodeReview => self.code_review_button_hover,
                ButtonName::NewTab => self.new_tab_button_hover,
                ButtonName::Settings => self.settings_button_hover,
                ButtonName::WindowMinimize => self.window_minimize_button_hover,
                ButtonName::WindowMaximize => self.window_maximize_button_hover,
                ButtonName::WindowClose => self.window_close_button_hover,
                ButtonName::TabClose => self.tab_close_button_hover,
                ButtonName::FontIncrease | ButtonName::FontDecrease => self.font_button_hover,
                ButtonName::FontReset => self.reset_button_hover,
                ButtonName::ModalSave => self.modal_confirm_button_hover,
                ButtonName::ModalDontSave => self.modal_danger_button_hover,
                ButtonName::ModalCancel => self.modal_cancel_button_hover,
            },
            ButtonState::Active => match button {
                ButtonName::SidebarToggle => self.sidebar_toggle_button_active,
                ButtonName::ExplorerToggle => self.explorer_toggle_button_active,
                ButtonName::CodeReview => self.code_review_button_active,
                ButtonName::NewTab => self.new_tab_button_active,
                ButtonName::Settings => self.settings_button_active,
                _ => self.button_active_background,
            },
        }
    }
}
