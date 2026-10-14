use super::types::{SettingsAction, SettingsTab};
use akssora_terminal_config::{Action, LineNumberMode, TabLayoutPreference};

pub struct SettingsView;

impl SettingsView {
    pub const NAV_WIDTH: f32 = 130.0;
    pub const CONTENT_OFFSET_X: f32 = 170.0;

    // ── Category Tab Bar Geometry (Vertical on the left) ─────────────────────────

    pub fn tab_button_rect(
        tab_idx: usize,
        workspace_x: f32,
        workspace_y: f32,
        cell_h: f32,
    ) -> (f32, f32, f32, f32) {
        let x = workspace_x + 30.0;
        let tab_w = Self::NAV_WIDTH;
        let tab_h = (cell_h + 10.0).max(30.0);
        let start_y = workspace_y + cell_h * 2.0 + 38.0;
        let y = start_y + tab_idx as f32 * (tab_h + 6.0);
        (x, y, tab_w, tab_h)
    }

    pub fn tab_content_start_y(workspace_y: f32, cell_h: f32) -> f32 {
        workspace_y + cell_h * 2.0 + 38.0
    }

    pub fn content_x(workspace_x: f32) -> f32 {
        workspace_x + Self::CONTENT_OFFSET_X
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

    /// Y offset (from workspace_y) where the settings content sections start.
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
        let max_w = (workspace_w - Self::CONTENT_OFFSET_X - 40.0).max(180.0);
        let w = max_w.clamp(200.0, 480.0);
        let x = Self::content_x(workspace_x);
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
            Self::content_x(workspace_x),
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
            Self::content_x(workspace_x) + bh + 4.0,
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
            Self::content_x(workspace_x) + bh + 4.0 + display_w + 4.0,
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
        let reset_x = Self::content_x(workspace_x) + bh + 4.0 + display_w + 4.0 + bh + 8.0;
        (reset_x, Self::font_btn_y(workspace_y, cell_h), bh * 2.4, bh)
    }

    // ── UI Font Size section geometry ─────────────────────────────────────────

    pub fn ui_font_section_y(workspace_y: f32, cell_h: f32) -> f32 {
        Self::font_btn_y(workspace_y, cell_h) + Self::btn_h(cell_h) + cell_h + 10.0
    }

    fn ui_font_btn_y(workspace_y: f32, cell_h: f32) -> f32 {
        Self::ui_font_section_y(workspace_y, cell_h) + cell_h * 2.0 + 8.0
    }

    pub fn ui_font_minus_button_rect(
        workspace_x: f32,
        workspace_y: f32,
        _workspace_w: f32,
        cell_h: f32,
    ) -> (f32, f32, f32, f32) {
        let bh = Self::btn_h(cell_h);
        (
            Self::content_x(workspace_x),
            Self::ui_font_btn_y(workspace_y, cell_h),
            bh,
            bh,
        )
    }

    pub fn ui_font_display_rect(
        workspace_x: f32,
        workspace_y: f32,
        _workspace_w: f32,
        cell_h: f32,
    ) -> (f32, f32, f32, f32) {
        let bh = Self::btn_h(cell_h);
        (
            Self::content_x(workspace_x) + bh + 4.0,
            Self::ui_font_btn_y(workspace_y, cell_h),
            bh * 2.2,
            bh,
        )
    }

    pub fn ui_font_plus_button_rect(
        workspace_x: f32,
        workspace_y: f32,
        _workspace_w: f32,
        cell_h: f32,
    ) -> (f32, f32, f32, f32) {
        let bh = Self::btn_h(cell_h);
        let display_w = bh * 2.2;
        (
            Self::content_x(workspace_x) + bh + 4.0 + display_w + 4.0,
            Self::ui_font_btn_y(workspace_y, cell_h),
            bh,
            bh,
        )
    }

    pub fn ui_font_reset_button_rect(
        workspace_x: f32,
        workspace_y: f32,
        _workspace_w: f32,
        cell_h: f32,
    ) -> (f32, f32, f32, f32) {
        let bh = Self::btn_h(cell_h);
        let display_w = bh * 2.2;
        let reset_x = Self::content_x(workspace_x) + bh + 4.0 + display_w + 4.0 + bh + 8.0;
        (
            reset_x,
            Self::ui_font_btn_y(workspace_y, cell_h),
            bh * 2.4,
            bh,
        )
    }

