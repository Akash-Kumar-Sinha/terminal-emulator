// INFO: Drop zone target within a pane during drag-and-drop rearrangement.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DropZone {
    Left,
    Right,
    Top,
    Bottom,
    Center,
}

impl DropZone {
    // INFO: Determine the drop zone within a given rectangle (x, y, w, h) for cursor (mx, my).
    pub fn from_point(mx: f32, my: f32, x: f32, y: f32, w: f32, h: f32) -> Self {
        if w <= 0.0 || h <= 0.0 {
            return Self::Center;
        }
        let rel_x = (mx - x) / w;
        let rel_y = (my - y) / h;

        // Quarter edge threshold: 25% on edges, center in the middle 50%
        let threshold = 0.25;
        if rel_x < threshold {
            Self::Left
        } else if rel_x > 1.0 - threshold {
            Self::Right
        } else if rel_y < threshold {
            Self::Top
        } else if rel_y > 1.0 - threshold {
            Self::Bottom
        } else {
            Self::Center
        }
    }

    // INFO: Computes preview indicator rectangle for this drop zone within the pane.
    pub fn preview_rect(&self, x: f32, y: f32, w: f32, h: f32) -> (f32, f32, f32, f32) {
        match self {
            Self::Left => (x, y, w * 0.5, h),
            Self::Right => (x + w * 0.5, y, w * 0.5, h),
            Self::Top => (x, y, w, h * 0.5),
            Self::Bottom => (x, y + h * 0.5, w, h * 0.5),
            Self::Center => (x, y, w, h),
        }
    }
}
