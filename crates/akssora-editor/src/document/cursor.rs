use super::Document;
use crate::wrap::WrapLayout;

impl Document {
    pub fn clamp_cursor(&mut self) {
        let max_row = self.buffer.line_count().saturating_sub(1);
        let row = self.cursor.0.min(max_row);
        let max_col = self.buffer.line_len(row);
        let col = self.cursor.1.min(max_col);
        self.cursor = (row, col);
    }

    pub fn move_cursor_left(&mut self, select: bool) {
        if select && self.selection_anchor.is_none() {
            self.selection_anchor = Some(self.cursor);
        } else if !select {
            self.selection_anchor = None;
        }

        let (row, col) = self.cursor;
        if col > 0 {
            self.cursor = (row, col - 1);
        } else if row > 0 {
            let prev_row = row - 1;
            let prev_col = self.buffer.line_len(prev_row);
            self.cursor = (prev_row, prev_col);
        }
    }

    pub fn move_cursor_right(&mut self, select: bool) {
        if select && self.selection_anchor.is_none() {
            self.selection_anchor = Some(self.cursor);
        } else if !select {
            self.selection_anchor = None;
        }

        let (row, col) = self.cursor;
        let line_len = self.buffer.line_len(row);
        if col < line_len {
            self.cursor = (row, col + 1);
        } else if row + 1 < self.buffer.line_count() {
            self.cursor = (row + 1, 0);
        }
    }

    pub fn move_cursor_up(&mut self, select: bool) {
        if select && self.selection_anchor.is_none() {
            self.selection_anchor = Some(self.cursor);
        } else if !select {
            self.selection_anchor = None;
        }

        let (row, col) = self.cursor;
        if row > 0 {
            let new_row = row - 1;
            let max_col = self.buffer.line_len(new_row);
            self.cursor = (new_row, col.min(max_col));
        } else {
            self.cursor = (0, 0);
        }
    }

    pub fn move_cursor_up_visual(&mut self, select: bool, wrap_width: usize) {
        if !self.line_wrap {
            self.move_cursor_up(select);
            return;
        }

        if select && self.selection_anchor.is_none() {
            self.selection_anchor = Some(self.cursor);
        } else if !select {
            self.selection_anchor = None;
        }

        let layout = self.wrap_layout(wrap_width);
        let (vis_row, vis_col) = layout.logical_to_visual(self.cursor.0, self.cursor.1);
        if vis_row > 0 {
            let target_vis_row = vis_row - 1;
            let (new_row, new_col) = layout.visual_to_logical(target_vis_row, vis_col);
            self.cursor = (new_row, new_col);
        } else {
            self.cursor = (0, 0);
        }
    }

    pub fn move_cursor_down(&mut self, select: bool) {
        if select && self.selection_anchor.is_none() {
            self.selection_anchor = Some(self.cursor);
        } else if !select {
            self.selection_anchor = None;
        }

        let (row, col) = self.cursor;
        if row + 1 < self.buffer.line_count() {
            let new_row = row + 1;
            let max_col = self.buffer.line_len(new_row);
            self.cursor = (new_row, col.min(max_col));
        } else {
            let last_row = self.buffer.line_count().saturating_sub(1);
            let last_col = self.buffer.line_len(last_row);
            self.cursor = (last_row, last_col);
        }
    }

    pub fn move_cursor_down_visual(&mut self, select: bool, wrap_width: usize) {
        if !self.line_wrap {
            self.move_cursor_down(select);
            return;
        }

        if select && self.selection_anchor.is_none() {
            self.selection_anchor = Some(self.cursor);
        } else if !select {
            self.selection_anchor = None;
        }

        let layout = self.wrap_layout(wrap_width);
        let (vis_row, vis_col) = layout.logical_to_visual(self.cursor.0, self.cursor.1);
        let total_vis = layout.total_visual_rows();
        if vis_row + 1 < total_vis {
            let target_vis_row = vis_row + 1;
            let (new_row, new_col) = layout.visual_to_logical(target_vis_row, vis_col);
            self.cursor = (new_row, new_col);
        } else {
            let last_row = self.buffer.line_count().saturating_sub(1);
            let last_col = self.buffer.line_len(last_row);
            self.cursor = (last_row, last_col);
        }
    }

