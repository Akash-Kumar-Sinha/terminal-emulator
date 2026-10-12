// INFO: Split orientation and node hierarchy for multi-pane terminal views.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SplitDirection {
    Horizontal, // Split horizontally: top / bottom panes
    Vertical,   // Split vertically: left / right panes
}

use crate::drop_zone::DropZone;

#[derive(Clone, Debug, PartialEq)]
pub enum PaneTree {
    Leaf(u64), // Terminal / block ID
    Split {
        direction: SplitDirection,
        ratio: f32, // Fraction of space allocated to first child (e.g. 0.5)
        first: Box<PaneTree>,
        second: Box<PaneTree>,
    },
}

impl PaneTree {
    pub const MIN_PANE_WIDTH: f32 = 120.0;
    pub const MIN_PANE_HEIGHT: f32 = 80.0;
    pub const DIVIDER_THICKNESS: f32 = 4.0;

    pub fn new_leaf(id: u64) -> Self {
        Self::Leaf(id)
    }

    pub fn all_leaf_ids(&self) -> Vec<u64> {
        let mut ids = Vec::new();
        self.collect_leaf_ids(&mut ids);
        ids
    }

    fn collect_leaf_ids(&self, out: &mut Vec<u64>) {
        match self {
            Self::Leaf(id) => out.push(*id),
            Self::Split { first, second, .. } => {
                first.collect_leaf_ids(out);
                second.collect_leaf_ids(out);
            }
        }
    }

    pub fn contains_leaf(&self, id: u64) -> bool {
        match self {
            Self::Leaf(leaf_id) => *leaf_id == id,
            Self::Split { first, second, .. } => {
                first.contains_leaf(id) || second.contains_leaf(id)
            }
        }
    }

    // INFO: Split the target leaf with a new pane ID and orientation.
    pub fn split_leaf(&mut self, target_id: u64, new_id: u64, direction: SplitDirection) -> bool {
        match self {
            Self::Leaf(id) if *id == target_id => {
                let current_leaf = Box::new(Self::Leaf(*id));
                let new_leaf = Box::new(Self::Leaf(new_id));
                *self = Self::Split {
                    direction,
                    ratio: 0.5,
                    first: current_leaf,
                    second: new_leaf,
                };
                true
            }
            Self::Split { first, second, .. } => {
                if first.split_leaf(target_id, new_id, direction) {
                    true
                } else {
                    second.split_leaf(target_id, new_id, direction)
                }
            }
            _ => false,
        }
    }

    // INFO: Split the target leaf with a new pane ID, orientation, and custom ratio.
    pub fn split_leaf_with_ratio(
        &mut self,
        target_id: u64,
        new_id: u64,
        direction: SplitDirection,
        first_is_original: bool,
        ratio: f32,
    ) -> bool {
        match self {
            Self::Leaf(id) if *id == target_id => {
                let orig = Box::new(Self::Leaf(*id));
                let new_leaf = Box::new(Self::Leaf(new_id));
                let (first, second) = if first_is_original {
                    (orig, new_leaf)
                } else {
                    (new_leaf, orig)
                };
                *self = Self::Split {
                    direction,
                    ratio: ratio.clamp(0.1, 0.9),
                    first,
                    second,
                };
                true
            }
            Self::Split { first, second, .. } => {
                if first.split_leaf_with_ratio(
                    target_id,
                    new_id,
                    direction,
                    first_is_original,
                    ratio,
                ) {
                    true
                } else {
                    second.split_leaf_with_ratio(
                        target_id,
                        new_id,
                        direction,
                        first_is_original,
                        ratio,
                    )
                }
            }
            _ => false,
        }
    }

    // INFO: Repositions source_id relative to target_id based on drop zone.
    pub fn reposition_leaf(&mut self, source_id: u64, target_id: u64, zone: DropZone) -> bool {
        if source_id == target_id {
            return false;
        }
        if !self.contains_leaf(source_id) || !self.contains_leaf(target_id) {
            return false;
        }

        // Handle center swap: simply swap leaf IDs
        if zone == DropZone::Center {
            return self.swap_leaves(source_id, target_id);
        }

        // If target is currently the only pane in root, or normal tree:
        // First remove source_id from tree
        if !self.remove_leaf(source_id) {
            return false;
        }

        // Now split target_id in the specified direction and placement
        let (dir, first_is_orig) = match zone {
            DropZone::Left => (SplitDirection::Vertical, false),
            DropZone::Right => (SplitDirection::Vertical, true),
            DropZone::Top => (SplitDirection::Horizontal, false),
            DropZone::Bottom => (SplitDirection::Horizontal, true),
            DropZone::Center => unreachable!(),
        };

        self.split_leaf_with_ratio(target_id, source_id, dir, first_is_orig, 0.5)
    }

    // INFO: Swaps two leaf nodes' IDs in the tree.
    pub fn swap_leaves(&mut self, id_a: u64, id_b: u64) -> bool {
        match self {
            Self::Leaf(id) => {
                if *id == id_a {
                    *id = id_b;
                    true
                } else if *id == id_b {
                    *id = id_a;
                    true
                } else {
                    false
                }
            }
            Self::Split { first, second, .. } => {
                let swapped_first = first.swap_leaves(id_a, id_b);
                let swapped_second = second.swap_leaves(id_a, id_b);
                swapped_first || swapped_second
            }
        }
    }

