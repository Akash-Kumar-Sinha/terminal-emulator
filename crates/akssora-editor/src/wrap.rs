use crate::buffer::TextBuffer;

// INFO: Soft-wrapping layout engine mapping logical lines to wrapped visual rows.

/// Represents a single visual row of a logical line when displayed in the editor.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct VisualRow {
    /// The 0-based index of the logical line in the document buffer.
    pub logical_row: usize,
    /// Character offset (inclusive) within the logical line where this visual row starts.
    pub char_start: usize,
    /// Character offset (exclusive) within the logical line where this visual row ends.
    pub char_end: usize,
    /// Whether this is a continuation row (i.e. 2nd, 3rd, etc. visual row of the logical line).
    pub is_continuation: bool,
}

impl VisualRow {
    #[inline]
    pub fn char_len(&self) -> usize {
        self.char_end.saturating_sub(self.char_start)
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.char_start == self.char_end
    }
}

/// Splits a single logical line into visual rows according to the specified column width.
///
/// Preserves the original document characters without modifying or dropping any characters.
/// Empty lines produce exactly one non-continuation visual row with len 0.
pub fn wrap_line(logical_row: usize, text: &str, wrap_width: usize) -> Vec<VisualRow> {
    let chars: Vec<char> = text.chars().collect();
    let total_chars = chars.len();
    let w = wrap_width.max(1);

    if total_chars == 0 {
        return vec![VisualRow {
            logical_row,
            char_start: 0,
            char_end: 0,
            is_continuation: false,
        }];
    }

    let mut rows = Vec::new();
    let mut start = 0;

    while start < total_chars {
        let remaining = total_chars - start;
        if remaining <= w {
            rows.push(VisualRow {
                logical_row,
                char_start: start,
                char_end: total_chars,
                is_continuation: !rows.is_empty(),
            });
            break;
        }

        let max_end = start + w;
        let mut break_idx = None;

        // Search backwards from max_end down to start + 1 for whitespace
        for i in (start + 1..=max_end).rev() {
            if chars[i - 1].is_whitespace() {
                break_idx = Some(i);
                break;
            }
        }

        // If no whitespace boundary is found, check common punctuation boundaries if line width is wide enough
        if break_idx.is_none() && w > 8 {
            for i in (start + (w / 2)..=max_end).rev() {
                let ch = chars[i - 1];
                if ch == ','
                    || ch == ';'
                    || ch == ':'
                    || ch == '('
                    || ch == ')'
                    || ch == '{'
                    || ch == '}'
                    || ch == '['
                    || ch == ']'
                {
                    break_idx = Some(i);
                    break;
                }
            }
        }

        let end = break_idx.unwrap_or(max_end);
        rows.push(VisualRow {
            logical_row,
            char_start: start,
            char_end: end,
            is_continuation: !rows.is_empty(),
        });
        start = end;
    }

    rows
}

/// Cached wrapped layout for the document buffer given a specific wrap column width.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WrapLayout {
    pub wrap_width: usize,
    /// Per-logical-line visual rows
    pub line_rows: Vec<Vec<VisualRow>>,
    /// Prefix sum: `line_start_vis[r]` is the visual row index where logical line `r` begins.
    /// Has length `line_rows.len() + 1`, where the last element equals `total_visual_rows()`.
    pub line_start_vis: Vec<usize>,
}

impl WrapLayout {
    /// Computes full wrap layout for a text buffer.
    pub fn compute(buffer: &TextBuffer, wrap_width: usize, line_wrap: bool) -> Self {
        let line_count = buffer.line_count();
        let mut line_rows = Vec::with_capacity(line_count);
        let mut line_start_vis = Vec::with_capacity(line_count + 1);
        let mut total_vis = 0;

        line_start_vis.push(0);

        for r in 0..line_count {
            let line_text = buffer.get_line(r).unwrap_or_default();
            let rows = if line_wrap {
                wrap_line(r, &line_text, wrap_width)
            } else {
                let len = line_text.chars().count();
                vec![VisualRow {
                    logical_row: r,
                    char_start: 0,
                    char_end: len,
                    is_continuation: false,
                }]
            };

            total_vis += rows.len();
            line_start_vis.push(total_vis);
            line_rows.push(rows);
        }

        Self {
            wrap_width,
            line_rows,
            line_start_vis,
        }
    }

