use akssora_terminal_config::Theme;

mod hit_test;
mod tab_layout;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum HeaderButton {
    Minimize,
    Maximize,
    Close,
}

pub const BUTTON_WIDTH: f32 = 40.0;
pub const SIDEBAR_TOGGLE_WIDTH: f32 = 36.0;
pub const TAB_MAX_WIDTH: f32 = 140.0;
pub const TAB_MIN_WIDTH: f32 = 80.0;
pub const TAB_HEIGHT: f32 = 26.0;
pub const TAB_BAR_HEIGHT: f32 = 32.0;

// INFO: Header bar element interaction targets.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum HeaderItem {
    SidebarToggle,
    WindowButton(HeaderButton),
    Settings,
    Tab(usize),
    TabClose(usize),
    NewTab,
}

// INFO: Custom frameless window header bar supporting window dragging, title rendering, tabs, and controls.
#[derive(Clone, Debug)]
pub struct Header {
    pub height: f32,
    pub bg_color: [f32; 4],
    pub title: String,
    pub title_color: [f32; 4],
    pub icon_color: [f32; 4],
    pub button_hover: [f32; 4],
    pub hovered: Option<HeaderButton>,
    pub hovered_item: Option<HeaderItem>,
    pub is_sidebar_expanded: bool,
    pub is_sidebar_toggle_focused: bool,
    pub is_sidebar_toggle_active: bool,
    pub is_settings_focused: bool,
    pub is_settings_active: bool,
    pub is_maximized: bool,
    pub focused_window_button: Option<HeaderButton>,
}

impl Default for Header {
    fn default() -> Self {
        Self::new("Akssora")
    }
}

impl Header {
    pub const DEFAULT_HEIGHT: f32 = 36.0;
    pub const TAB_BAR_HEIGHT: f32 = TAB_BAR_HEIGHT;

    pub fn new(title: impl Into<String>) -> Self {
        let theme = Theme::default();
        Self {
            height: Self::DEFAULT_HEIGHT,
            bg_color: theme.header_background,
            title: title.into(),
            title_color: theme.header_foreground,
            icon_color: theme.header_icon,
            button_hover: theme.button_hover_background,
            hovered: None,
            hovered_item: None,
            is_sidebar_expanded: true,
            is_sidebar_toggle_focused: false,
            is_sidebar_toggle_active: false,
            is_settings_focused: false,
            is_settings_active: false,
            is_maximized: false,
            focused_window_button: None,
        }
    }

    pub fn total_height(&self, show_tabs: bool) -> f32 {
        self.height + if show_tabs { Self::TAB_BAR_HEIGHT } else { 0.0 }
    }

    pub fn sidebar_toggle_rect(&self) -> (f32, f32, f32, f32) {
        (0.0, 0.0, SIDEBAR_TOGGLE_WIDTH, self.height)
    }

    // INFO: Visual bounding box for the sidebar toggle button aligned with header controls.
    pub fn sidebar_toggle_visual_rect(&self) -> (f32, f32, f32, f32) {
        let (stx, sty, stw, sth) = self.sidebar_toggle_rect();
        let btn_w = 28.0;
        let btn_h = 26.0;
        let bx = (stx + (stw - btn_w) * 0.5).round();
        let by = (sty + (sth - btn_h) * 0.5).round();
        (bx, by, btn_w, btn_h)
    }

    pub fn branding_rect(&self, show_toggle: bool) -> (f32, f32, f32, f32) {
        let x = if show_toggle {
            SIDEBAR_TOGGLE_WIDTH + 6.0
        } else {
            12.0
        };
        (x, 0.0, 68.0, self.height)
    }

    pub fn button_rect(&self, btn: HeaderButton, window_w: f32) -> (f32, f32, f32, f32) {
        let slot = match btn {
            HeaderButton::Close => 0.0,
            HeaderButton::Maximize => 1.0,
            HeaderButton::Minimize => 2.0,
        };
        let x = window_w - (slot + 1.0) * BUTTON_WIDTH;
        (x, 0.0, BUTTON_WIDTH, self.height)
    }

    // INFO: Visual bounding box for a window button hover/focus pill.
    pub fn button_visual_rect(&self, btn: HeaderButton, window_w: f32) -> (f32, f32, f32, f32) {
        let (bx, by, bw, bh) = self.button_rect(btn, window_w);
        let pill_w = 30.0;
        let pill_h = 22.0;
        let px = (bx + (bw - pill_w) * 0.5).round();
        let py = (by + (bh - pill_h) * 0.5).round();
        (px, py, pill_w, pill_h)
    }

    // INFO: Bounding box for the permanent settings gear button in the header bar.
    pub fn settings_button_rect(&self, window_w: f32) -> (f32, f32, f32, f32) {
        let btn_w = 28.0;
        let btn_h = 26.0;
        let spacing = 14.0;
        let x = (window_w - 3.0 * BUTTON_WIDTH - spacing - btn_w).round();
        let y = ((self.height - btn_h) * 0.5).round();
        (x, y, btn_w, btn_h)
    }
}
