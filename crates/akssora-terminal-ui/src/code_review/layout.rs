use akssora_git::FileDiff;

use super::model::CodeReviewModel;
use super::types::{CodeReviewAction, CodeReviewGutterHover, CodeReviewTreeItem};

// INFO: Bounding box and button geometry for the floating Code Review line action popup.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LineActionPopupGeometry {
    pub popup_rect: (f32, f32, f32, f32),
    pub add_btn_rect: (f32, f32, f32, f32),
    pub remove_btn_rect: (f32, f32, f32, f32),
    pub line_gutter_rect: (f32, f32, f32, f32),
}

impl LineActionPopupGeometry {
    pub fn contains_point(&self, x: f32, y: f32) -> bool {
        let (px, py, pw, ph) = self.popup_rect;
        let (gx, gy, gw, gh) = self.line_gutter_rect;

        // 1. Inside the popup card
        if x >= px && x <= px + pw && y >= py && y <= py + ph {
            return true;
        }

        // 2. Inside the hovered line's own gutter rect
        if x >= gx && x <= gx + gw && y >= gy && y <= gy + gh {
            return true;
        }

        // 3. Inside the bridge between popup bottom and line gutter top
        let bridge_top = (py + ph).min(gy);
        let bridge_bottom = (py + ph).max(gy);
        let bridge_left = px.min(gx);
        let bridge_right = (px + pw).max(gx + gw);
        if y >= bridge_top && y <= bridge_bottom && x >= bridge_left && x <= bridge_right {
            return true;
        }

        false
    }

    pub fn hit_test_button(&self, x: f32, y: f32) -> Option<akssora_editor::GutterActionKind> {
        let (ax, ay, aw, ah) = self.add_btn_rect;
        if x >= ax && x <= ax + aw && y >= ay && y <= ay + ah {
            return Some(akssora_editor::GutterActionKind::StageLine);
        }
        let (rx, ry, rw, rh) = self.remove_btn_rect;
        if x >= rx && x <= rx + rw && y >= ry && y <= ry + rh {
            return Some(akssora_editor::GutterActionKind::RevertLine);
        }
        None
    }
}

// INFO: Geometry calculator and hit-test locator for the Code Review user interface.
pub struct CodeReviewLayout;

impl CodeReviewLayout {
    pub const HEADER_HEIGHT: f32 = 42.0;
    pub const SIDEBAR_TOOLBAR_HEIGHT: f32 = 28.0;
    pub const FILE_ROW_HEIGHT: f32 = 28.0;
    pub const CARD_HEADER_HEIGHT: f32 = 36.0;
    pub const DIFF_LINE_HEIGHT: f32 = 20.0;
    pub const UNMODIFIED_LINES_HEIGHT: f32 = 24.0;
    pub const CARD_MARGIN_BOTTOM: f32 = 12.0;

    // INFO: Bounding box for the sidebar collapse/expand toggle button in the top header.
    pub fn sidebar_toggle_btn_rect(ws_x: f32, ws_y: f32) -> (f32, f32, f32, f32) {
        (ws_x + 12.0, ws_y + 8.0, 32.0, 26.0)
    }

    // INFO: Bounding box for the staged vs unstaged code toggle button in the top header (deprecated).
    pub fn stage_toggle_btn_rect(_ws_x: f32, _ws_y: f32) -> (f32, f32, f32, f32) {
        (0.0, 0.0, 0.0, 0.0)
    }

    // INFO: Bounding box for the toggle collapse/expand all button in the top header.
    pub fn toggle_collapse_all_btn_rect(ws_x: f32, ws_y: f32) -> (f32, f32, f32, f32) {
        (ws_x + 50.0, ws_y + 8.0, 110.0, 26.0)
    }

    // INFO: Bounding box for the collapse all accordions button in the top header (backwards compatible).
    pub fn collapse_all_btn_rect(ws_x: f32, ws_y: f32) -> (f32, f32, f32, f32) {
        Self::toggle_collapse_all_btn_rect(ws_x, ws_y)
    }

    // INFO: Bounding box for the expand all accordions button in the top header (backwards compatible).
    pub fn expand_all_btn_rect(ws_x: f32, ws_y: f32) -> (f32, f32, f32, f32) {
        Self::toggle_collapse_all_btn_rect(ws_x, ws_y)
    }

    // INFO: Compatibility alias for the stage toggle button rect.
    pub fn uncommitted_changes_rect(ws_x: f32, ws_y: f32) -> (f32, f32, f32, f32) {
        Self::stage_toggle_btn_rect(ws_x, ws_y)
    }

    // INFO: Bounding box for the Commit modal trigger button in the top header.
    pub fn commit_btn_rect(ws_x: f32, ws_y: f32, ws_w: f32) -> (f32, f32, f32, f32) {
        (ws_x + ws_w - 158.0, ws_y + 8.0, 84.0, 26.0)
    }

    // INFO: Bounding box for the expand/restore full view toggle button in the top header.
    pub fn expand_btn_rect(ws_x: f32, ws_y: f32, ws_w: f32) -> (f32, f32, f32, f32) {
        (ws_x + ws_w - 66.0, ws_y + 8.0, 26.0, 26.0)
    }

    // INFO: Bounding box for the close panel button in the top header.
    pub fn close_btn_rect(ws_x: f32, ws_y: f32, ws_w: f32) -> (f32, f32, f32, f32) {
        (ws_x + ws_w - 34.0, ws_y + 8.0, 26.0, 26.0)
    }

    // INFO: Bounding box for the sidebar Tree/List view toggle button in the sidebar toolbar.
    pub fn sidebar_mode_toggle_rect(sbx: f32, sby: f32, sbw: f32) -> (f32, f32, f32, f32) {
        let btn_w = 26.0;
        let btn_h = 22.0;
        let bx = sbx + sbw - btn_w - 8.0;
        let by = sby + (Self::SIDEBAR_TOOLBAR_HEIGHT - btn_h) * 0.5;
        (bx, by, btn_w, btn_h)
    }

    // INFO: Bounding box for the Full File view toggle button in an accordion card header.
    pub fn card_full_file_btn_rect(card_x: f32, card_y: f32, card_w: f32) -> (f32, f32, f32, f32) {
        let btn_w = 90.0;
        let spacing = 8.0;
        let right_offset = 86.0;
        let x = card_x + card_w - right_offset - spacing - btn_w;
        (x, card_y + 6.0, btn_w, 24.0)
    }

    // INFO: Bounding box for the Open File in Editor action button in an accordion card header.
    pub fn card_open_file_btn_rect(card_x: f32, card_y: f32, card_w: f32) -> (f32, f32, f32, f32) {
        (card_x + card_w - 30.0, card_y + 6.0, 24.0, 24.0)
    }

    // INFO: Bounding box for the Stage/Unstage File action button in an accordion card header.
    pub fn card_stage_btn_rect(card_x: f32, card_y: f32, card_w: f32) -> (f32, f32, f32, f32) {
        (card_x + card_w - 58.0, card_y + 6.0, 24.0, 24.0)
    }

    // INFO: Bounding box for the Discard Changes action button in an accordion card header.
    pub fn card_discard_btn_rect(card_x: f32, card_y: f32, card_w: f32) -> (f32, f32, f32, f32) {
        (card_x + card_w - 86.0, card_y + 6.0, 24.0, 24.0)
    }

    // INFO: Bounding box for the modal confirmation dialog when discarding file changes.
    pub fn discard_dialog_rect(ws_x: f32, ws_y: f32, ws_w: f32, ws_h: f32) -> (f32, f32, f32, f32) {
        let modal_w = 440.0;
        let modal_h = 160.0;
        let modal_x = ws_x + (ws_w - modal_w) * 0.5;
        let modal_y = ws_y + (ws_h - modal_h) * 0.5;
        (modal_x, modal_y, modal_w, modal_h)
    }

    // INFO: Bounding boxes for Cancel and Discard buttons in the discard confirmation dialog.
    #[allow(clippy::type_complexity)]
    pub fn discard_dialog_buttons_rect(
        ws_x: f32,
        ws_y: f32,
        ws_w: f32,
        ws_h: f32,
    ) -> ((f32, f32, f32, f32), (f32, f32, f32, f32)) {
        let (mx, my, mw, mh) = Self::discard_dialog_rect(ws_x, ws_y, ws_w, ws_h);
        let btn_y = my + mh - 42.0;
        let btn_h = 28.0;
        let btn_w = 80.0;
        let confirm_rect = (mx + mw - btn_w - 20.0, btn_y, btn_w, btn_h);
        let cancel_rect = (mx + mw - 2.0 * btn_w - 30.0, btn_y, btn_w, btn_h);
        (cancel_rect, confirm_rect)
    }

    pub const SIDEBAR_FILE_BTN_WIDTH: f32 = 22.0;
    pub const SIDEBAR_FILE_BTN_HEIGHT: f32 = 20.0;
    pub const SIDEBAR_FILE_BTN_GAP: f32 = 4.0;
    pub const SIDEBAR_FILE_MARGIN_RIGHT: f32 = 8.0;

