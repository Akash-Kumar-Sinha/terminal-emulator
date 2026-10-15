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
pub const EXPLORER_TOGGLE_WIDTH: f32 = 36.0;
pub const QUICK_SEARCH_WIDTH: f32 = 280.0;
pub const QUICK_SEARCH_HEIGHT: f32 = 24.0;
pub const TAB_MAX_WIDTH: f32 = 140.0;
pub const TAB_MIN_WIDTH: f32 = 80.0;
pub const TAB_HEIGHT: f32 = 26.0;
pub const TAB_BAR_HEIGHT: f32 = 32.0;

// INFO: Header bar element interaction targets.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum HeaderItem {
    SidebarToggle,
    ExplorerToggle,
    CodeReview,
    BranchDropdown,
    BranchItem(usize),
    QuickSearch,
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
    pub is_explorer_expanded: bool,
    pub is_explorer_toggle_focused: bool,
    pub is_explorer_toggle_active: bool,
    pub is_code_review_focused: bool,
    pub is_code_review_active: bool,
    pub is_settings_focused: bool,
    pub is_settings_active: bool,
    pub is_maximized: bool,
    pub focused_window_button: Option<HeaderButton>,
    pub git_branch: Option<String>,
    pub git_branches: Vec<String>,
    pub branch_dropdown_open: bool,
    pub hovered_branch_idx: Option<usize>,
    pub is_git_repo: bool,
    pub git_additions: usize,
    pub git_deletions: usize,
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
            is_explorer_expanded: false,
            is_explorer_toggle_focused: false,
            is_explorer_toggle_active: false,
            is_code_review_focused: false,
            is_code_review_active: false,
            is_settings_focused: false,
            is_settings_active: false,
            is_maximized: false,
            focused_window_button: None,
            git_branch: None,
            git_branches: Vec::new(),
            branch_dropdown_open: false,
            hovered_branch_idx: None,
            is_git_repo: false,
            git_additions: 0,
            git_deletions: 0,
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

    pub fn explorer_toggle_rect(&self) -> (f32, f32, f32, f32) {
        (
            SIDEBAR_TOGGLE_WIDTH,
            0.0,
            EXPLORER_TOGGLE_WIDTH,
            self.height,
        )
    }

    pub fn explorer_toggle_visual_rect(&self) -> (f32, f32, f32, f32) {
        let (etx, ety, etw, eth) = self.explorer_toggle_rect();
        let btn_w = 28.0;
        let btn_h = 26.0;
        let bx = (etx + (etw - btn_w) * 0.5).round();
        let by = (ety + (eth - btn_h) * 0.5).round();
        (bx, by, btn_w, btn_h)
    }

    // INFO: Bounding box for the center quick search bar in the header.
    pub fn quick_search_rect(&self, window_w: f32) -> (f32, f32, f32, f32) {
        let (bx, _, bw, _) = self.branding_rect_ex(true, true);
        let left_limit = if self.is_git_repo {
            let (br_x, _, br_w, _) = self.branch_button_rect(true, true);
            if br_w > 0.0 {
                br_x + br_w + 10.0
            } else {
                bx + bw + 10.0
            }
        } else {
            bx + bw + 10.0
        };

        let right_limit = if self.is_git_repo {
            let (cr_x, _, _, _) = self.code_review_button_rect(window_w);
            if cr_x > 0.0 {
                cr_x - 10.0
            } else {
                self.settings_button_rect(window_w).0 - 10.0
            }
        } else {
            self.settings_button_rect(window_w).0 - 10.0
        };

        let max_w = (right_limit - left_limit).max(60.0);
        let sw = QUICK_SEARCH_WIDTH.min(max_w).round();
        let sx = ((window_w - sw) * 0.5)
            .clamp(left_limit, (right_limit - sw).max(left_limit))
            .round();
        let sy = ((self.height - QUICK_SEARCH_HEIGHT) * 0.5).max(0.0).round();
        (sx, sy, sw, QUICK_SEARCH_HEIGHT)
    }

    pub fn branding_rect(&self, show_toggle: bool) -> (f32, f32, f32, f32) {
        self.branding_rect_ex(show_toggle, true)
    }

    pub fn branding_rect_ex(
        &self,
        show_sidebar_toggle: bool,
        show_explorer_toggle: bool,
    ) -> (f32, f32, f32, f32) {
        let mut x = 12.0;
        if show_sidebar_toggle && show_explorer_toggle {
            x = SIDEBAR_TOGGLE_WIDTH + EXPLORER_TOGGLE_WIDTH + 6.0;
        } else if show_sidebar_toggle {
            x = SIDEBAR_TOGGLE_WIDTH + 6.0;
        } else if show_explorer_toggle {
            x = EXPLORER_TOGGLE_WIDTH + 6.0;
        }
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

    // INFO: Bounding box for the git branch button on the left side of the header bar beside branding.
    pub fn branch_button_rect(
        &self,
        show_sidebar_toggle: bool,
        show_explorer_toggle: bool,
    ) -> (f32, f32, f32, f32) {
        if !self.is_git_repo {
            return (0.0, 0.0, 0.0, 0.0);
        }

        let (bx, _, bw, _) = self.branding_rect_ex(show_sidebar_toggle, show_explorer_toggle);
        let start_x = bx + bw + 10.0;
        let btn_h = 24.0;
        let branch_name = self.git_branch.as_deref().unwrap_or("detached");
        let mut width = 14.0 + 6.0 + branch_name.len() as f32 * 8.0 + 12.0;
        if self.git_branches.len() > 1 {
            width += 14.0;
        }

        let y = ((self.height - btn_h) * 0.5).round();
        (start_x.round(), y, width.max(54.0).round(), btn_h)
    }

    // INFO: Bounding box for the branch dropdown popup list below the left branch button.
    pub fn branch_dropdown_rect(
        &self,
        show_sidebar_toggle: bool,
        show_explorer_toggle: bool,
    ) -> (f32, f32, f32, f32) {
        let (bx, by, bw, bh) = self.branch_button_rect(show_sidebar_toggle, show_explorer_toggle);
        let item_h = 26.0;
        let menu_w = (bw + 20.0).max(150.0);
        let menu_h = self.git_branches.len() as f32 * item_h + 8.0;
        let menu_x = bx;
        let menu_y = by + bh + 4.0;
        (menu_x, menu_y, menu_w, menu_h)
    }

    // INFO: Bounding box for an individual branch item in the dropdown popup list.
    pub fn branch_dropdown_item_rect(
        &self,
        idx: usize,
        show_sidebar_toggle: bool,
        show_explorer_toggle: bool,
    ) -> (f32, f32, f32, f32) {
        let (mx, my, mw, _) = self.branch_dropdown_rect(show_sidebar_toggle, show_explorer_toggle);
        let item_h = 26.0;
        let iy = my + 4.0 + idx as f32 * item_h;
        (mx + 4.0, iy, mw - 8.0, item_h)
    }

    // INFO: Bounding box for the Code Review stats button on the right side beside Settings button.
    pub fn code_review_button_rect_stats(
        &self,
        window_w: f32,
        additions: usize,
        deletions: usize,
    ) -> (f32, f32, f32, f32) {
        if !self.is_git_repo {
            return (0.0, 0.0, 0.0, 0.0);
        }

        let (sx, _, _, _) = self.settings_button_rect(window_w);
        let btn_h = 24.0;
        let spacing = 8.0;

        let mut w = 28.0; // Left padding + 14px icon + padding
        if additions > 0 || deletions > 0 {
            if additions > 0 {
                let add_len = format!("+{additions}").len();
                w += add_len as f32 * 9.5 + 4.0;
            }
            if additions > 0 && deletions > 0 {
                w += 14.0; // "·" separator + padding
            }
            if deletions > 0 {
                let del_len = format!("-{deletions}").len();
                w += del_len as f32 * 9.5 + 4.0;
            }
            w += 6.0;
        }

        let btn_w = w.max(34.0).round();
        let x = (sx - spacing - btn_w).round();
        let y = ((self.height - btn_h) * 0.5).round();
        (x, y, btn_w, btn_h)
    }

    // INFO: Bounding box for the Code Review panel button in the header bar (beside settings button).
    pub fn code_review_button_rect(&self, window_w: f32) -> (f32, f32, f32, f32) {
        self.code_review_button_rect_stats(window_w, self.git_additions, self.git_deletions)
    }
}
