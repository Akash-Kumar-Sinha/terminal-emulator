use super::super::App;

// INFO: Mouse click routing and action handling for Code Review panel.
impl App {
    pub fn handle_code_review_click(
        &mut self,
        mx: f32,
        my: f32,
        orig_w: f32,
        orig_h: f32,
        orig_top_h: f32,
        orig_ws_x: f32,
    ) -> bool {
        // 4.5 Code Review interaction clicks
        if !self.is_code_review_open() {
            return false;
        }

        let ws_w = (orig_w - orig_ws_x).max(0.0);
        let ws_h = (orig_h - orig_top_h).max(0.0);

        let (cr_x, cr_y, cr_w, cr_h) = if !self.code_review.is_full_view
            && !self.active_tab_blocks().is_empty()
        {
            let total_w = (ws_w - akssora_terminal_ui::PaneTree::DIVIDER_THICKNESS).max(0.0);
            let min_w = 120.0_f32.min(total_w * 0.45);
            let ratio = self.code_review.split_ratio.clamp(0.15, 0.85);
            let term_w = (total_w * (1.0 - ratio)).max(min_w).min(total_w - min_w);
            let div_x = orig_ws_x + term_w;
            let x = div_x + akssora_terminal_ui::PaneTree::DIVIDER_THICKNESS;
            let width = (ws_w - term_w - akssora_terminal_ui::PaneTree::DIVIDER_THICKNESS).max(0.0);
            (x, orig_top_h, width, ws_h)
        } else {
            (orig_ws_x, orig_top_h, ws_w, ws_h)
        };

        if mx < cr_x || mx > cr_x + cr_w || my < cr_y || my > cr_y + cr_h {
            return false;
        }
        self.header.is_code_review_focused = true;

        let ws_x = cr_x;
        let top_h = cr_y;
        let w = cr_x + cr_w;
        let h = cr_y + cr_h;

        if self.code_review.hovered_sidebar_edge {
            self.code_review.is_resizing_sidebar = true;
            return true;
        }

        if true {
            let (vx, vy, vw, vh) = akssora_terminal_ui::CodeReviewLayout::diff_viewport_rect(
                &self.code_review,
                ws_x,
                top_h,
                (w - ws_x).max(0.0),
                (h - top_h).max(0.0),
            );
            let (stx, sty, stw, sth) =
                akssora_terminal_ui::CodeReviewLayout::diff_scrollbar_rect(vx, vy, vw, vh);
            if mx >= stx && mx <= stx + stw && my >= sty && my <= sty + sth {
                let max_scroll = self.code_review.max_diff_scroll(vh);
                let ratio = ((my - sty) / sth).clamp(0.0, 1.0);
                self.code_review.diff_scroll_y = ratio * max_scroll;
                self.sync_code_review_documents_scroll(vh);
                self.code_review.is_dragging_diff_scrollbar = true;
                self.code_review.diff_scrollbar_drag_start_y = my;
                self.code_review.diff_scrollbar_drag_start_scroll = self.code_review.diff_scroll_y;
                return true;
            }

            // 4.6 Floating line action popup click interception (Stage Change & Revert Changes buttons above line number)
            if let Some(hover) = self.code_review.hovered_gutter
                && let Some(geom) =
                    akssora_terminal_ui::CodeReviewLayout::line_action_popup_geometry(
                        &self.code_review,
                        ws_x,
                        top_h,
                        (w - ws_x).max(0.0),
                        (h - top_h).max(0.0),
                        hover.file_idx,
                        hover.row,
                        hover.is_deletion,
                    )
            {
                let (ax, ay, aw, ah) = geom.add_btn_rect;
                if mx >= ax && mx <= ax + aw && my >= ay && my <= ay + ah {
                    self.stage_single_line_in_code_review(
                        hover.file_idx,
                        hover.row,
                        hover.is_deletion,
                    );
                    return true;
                }
                let (rx, ry, rw, rh) = geom.remove_btn_rect;
                if mx >= rx && mx <= rx + rw && my >= ry && my <= ry + rh {
                    self.revert_single_line_in_code_review(
                        hover.file_idx,
                        hover.row,
                        hover.is_deletion,
                    );
                    return true;
                }
                let (px, py, pw, ph) = geom.popup_rect;
                if mx >= px && mx <= px + pw && my >= py && my <= py + ph {
                    return true;
                }
            }
        }

        if let Some(action) = akssora_terminal_ui::CodeReviewLayout::hit_test(
            &self.code_review,
            ws_x,
            top_h,
            (w - ws_x).max(0.0),
            (h - top_h).max(0.0),
            mx,
            my,
        ) {
            match action {
                akssora_terminal_ui::CodeReviewAction::SelectFile(idx) => {
                    self.code_review.select_file(idx);
                    self.prepare_code_review_editor_file(idx);
                }
                akssora_terminal_ui::CodeReviewAction::SelectFileForEdit(idx) => {
                    if let Some(file) = self.code_review.diff.files.get(idx) {
                        self.code_review.collapsed_files.remove(&file.path);
                    }
                    self.code_review.select_file(idx);
                    self.prepare_code_review_editor_file(idx);
                    let (_, _, _, dvh) = akssora_terminal_ui::CodeReviewLayout::diff_viewport_rect(
                        &self.code_review,
                        ws_x,
                        top_h,
                        (w - ws_x).max(0.0),
                        (h - top_h).max(0.0),
                    );
                    let offset = self.code_review.card_y_offset_for_file(idx, dvh);
                    let max_scroll = self.code_review.max_diff_scroll(dvh);
                    if offset < self.code_review.diff_scroll_y
                        || offset > self.code_review.diff_scroll_y + dvh
                    {
                        self.code_review.diff_scroll_y = offset.min(max_scroll).max(0.0);
                    }
                }
                akssora_terminal_ui::CodeReviewAction::ClickEditor {
                    orig_idx,
                    click_x,
                    click_y,
                    body_x,
                    body_y,
                    body_w: _,
                } => {
                    self.code_review.select_file(orig_idx);
                    self.prepare_code_review_editor_file(orig_idx);
                    let cell_w = self.fonts.as_ref().map(|f| f.cell_w).unwrap_or(9.0);
                    let _cell_h = self.fonts.as_ref().map(|f| f.cell_h).unwrap_or(18.0);
                    let is_full = self
                        .code_review
                        .diff
                        .files
                        .get(orig_idx)
                        .map(|f| self.code_review.is_full_file(&f.path))
                        .unwrap_or(false);

                    if !is_full {
                        let hit_info = if let Some(file) = self.code_review.diff.files.get(orig_idx)
                        {
                            let mut cur_y = body_y + 8.0;
                            let line_h = akssora_terminal_ui::CodeReviewLayout::DIFF_LINE_HEIGHT;
                            let hunk_hdr_h =
                                akssora_terminal_ui::CodeReviewLayout::UNMODIFIED_LINES_HEIGHT;
                            let mut found_row = None;
                            for hunk in &file.hunks {
                                cur_y += hunk_hdr_h;
                                for (l_idx, diff_line) in hunk.lines.iter().enumerate() {
                                    if click_y >= cur_y && click_y < cur_y + line_h {
                                        let target = if let Some(n) = diff_line.new_line_num {
                                            Some(n.saturating_sub(1))
                                        } else {
                                            // Deleted line: search forward then backward for nearest line with new_line_num
                                            hunk.lines[l_idx..]
                                                .iter()
                                                .find_map(|l| l.new_line_num)
                                                .or_else(|| {
                                                    hunk.lines[..l_idx]
                                                        .iter()
                                                        .rev()
                                                        .find_map(|l| l.new_line_num)
                                                })
                                                .map(|n| n.saturating_sub(1))
                                        };
                                        found_row = target;
                                        break;
                                    }
                                    cur_y += line_h;
                                }
                                if found_row.is_some() {
                                    break;
                                }
                            }
                            let first_hunk_target = file.hunks.first().and_then(|h| {
                                h.lines
                                    .iter()
                                    .find(|l| l.line_type == akssora_git::DiffLineType::Add)
                                    .or_else(|| h.lines.first())
                                    .and_then(|l| l.new_line_num.or(l.old_line_num))
                                    .map(|n| n.saturating_sub(1))
                            });
                            Some((found_row, first_hunk_target, file.path.clone()))
                        } else {
                            None
                        };

                        if let Some((found_row, first_hunk_target, _file_path)) = hit_info {
                            let curr_row = found_row.or(first_hunk_target).unwrap_or(0);
                            let text_x = body_x + 48.0 + 8.0;
                            let col = if click_x < text_x {
                                0
                            } else {
                                ((click_x - text_x).max(0.0) / cell_w).round() as usize
                            };
                            if let Some(doc) = self.active_editor_mut() {
                                let total = doc.buffer.line_count();
                                let safe_row = curr_row.min(total.saturating_sub(1));
                                let line_len = doc.buffer.line_len(safe_row);
                                doc.cursor = (safe_row, col.min(line_len));
                                self.mouse_press_pos = Some((click_x, click_y));
                            }
                            if let Some(win) = self.window.as_ref() {
                                win.request_redraw();
                            }
                            return true;
                        }
                    }

                    if is_full {
                        let text_x = body_x + 48.0 + 8.0;
                        let line_h = akssora_terminal_ui::CodeReviewLayout::DIFF_LINE_HEIGHT;
                        let rel_y = click_y - (body_y + 8.0);
                        let row = (rel_y / line_h).floor().max(0.0) as usize;
                        let col = if click_x < text_x {
                            0
                        } else {
                            ((click_x - text_x).max(0.0) / cell_w).round() as usize
                        };
                        if let Some(doc) = self.active_editor_mut() {
                            let total = doc.buffer.line_count();
                            let safe_row = row.min(total.saturating_sub(1));
                            let line_len = doc.buffer.line_len(safe_row);
                            doc.cursor = (safe_row, col.min(line_len));
                            self.mouse_press_pos = Some((click_x, click_y));
                        }
                        if let Some(win) = self.window.as_ref() {
                            win.request_redraw();
                        }
                        return true;
                    }
                }
                akssora_terminal_ui::CodeReviewAction::SaveFile(idx) => {
                    if let Some(file) = self.code_review.diff.files.get(idx) {
                        let full_path = self.code_review.repo_root.join(&file.path);
                        if let Some(doc) = self.editor_documents.get_mut(&full_path) {
                            let _ = doc.save();
                        }
                        self.code_review.refresh();
                    }
                }
                akssora_terminal_ui::CodeReviewAction::ToggleSidebarMode => {
                    self.code_review.toggle_sidebar_mode();
                }
                akssora_terminal_ui::CodeReviewAction::ToggleTreeDir(path) => {
                    self.code_review.toggle_tree_dir(path);
                }
                akssora_terminal_ui::CodeReviewAction::ToggleFileCollapse(idx) => {
                    if let Some(file) = self.code_review.diff.files.get(idx) {
                        let path = file.path.clone();
                        let was_collapsed = self.code_review.is_file_collapsed(&path);

                        let (_, vy, _, dvh) =
                            akssora_terminal_ui::CodeReviewLayout::diff_viewport_rect(
                                &self.code_review,
                                ws_x,
                                top_h,
                                (w - ws_x).max(0.0),
                                (h - top_h).max(0.0),
                            );
                        let offset = self.code_review.card_y_offset_for_file(idx, dvh);
                        let card_top = vy + offset - self.code_review.diff_scroll_y;

                        self.code_review.toggle_file_collapsed(&path);
                        if was_collapsed {
                            self.ensure_editor_document_for_diff_file(idx);
                            self.code_review.selected_file_idx = idx;
                            self.prepare_code_review_editor_file(idx);
                        } else {
                            if self.code_review.active_file_idx == Some(idx) {
                                self.code_review.active_file_idx = None;
                                self.code_review.active_editing_file = None;
                            }
                            // Scroll anchoring: if the accordion was scrolled above viewport top
                            // with its sticky header at the top (card_top < vy), keep its collapsed
                            // header anchored at vy so the viewport does not jump.
                            if card_top < vy {
                                self.code_review.diff_scroll_y = offset;
                            }
                        }

                        let max_scroll = self.code_review.max_diff_scroll(dvh);
                        self.code_review.diff_scroll_y =
                            self.code_review.diff_scroll_y.clamp(0.0, max_scroll);
                        self.sync_code_review_documents_scroll(dvh);
                    }
                }
                akssora_terminal_ui::CodeReviewAction::ToggleFullFile(idx) => {
                    if let Some(file) = self.code_review.diff.files.get(idx) {
                        let path = file.path.clone();
                        self.code_review.toggle_full_file(&path);
                        if self.code_review.is_full_file(&path) {
                            self.ensure_editor_document_for_diff_file(idx);
                            self.prepare_code_review_editor_file(idx);
                        }
                        self.code_review.invalidate_layout();
                        let (_, _, _, dvh) =
                            akssora_terminal_ui::CodeReviewLayout::diff_viewport_rect(
                                &self.code_review,
                                ws_x,
                                top_h,
                                (w - ws_x).max(0.0),
                                (h - top_h).max(0.0),
                            );
                        let max_scroll = self.code_review.max_diff_scroll(dvh);
                        self.code_review.diff_scroll_y =
                            self.code_review.diff_scroll_y.clamp(0.0, max_scroll);
                        self.sync_code_review_documents_scroll(dvh);
                    }
                }
                akssora_terminal_ui::CodeReviewAction::ExpandContextTop { file_idx, hunk_idx } => {
                    self.expand_code_review_hunk_context(file_idx, hunk_idx, true);
                }
                akssora_terminal_ui::CodeReviewAction::ExpandContextBottom {
                    file_idx,
                    hunk_idx,
                } => {
                    self.expand_code_review_hunk_context(file_idx, hunk_idx, false);
                }
                akssora_terminal_ui::CodeReviewAction::ToggleCollapseAll => {
                    self.code_review.toggle_collapse_all();
                    if !self.code_review.collapsed_files.is_empty() {
                        self.code_review.active_file_idx = None;
                        self.code_review.active_editing_file = None;
                    } else {
                        for i in 0..self.code_review.diff.files.len() {
                            self.ensure_editor_document_for_diff_file(i);
                        }
                    }
                    let (_, _, _, dvh) = akssora_terminal_ui::CodeReviewLayout::diff_viewport_rect(
                        &self.code_review,
                        ws_x,
                        top_h,
                        (w - ws_x).max(0.0),
                        (h - top_h).max(0.0),
                    );
                    let max_scroll = self.code_review.max_diff_scroll(dvh);
                    self.code_review.diff_scroll_y =
                        self.code_review.diff_scroll_y.clamp(0.0, max_scroll);
                    self.sync_code_review_documents_scroll(dvh);
                }
                akssora_terminal_ui::CodeReviewAction::CollapseAll => {
                    self.code_review.collapse_all();
                    self.code_review.active_file_idx = None;
                    self.code_review.active_editing_file = None;
                    let (_, _, _, dvh) = akssora_terminal_ui::CodeReviewLayout::diff_viewport_rect(
                        &self.code_review,
                        ws_x,
                        top_h,
                        (w - ws_x).max(0.0),
                        (h - top_h).max(0.0),
                    );
                    let max_scroll = self.code_review.max_diff_scroll(dvh);
                    self.code_review.diff_scroll_y =
                        self.code_review.diff_scroll_y.clamp(0.0, max_scroll);
                    self.sync_code_review_documents_scroll(dvh);
                }
                akssora_terminal_ui::CodeReviewAction::ExpandAll => {
                    self.code_review.expand_all();
                    for i in 0..self.code_review.diff.files.len() {
                        self.ensure_editor_document_for_diff_file(i);
                    }
                    let (_, _, _, dvh) = akssora_terminal_ui::CodeReviewLayout::diff_viewport_rect(
                        &self.code_review,
                        ws_x,
                        top_h,
                        (w - ws_x).max(0.0),
                        (h - top_h).max(0.0),
                    );
                    let max_scroll = self.code_review.max_diff_scroll(dvh);
                    self.code_review.diff_scroll_y =
                        self.code_review.diff_scroll_y.clamp(0.0, max_scroll);
                    self.sync_code_review_documents_scroll(dvh);
                }
                akssora_terminal_ui::CodeReviewAction::CopyFilePath(idx) => {
                    if let Some(file) = self.code_review.diff.files.get(idx) {
                        akssora_terminal_input::set_clipboard_text(&file.path.to_string_lossy());
                    }
                }
                akssora_terminal_ui::CodeReviewAction::OpenFileInEditor(idx) => {
                    if let Some(file) = self.code_review.diff.files.get(idx) {
                        let full_path = self.code_review.repo_root.join(&file.path);
                        self.open_editor_file(&full_path);
                    }
                }
                akssora_terminal_ui::CodeReviewAction::ClosePanel => {
                    self.close_code_review();
                }
                akssora_terminal_ui::CodeReviewAction::ToggleExpandView => {
                    self.code_review.toggle_full_view();
                    self.layout_blocks();
                    if let Some(win) = self.window.as_ref() {
                        win.request_redraw();
                    }
                }
                akssora_terminal_ui::CodeReviewAction::ToggleStaged => {
                    self.code_review.toggle_staged();
                    self.request_code_review_diff_reload();
                }
                akssora_terminal_ui::CodeReviewAction::ToggleDiffMode => {
                    self.code_review.toggle_diff_mode();
                }
                akssora_terminal_ui::CodeReviewAction::OpenCommitDialog => {
                    self.code_review.commit_dialog_open = true;
                }
                akssora_terminal_ui::CodeReviewAction::PushToRemote => {
                    let _ = self.code_review.push();
                }
                akssora_terminal_ui::CodeReviewAction::Refresh => {
                    self.request_code_review_diff_reload();
                }
                akssora_terminal_ui::CodeReviewAction::ToggleSidebar => {
                    self.code_review.sidebar_expanded = !self.code_review.sidebar_expanded;
                }
                akssora_terminal_ui::CodeReviewAction::StageFile(idx) => {
                    let affected = self.code_review.diff.files.get(idx).map(|f| f.path.clone());
                    self.code_review.stage_file_at(idx);
                    self.invalidate_code_review_data(affected.as_deref());
                }
                akssora_terminal_ui::CodeReviewAction::UnstageFile(idx) => {
                    let affected = self.code_review.diff.files.get(idx).map(|f| f.path.clone());
                    self.code_review.unstage_file_at(idx);
                    self.invalidate_code_review_data(affected.as_deref());
                }
                akssora_terminal_ui::CodeReviewAction::DiscardFile(idx) => {
                    let affected = self.code_review.diff.files.get(idx).map(|f| f.path.clone());
                    if let Some(ref path) = affected {
                        let full_path = self.code_review.repo_root.join(path);
                        self.editor_documents.remove(&full_path);
                    }
                    self.code_review.discard_file_at(idx);
                    self.invalidate_code_review_data(affected.as_deref());
                }
                akssora_terminal_ui::CodeReviewAction::CloseCommitDialog => {
                    self.code_review.commit_dialog_open = false;
                }
                akssora_terminal_ui::CodeReviewAction::SubmitCommit => {
                    let msg = self.code_review.commit_message.clone();
                    let _ = self.code_review.commit_staged(&msg);
                    self.invalidate_code_review_data(None);
                }
                akssora_terminal_ui::CodeReviewAction::OpenCommentInput { file_idx, line_idx } => {
                    self.code_review.active_comment_input =
                        Some((file_idx, line_idx, String::new()));
                }
                akssora_terminal_ui::CodeReviewAction::CancelCommentInput => {
                    self.code_review.active_comment_input = None;
                }
                akssora_terminal_ui::CodeReviewAction::SubmitComment => {
                    if let Some((f_idx, l_idx, body)) = self.code_review.active_comment_input.take()
                        && let Some(file) = self.code_review.diff.files.get(f_idx)
                    {
                        let text = if body.trim().is_empty() {
                            "Looks good!".to_string()
                        } else {
                            body
                        };
                        self.code_review
                            .add_comment(file.path.clone(), l_idx, "Reviewer", &text);
                    }
                }
                akssora_terminal_ui::CodeReviewAction::ToggleCommentResolved(id) => {
                    self.code_review.toggle_comment_resolved(id);
                }
                akssora_terminal_ui::CodeReviewAction::DeleteComment(id) => {
                    self.code_review.delete_comment(id);
                }
                akssora_terminal_ui::CodeReviewAction::ScrollDiff(dy) => {
                    let (_, _, _, dvh) = akssora_terminal_ui::CodeReviewLayout::diff_viewport_rect(
                        &self.code_review,
                        ws_x,
                        top_h,
                        (w - ws_x).max(0.0),
                        (h - top_h).max(0.0),
                    );
                    let max_scroll = self.code_review.max_diff_scroll(dvh);
                    self.code_review.diff_scroll_y =
                        (self.code_review.diff_scroll_y + dy).clamp(0.0, max_scroll);
                }
                akssora_terminal_ui::CodeReviewAction::ScrollSidebar(dy) => {
                    let sb_h = ((h - top_h).max(0.0)
                        - akssora_terminal_ui::CodeReviewLayout::HEADER_HEIGHT)
                        .max(0.0);
                    let sidebar_total_h = self.code_review.filtered_files().len() as f32
                        * akssora_terminal_ui::CodeReviewLayout::FILE_ROW_HEIGHT
                        + 16.0;
                    let max_sidebar_scroll = (sidebar_total_h - sb_h).max(0.0);
                    self.code_review.sidebar_scroll_y =
                        (self.code_review.sidebar_scroll_y + dy).clamp(0.0, max_sidebar_scroll);
                }
                akssora_terminal_ui::CodeReviewAction::SearchChangedFiles(q) => {
                    self.code_review.set_filter_query(q);
                }
                akssora_terminal_ui::CodeReviewAction::SelectTreeFile {
                    is_staged,
                    orig_idx: _,
                    path,
                } => {
                    if self.code_review.is_staged != is_staged {
                        self.code_review.toggle_staged();
                        self.request_code_review_diff_reload();
                    }
                    if let Some(idx) = self
                        .code_review
                        .diff
                        .files
                        .iter()
                        .position(|f| f.path == path)
                    {
                        self.code_review.collapsed_files.remove(&path);
                        self.code_review.select_file(idx);
                        self.prepare_code_review_editor_file(idx);
                        let (_, _, _, dvh) =
                            akssora_terminal_ui::CodeReviewLayout::diff_viewport_rect(
                                &self.code_review,
                                ws_x,
                                top_h,
                                (w - ws_x).max(0.0),
                                (h - top_h).max(0.0),
                            );
                        let offset = self.code_review.card_y_offset_for_file(idx, dvh);
                        let max_scroll = self.code_review.max_diff_scroll(dvh);
                        if offset < self.code_review.diff_scroll_y
                            || offset > self.code_review.diff_scroll_y + dvh
                        {
                            self.code_review.diff_scroll_y = offset.min(max_scroll).max(0.0);
                        }
                    }
                }
                akssora_terminal_ui::CodeReviewAction::ToggleSectionCollapse(is_staged) => {
                    self.code_review.toggle_section_collapse(is_staged);
                }
                akssora_terminal_ui::CodeReviewAction::StageAll => {
                    self.code_review.stage_all();
                    self.invalidate_code_review_data(None);
                }
                akssora_terminal_ui::CodeReviewAction::UnstageAll => {
                    self.code_review.unstage_all();
                    self.invalidate_code_review_data(None);
                }
                akssora_terminal_ui::CodeReviewAction::PromptDiscardAll => {
                    self.code_review.confirm_discard =
                        Some(akssora_terminal_ui::DiscardTarget::All);
                }
                akssora_terminal_ui::CodeReviewAction::StageDirectoryByPath(path) => {
                    self.code_review.stage_directory_by_path(&path);
                    self.invalidate_code_review_data(None);
                }
                akssora_terminal_ui::CodeReviewAction::UnstageDirectoryByPath(path) => {
                    self.code_review.unstage_directory_by_path(&path);
                    self.invalidate_code_review_data(None);
                }
                akssora_terminal_ui::CodeReviewAction::PromptDiscardDirectory(path) => {
                    self.code_review.confirm_discard =
                        Some(akssora_terminal_ui::DiscardTarget::Directory(path));
                }
                akssora_terminal_ui::CodeReviewAction::StageFileByPath(path) => {
                    self.code_review.stage_file_by_path(&path);
                    self.invalidate_code_review_data(Some(&path));
                }
                akssora_terminal_ui::CodeReviewAction::UnstageFileByPath(path) => {
                    self.code_review.unstage_file_by_path(&path);
                    self.invalidate_code_review_data(Some(&path));
                }
                akssora_terminal_ui::CodeReviewAction::DiscardFileByPath(path) => {
                    let full_path = self.code_review.repo_root.join(&path);
                    self.editor_documents.remove(&full_path);
                    self.code_review.discard_file_by_path(&path);
                    self.invalidate_code_review_data(Some(&path));
                }
                akssora_terminal_ui::CodeReviewAction::PromptDiscardFile(path) => {
                    self.code_review.confirm_discard =
                        Some(akssora_terminal_ui::DiscardTarget::File(path));
                }
                akssora_terminal_ui::CodeReviewAction::ConfirmDiscardOk => {
                    if let Some(target) = self.code_review.confirm_discard.take() {
                        match target {
                            akssora_terminal_ui::DiscardTarget::File(path) => {
                                let full_path = self.code_review.repo_root.join(&path);
                                self.editor_documents.remove(&full_path);
                                self.code_review.discard_file_by_path(&path);
                                self.invalidate_code_review_data(Some(&path));
                            }
                            akssora_terminal_ui::DiscardTarget::Directory(dir) => {
                                let full_dir = self.code_review.repo_root.join(&dir);
                                self.editor_documents
                                    .retain(|p, _| !p.starts_with(&full_dir));
                                self.code_review.discard_directory_by_path(&dir);
                                self.invalidate_code_review_data(None);
                            }
                            akssora_terminal_ui::DiscardTarget::All => {
                                self.editor_documents
                                    .retain(|p, _| !p.starts_with(&self.code_review.repo_root));
                                self.code_review.discard_all_changes();
                                self.invalidate_code_review_data(None);
                            }
                        }
                    }
                }
                akssora_terminal_ui::CodeReviewAction::ConfirmDiscardCancel => {
                    self.code_review.confirm_discard = None;
                }
                akssora_terminal_ui::CodeReviewAction::OpenFileInEditorByPath(path) => {
                    let full_path = self.code_review.repo_root.join(&path);
                    self.open_editor_file(&full_path);
                }
            }
            if let Some(win) = self.window.as_ref() {
                win.request_redraw();
            }
            return true;
        }

        false
    }
}
