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

    // INFO: Bounding box for the staged vs unstaged code toggle button in the top header.
    pub fn stage_toggle_btn_rect(ws_x: f32, ws_y: f32) -> (f32, f32, f32, f32) {
        (ws_x + 50.0, ws_y + 8.0, 180.0, 26.0)
    }

    // INFO: Bounding box for the toggle collapse/expand all button in the top header.
    pub fn toggle_collapse_all_btn_rect(ws_x: f32, ws_y: f32) -> (f32, f32, f32, f32) {
        (ws_x + 236.0, ws_y + 8.0, 120.0, 26.0)
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
        (ws_x + ws_w - 96.0, ws_y + 8.0, 84.0, 26.0)
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
        (card_x + card_w - 168.0, card_y + 6.0, 74.0, 24.0)
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
        // INFO: Top Header bar action hit-testing.
        if click_y >= ws_y && click_y <= ws_y + Self::HEADER_HEIGHT {
            let (sx, sy, sw, sh) = Self::sidebar_toggle_btn_rect(ws_x, ws_y);
            if click_x >= sx && click_x <= sx + sw && click_y >= sy && click_y <= sy + sh {
                return Some(CodeReviewAction::ToggleSidebar);
            }

            let (ux, uy, uw, uh) = Self::stage_toggle_btn_rect(ws_x, ws_y);
            if click_x >= ux && click_x <= ux + uw && click_y >= uy && click_y <= uy + uh {
                return Some(CodeReviewAction::ToggleStaged);
            }

            let (tx, ty, tw, th) = Self::toggle_collapse_all_btn_rect(ws_x, ws_y);
            if click_x >= tx && click_x <= tx + tw && click_y >= ty && click_y <= ty + th {
                return Some(CodeReviewAction::ToggleCollapseAll);
            }

            let (cx, cy, cw, ch) = Self::commit_btn_rect(ws_x, ws_y, ws_w);
            if click_x >= cx && click_x <= cx + cw && click_y >= cy && click_y <= cy + ch {
                return Some(CodeReviewAction::OpenCommitDialog);
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
                        CodeReviewTreeItem::Directory { path, .. } => {
                            return Some(CodeReviewAction::ToggleTreeDir(path.clone()));
                        }
                        CodeReviewTreeItem::File { orig_idx, .. } => {
                            return Some(CodeReviewAction::SelectFileForEdit(*orig_idx));
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
}
