use crate::buffer::TextBuffer;

// INFO: Editor search configuration defining match criteria.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SearchQuery {
    pub text: String,
    pub case_sensitive: bool,
    pub whole_word: bool,
}

// INFO: Coordinates of a search match in buffer line space.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SearchMatch {
    pub row: usize,
    pub col_start: usize,
    pub col_end: usize,
}

// INFO: Search and replace state model coordinating matches, active index, and replace buffer.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SearchModel {
    pub query: SearchQuery,
    pub replace_text: String,
    pub matches: Vec<SearchMatch>,
    pub active_match_idx: Option<usize>,
    pub is_open: bool,
    pub is_replace_open: bool,
}

impl SearchModel {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_query(&mut self, text: impl Into<String>, buffer: &TextBuffer) {
        self.query.text = text.into();
        self.refresh_matches(buffer);
    }

    pub fn refresh_matches(&mut self, buffer: &TextBuffer) {
        self.matches.clear();
        if self.query.text.is_empty() {
            self.active_match_idx = None;
            return;
        }

        let query_text = if self.query.case_sensitive {
            self.query.text.clone()
        } else {
            self.query.text.to_lowercase()
        };

        let total_lines = buffer.line_count();
        for row in 0..total_lines {
            let Some(line) = buffer.get_line(row) else {
                continue;
            };

            let search_line = if self.query.case_sensitive {
                line.clone()
            } else {
                line.to_lowercase()
            };

            let mut start_idx = 0;
            while let Some(found_byte) = search_line[start_idx..].find(&query_text) {
                let actual_byte_start = start_idx + found_byte;
                let actual_byte_end = actual_byte_start + query_text.len();

                // Convert byte positions to char offsets
                let col_start = line[..actual_byte_start].chars().count();
                let col_end = line[..actual_byte_end].chars().count();

                if self.query.whole_word {
                    let chars: Vec<char> = line.chars().collect();
                    let before_is_word = col_start > 0
                        && (chars[col_start - 1].is_alphanumeric() || chars[col_start - 1] == '_');
                    let after_is_word = col_end < chars.len()
                        && (chars[col_end].is_alphanumeric() || chars[col_end] == '_');

                    if !before_is_word && !after_is_word {
                        self.matches.push(SearchMatch {
                            row,
                            col_start,
                            col_end,
                        });
                    }
                } else {
                    self.matches.push(SearchMatch {
                        row,
                        col_start,
                        col_end,
                    });
                }

                start_idx = actual_byte_start + query_text.len().max(1);
                if start_idx >= search_line.len() {
                    break;
                }
            }
        }

        if !self.matches.is_empty() {
            if let Some(curr) = self.active_match_idx {
                self.active_match_idx = Some(curr.min(self.matches.len() - 1));
            } else {
                self.active_match_idx = Some(0);
            }
        } else {
            self.active_match_idx = None;
        }
    }

    pub fn current_match(&self) -> Option<SearchMatch> {
        self.active_match_idx
            .and_then(|idx| self.matches.get(idx).copied())
    }

    pub fn next_match(&mut self) -> Option<SearchMatch> {
        if self.matches.is_empty() {
            return None;
        }
        let next_idx = match self.active_match_idx {
            Some(i) => (i + 1) % self.matches.len(),
            None => 0,
        };
        self.active_match_idx = Some(next_idx);
        Some(self.matches[next_idx])
    }

    pub fn prev_match(&mut self) -> Option<SearchMatch> {
        if self.matches.is_empty() {
            return None;
        }
        let prev_idx = match self.active_match_idx {
            Some(0) | None => self.matches.len() - 1,
            Some(i) => i - 1,
        };
        self.active_match_idx = Some(prev_idx);
        Some(self.matches[prev_idx])
    }
}