    // ── Appearance section geometry ───────────────────────────────────────────

    pub fn cursor_style_rect(
        workspace_x: f32,
        workspace_y: f32,
        cell_h: f32,
    ) -> (f32, f32, f32, f32) {
        let bh = Self::btn_h(cell_h);
        let y = workspace_y + Self::header_height(cell_h) + cell_h + 8.0;
        (Self::content_x(workspace_x), y, 180.0, bh)
    }

    pub fn cursor_blink_rect(
        workspace_x: f32,
        workspace_y: f32,
        cell_h: f32,
    ) -> (f32, f32, f32, f32) {
        let bh = Self::btn_h(cell_h);
        let y = workspace_y + Self::header_height(cell_h) + cell_h + 8.0;
        (Self::content_x(workspace_x) + 190.0, y, 180.0, bh)
    }

    // ── Editor section geometry ───────────────────────────────────────────────

    pub fn tab_size_rect(workspace_x: f32, workspace_y: f32, cell_h: f32) -> (f32, f32, f32, f32) {
        let bh = Self::btn_h(cell_h);
        let y = workspace_y + Self::header_height(cell_h) + cell_h + 8.0;
        (Self::content_x(workspace_x), y, 200.0, bh)
    }

    // INFO: Bounding box for the Editor Line Height setting button.
    pub fn line_height_rect(
        workspace_x: f32,
        workspace_y: f32,
        cell_h: f32,
    ) -> (f32, f32, f32, f32) {
        let bh = Self::btn_h(cell_h);
        let y = workspace_y + Self::header_height(cell_h) + cell_h + 8.0;
        (Self::content_x(workspace_x) + 210.0, y, 220.0, bh)
    }

    pub const LINE_NUMBER_DROPDOWN_WIDTH: f32 = 150.0;

    pub fn line_number_dropdown_rect(
        workspace_x: f32,
        workspace_y: f32,
        cell_h: f32,
    ) -> (f32, f32, f32, f32) {
        let bh = Self::btn_h(cell_h);
        let y = workspace_y + Self::header_height(cell_h) + cell_h * 3.0 + 26.0;
        (
            Self::content_x(workspace_x),
            y,
            Self::LINE_NUMBER_DROPDOWN_WIDTH,
            bh,
        )
    }

    pub fn vim_mode_rect(workspace_x: f32, workspace_y: f32, cell_h: f32) -> (f32, f32, f32, f32) {
        let bh = Self::btn_h(cell_h);
        let y = workspace_y + Self::header_height(cell_h) + cell_h * 5.0 + 36.0;
        (Self::content_x(workspace_x), y, 220.0, bh)
    }

    pub fn vim_clipboard_rect(
        workspace_x: f32,
        workspace_y: f32,
        cell_h: f32,
    ) -> (f32, f32, f32, f32) {
        let bh = Self::btn_h(cell_h);
        let y = workspace_y + Self::header_height(cell_h) + cell_h * 5.0 + 36.0;
        (Self::content_x(workspace_x) + 230.0, y, 260.0, bh)
    }

    pub fn vim_status_bar_rect(
        workspace_x: f32,
        workspace_y: f32,
        cell_h: f32,
    ) -> (f32, f32, f32, f32) {
        let bh = Self::btn_h(cell_h);
        let y = workspace_y + Self::header_height(cell_h) + cell_h * 7.0 + 46.0;
        (Self::content_x(workspace_x), y, 220.0, bh)
    }

    pub fn autocomplete_symbols_rect(
        workspace_x: f32,
        workspace_y: f32,
        cell_h: f32,
    ) -> (f32, f32, f32, f32) {
        let bh = Self::btn_h(cell_h);
        let y = workspace_y + Self::header_height(cell_h) + cell_h * 7.0 + 46.0;
        (Self::content_x(workspace_x) + 230.0, y, 260.0, bh)
    }

    pub fn format_on_save_rect(
        workspace_x: f32,
        workspace_y: f32,
        cell_h: f32,
    ) -> (f32, f32, f32, f32) {
        let bh = Self::btn_h(cell_h);
        let y = workspace_y + Self::header_height(cell_h) + cell_h * 9.0 + 56.0;
        (Self::content_x(workspace_x), y, 220.0, bh)
    }

