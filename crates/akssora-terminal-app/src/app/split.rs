use akssora_terminal_core::Terminal;
use akssora_terminal_renderer::FontManager;
use akssora_terminal_ui::{Block, PaneTree, SplitDirection, TabKind};

use super::App;

const MARGIN: f32 = 2.0;

fn grid_dims(width: u32, height: u32, fonts: &FontManager) -> (usize, usize) {
    let usable_w = (width as f32 - 2.0 * MARGIN).max(0.0);
    let usable_h = (height as f32 - 2.0 * MARGIN).max(0.0);
    let cols = (usable_w / fonts.cell_w).floor() as usize;
    let rows = (usable_h / fonts.cell_h).floor() as usize;
    (rows.max(1), cols.max(1))
}

// INFO: Split pane management, creation, focus tracking, and tree layout.
impl App {
    // INFO: Creates a new split view in the currently active tab.
    pub fn split_active_pane(&mut self, direction: SplitDirection) -> Option<u64> {
        let active_cwd = self.active_working_directory();
        self.split_active_pane_in(direction, active_cwd.as_deref())
    }

    pub fn split_active_pane_in(
        &mut self,
        direction: SplitDirection,
        cwd: Option<&std::path::Path>,
    ) -> Option<u64> {
        let active_tab_id = self.tab_manager.active_id()?;

        // If active tab is not a terminal session, do not split
        if let Some(tab) = self.tab_manager.active_tab()
            && !matches!(tab.kind, TabKind::Terminal)
        {
            return None;
        }

        let focused_pane_id = self
            .split_manager
            .active_pane_id
            .filter(|id| {
                self.split_trees
                    .get(&active_tab_id)
                    .map(|tree| tree.contains_leaf(*id))
                    .unwrap_or(false)
            })
            .unwrap_or(active_tab_id);

        let new_pane_id = (rand::random::<u64>() % 900_000_000) + 100_000_000;
        let (rows, cols) = self.default_grid_dims();
        let terminal = match Terminal::new_in(rows, cols, cwd) {
            Ok(t) => t,
            Err(e) => {
                eprintln!("failed to create terminal split: {e:?}");
                return None;
            }
        };

        let mut block = Block::new(
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            Some(format!("Terminal {new_pane_id}")),
            terminal,
        );
        block.pane_id = Some(new_pane_id);
        self.blocks.insert(new_pane_id, block);

        if let Some(orig_block) = self.blocks.get_mut(&active_tab_id)
            && orig_block.pane_id.is_none()
        {
            orig_block.pane_id = Some(active_tab_id);
        }

        self.save_active_terminal_code_review();

        let tree = self
            .split_trees
            .entry(active_tab_id)
            .or_insert_with(|| PaneTree::new_leaf(active_tab_id));

        tree.split_leaf(focused_pane_id, new_pane_id, direction);
        self.split_manager.focus_pane(new_pane_id);
        self.load_active_terminal_code_review();

        self.layout_blocks();
        self.save_session_state();
        if let Some(win) = self.window.as_ref() {
            win.request_redraw();
        }

        Some(new_pane_id)
    }

    // INFO: Collects all Block IDs belonging to the currently active tab.
    pub fn active_tab_block_ids(&self) -> Vec<u64> {
        let Some(active_id) = self.tab_manager.active_id() else {
            return Vec::new();
        };

        if let Some(tree) = self.split_trees.get(&active_id) {
            let ids = tree.all_leaf_ids();
            if !ids.is_empty() {
                return ids;
            }
        }

        vec![active_id]
    }

    // INFO: Collects all Block references belonging to the currently active tab.
    pub fn active_tab_blocks(&self) -> Vec<&Block> {
        self.active_tab_block_ids()
            .into_iter()
            .filter_map(|id| self.blocks.get(&id))
            .collect()
    }

    // INFO: Retrieves an immutable reference to a block by its pane ID.
    pub fn get_block(&self, id: u64) -> Option<&Block> {
        self.blocks.get(&id)
    }

    // INFO: Retrieves a mutable reference to a block by its pane ID.
    pub fn get_block_mut(&mut self, id: u64) -> Option<&mut Block> {
        self.blocks.get_mut(&id)
    }

    // INFO: Selects the focused pane by cursor position if it hits any block in the active tab.
    pub fn focus_pane_at(&mut self, mx: f32, my: f32) -> bool {
        let Some(active_id) = self.tab_manager.active_id() else {
            return false;
        };

        let leaf_ids = if let Some(tree) = self.split_trees.get(&active_id) {
            tree.all_leaf_ids()
        } else {
            vec![active_id]
        };

        for id in leaf_ids {
            if let Some(block) = self.blocks.get(&id)
                && mx >= block.x
                && mx <= block.x + block.width as f32
                && my >= block.y
                && my <= block.y + block.height as f32
            {
                if self.split_manager.active_pane_id != Some(id) {
                    self.save_active_terminal_code_review();
                    self.split_manager.focus_pane(id);
                    self.load_active_terminal_code_review();
                    self.layout_blocks();
                }
                return true;
            }
        }
        false
    }

