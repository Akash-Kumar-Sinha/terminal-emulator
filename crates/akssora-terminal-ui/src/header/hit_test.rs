use super::{Header, HeaderButton, HeaderItem};

// INFO: Hit testing, cursor target detection, and titlebar window dragging region calculations.
impl Header {
    pub fn hit_test_item(
        &self,
        x: f32,
        y: f32,
        window_w: f32,
        tab_count: usize,
        show_tabs: bool,
    ) -> Option<HeaderItem> {
        self.hit_test_item_ex(x, y, window_w, tab_count, show_tabs, true, true)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn hit_test_item_ex(
        &self,
        x: f32,
        y: f32,
        window_w: f32,
        tab_count: usize,
        show_tabs: bool,
        show_toggle: bool,
        _show_header_settings: bool,
    ) -> Option<HeaderItem> {
        let total_h = self.total_height(show_tabs);
        if y < 0.0 || y > total_h {
            return None;
        }

        // Zone 1: Main Application Header (y < self.height)
        if y < self.height {
            // 1. Sidebar toggle (only when sidebar layout is active)
            if show_toggle {
                let (stx, sty, stw, sth) = self.sidebar_toggle_rect();
                if x >= stx && x < stx + stw && y >= sty && y < sty + sth {
                    return Some(HeaderItem::SidebarToggle);
                }
            }

            // 1.1 Explorer toggle button (beside the sidebar toggle button)
            let (etx, ety, etw, eth) = self.explorer_toggle_rect();
            if x >= etx && x < etx + etw && y >= ety && y < ety + eth {
                return Some(HeaderItem::ExplorerToggle);
            }

            // 1.2 Center quick search bar
            let (qs_x, qs_y, qs_w, qs_h) = self.quick_search_rect(window_w);
            if x >= qs_x && x < qs_x + qs_w && y >= qs_y && y < qs_y + qs_h {
                return Some(HeaderItem::QuickSearch);
            }

            // 1.3 Left-side git branch button (beside branding)
            if self.is_git_repo {
                let (br_x, br_y, br_w, br_h) = self.branch_button_rect(show_toggle, true);
                if br_w > 0.0 && x >= br_x && x < br_x + br_w && y >= br_y && y < br_y + br_h {
                    return Some(HeaderItem::BranchDropdown);
                }
            }

            // 1.4 Left-side branch dropdown popup (when open and multiple branches exist)
            if self.branch_dropdown_open && self.is_git_repo && self.git_branches.len() > 1 {
                let (mx, my, mw, mh) = self.branch_dropdown_rect(show_toggle, true);
                if x >= mx && x <= mx + mw && y >= my && y <= my + mh {
                    for i in 0..self.git_branches.len() {
                        let (ix, iy, iw, ih) = self.branch_dropdown_item_rect(i, show_toggle, true);
                        if x >= ix && x <= ix + iw && y >= iy && y <= iy + ih {
                            return Some(HeaderItem::BranchItem(i));
                        }
                    }
                    return Some(HeaderItem::BranchDropdown);
                }
            }

            // 2. Window control buttons
            for btn in [
                HeaderButton::Close,
                HeaderButton::Maximize,
                HeaderButton::Minimize,
            ] {
                let (bx, by, bw, bh) = self.button_rect(btn, window_w);
                if x >= bx && x < bx + bw && y >= by && y < by + bh {
                    return Some(HeaderItem::WindowButton(btn));
                }
            }

            // 3. Single settings button permanently visible in the corner
            let (sx, sy, sw, sh) = self.settings_button_rect(window_w);
            if x >= sx && x < sx + sw && y >= sy && y < sy + sh {
                return Some(HeaderItem::Settings);
            }

            // 4. Code review stats button beside settings (only if git repo is initialized)
            if self.is_git_repo {
                let (cr_x, cr_y, cr_w, cr_h) = self.code_review_button_rect(window_w);
                if cr_w > 0.0 && x >= cr_x && x < cr_x + cr_w && y >= cr_y && y < cr_y + cr_h {
                    return Some(HeaderItem::CodeReview);
                }
            }

            return None;
        }

        // Zone 2: Terminal Tab Bar directly below header (y >= self.height && y < self.height + TAB_BAR_HEIGHT)
        if show_tabs {
            for i in 0..tab_count {
                if let Some((cx, cy, cw, ch)) = self.tab_close_rect(i, tab_count, window_w)
                    && x >= cx
                    && x < cx + cw
                    && y >= cy
                    && y < cy + ch
                {
                    return Some(HeaderItem::TabClose(i));
                }
                if let Some((tx, ty, tw, th)) = self.tab_rect(i, tab_count, window_w)
                    && x >= tx
                    && x < tx + tw
                    && y >= ty
                    && y < ty + th
                {
                    return Some(HeaderItem::Tab(i));
                }
            }

            let (nx, ny, nw, nh) = self.new_tab_rect(tab_count, window_w);
            if x >= nx && x < nx + nw && y >= ny && y < ny + nh {
                return Some(HeaderItem::NewTab);
            }
        }

        None
    }

    pub fn hit_test(&self, x: f32, y: f32, window_w: f32) -> Option<HeaderButton> {
        if y < 0.0 || y > self.height {
            return None;
        }
        for btn in [
            HeaderButton::Minimize,
            HeaderButton::Maximize,
            HeaderButton::Close,
        ] {
            let (bx, by, bw, bh) = self.button_rect(btn, window_w);
            if x >= bx && x < bx + bw && y >= by && y < by + bh {
                return Some(btn);
            }
        }
        None
    }

    pub fn in_drag_region(&self, x: f32, y: f32, window_w: f32) -> bool {
        y >= 0.0 && y <= self.height && self.hit_test_item(x, y, window_w, 0, false).is_none()
    }

    pub fn in_drag_region_with_tabs(
        &self,
        x: f32,
        y: f32,
        window_w: f32,
        tab_count: usize,
        show_tabs: bool,
    ) -> bool {
        let total_h = self.total_height(show_tabs);
        y >= 0.0
            && y <= total_h
            && self
                .hit_test_item(x, y, window_w, tab_count, show_tabs)
                .is_none()
    }

    pub fn icon(btn: HeaderButton) -> char {
        match btn {
            HeaderButton::Minimize => '\u{2212}',
            HeaderButton::Maximize => '\u{25A1}',
            HeaderButton::Close => '\u{00D7}',
        }
    }
}