    pub fn line_wrap_rect(workspace_x: f32, workspace_y: f32, cell_h: f32) -> (f32, f32, f32, f32) {
        let bh = Self::btn_h(cell_h);
        let y = workspace_y + Self::header_height(cell_h) + cell_h * 9.0 + 56.0;
        (Self::content_x(workspace_x) + 230.0, y, 260.0, bh)
    }

    pub fn auto_save_rect(workspace_x: f32, workspace_y: f32, cell_h: f32) -> (f32, f32, f32, f32) {
        let bh = Self::btn_h(cell_h);
        let y = workspace_y + Self::header_height(cell_h) + cell_h * 11.0 + 66.0;
        (Self::content_x(workspace_x), y, 220.0, bh)
    }

    // ── Code Review section geometry ──────────────────────────────────────────

    pub fn code_review_diff_mode_rect(
        workspace_x: f32,
        workspace_y: f32,
        cell_h: f32,
    ) -> (f32, f32, f32, f32) {
        let bh = Self::btn_h(cell_h);
        let y = workspace_y + Self::header_height(cell_h) + cell_h + 8.0;
        (Self::content_x(workspace_x), y, 240.0, bh)
    }

    pub fn code_review_find_bar_rect(
        workspace_x: f32,
        workspace_y: f32,
        cell_h: f32,
    ) -> (f32, f32, f32, f32) {
        let bh = Self::btn_h(cell_h);
        let y = workspace_y + Self::header_height(cell_h) + cell_h * 3.0 + 26.0;
        (Self::content_x(workspace_x), y, 240.0, bh)
    }

    pub fn line_number_dropdown_item_rect(
        idx: usize,
        workspace_x: f32,
        workspace_y: f32,
        cell_h: f32,
    ) -> (f32, f32, f32, f32) {
        let (lx, ly, lw, lh) = Self::line_number_dropdown_rect(workspace_x, workspace_y, cell_h);
        let item_y = ly + lh + idx as f32 * Self::dropdown_item_height(cell_h);
        (lx, item_y, lw, Self::dropdown_item_height(cell_h))
    }

    // ── Keyboard shortcuts section ────────────────────────────────────

