use akssora_git::FileGitStatus;
use std::path::PathBuf;

// INFO: Open file descriptor displayed within the editor panel's dedicated file-tab bar.
#[derive(Clone, Debug, PartialEq)]
pub struct EditorFileTab {
    pub path: PathBuf,
    pub title: String,
    pub relative_path: String,
    pub is_active: bool,
    pub is_dirty: bool,
    pub is_preview: bool,
    pub is_pinned: bool,
    pub git_status: Option<FileGitStatus>,
}

// INFO: Interactive targets on the dedicated editor file-tab bar.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EditorTabTarget {
    Tab(usize),
    Close(usize),
    Pin(usize),
    FormatDocument,
    TogglePreview,
    ToggleSplitPreview,
    ToggleInlineDiff,
}

// INFO: Toolbar action buttons available for editor files.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EditorToolbarAction {
    FormatDocument,
    TogglePreview,
    ToggleSplitPreview,
    ToggleInlineDiff,
}

// INFO: Resolves an expressive Unicode icon for a given file name or extension.
#[inline]
pub fn file_type_icon(name: &str) -> &'static str {
    akssora_icons::file_type_icon(name)
}

// INFO: Fixed top tab bar within the code editor layout coordinating open file tabs, close targets, and geometry calculations.
pub struct EditorTabBar;

impl EditorTabBar {
    pub const HEIGHT: f32 = 28.0;
    pub const CLOSE_BTN_SIZE: f32 = 16.0;
    pub const PIN_BTN_SIZE: f32 = 14.0;
    pub const MIN_TAB_WIDTH: f32 = 90.0;
    pub const MAX_TAB_WIDTH: f32 = 180.0;

    // INFO: Computes tab item rectangular bounds (x, y, width, height) for the tab at index i.
    pub fn tab_rect(
        i: usize,
        total: usize,
        ws_x: f32,
        ws_y: f32,
        ws_w: f32,
    ) -> (f32, f32, f32, f32) {
        if total == 0 {
            return (ws_x, ws_y, 0.0, Self::HEIGHT);
        }
        let tab_w = (ws_w / total as f32).clamp(Self::MIN_TAB_WIDTH, Self::MAX_TAB_WIDTH);
        let x = ws_x + i as f32 * tab_w;
        (x, ws_y, tab_w, Self::HEIGHT)
    }

    // INFO: Computes close button rectangular bounds (cx, cy, cw, ch) for the tab at index i.
    pub fn close_rect(
        i: usize,
        total: usize,
        ws_x: f32,
        ws_y: f32,
        ws_w: f32,
    ) -> (f32, f32, f32, f32) {
        let (tx, ty, tw, th) = Self::tab_rect(i, total, ws_x, ws_y, ws_w);
        let size = Self::CLOSE_BTN_SIZE;
        let cx = tx + tw - size - 6.0;
        let cy = ty + (th - size) * 0.5;
        (cx, cy, size, size)
    }

    // INFO: Computes pin button rectangular bounds (px, py, pw, ph) for the tab at index i.
    pub fn pin_rect(
        i: usize,
        total: usize,
        ws_x: f32,
        ws_y: f32,
        ws_w: f32,
    ) -> (f32, f32, f32, f32) {
        let (cx, cy, _cw, _ch) = Self::close_rect(i, total, ws_x, ws_y, ws_w);
        let size = Self::PIN_BTN_SIZE;
        let px = cx - size - 4.0;
        let py = cy + 1.0;
        (px, py, size, size)
    }

    // INFO: Finds which tab index a horizontal coordinate falls within.
    pub fn tab_at_x(mx: f32, ws_x: f32, ws_w: f32, total: usize) -> Option<usize> {
        if total == 0 || mx < ws_x {
            return None;
        }
        let tab_w = (ws_w / total as f32).clamp(Self::MIN_TAB_WIDTH, Self::MAX_TAB_WIDTH);
        let rel_x = mx - ws_x;
        let idx = (rel_x / tab_w) as usize;
        if idx < total { Some(idx) } else { None }
    }

    pub const TOOLBAR_BTN_WIDTH: f32 = 28.0;
    pub const TOOLBAR_BTN_HEIGHT: f32 = 20.0;