    // INFO: Computes bounding rects for sidebar file hover action buttons.
    // For unstaged file: (open_rect, revert_rect, Some(stage_rect))
    // For staged file: (open_rect, unstage_rect, None)
    #[allow(clippy::type_complexity)]
    pub fn sidebar_file_action_rects(
        sbx: f32,
        sbw: f32,
        row_y: f32,
        is_staged: bool,
    ) -> (
        (f32, f32, f32, f32),
        (f32, f32, f32, f32),
        Option<(f32, f32, f32, f32)>,
    ) {
        let btn_y = row_y + (Self::FILE_ROW_HEIGHT - Self::SIDEBAR_FILE_BTN_HEIGHT) * 0.5;
        let right_x = sbx + sbw - Self::SIDEBAR_FILE_MARGIN_RIGHT;

        if is_staged {
            let unstage_x = right_x - Self::SIDEBAR_FILE_BTN_WIDTH;
            let unstage_rect = (
                unstage_x,
                btn_y,
                Self::SIDEBAR_FILE_BTN_WIDTH,
                Self::SIDEBAR_FILE_BTN_HEIGHT,
            );
            let open_x = unstage_x - Self::SIDEBAR_FILE_BTN_GAP - Self::SIDEBAR_FILE_BTN_WIDTH;
            let open_rect = (
                open_x,
                btn_y,
                Self::SIDEBAR_FILE_BTN_WIDTH,
                Self::SIDEBAR_FILE_BTN_HEIGHT,
            );
            (open_rect, unstage_rect, None)
        } else {
            let stage_x = right_x - Self::SIDEBAR_FILE_BTN_WIDTH;
            let stage_rect = (
                stage_x,
                btn_y,
                Self::SIDEBAR_FILE_BTN_WIDTH,
                Self::SIDEBAR_FILE_BTN_HEIGHT,
            );
            let revert_x = stage_x - Self::SIDEBAR_FILE_BTN_GAP - Self::SIDEBAR_FILE_BTN_WIDTH;
            let revert_rect = (
                revert_x,
                btn_y,
                Self::SIDEBAR_FILE_BTN_WIDTH,
                Self::SIDEBAR_FILE_BTN_HEIGHT,
            );
            let open_x = revert_x - Self::SIDEBAR_FILE_BTN_GAP - Self::SIDEBAR_FILE_BTN_WIDTH;
            let open_rect = (
                open_x,
                btn_y,
                Self::SIDEBAR_FILE_BTN_WIDTH,
                Self::SIDEBAR_FILE_BTN_HEIGHT,
            );
            (open_rect, revert_rect, Some(stage_rect))
        }
    }

    // INFO: Computes bounding rects for sidebar group (section or directory) hover action buttons.
    // For unstaged group: (revert_rect, Some(stage_rect))
    // For staged group: (unstage_rect, None)
    #[allow(clippy::type_complexity)]
    pub fn sidebar_group_action_rects(
        sbx: f32,
        sbw: f32,
        row_y: f32,
        is_staged: bool,
    ) -> ((f32, f32, f32, f32), Option<(f32, f32, f32, f32)>) {
        let btn_y = row_y + (Self::FILE_ROW_HEIGHT - Self::SIDEBAR_FILE_BTN_HEIGHT) * 0.5;
        let right_x = sbx + sbw - Self::SIDEBAR_FILE_MARGIN_RIGHT;

        if is_staged {
            let unstage_x = right_x - Self::SIDEBAR_FILE_BTN_WIDTH;
            let unstage_rect = (
                unstage_x,
                btn_y,
                Self::SIDEBAR_FILE_BTN_WIDTH,
                Self::SIDEBAR_FILE_BTN_HEIGHT,
            );
            (unstage_rect, None)
        } else {
            let stage_x = right_x - Self::SIDEBAR_FILE_BTN_WIDTH;
            let stage_rect = (
                stage_x,
                btn_y,
                Self::SIDEBAR_FILE_BTN_WIDTH,
                Self::SIDEBAR_FILE_BTN_HEIGHT,
            );
            let revert_x = stage_x - Self::SIDEBAR_FILE_BTN_GAP - Self::SIDEBAR_FILE_BTN_WIDTH;
            let revert_rect = (
                revert_x,
                btn_y,
                Self::SIDEBAR_FILE_BTN_WIDTH,
                Self::SIDEBAR_FILE_BTN_HEIGHT,
            );
            (revert_rect, Some(stage_rect))
        }
    }

    // Left Sidebar dimensions (changed files list on the left side)
    pub fn sidebar_rect(
        model: &CodeReviewModel,
        ws_x: f32,
        ws_y: f32,
        ws_w: f32,
        ws_h: f32,
    ) -> Option<(f32, f32, f32, f32)> {
        if !model.sidebar_expanded {
            return None;
        }
        let sb_w = model.sidebar_width.min(ws_w * 0.4).max(220.0);
        Some((
            ws_x,
            ws_y + Self::HEADER_HEIGHT,
            sb_w,
            ws_h - Self::HEADER_HEIGHT,
        ))
    }

    // Diff viewport dimensions (stacked cards on the right side)
    pub fn diff_viewport_rect(
        model: &CodeReviewModel,
        ws_x: f32,
        ws_y: f32,
        ws_w: f32,
        ws_h: f32,
    ) -> (f32, f32, f32, f32) {
        let sb_w = if model.sidebar_expanded {
            model.sidebar_width.min(ws_w * 0.4).max(220.0)
        } else {
            0.0
        };
        (
            ws_x + sb_w,
            ws_y + Self::HEADER_HEIGHT,
            (ws_w - sb_w).max(50.0),
            ws_h - Self::HEADER_HEIGHT,
        )
    }

    pub const EDITING_CARD_MIN_HEIGHT: f32 = 480.0;

    // Computes full visual height of a file diff card
    pub fn card_height_for_file(file: &FileDiff, collapsed: bool, viewport_h: f32) -> f32 {
        Self::card_height_for_file_state(file, collapsed, false, viewport_h)
    }

    pub fn card_height_for_file_state(
        file: &FileDiff,
        collapsed: bool,
        is_editing: bool,
        viewport_h: f32,
    ) -> f32 {
        Self::card_height_for_file_state_ex(file, collapsed, is_editing, false, viewport_h)
    }

    // INFO: Computes full visual height of a file diff card ensuring expanded cards fit their actual content.
    pub fn card_height_for_file_state_ex(
        file: &FileDiff,
        collapsed: bool,
        _is_editing: bool,
        is_full_file: bool,
        _viewport_h: f32,
    ) -> f32 {
        if collapsed {
            return Self::CARD_HEADER_HEIGHT;
        }

        if is_full_file {
            let lines_count = if file.line_count > 0 {
                file.line_count
            } else {
                (file.additions + file.deletions).max(1)
            };
            let content_h = lines_count as f32 * Self::DIFF_LINE_HEIGHT + 16.0;
            return Self::CARD_HEADER_HEIGHT + content_h;
        }

        let diff_lines: usize = file.hunks.iter().map(|h| h.lines.len()).sum();
        let hunk_headers: usize = file.hunks.len();
        let lines_count = if diff_lines > 0 {
            diff_lines
        } else if file.line_count > 0 && file.hunks.is_empty() {
            file.line_count
        } else {
            (file.additions + file.deletions).max(1)
        };

        let content_h = lines_count as f32 * Self::DIFF_LINE_HEIGHT
            + hunk_headers as f32 * Self::UNMODIFIED_LINES_HEIGHT
            + 16.0;
        Self::CARD_HEADER_HEIGHT + content_h
    }

    // INFO: Computes cumulative Y offset for file card at target index within diff viewport.
    pub fn card_y_offset_for_file(
        model: &CodeReviewModel,
        target_idx: usize,
        viewport_h: f32,
    ) -> f32 {
        let mut offset = 12.0;
        for (i, file) in model.diff.files.iter().enumerate().take(target_idx) {
            let is_collapsed = model.is_file_collapsed(&file.path);
            let is_editing = model.active_file_idx == Some(i);
            let is_full = model.is_full_file(&file.path);
            offset += Self::card_height_for_file_state_ex(
                file,
                is_collapsed,
                is_editing,
                is_full,
                viewport_h,
            ) + Self::CARD_MARGIN_BOTTOM;
        }
        offset
    }

    // INFO: Computes on-screen Y coordinate of an accordion card header, keeping it sticky within its accordion boundaries while scrolling.
    pub fn card_header_y(curr_y: f32, card_h: f32, is_collapsed: bool, vy: f32) -> f32 {
        if is_collapsed {
            curr_y
        } else {
            let sticky_y = curr_y.max(vy);
            let max_y = (curr_y + card_h - Self::CARD_HEADER_HEIGHT).max(curr_y);
            sticky_y.min(max_y)
        }
    }

