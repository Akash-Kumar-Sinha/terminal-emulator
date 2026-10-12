use crate::cell::Cell;

// INFO: Outcome of a terminal screen and scrollback reflow calculation during geometry changes.
pub struct ReflowResult {
    pub scrollback: Vec<Vec<Cell>>,
    pub scrollback_wrapped: Vec<bool>,
    pub lines: Vec<Vec<Cell>>,
    pub line_wrapped: Vec<bool>,
    pub cursor_row: usize,
    pub cursor_col: usize,
}

struct LogicalLine {
    cells: Vec<Cell>,
    cursor_offset: Option<usize>,
}

// INFO: Intelligently wraps or unwraps terminal lines, preserving logical lines, wrapped relationships, cursor position, and scrollback history.
#[allow(clippy::too_many_arguments)]
pub fn reflow_terminal_content(
    old_scrollback: &[Vec<Cell>],
    old_scrollback_wrapped: &[bool],
    old_lines: &[Vec<Cell>],
    old_line_wrapped: &[bool],
    old_cursor_row: usize,
    old_cursor_col: usize,
    new_rows: usize,
    new_cols: usize,
) -> ReflowResult {
    let new_rows = new_rows.max(1);
    let new_cols = new_cols.max(1);

    // 1. Determine active screen rows.
    let content_rows = {
        let mut last = 0;
        for (i, line) in old_lines.iter().enumerate() {
            if line.iter().any(|c| !c.is_empty()) {
                last = i + 1;
            }
        }
        last.max(1)
    };

    let active_screen_rows = if old_scrollback.is_empty() {
        (old_cursor_row + 1).max(content_rows).min(old_lines.len())
    } else {
        old_lines.len()
    };

    // 2. Assemble physical rows stream.
    let mut phys_rows: Vec<(&[Cell], bool)> = Vec::new();
    for (i, row) in old_scrollback.iter().enumerate() {
        let wrapped = old_scrollback_wrapped.get(i).copied().unwrap_or(false);
        phys_rows.push((row, wrapped));
    }

    let cursor_phys_idx =
        old_scrollback.len() + old_cursor_row.min(active_screen_rows.saturating_sub(1));

    for (i, row) in old_lines.iter().take(active_screen_rows).enumerate() {
        let wrapped = old_line_wrapped.get(i).copied().unwrap_or(false);
        phys_rows.push((row, wrapped));
    }

    // 3. Group physical rows into logical lines.
    let mut logical_lines: Vec<LogicalLine> = Vec::new();
    let mut cur_cells: Vec<Cell> = Vec::new();
    let mut cur_cursor: Option<usize> = None;

    for (p_idx, (row, is_wrapped)) in phys_rows.iter().enumerate() {
        let is_cursor_row = p_idx == cursor_phys_idx;

        if *is_wrapped {
            if is_cursor_row {
                cur_cursor =
                    Some(cur_cells.len() + old_cursor_col.min(row.len().saturating_sub(1)));
            }
            cur_cells.extend_from_slice(row);
        } else {
            let trimmed = row
                .iter()
                .rposition(|c| !c.is_empty())
                .map(|p| p + 1)
                .unwrap_or(0);
            let meaningful_len = if is_cursor_row {
                trimmed.max(old_cursor_col + 1).min(row.len())
            } else {
                trimmed
            };

            if is_cursor_row {
                cur_cursor =
                    Some(cur_cells.len() + old_cursor_col.min(meaningful_len.saturating_sub(1)));
            }

            cur_cells.extend_from_slice(&row[..meaningful_len]);

            logical_lines.push(LogicalLine {
                cells: std::mem::take(&mut cur_cells),
                cursor_offset: cur_cursor.take(),
            });
        }
    }

    if !cur_cells.is_empty() || cur_cursor.is_some() || logical_lines.is_empty() {
        logical_lines.push(LogicalLine {
            cells: cur_cells,
            cursor_offset: cur_cursor,
        });
    }

    // 4. Reflow logical lines into new_cols chunks.
    let mut reflowed_lines: Vec<Vec<Cell>> = Vec::new();
    let mut reflowed_wrapped: Vec<bool> = Vec::new();
    let mut target_cursor: Option<(usize, usize)> = None;

    for ll in logical_lines {
        if ll.cells.is_empty() {
            if let Some(c_offset) = ll.cursor_offset {
                target_cursor = Some((
                    reflowed_lines.len(),
                    c_offset.min(new_cols.saturating_sub(1)),
                ));
            }
            reflowed_lines.push(vec![Cell::default(); new_cols]);
            reflowed_wrapped.push(false);
        } else {
            let chunks: Vec<_> = ll.cells.chunks(new_cols).collect();
            let chunk_count = chunks.len();
            for (c_idx, chunk) in chunks.into_iter().enumerate() {
                let mut line = chunk.to_vec();
                line.resize(new_cols, Cell::default());
                let is_last = c_idx + 1 == chunk_count;
                let wrapped = !is_last;

                if let Some(c_offset) = ll.cursor_offset {
                    let chunk_start = c_idx * new_cols;
                    let chunk_end = chunk_start + new_cols;
                    if c_offset >= chunk_start && (c_offset < chunk_end || is_last) {
                        let col = (c_offset - chunk_start).min(new_cols.saturating_sub(1));
                        target_cursor = Some((reflowed_lines.len(), col));
                    }
                }

                reflowed_lines.push(line);
                reflowed_wrapped.push(wrapped);
            }
        }
    }

    if reflowed_lines.is_empty() {
        reflowed_lines.push(vec![Cell::default(); new_cols]);
        reflowed_wrapped.push(false);
    }

    let (target_line_idx, target_col) = target_cursor.unwrap_or_else(|| {
        (
            reflowed_lines.len().saturating_sub(1),
            old_cursor_col.min(new_cols.saturating_sub(1)),
        )
    });

    // 5. Partition reflowed lines into scrollback and active visible lines.
    let total = reflowed_lines.len();
    if total <= new_rows {
        let mut new_lines = reflowed_lines;
        let mut new_wrapped = reflowed_wrapped;
        while new_lines.len() < new_rows {
            new_lines.push(vec![Cell::default(); new_cols]);
            new_wrapped.push(false);
        }
        ReflowResult {
            scrollback: Vec::new(),
            scrollback_wrapped: Vec::new(),
            lines: new_lines,
            line_wrapped: new_wrapped,
            cursor_row: target_line_idx.min(new_rows.saturating_sub(1)),
            cursor_col: target_col.min(new_cols.saturating_sub(1)),
        }
    } else {
        let screen_start = (target_line_idx + 1)
            .saturating_sub(new_rows)
            .max(total.saturating_sub(new_rows));
        let mut sb: Vec<Vec<Cell>> = reflowed_lines.drain(0..screen_start).collect();
        let mut sb_wrapped: Vec<bool> = reflowed_wrapped.drain(0..screen_start).collect();

        if sb.len() > 10_000 {
            let excess = sb.len() - 10_000;
            sb.drain(0..excess);
            sb_wrapped.drain(0..excess);
        }

        let mut lines = reflowed_lines;
        let mut line_wrapped = reflowed_wrapped;
        while lines.len() < new_rows {
            lines.push(vec![Cell::default(); new_cols]);
            line_wrapped.push(false);
        }

        let cursor_row = target_line_idx
            .saturating_sub(screen_start)
            .min(new_rows.saturating_sub(1));

        ReflowResult {
            scrollback: sb,
            scrollback_wrapped: sb_wrapped,
            lines,
            line_wrapped,
            cursor_row,
            cursor_col: target_col.min(new_cols.saturating_sub(1)),
        }
    }
}
