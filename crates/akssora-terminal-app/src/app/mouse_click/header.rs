use akssora_terminal_ui::{HeaderButton, HeaderItem};
use winit::event_loop::ActiveEventLoop;

use crate::app::App;

impl App {
    pub(crate) fn handle_header_clicks(
        &mut self,
        event_loop: &ActiveEventLoop,
        mx: f32,
        my: f32,
        w: f32,
        show_tabs: bool,
        show_toggle: bool,
    ) -> bool {
        let hit_header = self.header.hit_test_item_ex(
            mx,
            my,
            w,
            self.tab_manager.len(),
            show_tabs,
            show_toggle,
            true,
        );

        if self.header.branch_dropdown_open
            && !matches!(
                hit_header,
                Some(HeaderItem::BranchDropdown) | Some(HeaderItem::BranchItem(_))
            )
        {
            self.header.branch_dropdown_open = false;
            if let Some(win) = self.window.as_ref() {
                win.request_redraw();
            }
        }

        match hit_header {
            Some(HeaderItem::SidebarToggle) => {
                self.header.is_sidebar_toggle_active = true;
                self.toggle_sidebar();
                if let Some(win) = self.window.as_ref() {
                    win.request_redraw();
                }
                true
            }
            Some(HeaderItem::ExplorerToggle) => {
                self.header.is_explorer_toggle_active = true;
                self.toggle_explorer();
                if let Some(win) = self.window.as_ref() {
                    win.request_redraw();
                }
                true
            }
            Some(HeaderItem::QuickSearch) => {
                self.toggle_quick_search();
                true
            }
            Some(HeaderItem::WindowButton(HeaderButton::Minimize)) => {
                if let Some(win) = self.window.as_ref() {
                    win.set_minimized(true);
                }
                true
            }
            Some(HeaderItem::WindowButton(HeaderButton::Maximize)) => {
                if let Some(win) = self.window.as_ref() {
                    let next_state = !win.is_maximized();
                    win.set_maximized(next_state);
                    self.header.is_maximized = next_state;
                    win.request_redraw();
                }
                true
            }
            Some(HeaderItem::WindowButton(HeaderButton::Close)) => {
                event_loop.exit();
                true
            }
            Some(HeaderItem::BranchDropdown) => {
                self.header.branch_dropdown_open = !self.header.branch_dropdown_open;
                if let Some(win) = self.window.as_ref() {
                    win.request_redraw();
                }
                true
            }
            Some(HeaderItem::BranchItem(idx)) => {
                if let Some(target_branch) = self.header.git_branches.get(idx).cloned() {
                    let target_dir = self.current_working_directory();
                    let repo_root =
                        akssora_git::GitInspector::repo_root(&target_dir).unwrap_or(target_dir);
                    let _ = akssora_git::GitInspector::switch_branch(&repo_root, &target_branch);
                    self.sync_code_review_with_active_terminal();
                }
                self.header.branch_dropdown_open = false;
                if let Some(win) = self.window.as_ref() {
                    win.request_redraw();
                }
                true
            }
            Some(HeaderItem::CodeReview) => {
                self.header.branch_dropdown_open = false;
                self.toggle_code_review();
                if let Some(win) = self.window.as_ref() {
                    win.request_redraw();
                }
                true
            }
            Some(HeaderItem::Settings) => {
                self.header.branch_dropdown_open = false;
                self.header.is_settings_active = true;
                self.open_settings_tab();
                if let Some(win) = self.window.as_ref() {
                    win.request_redraw();
                }
                true
            }
            Some(HeaderItem::Tab(idx)) => {
                self.tab_manager.select(idx);
                self.sync_active_tab_state();
                if let Some(active_id) = self.tab_manager.active_id()
                    && let Some(tree) = self.split_trees.get(&active_id)
                    && !tree.contains_leaf(self.split_manager.active_pane_id.unwrap_or(0))
                    && let Some(first_leaf) = tree.all_leaf_ids().first().copied()
                {
                    self.split_manager.focus_pane(first_leaf);
                }
                self.layout_blocks();
                if let Some(win) = self.window.as_ref() {
                    win.request_redraw();
                }
                true
            }
            Some(HeaderItem::TabClose(idx)) => {
                self.close_tab(idx);
                if let Some(win) = self.window.as_ref() {
                    win.request_redraw();
                }
                true
            }
            Some(HeaderItem::NewTab) => {
                self.open_terminal_tab();
                if let Some(win) = self.window.as_ref() {
                    win.request_redraw();
                }
                true
            }
            None => {
                if self.header.in_drag_region_with_tabs(
                    mx,
                    my,
                    w,
                    self.tab_manager.len(),
                    show_tabs,
                ) && let Some(win) = self.window.as_ref()
                {
                    let _ = win.drag_window();
                    return true;
                }
                false
            }
        }
    }
}
