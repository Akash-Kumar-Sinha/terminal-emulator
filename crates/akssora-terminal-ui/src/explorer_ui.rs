use akssora_file_explorer::{EntryKind, FileTree};
use akssora_terminal_config::Theme;
use std::path::{Path, PathBuf};

// INFO: Header actions available on the file explorer panel toolbar.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExplorerToolbarAction {
    NewFile,
    NewFolder,
    ToggleHidden,
    ChangeRoot,
}

// INFO: Context menu actions when right-clicking files, folders, or workspace canvas.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExplorerContextAction {
    OpenInEditor,
    OpenInSplit,
    OpenTerminalHere,
    NewFile,
    NewFolder,
    Rename,
    Copy,
    CopyPath,
    RevealInFileManager,
    Delete,
    ToggleHidden,
}

// INFO: Floating context menu modal for file explorer items.
#[derive(Clone, Debug, PartialEq)]
pub struct ExplorerContextMenu {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub target_path: Option<PathBuf>,
    pub items: Vec<ExplorerContextAction>,
    pub hovered_idx: Option<usize>,
}

impl ExplorerContextMenu {
    pub const ITEM_HEIGHT: f32 = 28.0;

    pub fn new(x: f32, y: f32, target_path: Option<PathBuf>) -> Self {
        let is_dir = target_path.as_ref().map(|p| p.is_dir()).unwrap_or(false);
        let items = if target_path.is_some() {
            if is_dir {
                vec![
                    ExplorerContextAction::OpenTerminalHere,
                    ExplorerContextAction::NewFile,
                    ExplorerContextAction::NewFolder,
                    ExplorerContextAction::Rename,
                    ExplorerContextAction::Copy,
                    ExplorerContextAction::CopyPath,
                    ExplorerContextAction::RevealInFileManager,
                    ExplorerContextAction::Delete,
                    ExplorerContextAction::ToggleHidden,
                ]
            } else {
                vec![
                    ExplorerContextAction::OpenInEditor,
                    ExplorerContextAction::OpenInSplit,
                    ExplorerContextAction::OpenTerminalHere,
                    ExplorerContextAction::NewFile,
                    ExplorerContextAction::NewFolder,
                    ExplorerContextAction::Rename,
                    ExplorerContextAction::Copy,
                    ExplorerContextAction::CopyPath,
                    ExplorerContextAction::RevealInFileManager,
                    ExplorerContextAction::Delete,
                    ExplorerContextAction::ToggleHidden,
                ]
            }
        } else {
            vec![
                ExplorerContextAction::OpenTerminalHere,
                ExplorerContextAction::NewFile,
                ExplorerContextAction::NewFolder,
                ExplorerContextAction::ToggleHidden,
            ]
        };
        let height = items.len() as f32 * Self::ITEM_HEIGHT + 8.0;
        Self {
            x,
            y,
            width: 195.0,
            height,
            target_path,
            items,
            hovered_idx: None,
        }
    }

