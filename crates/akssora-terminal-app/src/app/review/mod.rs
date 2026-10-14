pub mod worker;
pub use worker::{DiffBackgroundWorker, DiffWorkerRequest};

use super::App;

// INFO: Code Review panel integration and terminal synchronization methods.
impl App {
    // INFO: Dispatches an asynchronous request to the background diff worker to refresh repository diff.
    pub fn request_code_review_diff_reload(&mut self) {
        self.code_review.is_loading = true;
        self.code_review.diff_generation += 1;
        self.diff_worker.send_request(DiffWorkerRequest {
            generation: self.code_review.diff_generation,
            repo_root: self.code_review.repo_root.clone(),
            staged: self.code_review.is_staged,
        });
    }

    // INFO: Polls non-blocking responses from background diff worker and updates the indexed immutable model on UI thread.
    pub fn poll_diff_worker(&mut self) {
        if let Some(resp) = self.diff_worker.try_recv_response()
            && resp.repo_root == self.code_review.repo_root
            && resp.staged == self.code_review.is_staged
        {
            self.code_review
                .set_indexed_diff(resp.diff, resp.generation);
            self.ensure_code_review_documents();
            self.header.git_additions = self.code_review.diff.total_additions;
            self.header.git_deletions = self.code_review.diff.total_deletions;
            if let Some(win) = self.window.as_ref() {
                win.request_redraw();
            }
        }
    }

    // INFO: Invalidates affected editor documents and triggers background worker re-sync after Git staging / revert.
    pub fn invalidate_code_review_data(&mut self, affected_path: Option<&std::path::Path>) {
        if let Some(path) = affected_path {
            let full_path = self.code_review.repo_root.join(path);
            if let Some(doc) = self.editor_documents.get_mut(&full_path) {
                doc.set_git_original_content(None);
            }
        }
        self.code_review.invalidate_layout();
        self.request_code_review_diff_reload();
    }

    // INFO: Synchronizes Code Review model and header stats relative to the terminal's current directory.
    pub fn sync_code_review_with_active_terminal(&mut self) {
        let target_dir = self.current_working_directory();
        let repo_root_opt = akssora_git::GitInspector::repo_root(&target_dir);
        self.header.is_git_repo = repo_root_opt.is_some();
        if let Some(ref root) = repo_root_opt {
            self.header.git_branch = akssora_git::GitInspector::current_branch(root);
            self.header.git_branches = akssora_git::GitInspector::list_branches(root);
        } else {
            self.header.git_branch = None;
            self.header.git_branches.clear();
            self.header.branch_dropdown_open = false;
        }

        let is_code_review_tab = self
            .tab_manager
            .active_tab()
            .map(|t| matches!(t.kind, akssora_terminal_ui::TabKind::CodeReview { .. }))
            .unwrap_or(false);

        if !is_code_review_tab {
            let target_root = repo_root_opt.unwrap_or(target_dir);
            if self.code_review.repo_root != target_root {
                self.code_review = akssora_terminal_ui::CodeReviewModel::new(&target_root);
            }
        }
        self.request_code_review_diff_reload();
        self.header.git_additions = self.code_review.diff.total_additions;
        self.header.git_deletions = self.code_review.diff.total_deletions;
    }

    // INFO: Opens or creates a dedicated Code Review panel tab relative to the active terminal directory.
    pub fn open_code_review(&mut self) {
        self.sync_code_review_with_active_terminal();
        self.ensure_code_review_documents();
        if !self.code_review.diff.files.is_empty() {
            self.prepare_code_review_editor_file(self.code_review.selected_file_idx);
        }
        let repo_root = self.code_review.repo_root.to_string_lossy().to_string();
        let title = format!("Review: {}", self.code_review.branch);
        let _ = self.tab_manager.add_tab(
            title,
            akssora_terminal_ui::TabKind::CodeReview {
                repo_path: repo_root,
            },
        );
        self.header.is_code_review_active = true;
        if let Some(win) = self.window.as_ref() {
            win.request_redraw();
        }
    }

