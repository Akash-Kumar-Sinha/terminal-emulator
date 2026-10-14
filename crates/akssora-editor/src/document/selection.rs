use akssora_terminal_config::LineNumberMode;

use super::Document;

impl Document {
    pub fn has_selection(&self) -> bool {
        if let Some(anchor) = self.selection_anchor {
            anchor != self.cursor
        } else {
            false
        }
    }

    pub fn selection_range(&self) -> Option<((usize, usize), (usize, usize))> {
        let anchor = self.selection_anchor?;
        if anchor == self.cursor {
            return None;
        }
        if anchor <= self.cursor {
            Some((anchor, self.cursor))
        } else {
            Some((self.cursor, anchor))
        }
    }

    pub fn clear_selection(&mut self) {
        self.selection_anchor = None;
    }

    pub fn select_all(&mut self) {
        self.selection_anchor = Some((0, 0));
        let last_row = self.buffer.line_count().saturating_sub(1);
        let last_col = self.buffer.line_len(last_row);
        self.cursor = (last_row, last_col);
    }

    pub fn selected_text(&self) -> Option<String> {
        let ((start_row, start_col), (end_row, end_col)) = self.selection_range()?;
        let start_idx = self.buffer.pos_to_char_idx(start_row, start_col);
        let end_idx = self.buffer.pos_to_char_idx(end_row, end_col);
        if start_idx >= end_idx {
            return None;
        }
        let slice = self.buffer.rope().slice(start_idx..end_idx);
        Some(slice.to_string())
    }

    pub fn delete_selection(&mut self) {
        let Some(((start_row, start_col), (end_row, end_col))) = self.selection_range() else {
            return;
        };

        let start_idx = self.buffer.pos_to_char_idx(start_row, start_col);
        let end_idx = self.buffer.pos_to_char_idx(end_row, end_col);
        self.buffer.delete_range(start_idx, end_idx);
        self.cursor = (start_row, start_col);
        self.selection_anchor = None;
    }

    // INFO: Updates the selection cursor during mouse drag gestures relative to editor content bounds.
    #[allow(clippy::too_many_arguments)]
    pub fn handle_drag_selection(
        &mut self,
        drag_x: f32,
        drag_y: f32,
        content_x: f32,
        content_y: f32,
        content_w: f32,
        cell_w: f32,
        cell_h: f32,
    ) {
        if self.selection_anchor.is_none() {
            self.selection_anchor = Some(self.cursor);
        }

        let gutter_w = if self.line_numbers && self.line_number_mode != LineNumberMode::Off {
            48.0
        } else {
            0.0
        };
        let effective_line_h = (cell_h * self.line_height).max(cell_h);
        let text_x = content_x + gutter_w + 8.0;
        let vis_row = self.scroll_row + ((drag_y - content_y).max(0.0) / effective_line_h) as usize;
        let is_in_gutter = drag_x < text_x;

        if self.line_wrap {
            let wrap_w = if let Some(layout) = self.cached_wrap_layout.borrow().as_ref() {
                layout.wrap_width
            } else {
                let max_text_w = (content_w - gutter_w - 16.0 - 8.0).max(0.0);
                (max_text_w / cell_w).floor().max(1.0) as usize
            };
            let (actual_row, actual_col) = if is_in_gutter {
                let (r, _) = self.visual_to_logical(vis_row, 0, wrap_w);
                (r, 0)
            } else {
                let vis_col = ((drag_x - text_x).max(0.0) / cell_w).round() as usize;
                self.visual_to_logical(vis_row, vis_col, wrap_w)
            };
            self.cursor = (actual_row, actual_col);
        } else {
            let max_row = self.buffer.line_count().saturating_sub(1);
            let actual_row = vis_row.min(max_row);
            let line_len = self.buffer.line_len(actual_row);
            let actual_col = if is_in_gutter {
                0
            } else {
                let vis_col = ((drag_x - text_x).max(0.0) / cell_w).round() as usize;
                (self.scroll_col + vis_col).min(line_len)
            };
            self.cursor = (actual_row, actual_col);
        }
    }
}