    // INFO: Remove a leaf node from the split tree, promoting sibling if needed.
    pub fn remove_leaf(&mut self, target_id: u64) -> bool {
        match self {
            Self::Leaf(_) => false,
            Self::Split { first, second, .. } => {
                if let Self::Leaf(id) = **first
                    && id == target_id
                {
                    *self = *second.clone();
                    return true;
                }
                if let Self::Leaf(id) = **second
                    && id == target_id
                {
                    *self = *first.clone();
                    return true;
                }
                if first.remove_leaf(target_id) {
                    true
                } else {
                    second.remove_leaf(target_id)
                }
            }
        }
    }

    // INFO: Computes layout rectangles for all leaves given container bounds.
    pub fn compute_rects(&self, x: f32, y: f32, w: f32, h: f32) -> Vec<(u64, f32, f32, f32, f32)> {
        let mut out = Vec::new();
        self.collect_rects(x, y, w, h, &mut out);
        out
    }

    fn collect_rects(
        &self,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        out: &mut Vec<(u64, f32, f32, f32, f32)>,
    ) {
        match self {
            Self::Leaf(id) => {
                out.push((*id, x, y, w.max(0.0), h.max(0.0)));
            }
            Self::Split {
                direction,
                ratio,
                first,
                second,
            } => {
                let clamped_ratio = ratio.clamp(0.1, 0.9);
                match direction {
                    SplitDirection::Vertical => {
                        let total_w = (w - Self::DIVIDER_THICKNESS).max(0.0);
                        let min_w = Self::MIN_PANE_WIDTH.min(total_w * 0.45);
                        let mut w1 = (total_w * clamped_ratio).max(min_w);
                        let w2 = (total_w - w1).max(min_w);
                        w1 = total_w - w2;
                        let x2 = x + w1 + Self::DIVIDER_THICKNESS;

                        first.collect_rects(x, y, w1, h, out);
                        second.collect_rects(x2, y, w2, h, out);
                    }
                    SplitDirection::Horizontal => {
                        let total_h = (h - Self::DIVIDER_THICKNESS).max(0.0);
                        let min_h = Self::MIN_PANE_HEIGHT.min(total_h * 0.45);
                        let mut h1 = (total_h * clamped_ratio).max(min_h);
                        let h2 = (total_h - h1).max(min_h);
                        h1 = total_h - h2;
                        let y2 = y + h1 + Self::DIVIDER_THICKNESS;

                        first.collect_rects(x, y, w, h1, out);
                        second.collect_rects(x, y2, w, h2, out);
                    }
                }
            }
        }
    }

    // INFO: Computes all divider rects with their orientation for rendering and hit testing.
    pub fn compute_dividers(
        &self,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
    ) -> Vec<(SplitDirection, f32, f32, f32, f32, Vec<usize>)> {
        let mut out = Vec::new();
        self.collect_dividers(x, y, w, h, Vec::new(), &mut out);
        out
    }

    #[allow(clippy::type_complexity)]
    fn collect_dividers(
        &self,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        path: Vec<usize>,
        out: &mut Vec<(SplitDirection, f32, f32, f32, f32, Vec<usize>)>,
    ) {
        match self {
            Self::Leaf(_) => {}
            Self::Split {
                direction,
                ratio,
                first,
                second,
            } => {
                let clamped_ratio = ratio.clamp(0.1, 0.9);
                match direction {
                    SplitDirection::Vertical => {
                        let total_w = (w - Self::DIVIDER_THICKNESS).max(0.0);
                        let min_w = Self::MIN_PANE_WIDTH.min(total_w * 0.45);
                        let mut w1 = (total_w * clamped_ratio).max(min_w);
                        let w2 = (total_w - w1).max(min_w);
                        w1 = total_w - w2;
                        let div_x = x + w1;
                        let x2 = div_x + Self::DIVIDER_THICKNESS;

                        out.push((
                            *direction,
                            div_x,
                            y,
                            Self::DIVIDER_THICKNESS,
                            h,
                            path.clone(),
                        ));

                        let mut p1 = path.clone();
                        p1.push(0);
                        first.collect_dividers(x, y, w1, h, p1, out);

                        let mut p2 = path;
                        p2.push(1);
                        second.collect_dividers(x2, y, w2, h, p2, out);
                    }
                    SplitDirection::Horizontal => {
                        let total_h = (h - Self::DIVIDER_THICKNESS).max(0.0);
                        let min_h = Self::MIN_PANE_HEIGHT.min(total_h * 0.45);
                        let mut h1 = (total_h * clamped_ratio).max(min_h);
                        let h2 = (total_h - h1).max(min_h);
                        h1 = total_h - h2;
                        let div_y = y + h1;
                        let y2 = div_y + Self::DIVIDER_THICKNESS;

                        out.push((
                            *direction,
                            x,
                            div_y,
                            w,
                            Self::DIVIDER_THICKNESS,
                            path.clone(),
                        ));

                        let mut p1 = path.clone();
                        p1.push(0);
                        first.collect_dividers(x, y, w, h1, p1, out);

                        let mut p2 = path;
                        p2.push(1);
                        second.collect_dividers(x, y2, w, h2, p2, out);
                    }
                }
            }
        }
    }

    // INFO: Adjusts ratio at the specified node path by a drag delta.
    pub fn adjust_ratio(&mut self, path: &[usize], delta_ratio: f32) {
        if path.is_empty() {
            if let Self::Split { ratio, .. } = self {
                *ratio = (*ratio + delta_ratio).clamp(0.1, 0.9);
            }
            return;
        }

        if let Self::Split { first, second, .. } = self {
            if path[0] == 0 {
                first.adjust_ratio(&path[1..], delta_ratio);
            } else {
                second.adjust_ratio(&path[1..], delta_ratio);
            }
        }
    }
}