    // INFO: Toggles between Code Review panel and previous workspace tab.
    pub fn toggle_code_review(&mut self) {
        self.sync_code_review_with_active_terminal();
        self.ensure_code_review_documents();
        let is_active_review = self
            .tab_manager
            .active_tab()
            .map(|t| matches!(t.kind, akssora_terminal_ui::TabKind::CodeReview { .. }))
            .unwrap_or(false);

        if is_active_review {
            if let Some(first) = self.tab_manager.tabs().first() {
                self.tab_manager.select_by_id(first.id);
            }
            self.header.is_code_review_active = false;
        } else {
            if !self.code_review.diff.files.is_empty() {
                self.prepare_code_review_editor_file(self.code_review.selected_file_idx);
            }
            if let Some(review_tab) = self
                .tab_manager
                .tabs()
                .iter()
                .find(|t| matches!(t.kind, akssora_terminal_ui::TabKind::CodeReview { .. }))
            {
                let tab_id = review_tab.id;
                self.tab_manager.select_by_id(tab_id);
            } else {
                self.open_code_review();
            }
            self.header.is_code_review_active = true;
        }
        if let Some(win) = self.window.as_ref() {
            win.request_redraw();
        }
    }

    // INFO: Ensures expanded files in the code review diff have prepared editor documents.
    pub fn ensure_code_review_documents(&mut self) {
        let num_files = self.code_review.diff.files.len();
        if num_files == 0 {
            return;
        }

        let sel_idx = self
            .code_review
            .active_file_idx
            .unwrap_or(self.code_review.selected_file_idx)
            .min(num_files - 1);
        self.prepare_code_review_editor_file(sel_idx);

        let mut prepared = 0;
        for idx in 0..num_files {
            if idx == sel_idx {
                continue;
            }
            let is_collapsed = {
                let file = &self.code_review.diff.files[idx];
                self.code_review.is_file_collapsed(&file.path)
            };
            if !is_collapsed {
                self.ensure_editor_document_for_diff_file(idx);
                prepared += 1;
                if prepared >= 15 {
                    break;
                }
            }
        }
    }

    // INFO: Ensures an editor document exists and is synced for a given diff file.
    pub fn ensure_editor_document_for_diff_file(&mut self, file_idx: usize) {
        if let Some(file) = self.code_review.diff.files.get_mut(file_idx) {
            let full_path = self.code_review.repo_root.join(&file.path);
            let repo_root = self.code_review.repo_root.clone();
            let is_deleted = file.status == akssora_git::FileGitStatus::Deleted;
            let doc = self
                .editor_documents
                .entry(full_path.clone())
                .or_insert_with(|| {
                    let mut d = if is_deleted {
                        match akssora_git::GitInspector::file_deleted_content(
                            &repo_root,
                            &full_path,
                            Some(file),
                        ) {
                            Ok(content) => {
                                let mut doc =
                                    akssora_editor::Document::open(full_path.clone(), &content);
                                doc.set_deleted(true);
                                doc
                            }
                            Err(err) => {
                                let err_msg = format!("// {err}\n");
                                let mut doc =
                                    akssora_editor::Document::open(full_path.clone(), &err_msg);
                                doc.set_deleted(true);
                                doc
                            }
                        }
                    } else {
                        akssora_editor::Document::open_file(&full_path).unwrap_or_else(|_| {
                            akssora_editor::Document::open(full_path.clone(), "")
                        })
                    };
                    d.line_wrap = self.config.line_wrap;
                    d.line_numbers = self.config.line_numbers;
                    d.line_number_mode = self.config.line_number_mode;
                    d.tab_size = self.config.tab_size;
                    d.line_height = self.config.line_height;
                    d.vim_enabled = self.config.vim_mode;
                    d.inline_diff = true;
                    d
                });
            doc.line_numbers = self.config.line_numbers;
            doc.line_number_mode = self.config.line_number_mode;
            doc.line_height = self.config.line_height;
            doc.inline_diff = true;
            if is_deleted {
                doc.set_deleted(true);
            } else {
                doc.refresh_git_original_content(&repo_root);
            }
            file.line_count = doc.buffer.line_count();
        }
    }

    // INFO: Prepares or retrieves an editor document for in-place code review editing.
    pub fn prepare_code_review_editor_file(&mut self, file_idx: usize) {
        self.ensure_editor_document_for_diff_file(file_idx);
        if let Some(file) = self.code_review.diff.files.get(file_idx) {
            let full_path = self.code_review.repo_root.join(&file.path);
            self.code_review.active_file_idx = Some(file_idx);
            self.code_review.active_editing_file = Some(full_path.clone());
            if let Some(doc) = self.editor_documents.get_mut(&full_path)
                && doc.cursor == (0, 0)
                && let Some(first_hunk) = file.hunks.first()
            {
                let target_row = first_hunk
                    .lines
                    .iter()
                    .find(|l| l.line_type == akssora_git::DiffLineType::Add)
                    .or_else(|| first_hunk.lines.first())
                    .and_then(|l| l.new_line_num.or(l.old_line_num))
                    .map(|n| n.saturating_sub(1))
                    .unwrap_or(0);
                doc.cursor = (target_row, 0);
            }
        }
    }

