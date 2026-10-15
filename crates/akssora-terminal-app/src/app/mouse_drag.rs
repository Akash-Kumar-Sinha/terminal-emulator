use akssora_terminal_ui::Scrollbar;

use super::App;

// INFO: Dragging interactions including split divider resizing, scrollbar dragging, text selection, and pane rearrangements.
impl App {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn handle_mouse_drags(
        &mut self,
        mx: f32,
        my: f32,
        w: f32,
        h: f32,
        is_editor: bool,
        top_h: f32,
        tab_bar_h: f32,
        ws_x: f32,
    ) {
        // 0. Code review sidebar edge resizing
        if self.code_review.is_resizing_sidebar {
            let clamped_w = (mx - ws_x).clamp(180.0, (w - ws_x) * 0.6);
            self.code_review.sidebar_width = clamped_w;
            if let Some(win) = self.window.as_ref() {
                win.request_redraw();
            }
            return;
        }

        // 0.1 Code review diff scrollbar dragging
        if self.code_review.is_dragging_diff_scrollbar {
            let (_, _, _, vh) = akssora_terminal_ui::CodeReviewLayout::diff_viewport_rect(
                &self.code_review,
                ws_x,
                top_h,
                (w - ws_x).max(0.0),
                (h - top_h).max(0.0),
            );
            let track_h = (vh - 8.0).max(10.0);
            let max_scroll = self.code_review.max_diff_scroll(vh);
            let delta_y = my - self.code_review.diff_scrollbar_drag_start_y;
            let delta_scroll = (delta_y / track_h) * max_scroll;
            self.code_review.diff_scroll_y = (self.code_review.diff_scrollbar_drag_start_scroll
                + delta_scroll)
                .clamp(0.0, max_scroll);
            self.sync_code_review_documents_scroll(vh);
            if let Some(win) = self.window.as_ref() {
                win.request_redraw();
            }
            return;
        }

        // 0.9 Handle Code Review split divider dragging
        if self.code_review.is_resizing_split {
            let (win_w, _) = self
                .state
                .as_ref()
                .map(|s| s.size())
                .unwrap_or((self.config.window.width, self.config.window.height));
            let show_sidebar = self.config.tab_layout.show_sidebar();
            let sb_w = self.sidebar.effective_width(show_sidebar);
            let exp_w = self.explorer.effective_width();
            let ws_x = sb_w + exp_w;
            let ws_w = (win_w as f32 - ws_x).max(100.0);
            let total_w = (ws_w - akssora_terminal_ui::PaneTree::DIVIDER_THICKNESS).max(10.0);

            let new_term_w = (mx - ws_x).clamp(120.0, total_w - 120.0);
            let new_cr_w = total_w - new_term_w;
            let new_ratio = (new_cr_w / total_w).clamp(0.15, 0.85);
            self.code_review.set_split_ratio(new_ratio);
            self.layout_blocks();
            if let Some(win) = self.window.as_ref() {
                win.request_redraw();
            }
        }

        // 1. Handle divider dragging
        if let Some(drag) = self.split_manager.drag_state.clone() {
            let active_tab_id = drag.tab_id;
            if let Some(tree) = self.split_trees.get_mut(&active_tab_id) {
                let delta = match drag.direction {
                    akssora_terminal_ui::SplitDirection::Vertical => {
                        (mx - drag.start_x) / drag.container_w.max(1.0)
                    }
                    akssora_terminal_ui::SplitDirection::Horizontal => {
                        (my - drag.start_y) / drag.container_h.max(1.0)
                    }
                };
                tree.adjust_ratio(&drag.path, delta);
                self.split_manager.drag_state = Some(crate::app::split_state::DividerDragState {
                    tab_id: drag.tab_id,
                    path: drag.path,
                    direction: drag.direction,
                    start_x: mx,
                    start_y: my,
                    container_w: drag.container_w,
                    container_h: drag.container_h,
                });
                self.layout_blocks();
            }
        }

        // 2. Handle scrollbar dragging and hover (Terminal)
        if let Some(block) = self.active_block_mut() {
            let total_lines = block.shell.grid().scrollback_len() + block.shell.grid().rows();
            let visible_lines = block.shell.grid().rows();

            if block.scrollbar.is_dragging {
                let max_scroll = block.shell.grid().scrollback_len() as f32;
                let ratio = (visible_lines as f32 / total_lines.max(1) as f32).clamp(0.05, 1.0);
                let thumb_h = (block.scrollbar.height * ratio)
                    .max(Scrollbar::MIN_THUMB_HEIGHT)
                    .min(block.scrollbar.height);
                let travel = block.scrollbar.height - thumb_h;
                if travel > 0.0 {
                    let delta_y = my - block.scrollbar.drag_start_y;
                    let delta_offset = -(delta_y / travel) * max_scroll;
                    let new_offset =
                        (block.scrollbar.drag_start_offset + delta_offset).clamp(0.0, max_scroll);
                    block
                        .shell
                        .grid_mut()
                        .set_scroll_offset(new_offset.round() as usize);
                }
            } else {
                block.scrollbar.is_hovered = block.scrollbar.hit_test(mx, my);
            }
        }

        // 3. Handle editor scrollbars dragging and hover
        if is_editor
            && let Some((
                total_lines,
                visible_rows,
                max_line_len,
                visible_cols,
                v_overflow,
                h_overflow,
            )) = self.update_editor_scrollbar_geometries()
        {
            if self.editor_v_scrollbar.is_dragging {
                let max_scroll = total_lines.saturating_sub(visible_rows) as f32;
                let ratio = (visible_rows as f32 / total_lines.max(1) as f32).clamp(0.05, 1.0);
                let thumb_h = (self.editor_v_scrollbar.height * ratio)
                    .max(Scrollbar::MIN_THUMB_HEIGHT)
                    .min(self.editor_v_scrollbar.height);
                let travel = self.editor_v_scrollbar.height - thumb_h;
                if travel > 0.0 {
                    let delta_y = my - self.editor_v_scrollbar.drag_start_pos;
                    let delta_offset = (delta_y / travel) * max_scroll;
                    let new_offset = (self.editor_v_scrollbar.drag_start_offset + delta_offset)
                        .clamp(0.0, max_scroll);
                    if let Some(doc) = self.active_editor_mut() {
                        doc.scroll_row = new_offset.round() as usize;
                    }
                }
            } else {
                self.editor_v_scrollbar.is_hovered =
                    v_overflow && self.editor_v_scrollbar.hit_test(mx, my);
            }

            if self.editor_h_scrollbar.is_dragging {
                let max_scroll = max_line_len.saturating_sub(visible_cols) as f32;
                let ratio = (visible_cols as f32 / max_line_len.max(1) as f32).clamp(0.05, 1.0);
                let thumb_w = (self.editor_h_scrollbar.width * ratio)
                    .max(Scrollbar::MIN_THUMB_HEIGHT)
                    .min(self.editor_h_scrollbar.width);
                let travel = self.editor_h_scrollbar.width - thumb_w;
                if travel > 0.0 {
                    let delta_x = mx - self.editor_h_scrollbar.drag_start_pos;
                    let delta_offset = (delta_x / travel) * max_scroll;
                    let new_offset = (self.editor_h_scrollbar.drag_start_offset + delta_offset)
                        .clamp(0.0, max_scroll);
                    if let Some(doc) = self.active_editor_mut() {
                        doc.scroll_col = new_offset.round() as usize;
                    }
                }
            } else {
                self.editor_h_scrollbar.is_hovered =
                    h_overflow && self.editor_h_scrollbar.hit_test(mx, my);
            }
        }

        // 4. Handle mouse text selection drag
        if let Some((px, py)) = self.mouse_press_pos {
            let dist = (mx - px).hypot(my - py);
            let not_other_drag = !self.sidebar.is_resizing
                && !self.code_review.is_resizing_split
                && self.split_manager.drag_state.is_none()
                && self.split_manager.pane_drag.is_none()
                && !self.editor_v_scrollbar.is_dragging
                && !self.editor_h_scrollbar.is_dragging
                && self
                    .active_block_mut()
                    .map(|b| !b.scrollbar.is_dragging)
                    .unwrap_or(true);

            if dist >= 4.0 && not_other_drag {
                self.is_selecting_text = true;
                let (cell_w, cell_h) = self
                    .fonts
                    .as_ref()
                    .map(|f| (f.cell_w, f.cell_h))
                    .unwrap_or((8.0, 16.0));
                let is_code_review = self.is_code_review_open();
                let cr_body_rect = if is_code_review
                    && let Some((cr_x, cr_y, cr_w, cr_h)) = self.code_review_bounds()
                {
                    akssora_terminal_ui::CodeReviewLayout::active_card_body_rect(
                        &self.code_review,
                        cr_x,
                        cr_y,
                        cr_w,
                        cr_h,
                    )
                } else {
                    None
                };

                if is_editor && let Some(doc) = self.active_editor_mut() {
                    if is_code_review {
                        if let Some((body_x, body_y, body_w, body_h)) = cr_body_rect {
                            let clamped_y = my.clamp(body_y, body_y + body_h);
                            doc.handle_drag_selection(
                                mx, clamped_y, body_x, body_y, body_w, cell_w, cell_h,
                            );
                        }
                    } else {
                        let content_y = top_h + tab_bar_h;
                        let ws_w = (w - ws_x).max(100.0);
                        let split_limit =
                            if doc.markdown_mode == akssora_editor::MarkdownViewMode::Split {
                                ws_x + ((ws_w - 1.0) * 0.5).floor()
                            } else {
                                ws_x + ws_w
                            };
                        let editor_w = split_limit - ws_x;
                        let editor_bottom = (h - 22.0).max(content_y);

                        // Auto-scroll vertically when dragging above or below the content area
                        let line_h = (cell_h * doc.line_height).max(cell_h);
                        let visible_rows = ((editor_bottom - content_y) / line_h).floor() as usize;
                        let total_lines = if doc.line_wrap {
                            let wrap_w = doc.current_wrap_width().unwrap_or(1);
                            doc.total_visual_rows(wrap_w)
                        } else {
                            doc.buffer.line_count()
                        };
                        let max_scroll = total_lines.saturating_sub(visible_rows);

                        if my < content_y {
                            doc.scroll_row = doc.scroll_row.saturating_sub(1);
                        } else if my > editor_bottom {
                            doc.scroll_row = (doc.scroll_row + 1).min(max_scroll);
                        }

                        // Auto-scroll horizontally when dragging past left or right in non-wrap mode
                        if !doc.line_wrap {
                            let gutter_w = if doc.line_numbers
                                && doc.line_number_mode
                                    != akssora_terminal_config::LineNumberMode::Off
                            {
                                48.0
                            } else {
                                0.0
                            };
                            let text_start_x = ws_x + gutter_w + 8.0;
                            let max_line_len = doc.max_line_len();
                            let rough_visible_cols =
                                ((editor_w - gutter_w - 24.0).max(0.0) / cell_w)
                                    .floor()
                                    .max(1.0) as usize;
                            let max_h = max_line_len.saturating_sub(rough_visible_cols);

                            if mx > split_limit - 16.0 {
                                doc.scroll_col = (doc.scroll_col + 1).min(max_h);
                            } else if mx < text_start_x && doc.scroll_col > 0 {
                                doc.scroll_col = doc.scroll_col.saturating_sub(1);
                            }
                        }

                        let clamped_y = my.clamp(content_y, editor_bottom - 1.0);
                        doc.handle_drag_selection(
                            mx, clamped_y, ws_x, content_y, editor_w, cell_w, cell_h,
                        );
                    }
                } else {
                    self.update_mouse_selection_drag(mx, my);
                }
            }
        }

        // 5. Handle pane drag-and-drop rearrangement
        let target_hit = if self.split_manager.pane_drag.is_some() {
            self.pane_at(mx, my)
        } else {
            None
        };

        if let Some(pane_drag) = self.split_manager.pane_drag.as_mut() {
            pane_drag.current_x = mx;
            pane_drag.current_y = my;

            // Find pane under cursor (other than source)
            pane_drag.hover_target = None;
            if let Some((target_id, px, py, pw, ph)) = target_hit
                && target_id != pane_drag.source_pane_id
            {
                let zone = akssora_terminal_ui::DropZone::from_point(mx, my, px, py, pw, ph);
                pane_drag.hover_target = Some((target_id, zone, (px, py, pw, ph)));
            }
        }
    }
}
