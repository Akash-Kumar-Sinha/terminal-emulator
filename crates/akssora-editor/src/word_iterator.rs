use crate::buffer::TextBuffer;

// INFO: Distinguishes between standard word (breaks on punctuation) and big word / WORD (breaks only on whitespace).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WordType {
    Default,
    BigWord,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CharClass {
    Whitespace,
    Word,
    Punctuation,
}

fn classify_char(ch: char, word_type: WordType) -> CharClass {
    if ch.is_whitespace() {
        CharClass::Whitespace
    } else if word_type == WordType::BigWord || ch.is_alphanumeric() || ch == '_' {
        CharClass::Word
    } else {
        CharClass::Punctuation
    }
}

// INFO: Moves forward to the start of the next word ('w' / 'W').
pub fn next_word_start(
    buffer: &TextBuffer,
    row: usize,
    col: usize,
    word_type: WordType,
) -> (usize, usize) {
    let line_count = buffer.line_count();
    let mut curr_r = row;
    let curr_c = col;

    let Some(line) = buffer.get_line(curr_r) else {
        return (row, col);
    };
    let chars: Vec<char> = line.chars().collect();
    if curr_c >= chars.len() {
        if curr_r + 1 < line_count {
            return (curr_r + 1, 0);
        }
        return (curr_r, chars.len());
    }

    let initial_class = classify_char(chars[curr_c], word_type);
    let mut c = curr_c;

    // Skip current group
    while c < chars.len() && classify_char(chars[c], word_type) == initial_class {
        c += 1;
    }
    // Skip whitespace
    while c < chars.len() && classify_char(chars[c], word_type) == CharClass::Whitespace {
        c += 1;
    }

    if c < chars.len() {
        (curr_r, c)
    } else if curr_r + 1 < line_count {
        // Move to next line
        curr_r += 1;
        let next_line = buffer.get_line(curr_r).unwrap_or_default();
        let next_chars: Vec<char> = next_line.chars().collect();
        let mut nc = 0;
        while nc < next_chars.len()
            && classify_char(next_chars[nc], word_type) == CharClass::Whitespace
        {
            nc += 1;
        }
        (curr_r, nc)
    } else {
        (curr_r, chars.len())
    }
}

// INFO: Moves forward to the end of the current or next word ('e' / 'E').
pub fn next_word_end(
    buffer: &TextBuffer,
    row: usize,
    col: usize,
    word_type: WordType,
) -> (usize, usize) {
    let line_count = buffer.line_count();
    let mut curr_r = row;

    let Some(line) = buffer.get_line(curr_r) else {
        return (row, col);
    };
    let chars: Vec<char> = line.chars().collect();
    let mut c = col + 1;

    // Skip whitespace
    while c < chars.len() && classify_char(chars[c], word_type) == CharClass::Whitespace {
        c += 1;
    }

    if c >= chars.len() {
        if curr_r + 1 < line_count {
            curr_r += 1;
            let next_line = buffer.get_line(curr_r).unwrap_or_default();
            let next_chars: Vec<char> = next_line.chars().collect();
            let mut nc = 0;
            while nc < next_chars.len()
                && classify_char(next_chars[nc], word_type) == CharClass::Whitespace
            {
                nc += 1;
            }
            if nc < next_chars.len() {
                let target_class = classify_char(next_chars[nc], word_type);
                while nc + 1 < next_chars.len()
                    && classify_char(next_chars[nc + 1], word_type) == target_class
                {
                    nc += 1;
                }
                return (curr_r, nc);
            }
            return (curr_r, 0);
        }
        return (curr_r, chars.len().saturating_sub(1));
    }

    let target_class = classify_char(chars[c], word_type);
    while c + 1 < chars.len() && classify_char(chars[c + 1], word_type) == target_class {
        c += 1;
    }

    (curr_r, c)
}

// INFO: Moves backward to the start of the current or previous word ('b' / 'B').
pub fn prev_word_start(
    buffer: &TextBuffer,
    row: usize,
    col: usize,
    word_type: WordType,
) -> (usize, usize) {
    let mut curr_r = row;

    if col == 0 {
        if curr_r > 0 {
            curr_r -= 1;
            let prev_len = buffer.line_len(curr_r);
            return (curr_r, prev_len);
        }
        return (0, 0);
    }

    let Some(line) = buffer.get_line(curr_r) else {
        return (row, col);
    };
    let chars: Vec<char> = line.chars().collect();
    let mut c = col.saturating_sub(1);

    // Skip whitespace backward
    while c > 0 && classify_char(chars[c], word_type) == CharClass::Whitespace {
        c -= 1;
    }

    let target_class = classify_char(chars[c], word_type);
    while c > 0 && classify_char(chars[c - 1], word_type) == target_class {
        c -= 1;
    }

    (curr_r, c)
}

// INFO: Computes word text-object bounds (inner word 'iw' or around word 'aw').
pub fn word_text_object_range(
    buffer: &TextBuffer,
    row: usize,
    col: usize,
    around: bool,
) -> Option<((usize, usize), (usize, usize))> {
    let line = buffer.get_line(row)?;
    let chars: Vec<char> = line.chars().collect();
    if chars.is_empty() {
        return None;
    }

    let c = col.min(chars.len().saturating_sub(1));
    let target_class = classify_char(chars[c], WordType::Default);

    let mut start_c = c;
    while start_c > 0 && classify_char(chars[start_c - 1], WordType::Default) == target_class {
        start_c -= 1;
    }

    let mut end_c = c;
    while end_c + 1 < chars.len()
        && classify_char(chars[end_c + 1], WordType::Default) == target_class
    {
        end_c += 1;
    }

    if around {
        // Include trailing or leading whitespace
        while end_c + 1 < chars.len() && chars[end_c + 1].is_whitespace() {
            end_c += 1;
        }
        if end_c == c && start_c > 0 {
            while start_c > 0 && chars[start_c - 1].is_whitespace() {
                start_c -= 1;
            }
        }
    }

    Some(((row, start_c), (row, end_c + 1)))
}