    // INFO: Lays out all terminal blocks and panes according to the split tree geometry.
    pub(crate) fn layout_blocks_with_splits(&mut self) {
        let (win_w, win_h) = self
            .state
            .as_ref()
            .map(|s| s.size())
            .unwrap_or((self.config.window.width, self.config.window.height));
        let show_sidebar = self.config.tab_layout.show_sidebar();
        let sb_w = self.sidebar.effective_width(show_sidebar);
        let exp_w = self.explorer.effective_width();
        let content_x = sb_w + exp_w;
        let content_w = (win_w as f32 - content_x).max(0.0);
        let top_h = self
            .header
            .total_height(self.config.tab_layout.includes_title_bar());
        let content_y = top_h;
        let content_h = (win_h as f32 - top_h).max(0.0);

        let fonts = self.fonts.as_ref();

        let is_cr_open = self.is_code_review_open();
        if is_cr_open && self.code_review.is_full_view {
            // In full view, Code Review occupies the entire area; preserve existing terminal geometry
            return;
        }

        let (term_x, term_w) = if is_cr_open {
            let total_w = (content_w - akssora_terminal_ui::PaneTree::DIVIDER_THICKNESS).max(0.0);
            let min_w = 120.0_f32.min(total_w * 0.45);
            let ratio = self.code_review.split_ratio.clamp(0.15, 0.85);
            let tw = (total_w * (1.0 - ratio)).max(min_w).min(total_w - min_w);
            (content_x, tw)
        } else {
            (content_x, content_w)
        };

        for tab in self.tab_manager.tabs() {
            let tab_id = tab.id;
            if let Some(tree) = self.split_trees.get(&tab_id) {
                let rects = tree.compute_rects(term_x, content_y, term_w, content_h);
                for (pane_id, px, py, pw, ph) in rects {
                    if let Some(block) = self.blocks.get_mut(&pane_id) {
                        block.x = px;
                        block.y = py;
                        block.width = pw as u32;
                        block.height = ph as u32;
                        block.scrollbar.update_geometry(px, py, pw, ph);

                        let (rows, cols) = if let Some(f) = fonts {
                            grid_dims(pw as u32, ph as u32, f)
                        } else {
                            (24, 80)
                        };
                        block.shell.resize(rows, cols);
                    }
                }
            } else if let Some(block) = self.blocks.get_mut(&tab_id) {
                block.x = term_x;
                block.y = content_y;
                block.width = term_w as u32;
                block.height = content_h as u32;
                block
                    .scrollbar
                    .update_geometry(term_x, content_y, term_w, content_h);

                let (rows, cols) = if let Some(f) = fonts {
                    grid_dims(term_w as u32, content_h as u32, f)
                } else {
                    (24, 80)
                };
                block.shell.resize(rows, cols);
            }
        }
    }

    // INFO: Closes a specific split pane within its parent tab, updating groups and focus.
    pub fn close_split_pane(&mut self, pane_id: u64) {
        let owning_tab_id = self.split_trees.iter().find_map(|(tab_id, tree)| {
            if tree.contains_leaf(pane_id) {
                Some(*tab_id)
            } else {
                None
            }
        });

        let Some(tab_id) = owning_tab_id else {
            if let Some(pos) = self.tab_manager.tabs().iter().position(|t| t.id == pane_id) {
                self.close_tab(pos);
            }
            return;
        };

        let mut remove_tree = false;
        let mut single_remaining_id = None;
        let mut rekey_to = None;
        let mut next_active_id = None;

        if let Some(tree) = self.split_trees.get_mut(&tab_id) {
            tree.remove_leaf(pane_id);
            let remaining = tree.all_leaf_ids();

            if remaining.len() <= 1 {
                remove_tree = true;
                single_remaining_id = remaining.first().copied();
                next_active_id = single_remaining_id;
            } else {
                if pane_id == tab_id {
                    rekey_to = Some(remaining[0]);
                }
                if self.split_manager.active_pane_id == Some(pane_id) {
                    next_active_id = remaining.first().copied();
                }
            }
        }

        if remove_tree {
            self.split_trees.remove(&tab_id);
            self.sidebar.collapsed_tabs.remove(&tab_id);
            if let Some(rem_id) = single_remaining_id {
                if tab_id != rem_id {
                    self.tab_manager.update_tab_id(tab_id, rem_id);
                }
                if let Some(custom) = self.pane_custom_names.get(&rem_id) {
                    self.tab_manager.set_custom_title(rem_id, custom.clone());
                } else if let Some(last_cmd) = self.last_executed_commands.get(&rem_id) {
                    self.tab_manager.set_auto_title(rem_id, last_cmd);
                }
                self.split_manager.active_pane_id = Some(rem_id);
            }
        } else if let Some(new_tab_id) = rekey_to {
            if let Some(tree) = self.split_trees.remove(&tab_id) {
                self.split_trees.insert(new_tab_id, tree);
            }
            if self.sidebar.collapsed_tabs.remove(&tab_id) {
                self.sidebar.collapsed_tabs.insert(new_tab_id);
            }
            self.tab_manager.update_tab_id(tab_id, new_tab_id);
        }

        if let Some(act_id) = next_active_id {
            self.split_manager.active_pane_id = Some(act_id);
        }

        self.command_buffers.remove(&pane_id);
        self.last_executed_commands.remove(&pane_id);
        self.pane_custom_names.remove(&pane_id);
        self.terminal_code_reviews.remove(&pane_id);
        if let Some(mut block) = self.blocks.remove(&pane_id) {
            block.shell.kill();
        }

        self.load_active_terminal_code_review();
        self.layout_blocks();
        self.save_session_state();
        if let Some(win) = self.window.as_ref() {
            win.request_redraw();
        }
    }