    // INFO: Computes rectangular bounds (x, y, w, h) for a Markdown editor toolbar action button.
    pub fn toolbar_rect(
        action: EditorToolbarAction,
        ws_x: f32,
        ws_y: f32,
        ws_w: f32,
    ) -> (f32, f32, f32, f32) {
        let btn_w = Self::TOOLBAR_BTN_WIDTH;
        let btn_h = Self::TOOLBAR_BTN_HEIGHT;
        let right_margin = 8.0;
        let y = ws_y + (Self::HEIGHT - btn_h) * 0.5;

        let offset = match action {
            EditorToolbarAction::ToggleInlineDiff => 0.0,
            EditorToolbarAction::ToggleSplitPreview => btn_w + 4.0,
            EditorToolbarAction::TogglePreview => (btn_w + 4.0) * 2.0,
            EditorToolbarAction::FormatDocument => (btn_w + 4.0) * 3.0,
        };

        let x = (ws_x + ws_w - right_margin - btn_w - offset).max(ws_x);
        (x, y, btn_w, btn_h)
    }

    // INFO: Hit-tests cursor against the editor toolbar action buttons.
    pub fn hit_test_toolbar(
        mx: f32,
        my: f32,
        ws_x: f32,
        ws_y: f32,
        ws_w: f32,
    ) -> Option<EditorToolbarAction> {
        if my < ws_y || my >= ws_y + Self::HEIGHT {
            return None;
        }
        let actions = [
            EditorToolbarAction::ToggleInlineDiff,
            EditorToolbarAction::ToggleSplitPreview,
            EditorToolbarAction::TogglePreview,
        ];
        for action in actions {
            let (bx, by, bw, bh) = Self::toolbar_rect(action, ws_x, ws_y, ws_w);
            if mx >= bx && mx < bx + bw && my >= by && my < by + bh {
                return Some(action);
            }
        }
        None
    }

    // INFO: Hit-tests cursor coordinates against the editor file-tab bar.
    pub fn hit_test(
        mx: f32,
        my: f32,
        ws_x: f32,
        ws_y: f32,
        ws_w: f32,
        total: usize,
    ) -> Option<EditorTabTarget> {
        Self::hit_test_ex(mx, my, ws_x, ws_y, ws_w, total, true)
    }

    // INFO: Extended hit-test supporting tabs, close buttons, pin buttons, and editor toolbar controls.
    pub fn hit_test_ex(
        mx: f32,
        my: f32,
        ws_x: f32,
        ws_y: f32,
        ws_w: f32,
        total: usize,
        show_markdown_toolbar: bool,
    ) -> Option<EditorTabTarget> {
        if my < ws_y || my >= ws_y + Self::HEIGHT {
            return None;
        }

        if let Some(action) = Self::hit_test_toolbar(mx, my, ws_x, ws_y, ws_w) {
            match action {
                EditorToolbarAction::ToggleInlineDiff => {
                    return Some(EditorTabTarget::ToggleInlineDiff);
                }
                EditorToolbarAction::TogglePreview if show_markdown_toolbar => {
                    return Some(EditorTabTarget::TogglePreview);
                }
                EditorToolbarAction::ToggleSplitPreview if show_markdown_toolbar => {
                    return Some(EditorTabTarget::ToggleSplitPreview);
                }
                _ => {}
            }
        }

        if total == 0 {
            return None;
        }

        for i in 0..total {
            let (tx, ty, tw, th) = Self::tab_rect(i, total, ws_x, ws_y, ws_w);
            if mx >= tx && mx < tx + tw && my >= ty && my < ty + th {
                let (cx, cy, cw, ch) = Self::close_rect(i, total, ws_x, ws_y, ws_w);
                if mx >= cx && mx < cx + cw && my >= cy && my < cy + ch {
                    return Some(EditorTabTarget::Close(i));
                }
                let (px, py, pw, ph) = Self::pin_rect(i, total, ws_x, ws_y, ws_w);
                if mx >= px && mx < px + pw && my >= py && my < py + ph {
                    return Some(EditorTabTarget::Pin(i));
                }
                return Some(EditorTabTarget::Tab(i));
            }
        }
        None
    }
}
