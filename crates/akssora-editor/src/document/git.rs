use std::path::Path;

use akssora_terminal_config::LineNumberMode;

use super::Document;

impl Document {
    pub fn set_deleted(&mut self, deleted: bool) {
        self.is_deleted = deleted;
        self.cached_git_hash.set(0);
        self.cached_git_statuses.borrow_mut().take();
    }

    pub fn set_git_original_content(&mut self, content: Option<String>) {
        self.git_original_content = content;
        self.cached_git_hash.set(0);
        self.cached_git_statuses.borrow_mut().take();
    }

    pub fn refresh_git_original_content(&mut self, repo_root: &Path) {
        if let Some(path) = &self.path {
            self.git_original_content =
                akssora_git::GitInspector::file_original_content(repo_root, path);
            self.cached_git_hash.set(0);
            self.cached_git_statuses.borrow_mut().take();
        }
    }

    pub fn ensure_git_statuses_computed(&self) {
        if self.is_deleted {
            let mut cached = self.cached_git_statuses.borrow_mut();
            if cached.is_none() {
                *cached = Some(
                    (0..self.buffer.line_count())
                        .map(|_| akssora_git::LineGitStatus {
                            change: Some(akssora_git::LineGitChange::Deleted),
                            deleted_above: false,
                            deleted_below: false,
                            deleted_lines: Vec::new(),
                            original_line: None,
                        })
                        .collect(),
                );
            }
            return;
        }

        let Some(original) = &self.git_original_content else {
            *self.cached_git_statuses.borrow_mut() = Some(Vec::new());
            return;
        };

        let mut hash: u64 = 0xcbf29ce484222325;
        for chunk in self.buffer.rope().chunks() {
            for byte in chunk.as_bytes() {
                hash ^= *byte as u64;
                hash = hash.wrapping_mul(0x100000001b3);
            }
        }
        for byte in original.as_bytes() {
            hash ^= *byte as u64;
            hash = hash.wrapping_mul(0x100000001b3);
        }

        if self.cached_git_hash.get() == hash && self.cached_git_statuses.borrow().is_some() {
            return;
        }

        let line_count = self.buffer.line_count();
        let statuses = if original.is_empty() {
            vec![
                akssora_git::LineGitStatus {
                    change: Some(akssora_git::LineGitChange::Added),
                    deleted_above: false,
                    deleted_below: false,
                    deleted_lines: Vec::new(),
                    original_line: None,
                };
                line_count
            ]
        } else {
            let orig_lines: Vec<&str> = original
                .split('\n')
                .map(|s| s.strip_suffix('\r').unwrap_or(s))
                .collect();
            let curr_lines: Vec<String> = (0..line_count)
                .map(|r| self.buffer.get_line(r).unwrap_or_default())
                .collect();
            let curr_refs: Vec<&str> = curr_lines.iter().map(|s| s.as_str()).collect();
            akssora_git::compute_line_diff(&orig_lines, &curr_refs)
        };

        *self.cached_git_statuses.borrow_mut() = Some(statuses);
        self.cached_git_hash.set(hash);
    }

    pub fn with_git_line_statuses<R>(
        &self,
        f: impl FnOnce(&[akssora_git::LineGitStatus]) -> R,
    ) -> R {
        self.ensure_git_statuses_computed();
        let cached = self.cached_git_statuses.borrow();
        if let Some(statuses) = cached.as_ref() {
            f(statuses)
        } else {
            f(&[])
        }
    }

    pub fn git_line_statuses(&self) -> Vec<akssora_git::LineGitStatus> {
        self.with_git_line_statuses(|s| s.to_vec())
    }