    pub fn move_word_forward(&mut self, select: bool) {
        if select && self.selection_anchor.is_none() {
            self.selection_anchor = Some(self.cursor);
        } else if !select {
            self.selection_anchor = None;
        }

        let (row, col) = self.cursor;
        let line = self.buffer.get_line(row).unwrap_or_default();
        let chars: Vec<char> = line.chars().collect();
        let mut c = col;

        while c < chars.len() && !chars[c].is_whitespace() {
            c += 1;
        }
        while c < chars.len() && chars[c].is_whitespace() {
            c += 1;
        }

        if c < chars.len() {
            self.cursor = (row, c);
        } else if row + 1 < self.buffer.line_count() {
            self.cursor = (row + 1, 0);
        } else {
            self.cursor = (row, chars.len());
        }
    }

    pub fn move_word_backward(&mut self, select: bool) {
        if select && self.selection_anchor.is_none() {
            self.selection_anchor = Some(self.cursor);
        } else if !select {
            self.selection_anchor = None;
        }

        let (row, col) = self.cursor;
        if col == 0 {
            if row > 0 {
                let prev_row = row - 1;
                let prev_len = self.buffer.line_len(prev_row);
                self.cursor = (prev_row, prev_len);
            }
            return;
        }

        let line = self.buffer.get_line(row).unwrap_or_default();
        let chars: Vec<char> = line.chars().collect();
        let mut c = col.saturating_sub(1);

        while c > 0 && chars[c].is_whitespace() {
            c -= 1;
        }
        while c > 0 && !chars[c - 1].is_whitespace() {
            c -= 1;
        }

        self.cursor = (row, c);
    }

    pub fn move_to_line_start(&mut self, select: bool) {
        if select && self.selection_anchor.is_none() {
            self.selection_anchor = Some(self.cursor);
        } else if !select {
            self.selection_anchor = None;
        }
        self.cursor.1 = 0;
    }

    pub fn move_to_line_end(&mut self, select: bool) {
        if select && self.selection_anchor.is_none() {
            self.selection_anchor = Some(self.cursor);
        } else if !select {
            self.selection_anchor = None;
        }
        self.cursor.1 = self.buffer.line_len(self.cursor.0);
    }

    pub fn move_to_line_start_visual(&mut self, select: bool, wrap_width: usize) {
        if !self.line_wrap {
            self.move_to_line_start(select);
            return;
        }

        if select && self.selection_anchor.is_none() {
            self.selection_anchor = Some(self.cursor);
        } else if !select {
            self.selection_anchor = None;
        }

        let layout = self.wrap_layout(wrap_width);
        let r = self.cursor.0.min(layout.line_rows.len().saturating_sub(1));
        let sub_rows = &layout.line_rows[r];
        for vr in sub_rows.iter() {
            if self.cursor.1 > vr.char_start && self.cursor.1 <= vr.char_end {
                self.cursor.1 = vr.char_start;
                return;
            }
        }
        self.cursor.1 = 0;
    }

    pub fn move_to_line_end_visual(&mut self, select: bool, wrap_width: usize) {
        if !self.line_wrap {
            self.move_to_line_end(select);
            return;
        }

        if select && self.selection_anchor.is_none() {
            self.selection_anchor = Some(self.cursor);
        } else if !select {
            self.selection_anchor = None;
        }

        let layout = self.wrap_layout(wrap_width);
        let r = self.cursor.0.min(layout.line_rows.len().saturating_sub(1));
        let sub_rows = &layout.line_rows[r];
        let max_line_len = self.buffer.line_len(r);

        for vr in sub_rows.iter() {
            if self.cursor.1 >= vr.char_start && self.cursor.1 < vr.char_end {
                self.cursor.1 = vr.char_end;
                return;
            }
        }
        self.cursor.1 = max_line_len;
    }