    // INFO: Synchronizes all expanded code review documents' internal scroll offsets with panel viewport position.
    pub fn sync_code_review_documents_scroll(&mut self, dvh: f32) {
        let show_tabs = self.config.tab_layout.includes_title_bar();
        let top_h = self.header.total_height(show_tabs);
        let show_sidebar = self.config.tab_layout.show_sidebar();
        let sb_w = self.sidebar.effective_width(show_sidebar);
        let exp_w = self.explorer.effective_width();
        let ws_x = sb_w + exp_w;
        let ws_y = top_h;
        let (win_w, win_h) = self.state.as_ref().map(|s| s.size()).unwrap_or((1200, 800));
        let ws_w = (win_w as f32 - ws_x).max(100.0);
        let ws_h = (win_h as f32 - ws_y).max(100.0);
        let (_, vy, _, _) = akssora_terminal_ui::CodeReviewLayout::diff_viewport_rect(
            &self.code_review,
            ws_x,
            ws_y,
            ws_w,
            ws_h,
        );
        let cell_h = self.fonts.as_ref().map(|f| f.cell_h).unwrap_or(16.0);
        let mut cur_virtual_y = vy + 12.0;

        for (idx, file) in self.code_review.diff.files.iter().enumerate() {
            let is_collapsed = self.code_review.is_file_collapsed(&file.path);
            let is_editing = self.code_review.active_file_idx == Some(idx);
            let is_full = self.code_review.is_full_file(&file.path);
            let ch = akssora_terminal_ui::CodeReviewLayout::card_height_for_file_state_ex(
                file,
                is_collapsed,
                is_editing,
                is_full,
                dvh,
            );
            let cy = cur_virtual_y - self.code_review.diff_scroll_y;
            cur_virtual_y += ch + akssora_terminal_ui::CodeReviewLayout::CARD_MARGIN_BOTTOM;

            if !is_collapsed {
                let full_path = self.code_review.repo_root.join(&file.path);
                if let Some(doc) = self.editor_documents.get_mut(&full_path) {
                    let line_h = (cell_h * doc.line_height).max(cell_h);
                    let content_top =
                        cy + akssora_terminal_ui::CodeReviewLayout::CARD_HEADER_HEIGHT;
                    let sticky_top = vy + akssora_terminal_ui::CodeReviewLayout::CARD_HEADER_HEIGHT;
                    if content_top < sticky_top {
                        let scrolled_above = sticky_top - content_top;
                        let visible_body_h = (dvh
                            - akssora_terminal_ui::CodeReviewLayout::CARD_HEADER_HEIGHT)
                            .max(line_h);
                        let visible_rows = (visible_body_h / line_h).floor() as usize;
                        let max_scroll = doc.buffer.line_count().saturating_sub(visible_rows);
                        doc.scroll_row =
                            ((scrolled_above / line_h).floor() as usize).min(max_scroll);
                    } else {
                        doc.scroll_row = 0;
                    }
                }
            }
        }
    }

    // INFO: Stages or unstages a specific line in a Code Review file via Git patch application without touching unrelated lines.
    pub fn stage_or_unstage_line_in_code_review(&mut self, file_idx: usize, target_line: usize) {
        if let Some(file) = self.code_review.diff.files.get(file_idx) {
            let file_path = file.path.clone();
            let is_staged = self.code_review.is_staged;
            let repo_root = self.code_review.repo_root.clone();

            let _ = akssora_git::GitInspector::stage_or_unstage_line_at(
                &repo_root,
                &file_path,
                target_line,
                is_staged,
            );

            self.invalidate_code_review_data(Some(&file_path));
            let full_path = repo_root.join(&file_path);
            if let Some(doc) = self.editor_documents.get_mut(&full_path) {
                doc.refresh_git_original_content(&repo_root);
            }

            if let Some(win) = self.window.as_ref() {
                win.request_redraw();
            }
        }
    }

    // INFO: Stages a single specific line in a Code Review file without affecting unrelated changes.
    pub fn stage_single_line_in_code_review(
        &mut self,
        file_idx: usize,
        target_line: usize,
        _is_deletion: bool,
    ) {
        self.stage_or_unstage_line_in_code_review(file_idx, target_line);
    }

