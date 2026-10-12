use crate::tooltip::Tooltip;

// INFO: Reusable interactive UI button with geometry, tooltip, shortcut, and hit testing.
#[derive(Clone, Debug, PartialEq)]
pub struct ReusableButton {
    pub id: String,
    pub rect: (f32, f32, f32, f32),
    pub label: String,
    pub tooltip_function: String,
    pub shortcut: Option<String>,
    pub is_hovered: bool,
    pub is_focused: bool,
}

impl ReusableButton {
    pub fn new(
        id: impl Into<String>,
        rect: (f32, f32, f32, f32),
        label: impl Into<String>,
        tooltip_function: impl Into<String>,
        shortcut: Option<String>,
    ) -> Self {
        Self {
            id: id.into(),
            rect,
            label: label.into(),
            tooltip_function: tooltip_function.into(),
            shortcut,
            is_hovered: false,
            is_focused: false,
        }
    }

    pub fn hit_test(&self, mx: f32, my: f32) -> bool {
        let (x, y, w, h) = self.rect;
        mx >= x && mx < x + w && my >= y && my < y + h
    }

    pub fn tooltip(&self) -> Tooltip {
        Tooltip::for_target(
            self.tooltip_function.clone(),
            self.shortcut.clone(),
            self.rect,
        )
    }
}
