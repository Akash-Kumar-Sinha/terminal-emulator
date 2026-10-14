use crate::buffer::TextBuffer;

// INFO: Jumps forward to the next paragraph boundary (empty line) ('}').
pub fn next_paragraph(buffer: &TextBuffer, row: usize) -> (usize, usize) {
    let line_count = buffer.line_count();
    let mut r = row + 1;

    // If currently on empty line, skip empty lines first
    while r < line_count {
        let is_empty = buffer
            .get_line(r)
            .map(|l| l.trim().is_empty())
            .unwrap_or(true);
        if !is_empty {
            break;
        }
        r += 1;
    }

    // Now scan until next empty line
    while r < line_count {
        let is_empty = buffer
            .get_line(r)
            .map(|l| l.trim().is_empty())
            .unwrap_or(true);
        if is_empty {
            return (r, 0);
        }
        r += 1;
    }

    (
        line_count.saturating_sub(1),
        buffer.line_len(line_count.saturating_sub(1)),
    )
}

// INFO: Jumps backward to the previous paragraph boundary (empty line) ('{').
pub fn prev_paragraph(buffer: &TextBuffer, row: usize) -> (usize, usize) {
    if row == 0 {
        return (0, 0);
    }
    let mut r = row.saturating_sub(1);

    // If currently on empty line, skip empty lines backward
    while r > 0 {
        let is_empty = buffer
            .get_line(r)
            .map(|l| l.trim().is_empty())
            .unwrap_or(true);
        if !is_empty {
            break;
        }
        r -= 1;
    }

    // Now scan backward until next empty line
    while r > 0 {
        let is_empty = buffer
            .get_line(r)
            .map(|l| l.trim().is_empty())
            .unwrap_or(true);
        if is_empty {
            return (r, 0);
        }
        r -= 1;
    }

    (0, 0)
}
