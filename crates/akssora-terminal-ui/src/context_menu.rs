// INFO: Context menu actions triggered via terminal right-click popup menu.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ContextMenuAction {
    Copy,
    Paste,
    SelectAll,
}

#[derive(Clone, Debug)]
pub struct ContextMenuItem {
    pub label: &'static str,
    pub shortcut: &'static str,
    pub action: ContextMenuAction,
    pub enabled: bool,
}

// INFO: Floating popup context menu offering clipboard actions (Copy, Paste, Select All).
#[derive(Clone, Debug)]
pub struct ContextMenu {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub item_height: f32,
    pub items: Vec<ContextMenuItem>,
    pub hovered_idx: Option<usize>,
}

impl ContextMenu {
    pub const DEFAULT_WIDTH: f32 = 180.0;
    pub const DEFAULT_ITEM_HEIGHT: f32 = 28.0;

    pub fn new(x: f32, y: f32, can_copy: bool, can_paste: bool) -> Self {
        let items = vec![
            ContextMenuItem {
                label: "Copy",
                shortcut: "Ctrl+Shift+C",
                action: ContextMenuAction::Copy,
                enabled: can_copy,
            },
            ContextMenuItem {
                label: "Paste",
                shortcut: "Ctrl+Shift+V",
                action: ContextMenuAction::Paste,
                enabled: can_paste,
            },
            ContextMenuItem {
                label: "Select All",
                shortcut: "",
                action: ContextMenuAction::SelectAll,
                enabled: true,
            },
        ];

        Self {
            x,
            y,
            width: Self::DEFAULT_WIDTH,
            item_height: Self::DEFAULT_ITEM_HEIGHT,
            items,
            hovered_idx: None,
        }
    }

    pub fn total_height(&self) -> f32 {
        self.items.len() as f32 * self.item_height + 8.0
    }

    pub fn bounds(&self) -> (f32, f32, f32, f32) {
        (self.x, self.y, self.width, self.total_height())
    }

    pub fn contains(&self, mx: f32, my: f32) -> bool {
        let (x, y, w, h) = self.bounds();
        mx >= x && mx <= x + w && my >= y && my <= y + h
    }

    pub fn update_hover(&mut self, mx: f32, my: f32) {
        if !self.contains(mx, my) {
            self.hovered_idx = None;
            return;
        }

        let rel_y = my - (self.y + 4.0);
        if rel_y < 0.0 {
            self.hovered_idx = None;
            return;
        }

        let idx = (rel_y / self.item_height).floor() as usize;
        if idx < self.items.len() && self.items[idx].enabled {
            self.hovered_idx = Some(idx);
        } else {
            self.hovered_idx = None;
        }
    }

    pub fn hit_test(&self, mx: f32, my: f32) -> Option<ContextMenuAction> {
        if !self.contains(mx, my) {
            return None;
        }

        let rel_y = my - (self.y + 4.0);
        if rel_y < 0.0 {
            return None;
        }

        let idx = (rel_y / self.item_height).floor() as usize;
        if let Some(item) = self.items.get(idx)
            && item.enabled
        {
            return Some(item.action);
        }

        None
    }
}