    // INFO: Bounding box for the diff viewport vertical scrollbar track.
    pub fn diff_scrollbar_rect(vx: f32, vy: f32, vw: f32, vh: f32) -> (f32, f32, f32, f32) {
        let scrollbar_w = 6.0;
        let track_x = vx + vw - scrollbar_w - 4.0;
        let track_y = vy + 4.0;
        let track_h = (vh - 8.0).max(10.0);
        (track_x - 4.0, track_y, scrollbar_w + 8.0, track_h)
    }

    // INFO: Returns the content bounding box (x, y, w, h) of the active editing card body within diff viewport.
    pub fn active_card_body_rect(
        model: &CodeReviewModel,
        ws_x: f32,
        ws_y: f32,
        ws_w: f32,
        ws_h: f32,
    ) -> Option<(f32, f32, f32, f32)> {
        let active_idx = model.active_file_idx?;
        let (vx, vy, vw, vh) = Self::diff_viewport_rect(model, ws_x, ws_y, ws_w, ws_h);
        let card_x = vx + 16.0;
        let card_w = (vw - 32.0).max(100.0);
        let mut curr_y = vy + 12.0 - model.diff_scroll_y;

        for (orig_idx, file) in model.diff.files.iter().enumerate() {
            let is_collapsed = model.is_file_collapsed(&file.path);
            let is_editing = orig_idx == active_idx;
            let is_full = model.is_full_file(&file.path);
            let card_h =
                Self::card_height_for_file_state_ex(file, is_collapsed, is_editing, is_full, vh);

            if orig_idx == active_idx {
                let header_y = Self::card_header_y(curr_y, card_h, is_collapsed, vy);
                let body_top = (header_y + Self::CARD_HEADER_HEIGHT).max(vy);
                let body_bot = (curr_y + card_h).min(vy + vh);
                let body_h = (body_bot - body_top).max(0.0);
                return Some((card_x, body_top, card_w, body_h));
            }

            curr_y += card_h + Self::CARD_MARGIN_BOTTOM;
        }

        None
    }

    // INFO: Locates which card body the given coordinates fall into for routing internal scroll and clicks.
    pub fn card_body_at(
        model: &CodeReviewModel,
        ws_x: f32,
        ws_y: f32,
        ws_w: f32,
        ws_h: f32,
        x: f32,
        y: f32,
    ) -> Option<(usize, f32, f32, f32, f32)> {
        let (vx, vy, vw, vh) = Self::diff_viewport_rect(model, ws_x, ws_y, ws_w, ws_h);
        if x < vx || x > vx + vw || y < vy || y > vy + vh {
            return None;
        }
        let card_x = vx + 16.0;
        let card_w = (vw - 32.0).max(100.0);
        let mut curr_y = vy + 12.0 - model.diff_scroll_y;

        for (orig_idx, file) in model.diff.files.iter().enumerate() {
            let is_collapsed = model.is_file_collapsed(&file.path);
            let is_editing = model.active_file_idx == Some(orig_idx);
            let is_full = model.is_full_file(&file.path);
            let card_h =
                Self::card_height_for_file_state_ex(file, is_collapsed, is_editing, is_full, vh);

            if !is_collapsed {
                let header_y = Self::card_header_y(curr_y, card_h, is_collapsed, vy);
                let body_top = (header_y + Self::CARD_HEADER_HEIGHT).max(vy);
                let body_bot = (curr_y + card_h).min(vy + vh);
                if x >= card_x && x <= card_x + card_w && y >= body_top && y <= body_bot {
                    let body_h = (body_bot - body_top).max(0.0);
                    return Some((orig_idx, card_x, body_top, card_w, body_h));
                }
            }

            curr_y += card_h + Self::CARD_MARGIN_BOTTOM;
        }

        None
    }

    // INFO: Geometry coordinates for floating line action popup (Stage Change / Revert Changes buttons above line number).
    #[allow(clippy::too_many_arguments)]
    pub fn line_action_popup_geometry(
        model: &CodeReviewModel,
        ws_x: f32,
        ws_y: f32,
        ws_w: f32,
        ws_h: f32,
        target_file_idx: usize,
        target_row: usize,
        is_deletion: bool,
    ) -> Option<LineActionPopupGeometry> {
        let (dvx, dvy, dvw, dvh) = Self::diff_viewport_rect(model, ws_x, ws_y, ws_w, ws_h);
        let card_x = dvx + 16.0;
        let gutter_w = 48.0;
        let total_content_h = model.total_diff_content_height(dvh);
        let max_diff_scroll = (total_content_h - dvh).max(0.0);
        let clamped_scroll = model.diff_scroll_y.clamp(0.0, max_diff_scroll);
        let mut curr_y = dvy + 12.0 - clamped_scroll;

        for (orig_idx, file) in model.diff.files.iter().enumerate() {
            let is_collapsed = model.is_file_collapsed(&file.path);
            let is_editing = model.active_file_idx == Some(orig_idx);
            let is_full = model.is_full_file(&file.path);
            let card_h =
                Self::card_height_for_file_state_ex(file, is_collapsed, is_editing, is_full, dvh);

            if orig_idx == target_file_idx {
                if is_collapsed {
                    return None;
                }
                let header_y = Self::card_header_y(curr_y, card_h, is_collapsed, dvy);
                let body_top = (header_y + Self::CARD_HEADER_HEIGHT).max(dvy);
                let body_bot = (curr_y + card_h).min(dvy + dvh);

                let line_y = if is_full {
                    let y = curr_y
                        + Self::CARD_HEADER_HEIGHT
                        + 8.0
                        + target_row as f32 * Self::DIFF_LINE_HEIGHT;
                    if y >= body_top && y <= body_bot {
                        Some(y)
                    } else {
                        None
                    }
                } else {
                    let mut cur_y = curr_y + Self::CARD_HEADER_HEIGHT + 8.0;
                    let mut found_y = None;
                    'hunk_loop: for hunk in &file.hunks {
                        cur_y += Self::UNMODIFIED_LINES_HEIGHT;
                        for diff_line in &hunk.lines {
                            let is_del = diff_line.line_type == akssora_git::DiffLineType::Delete;
                            let r = diff_line
                                .new_line_num
                                .or(diff_line.old_line_num)
                                .map(|n| n.saturating_sub(1));
                            if r == Some(target_row) && is_del == is_deletion {
                                if cur_y >= body_top && cur_y <= body_bot {
                                    found_y = Some(cur_y);
                                }
                                break 'hunk_loop;
                            }
                            cur_y += Self::DIFF_LINE_HEIGHT;
                        }
                    }
                    found_y
                };

                if let Some(ly) = line_y {
                    let add_btn_w = 104.0;
                    let rem_btn_w = 120.0;
                    let pad = 6.0;
                    let gap = 6.0;
                    let popup_w = pad + add_btn_w + gap + rem_btn_w + pad;
                    let popup_h = 28.0;
                    let popup_x =
                        (card_x + 4.0).clamp(dvx + 4.0, (dvx + dvw - popup_w - 4.0).max(dvx + 4.0));

                    // Positioned above the line number
                    let mut popup_y = ly - popup_h - 4.0;
                    // Flip below if it would overlap sticky header or go past viewport top
                    if popup_y < body_top + 2.0 {
                        popup_y = ly + Self::DIFF_LINE_HEIGHT + 4.0;
                    }

                    let add_btn_rect = (popup_x + pad, popup_y + 4.0, add_btn_w, 20.0);
                    let remove_btn_rect = (
                        popup_x + pad + add_btn_w + gap,
                        popup_y + 4.0,
                        rem_btn_w,
                        20.0,
                    );
                    let line_gutter_rect = (card_x, ly, gutter_w, Self::DIFF_LINE_HEIGHT);

                    return Some(LineActionPopupGeometry {
                        popup_rect: (popup_x, popup_y, popup_w, popup_h),
                        add_btn_rect,
                        remove_btn_rect,
                        line_gutter_rect,
                    });
                }
                return None;
            }

            curr_y += card_h + Self::CARD_MARGIN_BOTTOM;
        }

