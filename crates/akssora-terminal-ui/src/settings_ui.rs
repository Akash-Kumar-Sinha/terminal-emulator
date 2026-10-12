use akssora_terminal_config::{Action, TabLayoutPreference};

// INFO: Category tabs for the Settings view.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SettingsTab {
    General,
    Appearance,
    Editor,
    Keybindings,
}

impl SettingsTab {
    pub const ALL: &'static [SettingsTab] = &[
        SettingsTab::General,
        SettingsTab::Appearance,
        SettingsTab::Editor,
        SettingsTab::Keybindings,
    ];

    pub fn label(&self) -> &'static str {
        match self {
            Self::General => "General",
            Self::Appearance => "Appearance",
            Self::Editor => "Editor",
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
    // Appearance settings
    SelectThemeMonochrome,
    SelectThemeSlate,
    ToggleCursorBlink,
    CycleCursorStyle,
    // Editor settings
    ToggleLineNumbers,
    CycleTabSize,
    // Keybindings settings
    KeybindingSearchClick,
    KeybindingResetAll,
    KeybindingRebind(Action),
}

// INFO: Settings interface layout calculator and hit-test helper.
// Every position is computed from `cell_h` so the layout scales with the font size.
#[derive(Clone, Copy, Debug, Default)]
pub struct SettingsView;

impl SettingsView {
    // ── Category Tab Bar Geometry ───────────────────────────────────────────

    pub fn tab_button_rect(
        tab_idx: usize,
        workspace_x: f32,
        workspace_y: f32,
        cell_h: f32,
    ) -> (f32, f32, f32, f32) {
        let x_start = workspace_x + 40.0;
        let y = workspace_y + cell_h * 2.0 + 38.0;
        let tab_w = 110.0;
        let tab_h = (cell_h + 8.0).max(28.0);
        let x = x_start + tab_idx as f32 * (tab_w + 8.0);
        (x, y, tab_w, tab_h)
    }

    pub fn tab_content_start_y(workspace_y: f32, cell_h: f32) -> f32 {
        let (_, y, _, h) = Self::tab_button_rect(0, 0.0, workspace_y, cell_h);
        y + h + 16.0
    }

    // ── Dropdown geometry ─────────────────────────────────────────────────────

    /// Height of the collapsed dropdown trigger row — 2× cell lines + padding.
    pub fn dropdown_height(cell_h: f32) -> f32 {
        (cell_h + 14.0).max(30.0)
    }

    /// Height of each option row inside the open list.
    pub fn dropdown_item_height(cell_h: f32) -> f32 {
        (cell_h + 10.0).max(26.0)
    }

    /// Number of layout options.
    pub const LAYOUT_OPTION_COUNT: usize = 2;

    // ── Whole-panel layout: where each section starts (relative to workspace_y) ──

    /// Y offset (from workspace_y) where the settings header block ends and
    /// the "Tab Layout" label begins.
    pub fn header_height(cell_h: f32) -> f32 {
        Self::tab_content_start_y(0.0, cell_h)
    }

    /// Bounding rect of the collapsed dropdown trigger.
    pub fn layout_dropdown_rect(
        workspace_x: f32,
        workspace_y: f32,
        workspace_w: f32,
        cell_h: f32,
    ) -> (f32, f32, f32, f32) {
        let w = (workspace_w - 80.0).clamp(220.0, 520.0);
        let x = workspace_x + 40.0;
        let y = workspace_y + Self::header_height(cell_h) + cell_h + 6.0;
        (x, y, w, Self::dropdown_height(cell_h))
    }

    /// Bounding rect for item `idx` inside the open dropdown list.
    pub fn layout_dropdown_item_rect(
        idx: usize,
        workspace_x: f32,
        workspace_y: f32,
        workspace_w: f32,
        cell_h: f32,
    ) -> (f32, f32, f32, f32) {
        let (dx, dy, dw, dh) =
            Self::layout_dropdown_rect(workspace_x, workspace_y, workspace_w, cell_h);
        let item_y = dy + dh + idx as f32 * Self::dropdown_item_height(cell_h);
        (dx, item_y, dw, Self::dropdown_item_height(cell_h))
    }

