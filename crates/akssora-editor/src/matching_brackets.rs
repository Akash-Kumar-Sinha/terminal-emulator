use crate::buffer::TextBuffer;

// INFO: Finds matching bracket pair for parentheses, square brackets, curly braces, and angle brackets.
pub fn find_matching_bracket(
    buffer: &TextBuffer,
    row: usize,
    col: usize,
) -> Option<(usize, usize)> {
    let line = buffer.get_line(row)?;
    let chars: Vec<char> = line.chars().collect();
    if col >= chars.len() {
        return None;
    }

    let ch = chars[col];
    let (match_target, forward) = match ch {
        '(' => (')', true),
        '[' => (']', true),
        '{' => ('}', true),
        '<' => ('>', true),
        ')' => ('(', false),
        ']' => ('[', false),
        '}' => ('{', false),
        '>' => ('<', false),
        _ => return None,
    };

    let total_lines = buffer.line_count();
    let mut depth = 0;

    if forward {
        for r in row..total_lines {
            let Some(curr_line) = buffer.get_line(r) else {
                continue;
            };
            let start_c = if r == row { col } else { 0 };
            for (c, curr_char) in curr_line.chars().enumerate().skip(start_c) {
                if curr_char == ch {
                    depth += 1;
                } else if curr_char == match_target {
                    depth -= 1;
                    if depth == 0 {
                        return Some((r, c));
                    }
                }
            }
        }
    } else {
        for r in (0..=row).rev() {
            let Some(curr_line) = buffer.get_line(r) else {
                continue;
            };
            let line_chars: Vec<char> = curr_line.chars().collect();
            let end_c = if r == row {
                col
            } else {
                line_chars.len().saturating_sub(1)
            };
            for c in (0..=end_c).rev() {
                if c < line_chars.len() {
                    let curr_char = line_chars[c];
                    if curr_char == ch {
                        depth += 1;
                    } else if curr_char == match_target {
                        depth -= 1;
                        if depth == 0 {
                            return Some((r, c));
                        }
                    }
                }
            }
        }
    }

    None
}