        None
    }

    // INFO: Hit-testing for gutter hover in Code Review cards (identifies hovered line and contextual action buttons).
    pub fn hit_test_gutter_hover(
        model: &CodeReviewModel,
        ws_x: f32,
        ws_y: f32,
        ws_w: f32,
        ws_h: f32,
        mx: f32,
        my: f32,
    ) -> Option<CodeReviewGutterHover> {
        let (vx, vy, vw, vh) = Self::diff_viewport_rect(model, ws_x, ws_y, ws_w, ws_h);
        if mx < vx || mx > vx + vw || my < vy || my > vy + vh {
            return None;
        }

        // 1. If a line popup is already active:
        // Buttons and current line gutter have top priority.
        let active_popup_geom = model.hovered_gutter.and_then(|prev| {
            Self::line_action_popup_geometry(
                model,
                ws_x,
                ws_y,
                ws_w,
                ws_h,
                prev.file_idx,
                prev.row,
                prev.is_deletion,
            )
            .map(|geom| (prev, geom))
        });

        if let Some((prev, ref geom)) = active_popup_geom {
            if let Some(action) = geom.hit_test_button(mx, my) {
                return Some(CodeReviewGutterHover {
                    file_idx: prev.file_idx,
                    row: prev.row,
                    is_deletion: prev.is_deletion,
                    action: Some(action),
                });
            }

            let in_current_gutter = mx >= geom.line_gutter_rect.0
                && mx <= geom.line_gutter_rect.0 + geom.line_gutter_rect.2
                && my >= geom.line_gutter_rect.1
                && my <= geom.line_gutter_rect.1 + geom.line_gutter_rect.3;

            if in_current_gutter {
                return Some(CodeReviewGutterHover {
                    file_idx: prev.file_idx,
                    row: prev.row,
                    is_deletion: prev.is_deletion,
                    action: None,
                });
            }
        }

        // 2. Check if pointer is hovering directly over any changed line's gutter
        let card_x = vx + 16.0;
        let gutter_w = 48.0;
        let total_content_h = model.total_diff_content_height(vh);
        let max_diff_scroll = (total_content_h - vh).max(0.0);
        let clamped_scroll = model.diff_scroll_y.clamp(0.0, max_diff_scroll);
        let mut curr_y = vy + 12.0 - clamped_scroll;

        for (orig_idx, file) in model.diff.files.iter().enumerate() {
            let is_collapsed = model.is_file_collapsed(&file.path);
            let is_editing = model.active_file_idx == Some(orig_idx);
            let is_full = model.is_full_file(&file.path);
            let card_h =
                Self::card_height_for_file_state_ex(file, is_collapsed, is_editing, is_full, vh);

            if !is_collapsed {
                let header_y = Self::card_header_y(curr_y, card_h, is_collapsed, vy);
                let body_top = (header_y + Self::CARD_HEADER_HEIGHT).max(vy);
                let body_bot = (curr_y + card_h).min(vy + vh);

                if my > body_top && my <= body_bot && mx >= card_x && mx < card_x + gutter_w {
                    if is_full {
                        let rel_y = my - (curr_y + Self::CARD_HEADER_HEIGHT + 8.0);
                        if rel_y >= 0.0 {
                            let row = (rel_y / Self::DIFF_LINE_HEIGHT).floor() as usize;
                            let matched = file.hunks.iter().find_map(|h| {
                                h.lines.iter().find(|l| {
                                    (l.line_type == akssora_git::DiffLineType::Add
                                        || l.line_type == akssora_git::DiffLineType::Delete)
                                        && (l.new_line_num == Some(row + 1)
                                            || l.old_line_num == Some(row + 1))
                                })
                            });
                            if let Some(diff_l) = matched {
                                return Some(CodeReviewGutterHover {
                                    file_idx: orig_idx,
                                    row,
                                    is_deletion: diff_l.line_type
                                        == akssora_git::DiffLineType::Delete,
                                    action: None,
                                });
                            }
                        }
                    } else {
                        let mut cur_y = curr_y + Self::CARD_HEADER_HEIGHT + 8.0;
                        for hunk in &file.hunks {
                            cur_y += Self::UNMODIFIED_LINES_HEIGHT;
                            for diff_line in &hunk.lines {
                                if my >= cur_y
                                    && my < cur_y + Self::DIFF_LINE_HEIGHT
                                    && (diff_line.line_type == akssora_git::DiffLineType::Add
                                        || diff_line.line_type == akssora_git::DiffLineType::Delete)
                                {
                                    let target_row = diff_line
                                        .new_line_num
                                        .or(diff_line.old_line_num)
                                        .map(|n| n.saturating_sub(1))
                                        .unwrap_or(0);
                                    return Some(CodeReviewGutterHover {
                                        file_idx: orig_idx,
                                        row: target_row,
                                        is_deletion: diff_line.line_type
                                            == akssora_git::DiffLineType::Delete,
                                        action: None,
                                    });
                                }
                                cur_y += Self::DIFF_LINE_HEIGHT;
                            }
                        }
                    }
                }
            }

            curr_y += card_h + Self::CARD_MARGIN_BOTTOM;
        }

        // 3. Keep popup open if hovering anywhere inside the popup card (e.g. background/padding)
        if let Some((prev, ref geom)) = active_popup_geom {
            let in_popup_card = mx >= geom.popup_rect.0
                && mx <= geom.popup_rect.0 + geom.popup_rect.2
                && my >= geom.popup_rect.1
                && my <= geom.popup_rect.1 + geom.popup_rect.3;
            if in_popup_card {
                return Some(CodeReviewGutterHover {
                    file_idx: prev.file_idx,
                    row: prev.row,
                    is_deletion: prev.is_deletion,
                    action: None,
                });
            }
        }

        None
    }

    // INFO: Hit-testing logic for user clicks across Code Review header, sidebar, and diff cards.
    pub fn hit_test(
        model: &CodeReviewModel,
        ws_x: f32,
        ws_y: f32,
        ws_w: f32,
        ws_h: f32,
        click_x: f32,
        click_y: f32,
    ) -> Option<CodeReviewAction> {
        // INFO: Modal Discard Confirmation dialog hit-testing (takes highest priority).
        if model.confirm_discard.is_some() {
            let (cancel_rect, confirm_rect) =
                Self::discard_dialog_buttons_rect(ws_x, ws_y, ws_w, ws_h);
            if click_x >= confirm_rect.0
                && click_x <= confirm_rect.0 + confirm_rect.2
                && click_y >= confirm_rect.1
                && click_y <= confirm_rect.1 + confirm_rect.3
            {
                return Some(CodeReviewAction::ConfirmDiscardOk);
            }
            if click_x >= cancel_rect.0
                && click_x <= cancel_rect.0 + cancel_rect.2
                && click_y >= cancel_rect.1
                && click_y <= cancel_rect.1 + cancel_rect.3
            {
                return Some(CodeReviewAction::ConfirmDiscardCancel);
            }
            let (mx, my, mw, mh) = Self::discard_dialog_rect(ws_x, ws_y, ws_w, ws_h);
            if click_x < mx || click_x > mx + mw || click_y < my || click_y > my + mh {
                return Some(CodeReviewAction::ConfirmDiscardCancel);
            }
            return None;
        }

        // INFO: Top Header bar action hit-testing.
        if click_y >= ws_y && click_y <= ws_y + Self::HEADER_HEIGHT {
            let (sx, sy, sw, sh) = Self::sidebar_toggle_btn_rect(ws_x, ws_y);
            if click_x >= sx && click_x <= sx + sw && click_y >= sy && click_y <= sy + sh {
                return Some(CodeReviewAction::ToggleSidebar);
            }

            let (tx, ty, tw, th) = Self::toggle_collapse_all_btn_rect(ws_x, ws_y);
            if click_x >= tx && click_x <= tx + tw && click_y >= ty && click_y <= ty + th {
                return Some(CodeReviewAction::ToggleCollapseAll);
            }

            let is_git = akssora_git::GitInspector::is_git_repo(&model.repo_root);
            if is_git {
                let (cx, cy, cw, ch) = Self::commit_btn_rect(ws_x, ws_y, ws_w);
                if click_x >= cx && click_x <= cx + cw && click_y >= cy && click_y <= cy + ch {
                    return Some(CodeReviewAction::OpenCommitDialog);
                }
            }

            let (ex, ey, ew, eh) = Self::expand_btn_rect(ws_x, ws_y, ws_w);
            if click_x >= ex && click_x <= ex + ew && click_y >= ey && click_y <= ey + eh {
                return Some(CodeReviewAction::ToggleExpandView);
            }

            let (clx, cly, clw, clh) = Self::close_btn_rect(ws_x, ws_y, ws_w);
            if click_x >= clx && click_x <= clx + clw && click_y >= cly && click_y <= cly + clh {
                return Some(CodeReviewAction::ClosePanel);
            }
        }

        // INFO: Left Sidebar (Changed Files) hit-testing.
        if let Some((sbx, sby, sbw, sbh)) = Self::sidebar_rect(model, ws_x, ws_y, ws_w, ws_h)
            && click_x >= sbx
            && click_x <= sbx + sbw
            && click_y >= sby
            && click_y <= sby + sbh
        {
            // INFO: Sidebar toolbar row at top.
            if click_y < sby + Self::SIDEBAR_TOOLBAR_HEIGHT {
                let (tx, ty, tw, th) = Self::sidebar_mode_toggle_rect(sbx, sby, sbw);
                if click_x >= tx && click_x <= tx + tw && click_y >= ty && click_y <= ty + th {
                    return Some(CodeReviewAction::ToggleSidebarMode);
                }
                return None;
            }

            let items = model.visible_tree_items();
            let total_items_h = items.len() as f32 * Self::FILE_ROW_HEIGHT + 16.0;
            let list_viewport_h = (sbh - Self::SIDEBAR_TOOLBAR_HEIGHT).max(0.0);
            let max_sidebar_scroll = (total_items_h - list_viewport_h).max(0.0);
            let clamped_scroll_y = model.sidebar_scroll_y.clamp(0.0, max_sidebar_scroll);

            let rel_y = click_y - (sby + Self::SIDEBAR_TOOLBAR_HEIGHT + 4.0) + clamped_scroll_y;
            if rel_y >= 0.0 {
                let row_idx = (rel_y / Self::FILE_ROW_HEIGHT).floor() as usize;
                if let Some(item) = items.get(row_idx) {
                    match item {
                        CodeReviewTreeItem::SectionHeader { is_staged, .. } => {
                            let row_top_y = sby + Self::SIDEBAR_TOOLBAR_HEIGHT + 4.0
                                - clamped_scroll_y
                                + row_idx as f32 * Self::FILE_ROW_HEIGHT;
                            let (main_rect, stage_opt) =
                                Self::sidebar_group_action_rects(sbx, sbw, row_top_y, *is_staged);
                            if *is_staged {
                                if click_x >= main_rect.0
                                    && click_x <= main_rect.0 + main_rect.2
                                    && click_y >= main_rect.1
                                    && click_y <= main_rect.1 + main_rect.3
                                {
                                    return Some(CodeReviewAction::UnstageAll);
                                }
                            } else {
                                if click_x >= main_rect.0
                                    && click_x <= main_rect.0 + main_rect.2
                                    && click_y >= main_rect.1
                                    && click_y <= main_rect.1 + main_rect.3
                                {
                                    return Some(CodeReviewAction::PromptDiscardAll);
                                }
                                if let Some(stage_rect) = stage_opt
                                    && click_x >= stage_rect.0
                                    && click_x <= stage_rect.0 + stage_rect.2
                                    && click_y >= stage_rect.1
                                    && click_y <= stage_rect.1 + stage_rect.3
                                {
                                    return Some(CodeReviewAction::StageAll);
                                }
                            }
                            return Some(CodeReviewAction::ToggleSectionCollapse(*is_staged));
                        }
                        CodeReviewTreeItem::Directory {
                            is_staged, path, ..
                        } => {
                            let row_top_y = sby + Self::SIDEBAR_TOOLBAR_HEIGHT + 4.0
                                - clamped_scroll_y
                                + row_idx as f32 * Self::FILE_ROW_HEIGHT;
                            let (main_rect, stage_opt) =
                                Self::sidebar_group_action_rects(sbx, sbw, row_top_y, *is_staged);
                            if *is_staged {
                                if click_x >= main_rect.0
                                    && click_x <= main_rect.0 + main_rect.2
                                    && click_y >= main_rect.1
                                    && click_y <= main_rect.1 + main_rect.3
                                {
                                    return Some(CodeReviewAction::UnstageDirectoryByPath(
                                        path.clone(),
                                    ));
                                }
                            } else {
                                if click_x >= main_rect.0
                                    && click_x <= main_rect.0 + main_rect.2
                                    && click_y >= main_rect.1
                                    && click_y <= main_rect.1 + main_rect.3
                                {
                                    return Some(CodeReviewAction::PromptDiscardDirectory(
                                        path.clone(),
                                    ));
                                }
                                if let Some(stage_rect) = stage_opt
                                    && click_x >= stage_rect.0
                                    && click_x <= stage_rect.0 + stage_rect.2
                                    && click_y >= stage_rect.1
                                    && click_y <= stage_rect.1 + stage_rect.3
                                {
                                    return Some(CodeReviewAction::StageDirectoryByPath(
                                        path.clone(),
                                    ));
                                }
                            }
                            return Some(CodeReviewAction::ToggleTreeDir(path.clone()));
                        }
                        CodeReviewTreeItem::File {
                            is_staged,
                            orig_idx,
                            path,
                            ..
                        } => {
                            let row_top_y = sby + Self::SIDEBAR_TOOLBAR_HEIGHT + 4.0
                                - clamped_scroll_y
                                + row_idx as f32 * Self::FILE_ROW_HEIGHT;
                            let (open_rect, mid_rect, stage_opt) =
                                Self::sidebar_file_action_rects(sbx, sbw, row_top_y, *is_staged);

                            if click_x >= open_rect.0
                                && click_x <= open_rect.0 + open_rect.2
                                && click_y >= open_rect.1
                                && click_y <= open_rect.1 + open_rect.3
                            {
                                return Some(CodeReviewAction::OpenFileInEditorByPath(
                                    path.clone(),
                                ));
                            }

                            if *is_staged {
                                if click_x >= mid_rect.0
                                    && click_x <= mid_rect.0 + mid_rect.2
                                    && click_y >= mid_rect.1
                                    && click_y <= mid_rect.1 + mid_rect.3
                                {
                                    return Some(CodeReviewAction::UnstageFileByPath(path.clone()));
                                }
                            } else {
                                if click_x >= mid_rect.0
                                    && click_x <= mid_rect.0 + mid_rect.2
                                    && click_y >= mid_rect.1
                                    && click_y <= mid_rect.1 + mid_rect.3
                                {
                                    return Some(CodeReviewAction::PromptDiscardFile(path.clone()));
                                }
                                if let Some(stage_rect) = stage_opt
                                    && click_x >= stage_rect.0
                                    && click_x <= stage_rect.0 + stage_rect.2
                                    && click_y >= stage_rect.1
                                    && click_y <= stage_rect.1 + stage_rect.3
                                {
                                    return Some(CodeReviewAction::StageFileByPath(path.clone()));
                                }
                            }

                            return Some(CodeReviewAction::SelectTreeFile {
                                is_staged: *is_staged,
                                orig_idx: *orig_idx,
                                path: path.clone(),
                            });
                        }
                    }
                }
            }
        }

        // INFO: Diff Viewport (File Cards on right) hit-testing.
        let (vx, vy, vw, vh) = Self::diff_viewport_rect(model, ws_x, ws_y, ws_w, ws_h);
        if click_x >= vx && click_x <= vx + vw && click_y >= vy && click_y <= vy + vh {
            let card_x = vx + 16.0;
            let card_w = (vw - 32.0).max(100.0);
            let mut curr_y = vy + 12.0 - model.diff_scroll_y;

            for (orig_idx, file) in model.diff.files.iter().enumerate() {
                let is_collapsed = model.is_file_collapsed(&file.path);
                let is_editing = model.active_file_idx == Some(orig_idx);
                let is_full = model.is_full_file(&file.path);
                let card_h = Self::card_height_for_file_state_ex(
                    file,
                    is_collapsed,
                    is_editing,
                    is_full,
                    vh,
                );

                let header_y = Self::card_header_y(curr_y, card_h, is_collapsed, vy);

                if click_y >= header_y
                    && click_y <= header_y + Self::CARD_HEADER_HEIGHT
                    && click_y >= vy
                    && click_y <= vy + vh
                    && click_x >= card_x
                    && click_x <= card_x + card_w
                {
                    // INFO: Action buttons on accordion trigger: Full File, Open file (↗), Stage/Unstage (+/−), and Discard (↺).
                    let (fx, fy, fw, fh) = Self::card_full_file_btn_rect(card_x, header_y, card_w);
                    if click_x >= fx && click_x <= fx + fw && click_y >= fy && click_y <= fy + fh {
                        return Some(CodeReviewAction::ToggleFullFile(orig_idx));
                    }
                    let (ox, oy, ow, oh) = Self::card_open_file_btn_rect(card_x, header_y, card_w);
                    if click_x >= ox && click_x <= ox + ow && click_y >= oy && click_y <= oy + oh {
                        return Some(CodeReviewAction::OpenFileInEditor(orig_idx));
                    }
                    let (sx, sy, sw, sh) = Self::card_stage_btn_rect(card_x, header_y, card_w);
                    if click_x >= sx && click_x <= sx + sw && click_y >= sy && click_y <= sy + sh {
                        return if model.is_staged {
                            Some(CodeReviewAction::UnstageFile(orig_idx))
                        } else {
                            Some(CodeReviewAction::StageFile(orig_idx))
                        };
                    }
                    let (dx, dy, dw, dh) = Self::card_discard_btn_rect(card_x, header_y, card_w);
                    if click_x >= dx && click_x <= dx + dw && click_y >= dy && click_y <= dy + dh {
                        return Some(CodeReviewAction::DiscardFile(orig_idx));
                    }
                    return Some(CodeReviewAction::ToggleFileCollapse(orig_idx));
                }

                // Clicking the card body: check hunk header expand arrows or route to editor
                if !is_collapsed {
                    let body_top = (header_y + Self::CARD_HEADER_HEIGHT).max(vy);
                    let body_bot = (curr_y + card_h).min(vy + vh);
                    if click_y > body_top
                        && click_y <= body_bot
                        && click_x >= card_x
                        && click_x <= card_x + card_w
                    {
                        if !is_full {
                            let mut cur_hunk_y = curr_y + Self::CARD_HEADER_HEIGHT + 8.0;
                            for (hunk_idx, hunk) in file.hunks.iter().enumerate() {
                                let top_bar_y = cur_hunk_y;
                                if click_y >= top_bar_y
                                    && click_y <= top_bar_y + Self::UNMODIFIED_LINES_HEIGHT
                                    && click_y >= body_top
                                    && click_y <= body_bot
                                {
                                    if hunk.can_expand_top()
                                        && click_x >= card_x + card_w - 48.0
                                        && click_x <= card_x + card_w - 28.0
                                    {
                                        return Some(CodeReviewAction::ExpandContextTop {
                                            file_idx: orig_idx,
                                            hunk_idx,
                                        });
                                    }
                                    if hunk.can_expand_bottom(file.line_count)
                                        && click_x >= card_x + card_w - 24.0
                                        && click_x <= card_x + card_w - 4.0
                                    {
                                        return Some(CodeReviewAction::ExpandContextBottom {
                                            file_idx: orig_idx,
                                            hunk_idx,
                                        });
                                    }
                                }
                                cur_hunk_y += Self::UNMODIFIED_LINES_HEIGHT
                                    + hunk.lines.len() as f32 * Self::DIFF_LINE_HEIGHT;
                            }
                        }

                        return Some(CodeReviewAction::ClickEditor {
                            orig_idx,
                            click_x,
                            click_y,
                            body_x: card_x,
                            body_y: curr_y + Self::CARD_HEADER_HEIGHT,
                            body_w: card_w,
                        });
                    }
                }

                curr_y += card_h + Self::CARD_MARGIN_BOTTOM;
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::DiscardTarget;
    use akssora_git::{DiffHunk, DiffLine, DiffLineType, FileDiff, FileGitStatus, RepoDiff};
    use std::path::PathBuf;

    #[test]
    fn test_hit_test_card_header_toggle_collapse() {
        let model = CodeReviewModel {
            diff: RepoDiff {
                files: vec![FileDiff {
                    path: PathBuf::from("foo.rs"),
                    old_path: None,
                    status: FileGitStatus::Modified,
                    hunks: vec![],
                    additions: 1,
                    deletions: 0,
                    is_binary: false,
                    line_count: 5,
                }],
                total_additions: 1,
                total_deletions: 0,
                files_changed: 1,
            },
            ..Default::default()
        };

        // Click on the header of the card (outside buttons)
        let (vx, vy, _vw, _vh) =
            CodeReviewLayout::diff_viewport_rect(&model, 0.0, 0.0, 1000.0, 800.0);
        let card_x = vx + 16.0;
        let card_y = vy + 12.0;

        let action = CodeReviewLayout::hit_test(
            &model,
            0.0,
            0.0,
            1000.0,
            800.0,
            card_x + 50.0,
            card_y + 10.0,
        );

        assert_eq!(action, Some(CodeReviewAction::ToggleFileCollapse(0)));
    }

    #[test]
    fn test_hit_test_hunk_context_expansion_arrows() {
        let mut model = CodeReviewModel::default();
        let hunk = DiffHunk {
            old_start: 10,
            old_lines: 5,
            new_start: 10,
            new_lines: 6,
            header: "@@ -10,5 +10,6 @@".to_string(),
            lines: vec![DiffLine {
                line_type: DiffLineType::Add,
                content: "added\n".to_string(),
                old_line_num: None,
                new_line_num: Some(10),
            }],
        };
        model.diff = RepoDiff {
            files: vec![FileDiff {
                path: PathBuf::from("foo.rs"),
                old_path: None,
                status: FileGitStatus::Modified,
                hunks: vec![hunk],
                additions: 1,
                deletions: 0,
                is_binary: false,
                line_count: 50,
            }],
            total_additions: 1,
            total_deletions: 0,
            files_changed: 1,
        };

        let (vx, vy, vw, _vh) =
            CodeReviewLayout::diff_viewport_rect(&model, 0.0, 0.0, 1000.0, 800.0);
        let card_x = vx + 16.0;
        let card_w = (vw - 32.0).max(100.0);
        let card_y = vy + 12.0;
        let hunk_header_y = card_y + CodeReviewLayout::CARD_HEADER_HEIGHT + 8.0;

        // Click top expand button (▲): card_x + card_w - 48.0 .. card_x + card_w - 28.0
        let click_x_top = card_x + card_w - 38.0;
        let click_y = hunk_header_y + 10.0;
        let action_top =
            CodeReviewLayout::hit_test(&model, 0.0, 0.0, 1000.0, 800.0, click_x_top, click_y);
        assert_eq!(
            action_top,
            Some(CodeReviewAction::ExpandContextTop {
                file_idx: 0,
                hunk_idx: 0,
            })
        );

        // Click bottom expand button (▼): card_x + card_w - 24.0 .. card_x + card_w - 4.0
        let click_x_bot = card_x + card_w - 14.0;
        let action_bot =
            CodeReviewLayout::hit_test(&model, 0.0, 0.0, 1000.0, 800.0, click_x_bot, click_y);
        assert_eq!(
            action_bot,
            Some(CodeReviewAction::ExpandContextBottom {
                file_idx: 0,
                hunk_idx: 0,
            })
        );
    }

    #[test]
    fn test_line_action_popup_geometry_and_hover() {
        let mut model = CodeReviewModel::default();
        let hunk = DiffHunk {
            old_start: 1,
            old_lines: 1,
            new_start: 1,
            new_lines: 2,
            header: "@@ -1,1 +1,2 @@".to_string(),
            lines: vec![
                DiffLine {
                    line_type: DiffLineType::Context,
                    content: "fn main() {\n".to_string(),
                    old_line_num: Some(1),
                    new_line_num: Some(1),
                },
                DiffLine {
                    line_type: DiffLineType::Add,
                    content: "    println!(\"hello\");\n".to_string(),
                    old_line_num: None,
                    new_line_num: Some(2),
                },
            ],
        };
        model.diff = RepoDiff {
            files: vec![FileDiff {
                path: PathBuf::from("main.rs"),
                old_path: None,
                status: FileGitStatus::Modified,
                hunks: vec![hunk],
                additions: 1,
                deletions: 0,
                is_binary: false,
                line_count: 2,
            }],
            total_additions: 1,
            total_deletions: 0,
            files_changed: 1,
        };

        let (vx, vy, _vw, _vh) =
            CodeReviewLayout::diff_viewport_rect(&model, 0.0, 0.0, 1000.0, 800.0);
        let card_x = vx + 16.0;
        let card_y = vy + 12.0;
        let hunk_header_y = card_y + CodeReviewLayout::CARD_HEADER_HEIGHT + 8.0;
        let line_0_y = hunk_header_y + CodeReviewLayout::UNMODIFIED_LINES_HEIGHT;
        let line_1_y = line_0_y + CodeReviewLayout::DIFF_LINE_HEIGHT;

        // Hover over line 0 (unchanged line) -> should be None
        let hover_unchanged = CodeReviewLayout::hit_test_gutter_hover(
            &model,
            0.0,
            0.0,
            1000.0,
            800.0,
            card_x + 10.0,
            line_0_y + 5.0,
        );
        assert_eq!(hover_unchanged, None);

        // Hover over line 1 (added line) -> should activate popup for row 1
        let hover_add_line = CodeReviewLayout::hit_test_gutter_hover(
            &model,
            0.0,
            0.0,
            1000.0,
            800.0,
            card_x + 10.0,
            line_1_y + 5.0,
        );
        assert_eq!(
            hover_add_line,
            Some(CodeReviewGutterHover {
                file_idx: 0,
                row: 1,
                is_deletion: false,
                action: None,
            })
        );

        // Now model has active popup
        model.hovered_gutter = hover_add_line;

        // Calculate popup geometry
        let geom = CodeReviewLayout::line_action_popup_geometry(
            &model, 0.0, 0.0, 1000.0, 800.0, 0, 1, false,
        )
        .expect("popup geometry should exist");

        // Popup should be positioned above line_1_y (or flipped below if clipped)
        assert!(geom.popup_rect.1 != line_1_y);

        // Hovering inside the Add button
        let (ax, ay, aw, ah) = geom.add_btn_rect;
        let hover_add_btn = CodeReviewLayout::hit_test_gutter_hover(
            &model,
            0.0,
            0.0,
            1000.0,
            800.0,
            ax + aw * 0.5,
            ay + ah * 0.5,
        );
        assert_eq!(
            hover_add_btn,
            Some(CodeReviewGutterHover {
                file_idx: 0,
                row: 1,
                is_deletion: false,
                action: Some(akssora_editor::GutterActionKind::StageLine),
            })
        );

        // Hovering inside the Remove button
        let (rx, ry, rw, rh) = geom.remove_btn_rect;
        let hover_rem_btn = CodeReviewLayout::hit_test_gutter_hover(
            &model,
            0.0,
            0.0,
            1000.0,
            800.0,
            rx + rw * 0.5,
            ry + rh * 0.5,
        );
        assert_eq!(
            hover_rem_btn,
            Some(CodeReviewGutterHover {
                file_idx: 0,
                row: 1,
                is_deletion: false,
                action: Some(akssora_editor::GutterActionKind::RevertLine),
            })
        );

        // Hovering in popup container/bridge keeps popup active
        let hover_popup_bg = CodeReviewLayout::hit_test_gutter_hover(
            &model,
            0.0,
            0.0,
            1000.0,
            800.0,
            geom.popup_rect.0 + 2.0,
            geom.popup_rect.1 + 2.0,
        );
        assert_eq!(
            hover_popup_bg,
            Some(CodeReviewGutterHover {
                file_idx: 0,
                row: 1,
                is_deletion: false,
                action: None,
            })
        );

        // Moving mouse away dismisses popup
        let hover_away = CodeReviewLayout::hit_test_gutter_hover(
            &model,
            0.0,
            0.0,
            1000.0,
            800.0,
            card_x + 300.0,
            line_1_y + 50.0,
        );
        assert_eq!(hover_away, None);
    }

    #[test]
    fn test_sidebar_tree_structure_compact_folders_and_sections() {
        let staged_diff = RepoDiff {
            files: vec![FileDiff {
                path: PathBuf::from("src/main.rs"),
                old_path: None,
                status: FileGitStatus::Modified,
                hunks: vec![],
                additions: 2,
                deletions: 1,
                is_binary: false,
                line_count: 10,
            }],
            total_additions: 2,
            total_deletions: 1,
            files_changed: 1,
        };
        let unstaged_diff = RepoDiff {
            files: vec![FileDiff {
                path: PathBuf::from("agents/skills/akssora/SKILL.md"),
                old_path: None,
                status: FileGitStatus::Untracked,
                hunks: vec![],
                additions: 50,
                deletions: 0,
                is_binary: false,
                line_count: 50,
            }],
            total_additions: 50,
            total_deletions: 0,
            files_changed: 1,
        };
        let mut model = CodeReviewModel {
            staged_diff,
            unstaged_diff,
            ..Default::default()
        };

        let items = model.visible_tree_items();
        // Item 0: Staged Section Header
        match &items[0] {
            CodeReviewTreeItem::SectionHeader {
                is_staged, count, ..
            } => {
                assert!(*is_staged);
                assert_eq!(*count, 1);
            }
            _ => panic!("Expected Staged SectionHeader at index 0"),
        }

        // Item 1: Staged file (under src/ dir or file)
        match &items[1] {
            CodeReviewTreeItem::Directory {
                is_staged,
                display_name,
                ..
            } => {
                assert!(*is_staged);
                assert_eq!(display_name, "src");
            }
            CodeReviewTreeItem::File {
                is_staged,
                display_name,
                ..
            } => {
                assert!(*is_staged);
                assert_eq!(display_name, "main.rs");
            }
            _ => panic!("Expected Directory or File in Staged section"),
        }

        // SectionHeader for Changes
        let unstaged_header_idx = items
            .iter()
            .position(|it| {
                matches!(
                    it,
                    CodeReviewTreeItem::SectionHeader {
                        is_staged: false,
                        ..
                    }
                )
            })
            .expect("Changes section header should exist");
        assert!(unstaged_header_idx > 0);

        // Verify compact folder in Changes: agents/skills/akssora should be merged into a single folder!
        let compact_dir = items.iter().find_map(|it| match it {
            CodeReviewTreeItem::Directory {
                is_staged: false,
                display_name,
                ..
            } => Some(display_name.clone()),
            _ => None,
        });
        assert_eq!(compact_dir.as_deref(), Some("agents/skills/akssora"));

        // Toggle staged section collapse
        model.toggle_section_collapse(true);
        let items_collapsed = model.visible_tree_items();
        assert!(items_collapsed.len() < items.len());
        assert_eq!(
            items_collapsed[0],
            CodeReviewTreeItem::SectionHeader {
                is_staged: true,
                count: 1,
                is_collapsed: true,
            }
        );
    }

    #[test]
    fn test_sidebar_hover_action_buttons_and_dialog_hit_test() {
        let unstaged_diff = RepoDiff {
            files: vec![FileDiff {
                path: PathBuf::from("test.rs"),
                old_path: None,
                status: FileGitStatus::Modified,
                hunks: vec![],
                additions: 1,
                deletions: 0,
                is_binary: false,
                line_count: 5,
            }],
            total_additions: 1,
            total_deletions: 0,
            files_changed: 1,
        };
        let mut model = CodeReviewModel {
            diff: unstaged_diff.clone(),
            unstaged_diff,
            ..Default::default()
        };

        let (sbx, sby, sbw, _sbh) =
            CodeReviewLayout::sidebar_rect(&model, 0.0, 0.0, 1000.0, 800.0).unwrap();

        // Row 0: Staged Changes header
        // Row 1: Changes header
        // Row 2: File test.rs
        let row_top = sby + CodeReviewLayout::SIDEBAR_TOOLBAR_HEIGHT + 4.0;
        let header_staged_y = row_top + 10.0;
        let act_header_staged = CodeReviewLayout::hit_test(
            &model,
            0.0,
            0.0,
            1000.0,
            800.0,
            sbx + 20.0,
            header_staged_y,
        );
        assert_eq!(
            act_header_staged,
            Some(CodeReviewAction::ToggleSectionCollapse(true))
        );

        let row_file_y = row_top + 2.0 * CodeReviewLayout::FILE_ROW_HEIGHT;
        let (open_rect, revert_rect, stage_opt) =
            CodeReviewLayout::sidebar_file_action_rects(sbx, sbw, row_file_y, false);

        // Click on Open File button
        let act_open = CodeReviewLayout::hit_test(
            &model,
            0.0,
            0.0,
            1000.0,
            800.0,
            open_rect.0 + 5.0,
            open_rect.1 + 5.0,
        );
        assert_eq!(
            act_open,
            Some(CodeReviewAction::OpenFileInEditorByPath(PathBuf::from(
                "test.rs"
            )))
        );

        // Click on Revert button
        let act_revert = CodeReviewLayout::hit_test(
            &model,
            0.0,
            0.0,
            1000.0,
            800.0,
            revert_rect.0 + 5.0,
            revert_rect.1 + 5.0,
        );
        assert_eq!(
            act_revert,
            Some(CodeReviewAction::PromptDiscardFile(PathBuf::from(
                "test.rs"
            )))
        );

        // Click on Stage button
        let stage_rect = stage_opt.unwrap();
        let act_stage = CodeReviewLayout::hit_test(
            &model,
            0.0,
            0.0,
            1000.0,
            800.0,
            stage_rect.0 + 5.0,
            stage_rect.1 + 5.0,
        );
        assert_eq!(
            act_stage,
            Some(CodeReviewAction::StageFileByPath(PathBuf::from("test.rs")))
        );

        // Click row outside action buttons selects file
        let act_select = CodeReviewLayout::hit_test(
            &model,
            0.0,
            0.0,
            1000.0,
            800.0,
            sbx + 30.0,
            row_file_y + 10.0,
        );
        assert_eq!(
            act_select,
            Some(CodeReviewAction::SelectTreeFile {
                is_staged: false,
                orig_idx: 0,
                path: PathBuf::from("test.rs"),
            })
        );

        // Discard Confirmation Modal hit test
        model.confirm_discard = Some(DiscardTarget::File(PathBuf::from("test.rs")));
        let (cancel_rect, confirm_rect) =
            CodeReviewLayout::discard_dialog_buttons_rect(0.0, 0.0, 1000.0, 800.0);

        let act_cancel = CodeReviewLayout::hit_test(
            &model,
            0.0,
            0.0,
            1000.0,
            800.0,
            cancel_rect.0 + 5.0,
            cancel_rect.1 + 5.0,
        );
        assert_eq!(act_cancel, Some(CodeReviewAction::ConfirmDiscardCancel));

        let act_confirm = CodeReviewLayout::hit_test(
            &model,
            0.0,
            0.0,
            1000.0,
            800.0,
            confirm_rect.0 + 5.0,
            confirm_rect.1 + 5.0,
        );
        assert_eq!(act_confirm, Some(CodeReviewAction::ConfirmDiscardOk));

        // Click outside dialog cancels
        let act_outside = CodeReviewLayout::hit_test(&model, 0.0, 0.0, 1000.0, 800.0, 10.0, 10.0);
        assert_eq!(act_outside, Some(CodeReviewAction::ConfirmDiscardCancel));

        // Test section header group buttons
        model.confirm_discard = None;
        let (unstage_all_rect, _) =
            CodeReviewLayout::sidebar_group_action_rects(sbx, sbw, row_top, true);
        let act_unstage_all = CodeReviewLayout::hit_test(
            &model,
            0.0,
            0.0,
            1000.0,
            800.0,
            unstage_all_rect.0 + 5.0,
            unstage_all_rect.1 + 5.0,
        );
        assert_eq!(act_unstage_all, Some(CodeReviewAction::UnstageAll));

        let row_changes_y = row_top + CodeReviewLayout::FILE_ROW_HEIGHT;
        let (revert_all_rect, stage_all_opt) =
            CodeReviewLayout::sidebar_group_action_rects(sbx, sbw, row_changes_y, false);
        let act_revert_all = CodeReviewLayout::hit_test(
            &model,
            0.0,
            0.0,
            1000.0,
            800.0,
            revert_all_rect.0 + 5.0,
            revert_all_rect.1 + 5.0,
        );
        assert_eq!(act_revert_all, Some(CodeReviewAction::PromptDiscardAll));

        let stage_all_rect = stage_all_opt.unwrap();
        let act_stage_all = CodeReviewLayout::hit_test(
            &model,
            0.0,
            0.0,
            1000.0,
            800.0,
            stage_all_rect.0 + 5.0,
            stage_all_rect.1 + 5.0,
        );
        assert_eq!(act_stage_all, Some(CodeReviewAction::StageAll));

        // Test Directory group action buttons (folder with children)
        let test_diff = RepoDiff {
            files: vec![
                FileDiff {
                    path: PathBuf::from("src/app/main.rs"),
                    old_path: None,
                    status: FileGitStatus::Modified,
                    hunks: vec![],
                    additions: 1,
                    deletions: 0,
                    is_binary: false,
                    line_count: 5,
                },
                FileDiff {
                    path: PathBuf::from("staged/lib.rs"),
                    old_path: None,
                    status: FileGitStatus::Modified,
                    hunks: vec![],
                    additions: 1,
                    deletions: 0,
                    is_binary: false,
                    line_count: 5,
                },
            ],
            total_additions: 2,
            total_deletions: 0,
            files_changed: 2,
        };
        let dir_model = CodeReviewModel {
            diff: test_diff.clone(),
            unstaged_diff: test_diff,
            is_staged: false,
            sidebar_expanded: true,
            ..Default::default()
        };
        let items = dir_model.visible_tree_items();
        // Find unstaged Directory item index
        let unstaged_dir_idx = items.iter().position(|it| match it {
            CodeReviewTreeItem::Directory { is_staged, .. } => !*is_staged,
            _ => false,
        });
        if let Some(dir_idx) = unstaged_dir_idx {
            let row_dir_y = row_top + dir_idx as f32 * CodeReviewLayout::FILE_ROW_HEIGHT;
            let (dir_revert_rect, dir_stage_opt) =
                CodeReviewLayout::sidebar_group_action_rects(sbx, sbw, row_dir_y, false);
            let act_dir_stage = CodeReviewLayout::hit_test(
                &dir_model,
                0.0,
                0.0,
                1000.0,
                800.0,
                dir_stage_opt.unwrap().0 + 5.0,
                dir_stage_opt.unwrap().1 + 5.0,
            );
            assert_eq!(
                act_dir_stage,
                Some(CodeReviewAction::StageDirectoryByPath(PathBuf::from(
                    "src/app"
                )))
            );

            let act_dir_revert = CodeReviewLayout::hit_test(
                &dir_model,
                0.0,
                0.0,
                1000.0,
                800.0,
                dir_revert_rect.0 + 5.0,
                dir_revert_rect.1 + 5.0,
            );
            assert_eq!(
                act_dir_revert,
                Some(CodeReviewAction::PromptDiscardDirectory(PathBuf::from(
                    "src/app"
                )))
            );
        }

        // Test DiscardTarget::Directory confirmation dialog hit test
        let mut dir_dialog_model = dir_model.clone();
        dir_dialog_model.confirm_discard = Some(DiscardTarget::Directory(PathBuf::from("src")));
        let (_cancel_rect, confirm_rect) =
            CodeReviewLayout::discard_dialog_buttons_rect(0.0, 0.0, 1000.0, 800.0);
        let act_dir_confirm = CodeReviewLayout::hit_test(
            &dir_dialog_model,
            0.0,
            0.0,
            1000.0,
            800.0,
            confirm_rect.0 + 5.0,
            confirm_rect.1 + 5.0,
        );
        assert_eq!(act_dir_confirm, Some(CodeReviewAction::ConfirmDiscardOk));

        // Test DiscardTarget::All confirmation dialog hit test
        let mut all_dialog_model = dir_model.clone();
        all_dialog_model.confirm_discard = Some(DiscardTarget::All);
        let act_all_confirm = CodeReviewLayout::hit_test(
            &all_dialog_model,
            0.0,
            0.0,
            1000.0,
            800.0,
            confirm_rect.0 + 5.0,
            confirm_rect.1 + 5.0,
        );
        assert_eq!(act_all_confirm, Some(CodeReviewAction::ConfirmDiscardOk));
    }

    #[test]
    fn test_changes_and_staged_tree_hierarchy_consistency_and_sorting() {
        let diff_files = vec![
            FileDiff {
                path: PathBuf::from("crates/akssora-terminal-renderer/src/sidebar.rs"),
                old_path: None,
                status: FileGitStatus::Modified,
                hunks: vec![],
                additions: 10,
                deletions: 2,
                is_binary: false,
                line_count: 50,
            },
            FileDiff {
                path: PathBuf::from("crates/akssora-terminal-renderer/src/header.rs"),
                old_path: None,
                status: FileGitStatus::Modified,
                hunks: vec![],
                additions: 5,
                deletions: 0,
                is_binary: false,
                line_count: 30,
            },
        ];

        let staged_diff = RepoDiff {
            files: diff_files.clone(),
            total_additions: 15,
            total_deletions: 2,
            files_changed: 2,
        };
        let unstaged_diff = RepoDiff {
            files: diff_files,
            total_additions: 15,
            total_deletions: 2,
            files_changed: 2,
        };

        let model = CodeReviewModel {
            staged_diff,
            unstaged_diff,
            ..Default::default()
        };

        let items = model.visible_tree_items();

        // 1. Both sections must compact the directory chain identically into "crates/akssora-terminal-renderer/src"
        let staged_dir = items.iter().find(|it| {
            matches!(
                it,
                CodeReviewTreeItem::Directory {
                    is_staged: true,
                    display_name,
                    ..
                } if display_name == "crates/akssora-terminal-renderer/src"
            )
        });
        assert!(
            staged_dir.is_some(),
            "Staged directory chain should be compacted"
        );

        let unstaged_dir = items.iter().find(|it| {
            matches!(
                it,
                CodeReviewTreeItem::Directory {
                    is_staged: false,
                    display_name,
                    ..
                } if display_name == "crates/akssora-terminal-renderer/src"
            )
        });
        assert!(
            unstaged_dir.is_some(),
            "Changes directory chain should be compacted identically"
        );

        // 2. Both sections must sort files alphabetically: "header.rs" before "sidebar.rs"
        let staged_files: Vec<_> = items
            .iter()
            .filter_map(|it| match it {
                CodeReviewTreeItem::File {
                    is_staged: true,
                    display_name,
                    depth,
                    ..
                } => Some((display_name.as_str(), *depth)),
                _ => None,
            })
            .collect();
        assert_eq!(staged_files, vec![("header.rs", 2), ("sidebar.rs", 2)]);

        let unstaged_files: Vec<_> = items
            .iter()
            .filter_map(|it| match it {
                CodeReviewTreeItem::File {
                    is_staged: false,
                    display_name,
                    depth,
                    ..
                } => Some((display_name.as_str(), *depth)),
                _ => None,
            })
            .collect();
        assert_eq!(unstaged_files, vec![("header.rs", 2), ("sidebar.rs", 2)]);

        // 3. Verify indentation consistency: depths match
        if let (
            Some(CodeReviewTreeItem::Directory { depth: sd, .. }),
            Some(CodeReviewTreeItem::Directory { depth: ud, .. }),
        ) = (staged_dir, unstaged_dir)
        {
            assert_eq!(
                sd, ud,
                "Directory depths between Staged and Changes must be identical"
            );
        }
    }

    #[test]
    fn test_card_full_file_btn_geometry_and_hit_test() {
        let mut model = CodeReviewModel::default();
        let file = FileDiff {
            path: PathBuf::from("src/main.rs"),
            old_path: None,
            status: FileGitStatus::Modified,
            additions: 1,
            deletions: 1,
            hunks: Vec::new(),
            is_binary: false,
            line_count: 2,
        };
        model.diff.files.push(file);
        model.sidebar_expanded = false;

        let (vx, vy, vw, _vh) =
            CodeReviewLayout::diff_viewport_rect(&model, 0.0, 0.0, 1000.0, 800.0);
        let card_x = vx + 16.0;
        let card_w = (vw - 32.0).max(100.0);
        let card_y = vy + 12.0;

        let (fx, fy, fw, fh) = CodeReviewLayout::card_full_file_btn_rect(card_x, card_y, card_w);
        assert_eq!(
            fw, 90.0,
            "Full File button width should be 90.0px to fit label comfortably"
        );
        assert_eq!(fh, 24.0);

        let (dx, _dy, _dw, _dh) = CodeReviewLayout::card_discard_btn_rect(card_x, card_y, card_w);
        assert!(
            fx + fw <= dx,
            "Full File button right edge ({}) must be <= discard button left edge ({})",
            fx + fw,
            dx
        );
        assert_eq!(
            dx - (fx + fw),
            8.0,
            "Spacing between Full File button and discard button should be 8.0px"
        );

        let hit = CodeReviewLayout::hit_test(&model, 0.0, 0.0, 1000.0, 800.0, fx + 10.0, fy + 10.0);
        assert_eq!(hit, Some(CodeReviewAction::ToggleFullFile(0)));
    }
}