    // ── Legacy alias ──────────────────────────────────────────────────────────
    pub fn option_rect(
        idx: usize,
        workspace_x: f32,
        workspace_y: f32,
        workspace_w: f32,
        cell_h: f32,
    ) -> (f32, f32, f32, f32) {
        Self::layout_dropdown_item_rect(idx, workspace_x, workspace_y, workspace_w, cell_h)
    }

    // ── Font section geometry ─────────────────────────────────────────────────

    /// Y of the "Terminal Font Size" section heading.
    pub fn font_section_y(workspace_y: f32, cell_h: f32) -> f32 {
        let (_, dy, _, dh) = Self::layout_dropdown_rect(0.0, workspace_y, 9999.0, cell_h);
        dy + dh + cell_h + 14.0
    }

    /// Button height — slightly taller than cell_h for comfortable clicking.
    pub fn btn_h(cell_h: f32) -> f32 {
        (cell_h + 10.0).max(28.0)
    }

    /// Y where all font control buttons start (below heading + hint).
    fn font_btn_y(workspace_y: f32, cell_h: f32) -> f32 {
        Self::font_section_y(workspace_y, cell_h) + cell_h * 2.0 + 8.0
    }

    pub fn font_minus_button_rect(
        workspace_x: f32,
        workspace_y: f32,
        _workspace_w: f32,
        cell_h: f32,
    ) -> (f32, f32, f32, f32) {
        let bh = Self::btn_h(cell_h);
        (
            workspace_x + 40.0,
            Self::font_btn_y(workspace_y, cell_h),
            bh,
            bh,
        )
    }

    pub fn font_display_rect(
        workspace_x: f32,
        workspace_y: f32,
        _workspace_w: f32,
        cell_h: f32,
    ) -> (f32, f32, f32, f32) {
        let bh = Self::btn_h(cell_h);
        (
            workspace_x + 40.0 + bh + 4.0,
            Self::font_btn_y(workspace_y, cell_h),
            bh * 2.2,
            bh,
        )
    }

    pub fn font_plus_button_rect(
        workspace_x: f32,
        workspace_y: f32,
        _workspace_w: f32,
        cell_h: f32,
    ) -> (f32, f32, f32, f32) {
        let bh = Self::btn_h(cell_h);
        let display_w = bh * 2.2;
        (
            workspace_x + 40.0 + bh + 4.0 + display_w + 4.0,
            Self::font_btn_y(workspace_y, cell_h),
            bh,
            bh,
        )
    }

    pub fn font_reset_button_rect(
        workspace_x: f32,
        workspace_y: f32,
        _workspace_w: f32,
        cell_h: f32,
    ) -> (f32, f32, f32, f32) {
        let bh = Self::btn_h(cell_h);
        let display_w = bh * 2.2;
        let reset_x = workspace_x + 40.0 + bh + 4.0 + display_w + 4.0 + bh + 8.0;
        (reset_x, Self::font_btn_y(workspace_y, cell_h), bh * 2.4, bh)
    }

    // ── Appearance section geometry ───────────────────────────────────────────

    pub fn theme_monochrome_rect(
        workspace_x: f32,
        workspace_y: f32,
        cell_h: f32,
    ) -> (f32, f32, f32, f32) {
        let bh = Self::btn_h(cell_h);
        let y = workspace_y + Self::header_height(cell_h) + cell_h + 8.0;
        (workspace_x + 40.0, y, 180.0, bh)
    }

    pub fn theme_slate_rect(
        workspace_x: f32,
        workspace_y: f32,
        cell_h: f32,
    ) -> (f32, f32, f32, f32) {
        let bh = Self::btn_h(cell_h);
        let y = workspace_y + Self::header_height(cell_h) + cell_h + 8.0;
        (workspace_x + 230.0, y, 180.0, bh)
    }

    pub fn cursor_style_rect(
        workspace_x: f32,
        workspace_y: f32,
        cell_h: f32,
    ) -> (f32, f32, f32, f32) {
        let bh = Self::btn_h(cell_h);
        let y = workspace_y + Self::header_height(cell_h) + cell_h * 3.0 + 26.0;
        (workspace_x + 40.0, y, 180.0, bh)
    }

    pub fn cursor_blink_rect(
        workspace_x: f32,
        workspace_y: f32,
        cell_h: f32,
    ) -> (f32, f32, f32, f32) {
        let bh = Self::btn_h(cell_h);
        let y = workspace_y + Self::header_height(cell_h) + cell_h * 3.0 + 26.0;
        (workspace_x + 230.0, y, 180.0, bh)
    }

