use akssora_terminal_input::scroll_delta_to_lines;
use akssora_terminal_ui::{CodeReviewLayout, ExplorerUi, TabKind};
use winit::event::MouseScrollDelta;

use super::App;

// INFO: Mouse wheel event routing and scroll offset clamping across all view types.
impl App {
    pub(crate) fn handle_mouse_wheel(&mut self, delta: MouseScrollDelta) {
        let cell_h = self.fonts.as_ref().map(|f| f.cell_h).unwrap_or(16.0);
        let lines = scroll_delta_to_lines(delta, cell_h);
        let (mx, my) = self.cursor_pos;
        let show_sidebar = self.config.tab_layout.show_sidebar();
        let sb_w = self.sidebar.effective_width(show_sidebar);
        let exp_w = self.explorer.effective_width();

        // 0. Mouse wheel inside open quick search palette
        if self.quick_search.is_open {
            if lines > 0.0 {
                self.quick_search.select_prev();
            } else if lines < 0.0 {
                self.quick_search.select_next();
            }
            if let Some(window) = self.window.as_ref() {
                window.request_redraw();
            }
            return;
        }

        // 1. Mouse wheel inside file explorer
        if self.explorer.is_expanded && mx >= sb_w && mx <= sb_w + exp_w {
            let (_, win_h) = self.state.as_ref().map(|s| s.size()).unwrap_or((1000, 700));
            let header_h = ExplorerUi::HEADER_HEIGHT;
            let top_offset = self.header.height;
            let viewport_h = (win_h as f32 - top_offset - header_h).max(0.0);
            let total_rows = self.explorer.tree.flatten_rows().len();
            let total_content_h = total_rows as f32 * ExplorerUi::ITEM_HEIGHT;
            let max_scroll = (total_content_h - viewport_h).max(0.0);

            self.explorer.scroll_y = (self.explorer.scroll_y - lines * 24.0).clamp(0.0, max_scroll);
            if let Some(window) = self.window.as_ref() {
                window.request_redraw();
            }
            return;
        }

        // 2. Mouse wheel inside active Code Review panel
        let is_code_review = self
            .tab_manager
            .active_tab()
            .map(|t| matches!(t.kind, TabKind::CodeReview { .. }))
            .unwrap_or(false);
        if is_code_review {
            let show_tabs = self.config.tab_layout.includes_title_bar();
            let top_h = self.header.total_height(show_tabs);
            let (win_w, win_h) = self.state.as_ref().map(|s| s.size()).unwrap_or((1000, 700));
            let ws_x = sb_w + exp_w;
            let ws_y = top_h;
            let ws_w = (win_w as f32 - ws_x).max(100.0);
            let ws_h = (win_h as f32 - ws_y).max(100.0);

            let left_sb_w = if self.code_review.sidebar_expanded {
                self.code_review.sidebar_width.min(ws_w * 0.4).max(220.0)
            } else {
                0.0
            };

            if self.code_review.sidebar_expanded && mx >= ws_x && mx <= ws_x + left_sb_w {
                let sb_h = (ws_h - akssora_terminal_ui::CodeReviewLayout::HEADER_HEIGHT).max(0.0);
                let sidebar_total_h = self.code_review.visible_tree_items().len() as f32
                    * akssora_terminal_ui::CodeReviewLayout::FILE_ROW_HEIGHT
                    + akssora_terminal_ui::CodeReviewLayout::SIDEBAR_TOOLBAR_HEIGHT
                    + 16.0;
                let max_sidebar_scroll = (sidebar_total_h - sb_h).max(0.0);
                self.code_review.sidebar_scroll_y = (self.code_review.sidebar_scroll_y
                    - lines * 28.0)
                    .clamp(0.0, max_sidebar_scroll);
            } else if mx > ws_x + left_sb_w && mx <= ws_x + ws_w {
                self.scroll_code_review_diff(lines, mx, my, ws_x, ws_y, ws_w, ws_h);
            }
            if let Some(window) = self.window.as_ref() {
                window.request_redraw();
            }
            return;
        }

        // 3. Mouse wheel inside active code editor
        let is_editor = self
            .tab_manager
            .active_tab()
            .map(|t| matches!(t.kind, TabKind::Editor { .. }))
            .unwrap_or(false);
        let is_shift = self.modifiers.shift_key();
        let cell_w = self.fonts.as_ref().map(|f| f.cell_w).unwrap_or(8.0);
        let (win_w, win_h) = self.state.as_ref().map(|s| s.size()).unwrap_or((1000, 700));
        let show_tabs = self.config.tab_layout.includes_title_bar();
        let top_h = self.header.total_height(show_tabs);
        let ws_y = top_h;
        let tab_bar_h = if !self.editor_open_files.is_empty() {
            akssora_terminal_ui::EditorTabBar::HEIGHT
        } else {
            0.0
        };
        let ws_x = sb_w + exp_w;
        let ws_w = (win_w as f32 - ws_x).max(100.0);
        let ws_h = (win_h as f32 - ws_y).max(100.0);
        let editor_content_h = (ws_h - tab_bar_h - 22.0).max(0.0);
        let visible_rows = (editor_content_h / cell_h).floor() as usize;

        if is_editor
            && mx >= ws_x
            && let Some(doc) = self.active_editor_mut()
        {
            let half_w = ((ws_w - 1.0) * 0.5).floor();
            let is_preview_target = doc.is_markdown()
                && (doc.markdown_mode == akssora_editor::MarkdownViewMode::Preview
                    || (doc.markdown_mode == akssora_editor::MarkdownViewMode::Split
                        && mx >= ws_x + half_w));

            if is_preview_target {
                let total_lines = doc.buffer.line_count();
                let max_preview_scroll = total_lines.saturating_sub(visible_rows);
                match delta {
                    MouseScrollDelta::LineDelta(_x, y) => {
                        if y > 0.0 {
                            doc.preview_scroll_row =
                                doc.preview_scroll_row.saturating_sub(y.ceil() as usize * 2);
                        } else if y < 0.0 {
                            doc.preview_scroll_row = (doc.preview_scroll_row
                                + (-y).ceil() as usize * 2)
                                .min(max_preview_scroll);
                        }
                    }
                    MouseScrollDelta::PixelDelta(pos) => {
                        let lines_y = pos.y as f32 / cell_h;
                        if lines_y > 0.0 {
                            doc.preview_scroll_row = doc
                                .preview_scroll_row
                                .saturating_sub(lines_y.ceil() as usize);
                        } else if lines_y < 0.0 {
                            doc.preview_scroll_row = (doc.preview_scroll_row
                                + (-lines_y).ceil() as usize)
                                .min(max_preview_scroll);
                        }
                    }
                }
            } else {
                let gutter_w = if doc.line_numbers
                    && doc.line_number_mode != akssora_terminal_config::LineNumberMode::Off
                {
                    48.0
                } else {
                    0.0
                };
                let content_w = (ws_w - gutter_w - 24.0).max(cell_w);
                let wrap_w = (content_w / cell_w).floor().max(1.0) as usize;
                let visible_cols = wrap_w;

                let total_lines = if doc.line_wrap {
                    doc.total_visual_rows(wrap_w)
                } else {
                    doc.buffer.line_count()
                };
                let max_v_scroll = total_lines.saturating_sub(visible_rows);
                let max_h_scroll = doc.max_line_len().saturating_sub(visible_cols);

                match delta {
                    MouseScrollDelta::LineDelta(x, y) => {
                        if !doc.line_wrap && (is_shift || x != 0.0) {
                            let h_delta = if x != 0.0 { x } else { -y };
                            if h_delta > 0.0 {
                                doc.scroll_col = (doc.scroll_col + (h_delta.ceil() as usize * 3))
                                    .min(max_h_scroll);
                            } else if h_delta < 0.0 {
                                doc.scroll_col = doc
                                    .scroll_col
                                    .saturating_sub((-h_delta).ceil() as usize * 3);
                            }
                        } else if y > 0.0 {
                            doc.scroll_row = doc.scroll_row.saturating_sub(y.ceil() as usize);
                        } else if y < 0.0 {
                            doc.scroll_row =
                                (doc.scroll_row + (-y).ceil() as usize).min(max_v_scroll);
                        }
                    }
                    MouseScrollDelta::PixelDelta(pos) => {
                        let lines_y = pos.y as f32 / cell_h;
                        let cols_x = pos.x as f32 / cell_w;
                        if !doc.line_wrap && (is_shift || cols_x.abs() > lines_y.abs()) {
                            let h_delta = if cols_x != 0.0 { cols_x } else { -lines_y };
                            if h_delta > 0.0 {
                                doc.scroll_col = (doc.scroll_col
                                    + h_delta.ceil().max(1.0) as usize)
                                    .min(max_h_scroll);
                            } else if h_delta < 0.0 {
                                doc.scroll_col = doc
                                    .scroll_col
                                    .saturating_sub((-h_delta).ceil().max(1.0) as usize);
                            }
                        } else if lines_y > 0.0 {
                            doc.scroll_row = doc.scroll_row.saturating_sub(lines_y.ceil() as usize);
                        } else if lines_y < 0.0 {
                            doc.scroll_row =
                                (doc.scroll_row + (-lines_y).ceil() as usize).min(max_v_scroll);
                        }
                    }
                }

                // Synchronize preview scroll proportionally in split mode
                if doc.is_markdown() && doc.markdown_mode == akssora_editor::MarkdownViewMode::Split
                {
                    let total_lines = doc.buffer.line_count().max(1);
                    let max_preview = total_lines.saturating_sub(visible_rows);
                    let ratio = doc.scroll_row as f32 / total_lines as f32;
                    doc.preview_scroll_row =
                        ((ratio * total_lines as f32).round() as usize).min(max_preview);
                }
            }

            if let Some(window) = self.window.as_ref() {
                window.request_redraw();
            }
            return;
        }

        // 4. Terminal pane mouse wheel scrolling
        let pane_id = self
            .pane_at(mx, my)
            .map(|(id, _, _, _, _)| id)
            .or_else(|| self.active_pane_id());

        if let Some(id) = pane_id
            && let Some(block) = self.blocks.get_mut(&id)
        {
            if lines > 0.0 {
                block.shell.scroll_up(lines.ceil().max(1.0) as usize);
            } else if lines < 0.0 {
                block.shell.scroll_down((-lines).ceil().max(1.0) as usize);
            }
            if let Some(window) = self.window.as_ref() {
                window.request_redraw();
            }
        }
    }

    // INFO: Nested natural scrolling for Code Review panel: panel moves first, header sticks at top, content scrolls to boundary, then panel resumes scrolling.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn scroll_code_review_diff(
        &mut self,
        lines: f32,
        _mx: f32,
        _my: f32,
        ws_x: f32,
        ws_y: f32,
        ws_w: f32,
        ws_h: f32,
    ) {
        if lines == 0.0 || self.code_review.diff.files.is_empty() {
            return;
        }

        let (_, _vy, _, dvh) =
            CodeReviewLayout::diff_viewport_rect(&self.code_review, ws_x, ws_y, ws_w, ws_h);
        let total_content_h = self.code_review.total_diff_content_height(dvh);
        let max_diff_scroll = (total_content_h - dvh).max(0.0);
        let scroll_delta = lines * 36.0;
        self.code_review.diff_scroll_y =
            (self.code_review.diff_scroll_y - scroll_delta).clamp(0.0, max_diff_scroll);
        self.sync_code_review_documents_scroll(dvh);
    }
}