    pub fn scroll_to_cursor(&mut self, visible_rows: usize, visible_cols: usize) {
        if self.line_wrap {
            let layout = self.wrap_layout(visible_cols);
            let (vis_row, _) = layout.logical_to_visual(self.cursor.0, self.cursor.1);
            if vis_row < self.scroll_row {
                self.scroll_row = vis_row;
            } else if vis_row >= self.scroll_row + visible_rows {
                self.scroll_row = vis_row.saturating_sub(visible_rows.saturating_sub(1));
            }
            self.scroll_col = 0;
            return;
        }

        let (row, col) = self.cursor;
        if row < self.scroll_row {
            self.scroll_row = row;
        } else if row >= self.scroll_row + visible_rows {
            self.scroll_row = row.saturating_sub(visible_rows - 1);
        }

        if col < self.scroll_col {
            self.scroll_col = col;
        } else if col >= self.scroll_col + visible_cols {
            self.scroll_col = col.saturating_sub(visible_cols - 1);
        }

        self.clamp_scroll(visible_rows, visible_cols);
    }

    pub fn clamp_scroll(&mut self, visible_rows: usize, visible_cols: usize) {
        let total_lines = if self.line_wrap {
            self.total_visual_rows(visible_cols)
        } else {
            self.buffer.line_count()
        };
        let max_v = total_lines.saturating_sub(visible_rows);
        self.scroll_row = self.scroll_row.min(max_v);

        if self.line_wrap {
            self.scroll_col = 0;
        } else {
            let max_h = self.max_line_len().saturating_sub(visible_cols);
            self.scroll_col = self.scroll_col.min(max_h);
        }

        let max_preview = self.buffer.line_count().saturating_sub(visible_rows);
        self.preview_scroll_row = self.preview_scroll_row.min(max_preview);
    }

    pub fn wrap_layout(&self, wrap_width: usize) -> WrapLayout {
        let mut cached = self.cached_wrap_layout.borrow_mut();
        if let Some(layout) = cached.as_ref()
            && layout.wrap_width == wrap_width
        {
            return layout.clone();
        }
        let new_layout = WrapLayout::compute(&self.buffer, wrap_width, self.line_wrap);
        *cached = Some(new_layout.clone());
        new_layout
    }

    pub fn current_wrap_width(&self) -> Option<usize> {
        self.cached_wrap_layout
            .borrow()
            .as_ref()
            .map(|l| l.wrap_width)
    }

    pub fn total_visual_rows(&self, wrap_width: usize) -> usize {
        if !self.line_wrap {
            self.buffer.line_count()
        } else {
            self.wrap_layout(wrap_width).total_visual_rows()
        }
    }

    pub fn logical_to_visual(&self, row: usize, col: usize, wrap_width: usize) -> (usize, usize) {
        if !self.line_wrap {
            (row, col)
        } else {
            self.wrap_layout(wrap_width).logical_to_visual(row, col)
        }
    }

    pub fn visual_to_logical(
        &self,
        vis_row: usize,
        vis_col: usize,
        wrap_width: usize,
    ) -> (usize, usize) {
        if !self.line_wrap {
            let max_row = self.buffer.line_count().saturating_sub(1);
            let actual_row = vis_row.min(max_row);
            let max_col = self.buffer.line_len(actual_row);
            (actual_row, vis_col.min(max_col))
        } else {
            self.wrap_layout(wrap_width)
                .visual_to_logical(vis_row, vis_col)
        }
    }

    pub fn invalidate_wrap_cache(&self) {
        self.cached_wrap_layout.borrow_mut().take();
    }

    // INFO: Returns the maximum line length, caching the result to avoid re-scanning on every scroll event.
    pub fn max_line_len(&self) -> usize {
        if let Some(cached) = self.cached_max_line_len.get() {
            return cached;
        }
        let count = self.buffer.line_count();
        let max_len = (0..count)
            .map(|r| self.buffer.line_len(r))
            .max()
            .unwrap_or(0);
        self.cached_max_line_len.set(Some(max_len));
        max_len
    }
}