    pub fn hit_test(&self, mx: f32, my: f32) -> Option<ExplorerContextAction> {
        if mx < self.x || mx > self.x + self.width || my < self.y || my > self.y + self.height {
            return None;
        }
        let rel_y = my - self.y - 4.0;
        if rel_y < 0.0 {
            return None;
        }
        let idx = (rel_y / Self::ITEM_HEIGHT) as usize;
        self.items.get(idx).copied()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum InlineInputMode {
    NewFile,
    NewFolder,
    Rename { target: PathBuf },
    ChangeRoot,
}

#[derive(Clone, Debug, PartialEq)]
pub struct InlineInputState {
    pub mode: InlineInputMode,
    pub text: String,
    pub parent_dir: PathBuf,
    pub error: Option<String>,
}

// INFO: Target interactable elements inside the file explorer navigation panel.
#[derive(Clone, Debug, PartialEq)]
pub enum ExplorerTarget {
    Toolbar(ExplorerToolbarAction),
    RowItem {
        index: usize,
        path: PathBuf,
        kind: EntryKind,
    },
    RowToggle(PathBuf),
    ConfirmDeleteOk,
    ConfirmDeleteCancel,
    ChangeRootPrompt,
}

// INFO: Left sidebar hierarchical file explorer UI component coordinating tree display, file CRUD, and resize handling.
#[derive(Clone, Debug)]
pub struct ExplorerUi {
    pub width: f32,
    pub saved_width: f32,
    pub is_expanded: bool,
    pub is_resizing: bool,
    pub hovered_edge: bool,
    pub hovered_row: Option<usize>,
    pub hovered_toolbar: Option<ExplorerToolbarAction>,
    pub tree: FileTree,
    pub scroll_y: f32,
    pub inline_input: Option<InlineInputState>,
    pub confirm_delete: Option<Vec<PathBuf>>,
    pub context_menu: Option<ExplorerContextMenu>,
    pub active_editor_file: Option<PathBuf>,
    pub git_status: Option<akssora_git::GitRepoStatus>,
}

impl ExplorerUi {
    pub const DEFAULT_WIDTH: f32 = 240.0;
    pub const MIN_WIDTH: f32 = 160.0;
    pub const MAX_WIDTH: f32 = 480.0;
    pub const ITEM_HEIGHT: f32 = 24.0;
    pub const HEADER_HEIGHT: f32 = 32.0;

    pub fn new(root_path: impl AsRef<Path>) -> Self {
        let tree = FileTree::new(root_path.as_ref()).unwrap_or_else(|_| {
            let curr = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
            FileTree::new(curr).expect("valid fallback root")
        });
        Self {
            width: 0.0,
            saved_width: Self::DEFAULT_WIDTH,
            is_expanded: false,
            is_resizing: false,
            hovered_edge: false,
            hovered_row: None,
            hovered_toolbar: None,
            tree,
            scroll_y: 0.0,
            inline_input: None,
            confirm_delete: None,
            context_menu: None,
            active_editor_file: None,
            git_status: None,
        }
    }

    pub fn toggle(&mut self) {
        self.is_expanded = !self.is_expanded;
        if self.is_expanded {
            self.width = if self.saved_width >= Self::MIN_WIDTH {
                self.saved_width
            } else {
                Self::DEFAULT_WIDTH
            };
        } else {
            if self.width >= Self::MIN_WIDTH {
                self.saved_width = self.width;
            }
            self.width = 0.0;
        }
    }

    pub fn effective_width(&self) -> f32 {
        if self.is_expanded { self.width } else { 0.0 }
    }

    pub fn max_scroll(&self, viewport_h: f32) -> f32 {
        let total_rows = self.tree.flatten_rows().len();
        let total_content_h = total_rows as f32 * Self::ITEM_HEIGHT;
        (total_content_h - viewport_h).max(0.0)
    }

    pub fn clamp_scroll(&mut self, viewport_h: f32) {
        let max = self.max_scroll(viewport_h);
        self.scroll_y = self.scroll_y.clamp(0.0, max);
    }

    pub fn toolbar_buttons_rect(&self) -> (f32, f32, f32, f32) {
        (0.0, 0.0, self.width, Self::HEADER_HEIGHT)
    }

    pub fn header_rect(&self, top_offset: f32) -> (f32, f32, f32, f32) {
        (0.0, top_offset, self.width, Self::HEADER_HEIGHT)
    }

    pub fn hit_test_toolbar(
        &self,
        mx: f32,
        my: f32,
        top_offset: f32,
    ) -> Option<ExplorerToolbarAction> {
        self.hit_test_toolbar_at(mx, my, top_offset, 0.0)
    }

    pub fn hit_test_toolbar_at(
        &self,
        mx: f32,
        my: f32,
        top_offset: f32,
        x_offset: f32,
    ) -> Option<ExplorerToolbarAction> {
        let h_y = top_offset;
        if my < h_y || my > h_y + Self::HEADER_HEIGHT {
            return None;
        }

        let rel_x = mx - x_offset;
        // Clicking the folder root name opens ChangeRoot
        if rel_x >= 8.0 && rel_x < self.width - 8.0 {
            return Some(ExplorerToolbarAction::ChangeRoot);
        }

        None
    }

    pub fn hit_test(&self, mx: f32, my: f32, top_offset: f32) -> Option<ExplorerTarget> {
        self.hit_test_at(mx, my, top_offset, 0.0)
    }

    pub fn hit_test_at(
        &self,
        mx: f32,
        my: f32,
        top_offset: f32,
        x_offset: f32,
    ) -> Option<ExplorerTarget> {
        if !self.is_expanded || mx < x_offset || mx > x_offset + self.width {
            return None;
        }

        let rel_x = mx - x_offset;

        // 1. Toolbar target
        if let Some(action) = self.hit_test_toolbar_at(mx, my, top_offset, x_offset) {
            return Some(ExplorerTarget::Toolbar(action));
        }

        // 2. Tree row targets
        let rows = self.tree.flatten_rows();
        let content_start_y = top_offset + Self::HEADER_HEIGHT - self.scroll_y;

        for (idx, row) in rows.iter().enumerate() {
            let row_y = content_start_y + idx as f32 * Self::ITEM_HEIGHT;
            if my >= row_y && my < row_y + Self::ITEM_HEIGHT {
                let arrow_w = 18.0;
                let indent_x = (row.depth as f32 * 14.0) + 6.0;
                if row.kind == EntryKind::Directory
                    && rel_x >= indent_x
                    && rel_x < indent_x + arrow_w
                {
                    return Some(ExplorerTarget::RowToggle(row.path.clone()));
                }
                return Some(ExplorerTarget::RowItem {
                    index: idx,
                    path: row.path.clone(),
                    kind: row.kind,
                });
            }
        }

        None
    }

    // INFO: Confirmation dialog bounds and button hit testing for delete operations.
    pub fn confirm_dialog_rect(screen_w: f32, screen_h: f32) -> (f32, f32, f32, f32) {
        let dw = 340.0;
        let dh = 130.0;
        let dx = ((screen_w - dw) / 2.0).max(0.0);
        let dy = ((screen_h - dh) / 2.0).max(0.0);
        (dx, dy, dw, dh)
    }

    pub fn hit_test_confirm_dialog(
        mx: f32,
        my: f32,
        screen_w: f32,
        screen_h: f32,
    ) -> Option<ExplorerTarget> {
        let (dx, dy, dw, dh) = Self::confirm_dialog_rect(screen_w, screen_h);
        if mx < dx || mx > dx + dw || my < dy || my > dy + dh {
            return None;
        }
        let btn_y = dy + dh - 38.0;
        let btn_h = 28.0;
        let del_w = 90.0;
        let can_w = 90.0;
        let btn_gap = 14.0;
        let total_buttons_w = del_w + can_w + btn_gap;
        let start_btn_x = dx + (dw - total_buttons_w) / 2.0;

        let del_x = start_btn_x;
        let can_x = del_x + del_w + btn_gap;

        if my >= btn_y && my <= btn_y + btn_h {
            // Delete button
            if mx >= del_x && mx <= del_x + del_w {
                return Some(ExplorerTarget::ConfirmDeleteOk);
            }
            // Cancel button
            if mx >= can_x && mx <= can_x + can_w {
                return Some(ExplorerTarget::ConfirmDeleteCancel);
            }
        }
        None
    }

    pub fn bg_color(&self, theme: &Theme) -> [f32; 4] {
        theme.sidebar_background
    }

    pub fn border_color(&self, theme: &Theme) -> [f32; 4] {
        theme.border_subtle
    }
}