    // INFO: Cycles focus to the next split pane in the active tab.
    pub fn focus_next_split(&mut self) {
        let Some(active_id) = self.tab_manager.active_id() else {
            return;
        };
        if let Some(tree) = self.split_trees.get(&active_id) {
            let leaves = tree.all_leaf_ids();
            if leaves.len() > 1 {
                let current = self.split_manager.active_pane_id.unwrap_or(active_id);
                if let Some(pos) = leaves.iter().position(|&id| id == current) {
                    let next_pos = (pos + 1) % leaves.len();
                    self.save_active_terminal_code_review();
                    self.split_manager.focus_pane(leaves[next_pos]);
                    self.load_active_terminal_code_review();
                    self.layout_blocks();
                    if let Some(win) = self.window.as_ref() {
                        win.request_redraw();
                    }
                }
            }
        }
    }

    // INFO: Cycles focus to the previous split pane in the active tab.
    pub fn focus_prev_split(&mut self) {
        let Some(active_id) = self.tab_manager.active_id() else {
            return;
        };
        if let Some(tree) = self.split_trees.get(&active_id) {
            let leaves = tree.all_leaf_ids();
            if leaves.len() > 1 {
                let current = self.split_manager.active_pane_id.unwrap_or(active_id);
                if let Some(pos) = leaves.iter().position(|&id| id == current) {
                    let prev_pos = if pos == 0 { leaves.len() - 1 } else { pos - 1 };
                    self.save_active_terminal_code_review();
                    self.split_manager.focus_pane(leaves[prev_pos]);
                    self.load_active_terminal_code_review();
                    self.layout_blocks();
                    if let Some(win) = self.window.as_ref() {
                        win.request_redraw();
                    }
                }
            }
        }
    }

    // INFO: Returns the pane ID and rectangle under the given coordinates in the active tab.
    pub fn pane_at(&self, mx: f32, my: f32) -> Option<(u64, f32, f32, f32, f32)> {
        let active_id = self.tab_manager.active_id()?;
        let leaf_ids = if let Some(tree) = self.split_trees.get(&active_id) {
            tree.all_leaf_ids()
        } else {
            vec![active_id]
        };

        for id in leaf_ids {
            if let Some(block) = self.blocks.get(&id)
                && mx >= block.x
                && mx <= block.x + block.width as f32
                && my >= block.y
                && my <= block.y + block.height as f32
            {
                return Some((
                    id,
                    block.x,
                    block.y,
                    block.width as f32,
                    block.height as f32,
                ));
            }
        }
        None
    }

    // INFO: Repositions source pane relative to target pane within the active tab using drop zone.
    pub fn reposition_pane(
        &mut self,
        source_id: u64,
        target_id: u64,
        zone: akssora_terminal_ui::DropZone,
    ) -> bool {
        let Some(active_id) = self.tab_manager.active_id() else {
            return false;
        };
        let Some(tree) = self.split_trees.get_mut(&active_id) else {
            return false;
        };

        let success = tree.reposition_leaf(source_id, target_id, zone);
        if success {
            self.split_manager.focus_pane(source_id);
            self.layout_blocks();
            if let Some(win) = self.window.as_ref() {
                win.request_redraw();
            }
        }
        success
    }
}
