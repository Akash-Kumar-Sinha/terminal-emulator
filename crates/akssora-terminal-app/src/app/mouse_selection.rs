use akssora_terminal_input::{get_clipboard_text, set_clipboard_text};
use akssora_terminal_screen::{SelectionRange, TerminalPoint};
use akssora_terminal_ui::{Block, ContextMenu, ContextMenuAction};

use super::App;

const MARGIN: f32 = 2.0;

// INFO: Maps physical window coordinates into the terminal character cell grid for a specific block.
pub fn pixel_to_grid_cell(
    block: &Block,
    px: f32,
    py: f32,
    cell_w: f32,
    cell_h: f32,
) -> Option<(usize, usize)> {
    if px < block.x
        || px > block.x + block.width as f32
        || py < block.y
        || py > block.y + block.height as f32
    {
        return None;
    }

    let rel_x = (px - block.x - MARGIN).max(0.0);
    let rel_y = (py - block.y - MARGIN).max(0.0);

    let grid = block.shell.grid();
    let lines_count = grid.visible_lines().len();
    let cols_count = grid.cols();

    let y_offset = if block.bottom_up && block.shell.scroll_offset() == 0 {
        let (crow, _) = block.shell.cursor();
        let active = (crow + 1).max(grid.content_rows()).min(lines_count);
        (lines_count.saturating_sub(active)) as f32 * cell_h
    } else {
        0.0
    };

    let adjusted_y = (rel_y - y_offset).max(0.0);
    let row = (adjusted_y / cell_h).floor() as usize;
    let col = (rel_x / cell_w).floor() as usize;

    Some((
        row.min(lines_count.saturating_sub(1)),
        col.min(cols_count.saturating_sub(1)),
    ))
}

impl App {
    // INFO: Copies currently selected text from the active terminal pane to the system clipboard.
    pub fn copy_active_selection(&mut self) -> bool {
        if let Some(block) = self.active_block_mut()
            && let Some(text) = block.shell.selected_text()
            && !text.is_empty()
        {
            set_clipboard_text(&text);
            return true;
        }
        false
    }

    // INFO: Pastes system clipboard contents into the active terminal shell.
    pub fn paste_into_active(&mut self) -> bool {
        if let Some(text) = get_clipboard_text()
            && !text.is_empty()
            && let Some(block) = self.active_block_mut()
        {
            block.shell.reset_scroll();
            block.shell.paste_text(&text);
            if let Some(win) = self.window.as_ref() {
                win.request_redraw();
            }
            return true;
        }
        false
    }

    // INFO: Selects all visible text on the active terminal grid.
    pub fn select_all_in_active(&mut self) {
        if let Some(block) = self.active_block_mut() {
            let rows = block.shell.grid().visible_lines().len();
            let cols = block.shell.grid().cols();
            if rows > 0 && cols > 0 {
                block.shell.set_selection(Some(SelectionRange::new(
                    TerminalPoint::new(0, 0),
                    TerminalPoint::new(rows - 1, cols.saturating_sub(1)),
                )));
                if let Some(win) = self.window.as_ref() {
                    win.request_redraw();
                }
            }
        }
    }

    // INFO: Opens the context menu popup at the specified screen coordinates.
    pub fn open_context_menu(&mut self, mx: f32, my: f32) {
        let can_copy = self
            .active_block_mut()
            .and_then(|b| b.shell.selected_text())
            .map(|t| !t.is_empty())
            .unwrap_or(false);

        let can_paste = get_clipboard_text().map(|t| !t.is_empty()).unwrap_or(false);

        // Clamp menu coordinates so it doesn't overflow screen bounds
        let (win_w, win_h) = self.state.as_ref().map(|s| s.size()).unwrap_or((1200, 800));
        let menu_w = ContextMenu::DEFAULT_WIDTH;
        let menu_h = 3.0 * ContextMenu::DEFAULT_ITEM_HEIGHT + 8.0;

        let clamped_x = mx.min(win_w as f32 - menu_w - 4.0).max(4.0);
        let clamped_y = my.min(win_h as f32 - menu_h - 4.0).max(4.0);

        self.context_menu = Some(ContextMenu::new(clamped_x, clamped_y, can_copy, can_paste));
        if let Some(win) = self.window.as_ref() {
            win.request_redraw();
        }
    }

    // INFO: Dispatches context menu item action and closes the menu.
    pub fn handle_context_menu_action(&mut self, action: ContextMenuAction) {
        self.context_menu = None;
        match action {
            ContextMenuAction::Copy => {
                self.copy_active_selection();
            }
            ContextMenuAction::Paste => {
                self.paste_into_active();
            }
            ContextMenuAction::SelectAll => {
                self.select_all_in_active();
            }
        }
        if let Some(win) = self.window.as_ref() {
            win.request_redraw();
        }
    }

    // INFO: Handles clicking inside a terminal block for cursor repositioning or application mouse event dispatch.
    pub fn handle_terminal_click_in_pane(&mut self, pane_id: u64, mx: f32, my: f32) {
        let (cell_w, cell_h) = self
            .fonts
            .as_ref()
            .map(|f| (f.cell_w, f.cell_h))
            .unwrap_or((9.0, 18.0));

        let shift = self.modifiers.shift_key();

        let Some(block) = self.blocks.get_mut(&pane_id) else {
            return;
        };

        // If interactive application captures mouse (Vim, etc.) and Shift is not held, forward SGR mouse sequence
        if block.shell.mouse_tracking() && !shift {
            if let Some((row, col)) = pixel_to_grid_cell(block, mx, my, cell_w, cell_h) {
                let seq = akssora_terminal_input::encode_sgr_mouse(col, row, 0, true);
                block.shell.send(&seq);
                let rel_seq = akssora_terminal_input::encode_sgr_mouse(col, row, 0, false);
                block.shell.send(&rel_seq);
            }
            return;
        }

        // Standard terminal click: clear existing selection
        block.shell.clear_selection();

        // Check if user clicked on active cursor line for cursor-based command editing
        if block.shell.scroll_offset() == 0
            && let Some((row, col)) = pixel_to_grid_cell(block, mx, my, cell_w, cell_h)
        {
            let (crow, ccol) = block.shell.cursor();
            if row == crow {
                if col < ccol {
                    let diff = ccol - col;
                    let seq = b"\x1b[D".repeat(diff);
                    block.shell.send(&seq);
                } else if col > ccol {
                    let diff = col - ccol;
                    let seq = b"\x1b[C".repeat(diff);
                    block.shell.send(&seq);
                }
            }
        }

        if let Some(win) = self.window.as_ref() {
            win.request_redraw();
        }
    }

    // INFO: Updates mouse drag text selection across visible terminal cells.
    pub fn update_mouse_selection_drag(&mut self, mx: f32, my: f32) {
        let Some(pane_id) = self.selection_start_pane else {
            return;
        };
        let Some(start_pt) = self.selection_start_pt else {
            return;
        };

        let (cell_w, cell_h) = self
            .fonts
            .as_ref()
            .map(|f| (f.cell_w, f.cell_h))
            .unwrap_or((9.0, 18.0));

        let Some(block) = self.blocks.get_mut(&pane_id) else {
            return;
        };

        if let Some((row, col)) = pixel_to_grid_cell(block, mx, my, cell_w, cell_h) {
            let end_pt = TerminalPoint::new(row, col);
            block
                .shell
                .set_selection(Some(SelectionRange::new(start_pt, end_pt)));
            if let Some(win) = self.window.as_ref() {
                win.request_redraw();
            }
        }
    }
}
