use akssora_terminal_ui::{PaneTree, SplitDirection};

// INFO: Tracks active divider dragging interaction state across mouse moves.
#[derive(Clone, Debug)]
pub struct DividerDragState {
    pub tab_id: u64,
    pub path: Vec<usize>,
    pub direction: SplitDirection,
    pub start_x: f32,
    pub start_y: f32,
    pub container_w: f32,
    pub container_h: f32,
}

// INFO: Target pane and zone for drag-and-drop pane rearrangement.
pub type PaneDropTarget = (u64, akssora_terminal_ui::DropZone, (f32, f32, f32, f32));

// INFO: Tracks active pane dragging state across mouse moves for layout rearrangement.
#[derive(Clone, Debug, PartialEq)]
pub struct PaneDragState {
    pub source_pane_id: u64,
    pub tab_id: u64,
    pub start_x: f32,
    pub start_y: f32,
    pub current_x: f32,
    pub current_y: f32,
    pub hover_target: Option<PaneDropTarget>,
}

// INFO: Manages split layout tree per tab and tracks active focused pane.
#[derive(Clone, Debug, Default)]
pub struct SplitManager {
    pub active_pane_id: Option<u64>,
    pub drag_state: Option<DividerDragState>,
    pub pane_drag: Option<PaneDragState>,
}

impl SplitManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn focus_pane(&mut self, pane_id: u64) {
        self.active_pane_id = Some(pane_id);
    }

    pub fn active_pane(&self, default_id: Option<u64>) -> Option<u64> {
        self.active_pane_id.or(default_id)
    }

    // INFO: Hit-tests divider lines in the given tree given content offset.
    #[allow(clippy::too_many_arguments)]
    pub fn hit_test_divider(
        &self,
        tree: &PaneTree,
        mx: f32,
        my: f32,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
    ) -> Option<(SplitDirection, Vec<usize>, f32, f32)> {
        let dividers = tree.compute_dividers(x, y, w, h);
        let hit_margin = 4.0;
        for (dir, dx, dy, dw, dh, path) in dividers {
            match dir {
                SplitDirection::Vertical => {
                    if mx >= dx - hit_margin
                        && mx <= dx + dw + hit_margin
                        && my >= dy
                        && my <= dy + dh
                    {
                        return Some((dir, path, w, h));
                    }
                }
                SplitDirection::Horizontal => {
                    if my >= dy - hit_margin
                        && my <= dy + dh + hit_margin
                        && mx >= dx
                        && mx <= dx + dw
                    {
                        return Some((dir, path, w, h));
                    }
                }
            }
        }
        None
    }
}
