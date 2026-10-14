use std::fs;
use std::path::{Path, PathBuf};

// INFO: Search candidate matching a file path within the workspace root.
#[derive(Clone, Debug, PartialEq)]
pub struct QuickSearchResult {
    pub path: PathBuf,
    pub relative_path: String,
    pub filename: String,
    pub score: i32,
}

// INFO: Command-palette quick file search state and fuzzy matching controller.
#[derive(Clone, Debug)]
pub struct QuickSearchUi {
    pub is_open: bool,
    pub query: String,
    pub selected_index: usize,
    pub results: Vec<QuickSearchResult>,
    pub max_results: usize,
    pub scroll_offset: usize,
}

impl Default for QuickSearchUi {
    fn default() -> Self {
        Self::new()
    }
}

impl QuickSearchUi {
    pub const OVERLAY_WIDTH: f32 = 560.0;
    pub const INPUT_HEIGHT: f32 = 36.0;
    pub const ITEM_HEIGHT: f32 = 28.0;
    pub const MAX_VISIBLE_ITEMS: usize = 10;

    pub fn new() -> Self {
        Self {
            is_open: false,
            query: String::new(),
            selected_index: 0,
            results: Vec::new(),
            max_results: 50,
            scroll_offset: 0,
        }
    }

    // INFO: Opens the quick search palette, clears query, and scans initial workspace files.
    pub fn open(&mut self, root: &Path, exclusions: &[String], show_hidden: bool) {
        self.is_open = true;
        self.query.clear();
        self.selected_index = 0;
        self.scroll_offset = 0;
        self.update_search(root, exclusions, show_hidden);
    }

    // INFO: Closes the quick search palette and clears search state.
    pub fn close(&mut self) {
        self.is_open = false;
        self.query.clear();
        self.results.clear();
        self.selected_index = 0;
        self.scroll_offset = 0;
    }

    // INFO: Toggles the visibility of the quick search overlay.
    pub fn toggle(&mut self, root: &Path, exclusions: &[String], show_hidden: bool) {
        if self.is_open {
            self.close();
        } else {
            self.open(root, exclusions, show_hidden);
        }
    }

    pub fn select_next(&mut self) {
        if !self.results.is_empty() {
            self.selected_index = (self.selected_index + 1).min(self.results.len() - 1);
            if self.selected_index >= self.scroll_offset + Self::MAX_VISIBLE_ITEMS {
                self.scroll_offset = self.selected_index + 1 - Self::MAX_VISIBLE_ITEMS;
            }
        }
    }

    pub fn select_prev(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
            if self.selected_index < self.scroll_offset {
                self.scroll_offset = self.selected_index;
            }
        }
    }

    pub fn selected_result(&self) -> Option<&QuickSearchResult> {
        self.results.get(self.selected_index)
    }

    // INFO: Updates search results based on current query and workspace directory contents.
    pub fn update_search(&mut self, root: &Path, exclusions: &[String], show_hidden: bool) {
        let mut candidates = Vec::new();
        let mut scanned_count = 0;
        scan_dir_files(
            root,
            root,
            exclusions,
            show_hidden,
            &mut candidates,
            &mut scanned_count,
            3000,
        );

        let query = self.query.trim();
        let mut scored: Vec<QuickSearchResult> = candidates
            .into_iter()
            .filter_map(|(path, rel_path, filename)| {
                if query.is_empty() {
                    Some(QuickSearchResult {
                        path,
                        relative_path: rel_path,
                        filename,
                        score: 0,
                    })
                } else {
                    fuzzy_match(query, &filename, &rel_path).map(|score| QuickSearchResult {
                        path,
                        relative_path: rel_path,
                        filename,
                        score,
                    })
                }
            })
            .collect();

        if query.is_empty() {
            scored.sort_by(|a, b| a.relative_path.cmp(&b.relative_path));
        } else {
            scored.sort_by(|a, b| {
                b.score
                    .cmp(&a.score)
                    .then_with(|| a.filename.len().cmp(&b.filename.len()))
                    .then_with(|| a.relative_path.cmp(&b.relative_path))
            });
        }

        if scored.len() > self.max_results {
            scored.truncate(self.max_results);
        }

        self.results = scored;
        if self.selected_index >= self.results.len() {
            self.selected_index = 0;
            self.scroll_offset = 0;
        }
    }

    // INFO: Computes screen geometry for the command-palette overlay.
    pub fn overlay_rect(screen_w: f32, top_y: f32, results_count: usize) -> (f32, f32, f32, f32) {
        let w = Self::OVERLAY_WIDTH.min(screen_w - 20.0);
        let x = ((screen_w - w) / 2.0).max(10.0);
        let y = top_y + 4.0;
        let visible_items = results_count.min(Self::MAX_VISIBLE_ITEMS);
        let bottom_pad = if visible_items > 0 { 6.0 } else { 8.0 };
        let h = Self::INPUT_HEIGHT + (visible_items as f32 * Self::ITEM_HEIGHT) + bottom_pad;
        (x, y, w, h)
    }

    // INFO: Hit tests item click in the quick search results list.
    pub fn hit_test_item(&self, mx: f32, my: f32, screen_w: f32, top_y: f32) -> Option<usize> {
        let (ox, oy, ow, _oh) = Self::overlay_rect(screen_w, top_y, self.results.len());
        if mx < ox || mx > ox + ow {
            return None;
        }
        let list_y = oy + Self::INPUT_HEIGHT;
        if my < list_y {
            return None;
        }
        let rel_y = my - list_y;
        let item_idx = (rel_y / Self::ITEM_HEIGHT) as usize + self.scroll_offset;
        if item_idx < self.results.len() {
            Some(item_idx)
        } else {
            None
        }
    }
}