    pub fn shortcuts_search_rect(
        workspace_x: f32,
        workspace_y: f32,
        workspace_w: f32,
        cell_h: f32,
    ) -> (f32, f32, f32, f32) {
        let y = workspace_y + Self::header_height(cell_h);
        let max_w = (workspace_w - Self::CONTENT_OFFSET_X - 160.0).max(180.0);
        let w = max_w.clamp(200.0, 480.0);
        let h = (cell_h + 10.0).max(28.0);
        (Self::content_x(workspace_x), y, w, h)
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
            false,
            false,
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
        layout_dropdown_open: bool,
        line_number_dropdown_open: bool,
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
                // Dropdown items (only interactive when open and sitting on top)
                if layout_dropdown_open
                    && let Some(opt) =
                        Self::hit_test_option(mx, my, workspace_x, workspace_y, workspace_w, cell_h)
                {
                    return Some(SettingsAction::SelectLayout(opt));
                }

                // Dropdown trigger
                let (ddx, ddy, ddw, ddh) =
                    Self::layout_dropdown_rect(workspace_x, workspace_y, workspace_w, cell_h);
                if mx >= ddx && mx < ddx + ddw && my >= ddy && my < ddy + ddh {
                    return Some(SettingsAction::ToggleLayoutDropdown);
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

                // UI font size controls
                let (umx, umy, umw, umh) =
                    Self::ui_font_minus_button_rect(workspace_x, workspace_y, workspace_w, cell_h);
                if mx >= umx && mx < umx + umw && my >= umy && my < umy + umh {
                    return Some(SettingsAction::DecreaseUiFontSize);
                }

                let (upx, upy, upw, uph) =
                    Self::ui_font_plus_button_rect(workspace_x, workspace_y, workspace_w, cell_h);
                if mx >= upx && mx < upx + upw && my >= upy && my < upy + uph {
                    return Some(SettingsAction::IncreaseUiFontSize);
                }

                let (urx, ury, urw, urh) =
                    Self::ui_font_reset_button_rect(workspace_x, workspace_y, workspace_w, cell_h);
                if mx >= urx && mx < urx + urw && my >= ury && my < ury + urh {
                    return Some(SettingsAction::ResetUiFontSize);
                }
            }

            SettingsTab::Appearance => {
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
                // Dropdown items (only interactive when open and sitting on top)
                if line_number_dropdown_open {
                    for (idx, &mode) in LineNumberMode::ALL.iter().enumerate() {
                        let (ix, iy, iw, ih) = Self::line_number_dropdown_item_rect(
                            idx,
                            workspace_x,
                            workspace_y,
                            cell_h,
                        );
                        if mx >= ix && mx < ix + iw && my >= iy && my < iy + ih {
                            return Some(SettingsAction::SelectLineNumberMode(mode));
                        }
                    }
                }

                // Dropdown trigger
                let (lx, ly, lw, lh) =
                    Self::line_number_dropdown_rect(workspace_x, workspace_y, cell_h);
                if mx >= lx && mx < lx + lw && my >= ly && my < ly + lh {
                    return Some(SettingsAction::ToggleLineNumberDropdown);
                }

                let (tx, ty, tw, th) = Self::tab_size_rect(workspace_x, workspace_y, cell_h);
                if mx >= tx && mx < tx + tw && my >= ty && my < ty + th {
                    return Some(SettingsAction::CycleTabSize);
                }

                let (lh_x, lh_y, lh_w, lh_h) =
                    Self::line_height_rect(workspace_x, workspace_y, cell_h);
                if mx >= lh_x && mx < lh_x + lh_w && my >= lh_y && my < lh_y + lh_h {
                    return Some(SettingsAction::CycleLineHeight);
                }

                let (vx, vy, vw, vh) = Self::vim_mode_rect(workspace_x, workspace_y, cell_h);
                if mx >= vx && mx < vx + vw && my >= vy && my < vy + vh {
                    return Some(SettingsAction::ToggleVimMode);
                }

                let (vc_x, vc_y, vc_w, vc_h) =
                    Self::vim_clipboard_rect(workspace_x, workspace_y, cell_h);
                if mx >= vc_x && mx < vc_x + vc_w && my >= vc_y && my < vc_y + vc_h {
                    return Some(SettingsAction::ToggleVimClipboard);
                }

                let (vs_x, vs_y, vs_w, vs_h) =
                    Self::vim_status_bar_rect(workspace_x, workspace_y, cell_h);
                if mx >= vs_x && mx < vs_x + vs_w && my >= vs_y && my < vs_y + vs_h {
                    return Some(SettingsAction::ToggleVimStatusBar);
                }

                let (ac_x, ac_y, ac_w, ac_h) =
                    Self::autocomplete_symbols_rect(workspace_x, workspace_y, cell_h);
                if mx >= ac_x && mx < ac_x + ac_w && my >= ac_y && my < ac_y + ac_h {
                    return Some(SettingsAction::ToggleAutocompleteSymbols);
                }

                let (fs_x, fs_y, fs_w, fs_h) =
                    Self::format_on_save_rect(workspace_x, workspace_y, cell_h);
                if mx >= fs_x && mx < fs_x + fs_w && my >= fs_y && my < fs_y + fs_h {
                    return Some(SettingsAction::ToggleFormatOnSave);
                }

                let (lw_x, lw_y, lw_w, lw_h) =
                    Self::line_wrap_rect(workspace_x, workspace_y, cell_h);
                if mx >= lw_x && mx < lw_x + lw_w && my >= lw_y && my < lw_y + lw_h {
                    return Some(SettingsAction::ToggleLineWrap);
                }

                let (as_x, as_y, as_w, as_h) =
                    Self::auto_save_rect(workspace_x, workspace_y, cell_h);
                if mx >= as_x && mx < as_x + as_w && my >= as_y && my < as_y + as_h {
                    return Some(SettingsAction::ToggleAutoSave);
                }
            }

            SettingsTab::CodeReview => {
                let (dx, dy, dw, dh) =
                    Self::code_review_diff_mode_rect(workspace_x, workspace_y, cell_h);
                if mx >= dx && mx < dx + dw && my >= dy && my < dy + dh {
                    return Some(SettingsAction::ToggleCodeReviewDiffMode);
                }

                let (fx, fy, fw, fh) =
                    Self::code_review_find_bar_rect(workspace_x, workspace_y, cell_h);
                if mx >= fx && mx < fx + fw && my >= fy && my < fy + fh {
                    return Some(SettingsAction::ToggleCodeReviewFindBar);
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