    // ── Editor section geometry ───────────────────────────────────────────────

    pub fn tab_size_rect(workspace_x: f32, workspace_y: f32, cell_h: f32) -> (f32, f32, f32, f32) {
        let bh = Self::btn_h(cell_h);
        let y = workspace_y + Self::header_height(cell_h) + cell_h + 8.0;
        (workspace_x + 40.0, y, 200.0, bh)
    }

    pub fn line_numbers_rect(
        workspace_x: f32,
        workspace_y: f32,
        cell_h: f32,
    ) -> (f32, f32, f32, f32) {
        let bh = Self::btn_h(cell_h);
        let y = workspace_y + Self::header_height(cell_h) + cell_h * 3.0 + 26.0;
        (workspace_x + 40.0, y, 200.0, bh)
    }

    // ── Keyboard shortcuts section ────────────────────────────────────────────

    pub fn shortcuts_search_rect(
        workspace_x: f32,
        workspace_y: f32,
        workspace_w: f32,
        cell_h: f32,
    ) -> (f32, f32, f32, f32) {
        let y = workspace_y + Self::header_height(cell_h);
        let w = (workspace_w - 80.0).clamp(200.0, 480.0);
        let h = (cell_h + 10.0).max(28.0);
        (workspace_x + 40.0, y, w, h)
    }

    pub fn shortcuts_reset_rect(
        workspace_x: f32,
        workspace_y: f32,
        workspace_w: f32,
        cell_h: f32,
    ) -> (f32, f32, f32, f32) {
        let (sx, sy, sw, sh) =
            Self::shortcuts_search_rect(workspace_x, workspace_y, workspace_w, cell_h);
        (sx + sw + 12.0, sy, 110.0, sh)
    }

    pub fn shortcuts_row_y(workspace_y: f32, cell_h: f32, row_idx: usize) -> f32 {
        let start_y = workspace_y + Self::header_height(cell_h) + cell_h + 30.0;
        start_y + row_idx as f32 * (cell_h + 8.0)
    }

    pub fn shortcuts_section_y(workspace_y: f32, cell_h: f32) -> f32 {
        workspace_y + Self::header_height(cell_h)
    }

    // ── Hit testing ──────────────────────────────────────────────────────────

    pub fn hit_test_option(
        mx: f32,
        my: f32,
        workspace_x: f32,
        workspace_y: f32,
        workspace_w: f32,
        cell_h: f32,
    ) -> Option<TabLayoutPreference> {
        let options = [TabLayoutPreference::Sidebar, TabLayoutPreference::TitleBar];
        for (i, opt) in options.into_iter().enumerate() {
            let (ox, oy, ow, oh) =
                Self::layout_dropdown_item_rect(i, workspace_x, workspace_y, workspace_w, cell_h);
            if mx >= ox && mx < ox + ow && my >= oy && my < oy + oh {
                return Some(opt);
            }
        }
        None
    }

