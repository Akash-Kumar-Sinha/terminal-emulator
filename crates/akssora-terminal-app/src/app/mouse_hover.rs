use akssora_terminal_ui::{HeaderItem, SettingsView, Sidebar, TabKind};
use winit::window::CursorIcon;

use super::App;

// INFO: Hover state calculation, hit-testing across UI components, and dynamic cursor icon updates.
impl App {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn update_hover_states(
        &mut self,
        mx: f32,
        my: f32,
        w: f32,
        h: f32,
        top_h: f32,
        sb_w: f32,
        exp_w: f32,
        show_sidebar: bool,
        show_tabs: bool,
        show_toggle: bool,
    ) {
        // 1. Header hover
        let item = self.header.hit_test_item_ex(
            mx,
            my,
            w,
            self.tab_manager.len(),
            show_tabs,
            show_toggle,
            true,
        );
        self.header.hovered_item = item;
        self.header.hovered_branch_idx = match item {
            Some(HeaderItem::BranchItem(idx)) => Some(idx),
            _ => None,
        };
        self.header.hovered = match item {
            Some(HeaderItem::WindowButton(b)) => Some(b),
            _ => None,
        };

        // 2. Update context menu hover
        if let Some(menu) = self.context_menu.as_mut() {
            menu.update_hover(mx, my);
            if let Some(win) = self.window.as_ref() {
                win.request_redraw();
            }
        }

        // 3. Sidebar edge hover and resizing (debounced by cell width)
        if show_sidebar && self.sidebar.is_expanded {
            let on_edge = (mx - sb_w).abs() <= 5.0 && my >= top_h;
            self.sidebar.hovered_edge = on_edge;

            if self.sidebar.is_resizing {
                let clamped_w = mx.clamp(Sidebar::MIN_WIDTH, Sidebar::MAX_WIDTH);
                let diff = (clamped_w - self.sidebar.width).abs();
                self.sidebar.width = clamped_w;
                self.sidebar.saved_width = clamped_w;
                self.config.sidebar_width = clamped_w;
                let cell_w = self.fonts.as_ref().map(|f| f.cell_w).unwrap_or(9.0);
                let now = std::time::Instant::now();
                let time_ok = self
                    .last_layout_time
                    .map(|t| now.duration_since(t).as_millis() >= 16)
                    .unwrap_or(true);
                if diff >= cell_w && time_ok {
                    self.last_layout_time = Some(now);
                    self.layout_blocks();
                }
            }
        } else {
            self.sidebar.hovered_edge = false;
        }

        // 4. Explorer edge hover and resizing
        if self.explorer.is_expanded {
            let exp_right = sb_w + exp_w;
            let on_exp_edge = (mx - exp_right).abs() <= 5.0 && my >= top_h;
            self.explorer.hovered_edge = on_exp_edge;

            if self.explorer.is_resizing {
                let clamped_w = (mx - sb_w).clamp(
                    akssora_terminal_ui::ExplorerUi::MIN_WIDTH,
                    akssora_terminal_ui::ExplorerUi::MAX_WIDTH,
                );
                self.explorer.width = clamped_w;
                self.explorer.saved_width = clamped_w;
                self.layout_blocks();
            }

            if mx >= sb_w && mx <= exp_right && my >= top_h {
                self.explorer.hovered_toolbar =
                    self.explorer.hit_test_toolbar_at(mx, my, top_h, sb_w);
                let rel_y = my
                    - (top_h + akssora_terminal_ui::ExplorerUi::HEADER_HEIGHT
                        - self.explorer.scroll_y);
                if rel_y >= 0.0 {
                    let row_idx = (rel_y / akssora_terminal_ui::ExplorerUi::ITEM_HEIGHT) as usize;
                    let rows = self.explorer.tree.flatten_rows();
                    if row_idx < rows.len() {
                        self.explorer.hovered_row = Some(row_idx);
                        if !self.explorer.tree.dragged_paths.is_empty() {
                            let target_p = &rows[row_idx].path;
                            let target_dir = if rows[row_idx].kind
                                == akssora_file_explorer::EntryKind::Directory
                            {
                                target_p.clone()
                            } else {
                                target_p
                                    .parent()
                                    .unwrap_or(&self.explorer.tree.root_path)
                                    .to_path_buf()
                            };
                            self.explorer.tree.update_drop_target(Some(target_dir));
                        }
                    } else {
                        self.explorer.hovered_row = None;
                    }
                }
            } else {
                self.explorer.hovered_toolbar = None;
                self.explorer.hovered_row = None;
            }
        }

        // 5. Sidebar sessions and tabs hit test
        let content_start_y = top_h + 8.0;
        if show_sidebar && self.sidebar.is_expanded && mx <= sb_w && !self.sidebar.hovered_edge {
            self.sidebar.hovered_toggle = false;
            let items = self.collect_session_items();
            let target = self
                .sidebar
                .hit_test_sessions(mx, my, &items, content_start_y)
                .or_else(|| {
                    self.sidebar.hit_test(
                        mx,
                        my,
                        self.tab_manager.tabs(),
                        &self.split_trees,
                        content_start_y,
                    )
                });
            match target {
                Some(akssora_terminal_ui::SidebarTarget::Tab(idx)) => {
                    self.sidebar.hovered_tab = Some(idx);
                    self.sidebar.hovered_close = None;
                    self.sidebar.hovered_pane = None;
                    self.sidebar.hovered_pane_close = None;
                    self.sidebar.hovered_collapse_tab = None;
                    self.sidebar.hovered_new_tab = false;
                }
                Some(akssora_terminal_ui::SidebarTarget::TabCollapse(idx)) => {
                    self.sidebar.hovered_tab = Some(idx);
                    self.sidebar.hovered_close = None;
                    self.sidebar.hovered_pane = None;
                    self.sidebar.hovered_pane_close = None;
                    self.sidebar.hovered_collapse_tab = Some(idx);
                    self.sidebar.hovered_new_tab = false;
                }
                Some(akssora_terminal_ui::SidebarTarget::TabClose(idx)) => {
                    self.sidebar.hovered_tab = Some(idx);
                    self.sidebar.hovered_close = Some(idx);
                    self.sidebar.hovered_pane = None;
                    self.sidebar.hovered_pane_close = None;
                    self.sidebar.hovered_collapse_tab = None;
                    self.sidebar.hovered_new_tab = false;
                }
                Some(akssora_terminal_ui::SidebarTarget::SplitPane { tab_idx, pane_id }) => {
                    self.sidebar.hovered_tab = Some(tab_idx);
                    self.sidebar.hovered_close = None;
                    self.sidebar.hovered_pane = Some(pane_id);
                    self.sidebar.hovered_pane_close = None;
                    self.sidebar.hovered_collapse_tab = None;
                    self.sidebar.hovered_new_tab = false;
                }
                Some(akssora_terminal_ui::SidebarTarget::SplitClose { tab_idx, pane_id }) => {
                    self.sidebar.hovered_tab = Some(tab_idx);
                    self.sidebar.hovered_close = None;
                    self.sidebar.hovered_pane = Some(pane_id);
                    self.sidebar.hovered_pane_close = Some(pane_id);
                    self.sidebar.hovered_collapse_tab = None;
                    self.sidebar.hovered_new_tab = false;
                }
                Some(akssora_terminal_ui::SidebarTarget::Session { tab_idx, pane_id }) => {
                    self.sidebar.hovered_tab = Some(tab_idx);
                    self.sidebar.hovered_close = None;
                    self.sidebar.hovered_pane = Some(pane_id);
                    self.sidebar.hovered_pane_close = None;
                    self.sidebar.hovered_collapse_tab = None;
                    self.sidebar.hovered_new_tab = false;
                }
                Some(akssora_terminal_ui::SidebarTarget::SessionClose { tab_idx, pane_id }) => {
                    self.sidebar.hovered_tab = Some(tab_idx);
                    self.sidebar.hovered_close = None;
                    self.sidebar.hovered_pane = Some(pane_id);
                    self.sidebar.hovered_pane_close = Some(pane_id);
                    self.sidebar.hovered_collapse_tab = None;
                    self.sidebar.hovered_new_tab = false;
                }
                Some(akssora_terminal_ui::SidebarTarget::ProjectCollapse(_)) => {
                    self.sidebar.hovered_tab = None;
                    self.sidebar.hovered_close = None;
                    self.sidebar.hovered_pane = None;
                    self.sidebar.hovered_pane_close = None;
                    self.sidebar.hovered_collapse_tab = None;
                    self.sidebar.hovered_new_tab = false;
                }
                Some(akssora_terminal_ui::SidebarTarget::NewTab) => {
                    self.sidebar.hovered_tab = None;
                    self.sidebar.hovered_close = None;
                    self.sidebar.hovered_pane = None;
                    self.sidebar.hovered_pane_close = None;
                    self.sidebar.hovered_collapse_tab = None;
                    self.sidebar.hovered_new_tab = true;
                }
                None => {
                    self.sidebar.hovered_tab = None;
                    self.sidebar.hovered_close = None;
                    self.sidebar.hovered_pane = None;
                    self.sidebar.hovered_pane_close = None;
                    self.sidebar.hovered_collapse_tab = None;
                    self.sidebar.hovered_new_tab = false;
                }
            }
        } else {
            self.sidebar.hovered_toggle = false;
            self.sidebar.hovered_tab = None;
            self.sidebar.hovered_close = None;
            self.sidebar.hovered_pane = None;
            self.sidebar.hovered_pane_close = None;
            self.sidebar.hovered_collapse_tab = None;
            self.sidebar.hovered_new_tab = false;
        }

        // 6. Hit test settings actions if settings tab is active
        let is_settings = self
            .tab_manager
            .active_tab()
            .map(|t| matches!(t.kind, TabKind::Settings))
            .unwrap_or(false);
        if is_settings {
            let exp_w = self.explorer.effective_width();
            let ws_x = sb_w + exp_w;
            let cell_h = self.ui_fonts.as_ref().map(|f| f.cell_h).unwrap_or(18.0);
            self.hovered_setting_action = SettingsView::hit_test_action_tab(
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
        } else {
            self.hovered_setting_action = None;
        }

        // 7. Track hover state for editor file-tab bar
        let ws_x = sb_w + exp_w;
        let is_editor = self
            .tab_manager
            .active_tab()
            .map(|t| matches!(t.kind, TabKind::Editor { .. }))
            .unwrap_or(false);
        let tab_bar_h = if !self.editor_open_files.is_empty() {
            akssora_terminal_ui::EditorTabBar::HEIGHT
        } else {
            0.0
        };

        if is_editor && tab_bar_h > 0.0 && mx >= ws_x && my >= top_h && my < top_h + tab_bar_h {
            if let Some(target) = akssora_terminal_ui::EditorTabBar::hit_test(
                mx,
                my,
                ws_x,
                top_h,
                (w - ws_x).max(0.0),
                self.editor_open_files.len(),
            ) {
                match target {
                    akssora_terminal_ui::EditorTabTarget::Tab(idx) => {
                        self.editor_hovered_tab = Some(idx);
                        self.editor_hovered_close = None;
                        self.editor_hovered_toolbar = None;
                    }
                    akssora_terminal_ui::EditorTabTarget::Pin(idx) => {
                        self.editor_hovered_tab = Some(idx);
                        self.editor_hovered_close = None;
                        self.editor_hovered_toolbar = None;
                    }
                    akssora_terminal_ui::EditorTabTarget::Close(idx) => {
                        self.editor_hovered_tab = Some(idx);
                        self.editor_hovered_close = Some(idx);
                        self.editor_hovered_toolbar = None;
                    }
                    akssora_terminal_ui::EditorTabTarget::FormatDocument => {
                        self.editor_hovered_tab = None;
                        self.editor_hovered_close = None;
                        self.editor_hovered_toolbar =
                            Some(akssora_terminal_ui::EditorToolbarAction::FormatDocument);
                    }
                    akssora_terminal_ui::EditorTabTarget::TogglePreview => {
                        self.editor_hovered_tab = None;
                        self.editor_hovered_close = None;
                        self.editor_hovered_toolbar =
                            Some(akssora_terminal_ui::EditorToolbarAction::TogglePreview);
                    }
                    akssora_terminal_ui::EditorTabTarget::ToggleSplitPreview => {
                        self.editor_hovered_tab = None;
                        self.editor_hovered_close = None;
                        self.editor_hovered_toolbar =
                            Some(akssora_terminal_ui::EditorToolbarAction::ToggleSplitPreview);
                    }
                    akssora_terminal_ui::EditorTabTarget::ToggleInlineDiff => {
                        self.editor_hovered_tab = None;
                        self.editor_hovered_close = None;
                        self.editor_hovered_toolbar =
                            Some(akssora_terminal_ui::EditorToolbarAction::ToggleInlineDiff);
                    }
                }
            } else {
                self.editor_hovered_tab = None;
                self.editor_hovered_close = None;
                self.editor_hovered_toolbar = None;
            }
        } else {
            self.editor_hovered_tab = None;
            self.editor_hovered_close = None;
            self.editor_hovered_toolbar = None;
        }

        // 7b. Track hover state for Editor Git gutter actions
        let content_y = top_h + tab_bar_h;
        if is_editor && my >= content_y && my < h - 22.0 {
            let split_limit = if let Some(doc) = self.active_editor() {
                if doc.markdown_mode == akssora_editor::MarkdownViewMode::Split {
                    ws_x + ((w - ws_x - 1.0).max(0.0) * 0.5).floor()
                } else {
                    w
                }
            } else {
                w
            };

            let cell_w = self.fonts.as_ref().map(|f| f.cell_w).unwrap_or(9.0);
            let cell_h = self.fonts.as_ref().map(|f| f.cell_h).unwrap_or(18.0);

            if let Some(doc) = self.active_editor() {
                self.editor_hovered_gutter = doc.hit_test_gutter_hover(
                    mx,
                    my,
                    ws_x,
                    content_y,
                    (split_limit - ws_x).max(0.0),
                    cell_w,
                    cell_h,
                );
            } else {
                self.editor_hovered_gutter = None;
            }
        } else {
            self.editor_hovered_gutter = None;
        }

        // 8. Track hover state for Code Review actions
        let is_code_review = self
            .tab_manager
            .active_tab()
            .map(|t| matches!(t.kind, TabKind::CodeReview { .. }))
            .unwrap_or(false);
        if is_code_review {
            let ws_y = top_h;
            let ws_w = (w - ws_x).max(100.0);
            let ws_h = (h - ws_y).max(100.0);
            if let Some((sbx, sby, sbw, _)) = akssora_terminal_ui::CodeReviewLayout::sidebar_rect(
                &self.code_review,
                ws_x,
                ws_y,
                ws_w,
                ws_h,
            ) {
                let on_edge = (mx - (sbx + sbw)).abs() <= 5.0 && my >= sby;
                self.code_review.hovered_sidebar_edge = on_edge;
                if self.code_review.is_resizing_sidebar {
                    self.code_review.sidebar_width = (mx - sbx).clamp(180.0, ws_w * 0.6);
                }
            } else {
                self.code_review.hovered_sidebar_edge = false;
            }

            self.hovered_code_review_action = akssora_terminal_ui::CodeReviewLayout::hit_test(
                &self.code_review,
                ws_x,
                ws_y,
                ws_w,
                ws_h,
                mx,
                my,
            );

            let gutter_hover = akssora_terminal_ui::CodeReviewLayout::hit_test_gutter_hover(
                &self.code_review,
                ws_x,
                ws_y,
                ws_w,
                ws_h,
                mx,
                my,
            );
            self.code_review.hovered_gutter = gutter_hover;
            self.editor_hovered_gutter = gutter_hover.map(|h| (h.row, h.action));
        } else {
            self.hovered_code_review_action = None;
            self.code_review.hovered_sidebar_edge = false;
            self.code_review.hovered_gutter = None;
        }

        // 9. Update active tooltip
        self.update_tooltips(mx, my, w, h);
    }

    pub fn is_button_hovered(&self, mx: f32, my: f32, w: f32, h: f32, _top_h: f32) -> bool {
        // 1. Unsaved prompt modal buttons
        if self.unsaved_prompt_target.is_some() {
            let modal_w = 480.0f32.min(w - 40.0);
            let modal_h = 160.0f32;
            let mx_box = (w - modal_w) / 2.0;
            let my_box = (h - modal_h) / 2.0;
            let btn_y = my_box + modal_h - 48.0;
            let btn_h = 32.0;
            let btn_gap = 12.0;
            let is_all = matches!(
                self.unsaved_prompt_target,
                Some(super::UnsavedCloseAction::CloseAll)
            );
            let save_w = if is_all { 145.0 } else { 125.0 };
            let dont_w = 135.0;
            let cancel_w = 125.0;
            let total_buttons_w = save_w + dont_w + cancel_w + (btn_gap * 2.0);
            let start_btn_x = mx_box + (modal_w - total_buttons_w) / 2.0;
            let save_x = start_btn_x;
            let dont_x = save_x + save_w + btn_gap;
            let cancel_x = dont_x + dont_w + btn_gap;
            if (mx >= save_x && mx < save_x + save_w && my >= btn_y && my < btn_y + btn_h)
                || (mx >= dont_x && mx < dont_x + dont_w && my >= btn_y && my < btn_y + btn_h)
                || (mx >= cancel_x && mx < cancel_x + cancel_w && my >= btn_y && my < btn_y + btn_h)
            {
                return true;
            }
        }

        // 2. Header bar items (window controls, tabs, settings, code review, toggles)
        if let Some(item) = self.header.hovered_item {
            match item {
                akssora_terminal_ui::HeaderItem::BranchDropdown
                | akssora_terminal_ui::HeaderItem::BranchItem(_) => {}
                _ => return true,
            }
        }

        // 3. Settings actions (tab selection, toggles, buttons)
        if let Some(action) = self.hovered_setting_action {
            match action {
                akssora_terminal_ui::SettingsAction::ToggleLayoutDropdown
                | akssora_terminal_ui::SettingsAction::SelectLayout(_)
                | akssora_terminal_ui::SettingsAction::ToggleLineNumberDropdown
                | akssora_terminal_ui::SettingsAction::SelectLineNumberMode(_) => {}
                _ => return true,
            }
        }

        // 4. Code review actions (sidebar toggle, staged toggle, collapse/expand all, commit, card actions, file selection)
        if let Some(action) = &self.hovered_code_review_action {
            match action {
                akssora_terminal_ui::CodeReviewAction::ClickEditor {
                    orig_idx,
                    click_x,
                    click_y,
                    body_x,
                    body_y,
                    body_w,
                } => {
                    if let Some(file) = self.code_review.diff.files.get(*orig_idx) {
                        let full_path = self.code_review.repo_root.join(&file.path);
                        if let Some(doc) = self.editor_documents.get(&full_path) {
                            let cell_h = self.fonts.as_ref().map(|f| f.cell_h).unwrap_or(18.0);
                            let cell_w = self.fonts.as_ref().map(|f| f.cell_w).unwrap_or(8.0);
                            if doc
                                .hit_test_gutter_action(
                                    *click_x, *click_y, *body_x, *body_y, *body_w, cell_w, cell_h,
                                )
                                .is_some()
                            {
                                return true;
                            }
                        }
                    }
                }
                akssora_terminal_ui::CodeReviewAction::ScrollDiff(_)
                | akssora_terminal_ui::CodeReviewAction::ScrollSidebar(_) => {}
                _ => return true,
            }
        }

        // 5. Editor tab bar tabs, pin/close buttons, toolbar action buttons, and gutter action buttons
        if self.editor_hovered_tab.is_some()
            || self.editor_hovered_close.is_some()
            || self.editor_hovered_toolbar.is_some()
            || self
                .editor_hovered_gutter
                .and_then(|(_, act)| act)
                .is_some()
            || self
                .code_review
                .hovered_gutter
                .and_then(|h| h.action)
                .is_some()
        {
            return true;
        }

        // 6. Explorer toolbar buttons (new file, new folder, refresh, collapse)
        if self.explorer.hovered_toolbar.is_some() {
            return true;
        }

        // 7. Context menu items
        if self
            .context_menu
            .as_ref()
            .map(|m| m.hovered_idx.is_some())
            .unwrap_or(false)
        {
            return true;
        }

        false
    }

    pub fn determine_cursor_icon(
        &self,
        mx: f32,
        my: f32,
        w: f32,
        h: f32,
        top_h: f32,
        sb_w: f32,
    ) -> CursorIcon {
        if self.sidebar.is_resizing
            || self.sidebar.hovered_edge
            || self.explorer.is_resizing
            || self.explorer.hovered_edge
            || self.code_review.is_resizing_sidebar
            || self.code_review.hovered_sidebar_edge
        {
            CursorIcon::ColResize
        } else if self.split_manager.pane_drag.is_some() {
            CursorIcon::Grabbing
        } else if let Some(drag) = &self.split_manager.drag_state {
            match drag.direction {
                akssora_terminal_ui::SplitDirection::Vertical => CursorIcon::ColResize,
                akssora_terminal_ui::SplitDirection::Horizontal => CursorIcon::RowResize,
            }
        } else if self.is_button_hovered(mx, my, w, h, top_h) {
            CursorIcon::Pointer
        } else if let Some(active_tab_id) = self.tab_manager.active_id()
            && let Some(tree) = self.split_trees.get(&active_tab_id)
        {
            let content_x = sb_w;
            let content_w = (w - sb_w).max(0.0);
            let content_y = top_h;
            let content_h = (h - top_h).max(0.0);
            if let Some((dir, _, _, _)) = self
                .split_manager
                .hit_test_divider(tree, mx, my, content_x, content_y, content_w, content_h)
            {
                match dir {
                    akssora_terminal_ui::SplitDirection::Vertical => CursorIcon::ColResize,
                    akssora_terminal_ui::SplitDirection::Horizontal => CursorIcon::RowResize,
                }
            } else {
                CursorIcon::Default
            }
        } else {
            CursorIcon::Default
        }
    }
}