// INFO: Fast recursive scanner for workspace files respecting exclusion lists and hidden file settings.
fn scan_dir_files(
    root: &Path,
    current: &Path,
    exclusions: &[String],
    show_hidden: bool,
    candidates: &mut Vec<(PathBuf, String, String)>,
    count: &mut usize,
    max_scan: usize,
) {
    if *count >= max_scan {
        return;
    }

    let Ok(entries) = fs::read_dir(current) else {
        return;
    };

    for res in entries.flatten() {
        if *count >= max_scan {
            break;
        }

        let path = res.path();
        let Some(file_name) = path.file_name() else {
            continue;
        };
        let name_str = file_name.to_string_lossy().to_string();

        let is_hidden = name_str.starts_with('.');
        if !show_hidden && is_hidden {
            continue;
        }
        if exclusions.iter().any(|exc| exc == &name_str) {
            continue;
        }

        *count += 1;
        let is_dir = res.file_type().map(|ft| ft.is_dir()).unwrap_or(false);

        if is_dir {
            scan_dir_files(
                root,
                &path,
                exclusions,
                show_hidden,
                candidates,
                count,
                max_scan,
            );
        } else {
            let rel_path = path
                .strip_prefix(root)
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|_| name_str.clone());
            candidates.push((path, rel_path, name_str));
        }
    }
}

// INFO: Calculates fuzzy match score between search query and file path.
pub fn fuzzy_match(query: &str, filename: &str, rel_path: &str) -> Option<i32> {
    let q_lower = query.to_lowercase();
    let f_lower = filename.to_lowercase();
    let p_lower = rel_path.to_lowercase();

    if q_lower.is_empty() {
        return Some(0);
    }

    // Exact filename match
    if f_lower == q_lower {
        return Some(1000);
    }

    // Filename starts with query
    if f_lower.starts_with(&q_lower) {
        return Some(500 + (100 - f_lower.len() as i32).max(0));
    }

    // Filename contains query substring
    if f_lower.contains(&q_lower) {
        return Some(250 + (100 - f_lower.len() as i32).max(0));
    }

    // Path contains query substring
    if p_lower.contains(&q_lower) {
        return Some(150 + (100 - p_lower.len() as i32).max(0));
    }

    // Subsequence matching on filename
    if let Some(score) = subsequence_score(&q_lower, &f_lower) {
        return Some(100 + score);
    }

    // Subsequence matching on full path
    if let Some(score) = subsequence_score(&q_lower, &p_lower) {
        return Some(50 + score);
    }

    None
}

// INFO: Scores ordered character subsequence matches with consecutive character bonuses.
fn subsequence_score(query: &str, target: &str) -> Option<i32> {
    let mut q_chars = query.chars().peekable();
    let mut score = 0;
    let mut prev_idx: Option<usize> = None;

    for (idx, t_ch) in target.chars().enumerate() {
        if let Some(&q_ch) = q_chars.peek() {
            if q_ch == t_ch {
                q_chars.next();
                score += 10;
                if let Some(p) = prev_idx
                    && idx == p + 1
                {
                    score += 15;
                }
                prev_idx = Some(idx);
            }
        } else {
            break;
        }
    }

    if q_chars.peek().is_none() {
        Some(score)
    } else {
        None
    }
}