    pub fn hit_test_action(
        mx: f32,
        my: f32,
        workspace_x: f32,
        workspace_y: f32,
        workspace_w: f32,
        cell_h: f32,
    ) -> Option<SettingsAction> {
        Self::hit_test_action_tab(
            mx,
            my,
            workspace_x,
            workspace_y,
            workspace_w,
            cell_h,
            SettingsTab::General,
            &[],
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn hit_test_action_tab(
        mx: f32,
        my: f32,
        workspace_x: f32,
        workspace_y: f32,
        workspace_w: f32,
        cell_h: f32,
        current_tab: SettingsTab,
        visible_actions: &[Action],
    ) -> Option<SettingsAction> {
        // 1. Category Tab Bar Buttons
        for (idx, &tab) in SettingsTab::ALL.iter().enumerate() {
            let (tx, ty, tw, th) = Self::tab_button_rect(idx, workspace_x, workspace_y, cell_h);
            if mx >= tx && mx < tx + tw && my >= ty && my < ty + th {
                return Some(SettingsAction::SelectTab(tab));
            }
        }

        // 2. Tab-specific hit testing
        match current_tab {
            SettingsTab::General => {
                // Dropdown trigger
                let (ddx, ddy, ddw, ddh) =
                    Self::layout_dropdown_rect(workspace_x, workspace_y, workspace_w, cell_h);
                if mx >= ddx && mx < ddx + ddw && my >= ddy && my < ddy + ddh {
                    return Some(SettingsAction::ToggleLayoutDropdown);
                }

                // Dropdown items
                if let Some(opt) =
                    Self::hit_test_option(mx, my, workspace_x, workspace_y, workspace_w, cell_h)
                {
                    return Some(SettingsAction::SelectLayout(opt));
                }

                // Font size controls
                let (mx1, my1, mw1, mh1) =
                    Self::font_minus_button_rect(workspace_x, workspace_y, workspace_w, cell_h);
                if mx >= mx1 && mx < mx1 + mw1 && my >= my1 && my < my1 + mh1 {
                    return Some(SettingsAction::DecreaseFontSize);
                }

                let (px1, py1, pw1, ph1) =
                    Self::font_plus_button_rect(workspace_x, workspace_y, workspace_w, cell_h);
                if mx >= px1 && mx < px1 + pw1 && my >= py1 && my < py1 + ph1 {
                    return Some(SettingsAction::IncreaseFontSize);
                }

                let (rx1, ry1, rw1, rh1) =
                    Self::font_reset_button_rect(workspace_x, workspace_y, workspace_w, cell_h);
                if mx >= rx1 && mx < rx1 + rw1 && my >= ry1 && my < ry1 + rh1 {
                    return Some(SettingsAction::ResetFontSize);
                }
            }

            SettingsTab::Appearance => {
                let (bx, by, bw, bh) =
                    Self::theme_monochrome_rect(workspace_x, workspace_y, cell_h);
                if mx >= bx && mx < bx + bw && my >= by && my < by + bh {
                    return Some(SettingsAction::SelectThemeMonochrome);
                }

                let (sx, sy, sw, sh) = Self::theme_slate_rect(workspace_x, workspace_y, cell_h);
                if mx >= sx && mx < sx + sw && my >= sy && my < sy + sh {
                    return Some(SettingsAction::SelectThemeSlate);
                }

                let (cx, cy, cw, ch) = Self::cursor_style_rect(workspace_x, workspace_y, cell_h);
                if mx >= cx && mx < cx + cw && my >= cy && my < cy + ch {
                    return Some(SettingsAction::CycleCursorStyle);
                }

                let (kx, ky, kw, kh) = Self::cursor_blink_rect(workspace_x, workspace_y, cell_h);
                if mx >= kx && mx < kx + kw && my >= ky && my < ky + kh {
                    return Some(SettingsAction::ToggleCursorBlink);
                }
            }

            SettingsTab::Editor => {
                let (tx, ty, tw, th) = Self::tab_size_rect(workspace_x, workspace_y, cell_h);
                if mx >= tx && mx < tx + tw && my >= ty && my < ty + th {
                    return Some(SettingsAction::CycleTabSize);
                }

                let (lx, ly, lw, lh) = Self::line_numbers_rect(workspace_x, workspace_y, cell_h);
                if mx >= lx && mx < lx + lw && my >= ly && my < ly + lh {
                    return Some(SettingsAction::ToggleLineNumbers);
                }
            }

            SettingsTab::Keybindings => {
                let (sx, sy, sw, sh) =
                    Self::shortcuts_search_rect(workspace_x, workspace_y, workspace_w, cell_h);
                if mx >= sx && mx < sx + sw && my >= sy && my < sy + sh {
                    return Some(SettingsAction::KeybindingSearchClick);
                }

                let (rx, ry, rw, rh) =
                    Self::shortcuts_reset_rect(workspace_x, workspace_y, workspace_w, cell_h);
                if mx >= rx && mx < rx + rw && my >= ry && my < ry + rh {
                    return Some(SettingsAction::KeybindingResetAll);
                }

                for (idx, &act) in visible_actions.iter().enumerate() {
                    let ry = Self::shortcuts_row_y(workspace_y, cell_h, idx);
                    let rh = cell_h + 6.0;
                    if my >= ry
                        && my < ry + rh
                        && mx >= workspace_x + 40.0
                        && mx < workspace_x + 600.0
                    {
                        return Some(SettingsAction::KeybindingRebind(act));
                    }
                }
            }
        }

        None
    }
}