    pub fn git_line_status(&self, logical_row: usize) -> akssora_git::LineGitStatus {
        self.with_git_line_statuses(|statuses| {
            if logical_row < statuses.len() {
                statuses[logical_row].clone()
            } else {
                akssora_git::LineGitStatus::default()
            }
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GutterActionKind {
    StageLine,
    RevertLine,
}

impl Document {
    // INFO: Hit-tests whether a mouse position hovers over a gutter line or specific action button.
    #[allow(clippy::too_many_arguments)]
    pub fn hit_test_gutter_hover(
        &self,
        mx: f32,
        my: f32,
        content_x: f32,
        content_y: f32,
        content_w: f32,
        cell_w: f32,
        cell_h: f32,
    ) -> Option<(usize, Option<GutterActionKind>)> {
        let gutter_w = if self.line_numbers && self.line_number_mode != LineNumberMode::Off {
            48.0
        } else {
            0.0
        };
        if gutter_w <= 0.0 {
            return None;
        }

        if mx < content_x || mx > content_x + gutter_w {
            return None;
        }

        let effective_line_h = (cell_h * self.line_height).max(cell_h);
        if my < content_y {
            return None;
        }
        let vis_row = self.scroll_row + ((my - content_y) / effective_line_h).floor() as usize;

        let actual_row = if self.line_wrap {
            let wrap_w = if let Some(layout) = self.cached_wrap_layout.borrow().as_ref() {
                layout.wrap_width
            } else {
                let max_text_w = (content_w - gutter_w - 16.0 - 8.0).max(0.0);
                (max_text_w / cell_w).floor().max(1.0) as usize
            };
            self.visual_to_logical(vis_row, 0, wrap_w).0
        } else {
            vis_row
        };

        if actual_row < self.buffer.line_count() {
            let git_st = self.git_line_status(actual_row);
            if git_st.change.is_some() || git_st.deleted_above || git_st.deleted_below {
                let line_y = content_y + ((vis_row - self.scroll_row) as f32 * effective_line_h);
                let btn_y = line_y + (effective_line_h - 15.0) * 0.5;
                let btn_h = 15.0;

                // Button 1: Add This Line (+)
                let btn_add_x = content_x + gutter_w - 33.0;
                let btn_add_w = 15.0;

                // Button 2: Revert This Line (↶)
                let btn_rev_x = content_x + gutter_w - 17.0;
                let btn_rev_w = 15.0;

                if my >= btn_y && my < btn_y + btn_h {
                    if mx >= btn_add_x && mx < btn_add_x + btn_add_w {
                        return Some((actual_row, Some(GutterActionKind::StageLine)));
                    } else if mx >= btn_rev_x && mx < btn_rev_x + btn_rev_w {
                        return Some((actual_row, Some(GutterActionKind::RevertLine)));
                    }
                }

                return Some((actual_row, None));
            }
        }
        None
    }

    #[allow(clippy::too_many_arguments)]
    pub fn hit_test_gutter_click(
        &self,
        mx: f32,
        my: f32,
        content_x: f32,
        content_y: f32,
        content_w: f32,
        cell_w: f32,
        cell_h: f32,
    ) -> Option<(usize, GutterActionKind)> {
        let (row, action_opt) =
            self.hit_test_gutter_hover(mx, my, content_x, content_y, content_w, cell_w, cell_h)?;
        action_opt.map(|act| (row, act))
    }

    // INFO: Hit-tests whether a click falls inside the compact Git gutter change action badge on a line.
    #[allow(clippy::too_many_arguments)]
    pub fn hit_test_gutter_action(
        &self,
        click_x: f32,
        click_y: f32,
        content_x: f32,
        content_y: f32,
        content_w: f32,
        cell_w: f32,
        cell_h: f32,
    ) -> Option<usize> {
        self.hit_test_gutter_click(
            click_x, click_y, content_x, content_y, content_w, cell_w, cell_h,
        )
        .map(|(r, _)| r)
    }

    // INFO: Strictly reverts a single logical line in the document buffer to its original Git baseline.
    pub fn revert_line_at(&mut self, target_line: usize) -> Result<(), String> {
        let statuses = self.git_line_statuses();
        if target_line >= statuses.len() {
            return Err("Target line is out of range".to_string());
        }

        let st = &statuses[target_line];
        if st.change.is_none() && !st.deleted_above && !st.deleted_below {
            return Err("No Git change detected on target line".to_string());
        }

        self.record_undo();

        if let Some(change) = st.change {
            match change {
                akssora_git::LineGitChange::Added => {
                    let line_count = self.buffer.line_count();
                    if line_count <= 1 {
                        self.buffer.clear();
                    } else {
                        let start_char = self.buffer.rope().line_to_char(target_line);
                        let end_char = if target_line + 1 < self.buffer.rope().len_lines() {
                            self.buffer.rope().line_to_char(target_line + 1)
                        } else {
                            self.buffer.rope().len_chars()
                        };
                        self.buffer.delete_range(start_char, end_char);
                    }
                }
                akssora_git::LineGitChange::Modified => {
                    if let Some(ref orig) = st.original_line {
                        let start_char = self.buffer.rope().line_to_char(target_line);
                        let line_len = self.buffer.line_len(target_line);
                        self.buffer.delete_range(start_char, start_char + line_len);
                        self.buffer.insert_str(target_line, 0, orig);
                    } else {
                        return Err(
                            "Original content for modified line is not available".to_string()
                        );
                    }
                }
                akssora_git::LineGitChange::Deleted => {
                    return Err("Cannot revert deleted file line-by-line".to_string());
                }
            }
        } else if st.deleted_above
            && let Some(del_line) = st.deleted_lines.first()
        {
            self.buffer
                .insert_str(target_line, 0, &format!("{del_line}\n"));
        } else if st.deleted_below
            && let Some(del_line) = st.deleted_lines.first()
        {
            let insert_row = target_line + 1;
            if insert_row < self.buffer.line_count() {
                self.buffer
                    .insert_str(insert_row, 0, &format!("{del_line}\n"));
            } else {
                let last_row = self.buffer.line_count() - 1;
                let last_len = self.buffer.line_len(last_row);
                self.buffer
                    .insert_str(last_row, last_len, &format!("\n{del_line}"));
            }
        }

        let _ = self.save();
        self.cached_git_hash.set(0);
        self.cached_git_statuses.borrow_mut().take();
        self.cached_wrap_layout.borrow_mut().take();
        self.cached_max_line_len.set(None);

        Ok(())
    }
}