    #[inline]
    pub fn total_visual_rows(&self) -> usize {
        *self.line_start_vis.last().unwrap_or(&0)
    }

    /// Maps a logical position `(logical_row, char_col)` to `(visual_row, visual_col)`.
    pub fn logical_to_visual(&self, logical_row: usize, char_col: usize) -> (usize, usize) {
        if self.line_rows.is_empty() {
            return (0, 0);
        }

        let r = logical_row.min(self.line_rows.len() - 1);
        let start_vis = self.line_start_vis[r];
        let sub_rows = &self.line_rows[r];

        if sub_rows.is_empty() {
            return (start_vis, 0);
        }

        for (idx, vr) in sub_rows.iter().enumerate() {
            let is_last = idx == sub_rows.len() - 1;
            if char_col >= vr.char_start && (char_col < vr.char_end || is_last) {
                let vis_row = start_vis + idx;
                let vis_col = char_col.saturating_sub(vr.char_start);
                return (vis_row, vis_col);
            }
        }

        // Past the last visual row
        let last_idx = sub_rows.len() - 1;
        let last_vr = &sub_rows[last_idx];
        let vis_row = start_vis + last_idx;
        let vis_col = char_col.saturating_sub(last_vr.char_start);
        (vis_row, vis_col)
    }

    /// Maps a visual position `(visual_row, visual_col)` to `(logical_row, char_col)`.
    pub fn visual_to_logical(&self, visual_row: usize, visual_col: usize) -> (usize, usize) {
        let total = self.total_visual_rows();
        if total == 0 {
            return (0, 0);
        }

        let target_vis = visual_row.min(total - 1);

        // Binary search in `line_start_vis` to find the logical line
        let r = match self.line_start_vis.binary_search(&target_vis) {
            Ok(idx) => {
                if idx < self.line_rows.len() {
                    idx
                } else {
                    idx.saturating_sub(1)
                }
            }
            Err(idx) => idx.saturating_sub(1),
        };

        let start_vis = self.line_start_vis[r];
        let sub_idx = target_vis.saturating_sub(start_vis);
        let sub_rows = &self.line_rows[r];

        if sub_idx < sub_rows.len() {
            let vr = &sub_rows[sub_idx];
            let is_last_sub = sub_idx == sub_rows.len() - 1;
            let max_char = if is_last_sub {
                vr.char_end
            } else {
                vr.char_end.saturating_sub(1).max(vr.char_start)
            };
            let char_col = (vr.char_start + visual_col).min(max_char);
            (r, char_col)
        } else if let Some(last_vr) = sub_rows.last() {
            (r, last_vr.char_end)
        } else {
            (r, 0)
        }
    }

    /// Returns the `VisualRow` at the given visual row index.
    pub fn get_visual_row(&self, visual_row: usize) -> Option<&VisualRow> {
        let total = self.total_visual_rows();
        if visual_row >= total {
            return None;
        }

        let r = match self.line_start_vis.binary_search(&visual_row) {
            Ok(idx) => {
                if idx < self.line_rows.len() {
                    idx
                } else {
                    idx.saturating_sub(1)
                }
            }
            Err(idx) => idx.saturating_sub(1),
        };

        let start_vis = self.line_start_vis[r];
        let sub_idx = visual_row - start_vis;
        self.line_rows.get(r)?.get(sub_idx)
    }

    /// Incremental line update: re-wraps only `logical_row` and updates visual offsets.
    pub fn update_line(&mut self, buffer: &TextBuffer, logical_row: usize, line_wrap: bool) {
        if logical_row >= self.line_rows.len() || buffer.line_count() != self.line_rows.len() {
            *self = Self::compute(buffer, self.wrap_width, line_wrap);
            return;
        }

        let line_text = buffer.get_line(logical_row).unwrap_or_default();
        let new_rows = if line_wrap {
            wrap_line(logical_row, &line_text, self.wrap_width)
        } else {
            let len = line_text.chars().count();
            vec![VisualRow {
                logical_row,
                char_start: 0,
                char_end: len,
                is_continuation: false,
            }]
        };

        self.line_rows[logical_row] = new_rows;

        // Re-accumulate line_start_vis from logical_row to the end
        let mut total = self.line_start_vis[logical_row];
        for r in logical_row..self.line_rows.len() {
            total += self.line_rows[r].len();
            self.line_start_vis[r + 1] = total;
        }
    }
}