    // INFO: Reverts / removes a specific line in a Code Review file (restores original or removes addition).
    pub fn revert_single_line_in_code_review(
        &mut self,
        file_idx: usize,
        target_line: usize,
        is_deletion: bool,
    ) {
        if let Some(file) = self.code_review.diff.files.get(file_idx) {
            let file_path = file.path.clone();
            let repo_root = self.code_review.repo_root.clone();
            let full_path = repo_root.join(&file_path);

            let mut del_content = None;
            if is_deletion {
                for hunk in &file.hunks {
                    for diff_line in &hunk.lines {
                        let r = diff_line
                            .new_line_num
                            .or(diff_line.old_line_num)
                            .map(|n| n.saturating_sub(1));
                        if r == Some(target_line)
                            && diff_line.line_type == akssora_git::DiffLineType::Delete
                        {
                            del_content = Some(diff_line.content.clone());
                            break;
                        }
                    }
                    if del_content.is_some() {
                        break;
                    }
                }
            }

            self.prepare_code_review_editor_file(file_idx);

            if let Some(doc) = self.editor_documents.get_mut(&full_path) {
                if let Some(content) = del_content {
                    let insert_row = target_line.min(doc.buffer.line_count());
                    doc.record_undo();
                    doc.buffer.insert_str(insert_row, 0, &content);
                    let _ = doc.save();
                } else if doc.revert_line_at(target_line).is_ok() {
                    let _ = doc.save();
                } else {
                    let line_count = doc.buffer.line_count();
                    if target_line < line_count {
                        doc.record_undo();
                        if line_count <= 1 {
                            doc.buffer.clear();
                        } else {
                            let start_char = doc.buffer.rope().line_to_char(target_line);
                            let end_char = if target_line + 1 < doc.buffer.rope().len_lines() {
                                doc.buffer.rope().line_to_char(target_line + 1)
                            } else {
                                doc.buffer.rope().len_chars()
                            };
                            doc.buffer.delete_range(start_char, end_char);
                        }
                        let _ = doc.save();
                    }
                }
                doc.refresh_git_original_content(&repo_root);
            }

            self.invalidate_code_review_data(Some(&file_path));
            self.refresh_git_status();

            if let Some(win) = self.window.as_ref() {
                win.request_redraw();
            }
        }
    }

    // INFO: Dynamically reveals additional unchanged context lines from the original file at a hunk boundary.
    pub fn expand_code_review_hunk_context(
        &mut self,
        file_idx: usize,
        hunk_idx: usize,
        is_top: bool,
    ) {
        if let Some(file) = self.code_review.diff.files.get_mut(file_idx) {
            let file_path = file.path.clone();
            let full_path = self.code_review.repo_root.join(&file_path);
            let content_opt = std::fs::read_to_string(&full_path).ok();
            let all_lines: Vec<&str> = if let Some(ref text) = content_opt {
                text.lines().collect()
            } else {
                Vec::new()
            };
            if file.line_count == 0 && !all_lines.is_empty() {
                file.line_count = all_lines.len();
            }

            if let Some(hunk) = file.hunks.get_mut(hunk_idx) {
                const EXPAND_COUNT: usize = 20;
                if is_top {
                    let first_line_num = hunk.first_line_num();
                    if first_line_num > 1 {
                        let start_idx = first_line_num.saturating_sub(EXPAND_COUNT + 1);
                        let end_idx = first_line_num.saturating_sub(1);
                        let mut added_lines = Vec::new();
                        for (i, line_str) in
                            all_lines.iter().enumerate().take(end_idx).skip(start_idx)
                        {
                            let line_num = i + 1;
                            added_lines.push(akssora_git::DiffLine {
                                line_type: akssora_git::DiffLineType::Context,
                                content: format!("{}\n", line_str),
                                old_line_num: Some(line_num),
                                new_line_num: Some(line_num),
                            });
                        }
                        if !added_lines.is_empty() {
                            let mut new_lines = added_lines;
                            new_lines.append(&mut hunk.lines);
                            hunk.lines = new_lines;
                        }
                    }
                } else {
                    let last_line_num = hunk.last_line_num();
                    if last_line_num < all_lines.len() {
                        let start_idx = last_line_num;
                        let end_idx = (start_idx + EXPAND_COUNT).min(all_lines.len());
                        for (i, line_str) in
                            all_lines.iter().enumerate().take(end_idx).skip(start_idx)
                        {
                            let line_num = i + 1;
                            hunk.lines.push(akssora_git::DiffLine {
                                line_type: akssora_git::DiffLineType::Context,
                                content: format!("{}\n", line_str),
                                old_line_num: Some(line_num),
                                new_line_num: Some(line_num),
                            });
                        }
                    }
                }
            }
            self.code_review
                .set_context_expanded(&file_path, hunk_idx, is_top);
            self.code_review.invalidate_layout();
        }
        if let Some(win) = self.window.as_ref() {
            win.request_redraw();
        }
    }
}
