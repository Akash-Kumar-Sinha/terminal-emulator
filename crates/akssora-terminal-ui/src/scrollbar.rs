use akssora_terminal_config::Theme;

// INFO: Orientation for scrollbars supporting both vertical and horizontal layouts.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum ScrollbarOrientation {
    #[default]
    Vertical,
    Horizontal,
}

// INFO: Shared scrollbar with dynamic thumb sizing, hover states, drag tracking, and orientation support.
#[derive(Clone, Debug)]
pub struct Scrollbar {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub orientation: ScrollbarOrientation,
    pub is_hovered: bool,
    pub is_dragging: bool,
    pub drag_start_y: f32,
    pub drag_start_pos: f32,
    pub drag_start_offset: f32,
}

impl Default for Scrollbar {
    fn default() -> Self {
        Self::new()
    }
}

impl Scrollbar {
    pub const VISIBLE_WIDTH: f32 = 6.0;
    pub const HIT_WIDTH: f32 = 14.0;
    pub const MIN_THUMB_HEIGHT: f32 = 24.0;

    pub fn new() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            width: Self::VISIBLE_WIDTH,
            height: 100.0,
            orientation: ScrollbarOrientation::Vertical,
            is_hovered: false,
            is_dragging: false,
            drag_start_y: 0.0,
            drag_start_pos: 0.0,
            drag_start_offset: 0.0,
        }
    }

    pub fn new_horizontal() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: Self::VISIBLE_WIDTH,
            orientation: ScrollbarOrientation::Horizontal,
            is_hovered: false,
            is_dragging: false,
            drag_start_y: 0.0,
            drag_start_pos: 0.0,
            drag_start_offset: 0.0,
        }
    }

    pub fn update_geometry(&mut self, block_x: f32, block_y: f32, block_w: f32, output_h: f32) {
        self.orientation = ScrollbarOrientation::Vertical;
        self.x = (block_x + block_w - Self::VISIBLE_WIDTH - 2.0).max(0.0);
        self.y = block_y;
        self.width = Self::VISIBLE_WIDTH;
        self.height = output_h.max(0.0);
    }

    pub fn update_geometry_horizontal(
        &mut self,
        block_x: f32,
        block_y: f32,
        block_w: f32,
        block_h: f32,
    ) {
        self.orientation = ScrollbarOrientation::Horizontal;
        self.x = block_x;
        self.y = (block_y + block_h - Self::VISIBLE_WIDTH - 2.0).max(0.0);
        self.width = block_w.max(0.0);
        self.height = Self::VISIBLE_WIDTH;
    }

    pub fn hit_test(&self, mx: f32, my: f32) -> bool {
        match self.orientation {
            ScrollbarOrientation::Vertical => {
                if self.height <= 0.0 {
                    return false;
                }
                let hit_x0 = self.x + self.width - Self::HIT_WIDTH;
                let hit_x1 = self.x + self.width + 4.0;
                mx >= hit_x0 && mx <= hit_x1 && my >= self.y && my <= self.y + self.height
            }
            ScrollbarOrientation::Horizontal => {
                if self.width <= 0.0 {
                    return false;
                }
                let hit_y0 = self.y + self.height - Self::HIT_WIDTH;
                let hit_y1 = self.y + self.height + 4.0;
                mx >= self.x && mx <= self.x + self.width && my >= hit_y0 && my <= hit_y1
            }
        }
    }

    pub fn thumb_rect(
        &self,
        total_lines: usize,
        visible_lines: usize,
        scroll_offset: f32,
    ) -> Option<(f32, f32, f32, f32)> {
        if total_lines <= visible_lines || self.height <= 0.0 || total_lines == 0 {
            return None;
        }
        let max_scroll = (total_lines.saturating_sub(visible_lines)) as f32;
        if max_scroll <= 0.0 {
            return None;
        }

        let ratio = (visible_lines as f32 / total_lines as f32).clamp(0.05, 1.0);
        let thumb_h = (self.height * ratio)
            .max(Self::MIN_THUMB_HEIGHT)
            .min(self.height);
        let travel = self.height - thumb_h;
        let norm = 1.0 - (scroll_offset / max_scroll).clamp(0.0, 1.0);
        let thumb_y = self.y + travel * norm;

        Some((self.x, thumb_y, self.width, thumb_h))
    }

    // INFO: Calculates top-down thumb geometry for editors and standard document viewports.
    pub fn thumb_rect_top_down(
        &self,
        total_items: usize,
        visible_items: usize,
        scroll_offset: usize,
    ) -> Option<(f32, f32, f32, f32)> {
        if total_items <= visible_items || total_items == 0 {
            return None;
        }
        let max_scroll = (total_items.saturating_sub(visible_items)) as f32;
        if max_scroll <= 0.0 {
            return None;
        }

        match self.orientation {
            ScrollbarOrientation::Vertical => {
                if self.height <= 0.0 {
                    return None;
                }
                let ratio = (visible_items as f32 / total_items as f32).clamp(0.05, 1.0);
                let thumb_h = (self.height * ratio)
                    .max(Self::MIN_THUMB_HEIGHT)
                    .min(self.height);
                let travel = self.height - thumb_h;
                let norm = (scroll_offset as f32 / max_scroll).clamp(0.0, 1.0);
                let thumb_y = self.y + travel * norm;
                Some((self.x, thumb_y, self.width, thumb_h))
            }
            ScrollbarOrientation::Horizontal => {
                if self.width <= 0.0 {
                    return None;
                }
                let ratio = (visible_items as f32 / total_items as f32).clamp(0.05, 1.0);
                let thumb_w = (self.width * ratio)
                    .max(Self::MIN_THUMB_HEIGHT)
                    .min(self.width);
                let travel = self.width - thumb_w;
                let norm = (scroll_offset as f32 / max_scroll).clamp(0.0, 1.0);
                let thumb_x = self.x + travel * norm;
                Some((thumb_x, self.y, thumb_w, self.height))
            }
        }
    }

    pub fn hit_test_thumb(
        &self,
        mx: f32,
        my: f32,
        total_lines: usize,
        visible_lines: usize,
        scroll_offset: f32,
    ) -> bool {
        if let Some((_, ty, _, th)) = self.thumb_rect(total_lines, visible_lines, scroll_offset) {
            let hit_x0 = self.x + self.width - Self::HIT_WIDTH;
            let hit_x1 = self.x + self.width + 4.0;
            mx >= hit_x0 && mx <= hit_x1 && my >= ty && my <= ty + th
        } else {
            false
        }
    }

    pub fn hit_test_thumb_top_down(
        &self,
        mx: f32,
        my: f32,
        total_items: usize,
        visible_items: usize,
        scroll_offset: usize,
    ) -> bool {
        if let Some((tx, ty, tw, th)) =
            self.thumb_rect_top_down(total_items, visible_items, scroll_offset)
        {
            match self.orientation {
                ScrollbarOrientation::Vertical => {
                    let hit_x0 = self.x + self.width - Self::HIT_WIDTH;
                    let hit_x1 = self.x + self.width + 4.0;
                    mx >= hit_x0 && mx <= hit_x1 && my >= ty && my <= ty + th
                }
                ScrollbarOrientation::Horizontal => {
                    let hit_y0 = self.y + self.height - Self::HIT_WIDTH;
                    let hit_y1 = self.y + self.height + 4.0;
                    mx >= tx && mx <= tx + tw && my >= hit_y0 && my <= hit_y1
                }
            }
        } else {
            false
        }
    }

    pub fn offset_from_y(&self, my: f32, total_lines: usize, visible_lines: usize) -> f32 {
        if total_lines <= visible_lines || self.height <= 0.0 {
            return 0.0;
        }
        let max_scroll = (total_lines.saturating_sub(visible_lines)) as f32;
        let ratio = (visible_lines as f32 / total_lines as f32).clamp(0.05, 1.0);
        let thumb_h = (self.height * ratio)
            .max(Self::MIN_THUMB_HEIGHT)
            .min(self.height);
        let travel = self.height - thumb_h;
        if travel <= 0.0 {
            return 0.0;
        }
        let norm = ((my - self.y - thumb_h / 2.0) / travel).clamp(0.0, 1.0);
        (1.0 - norm) * max_scroll
    }

    pub fn offset_from_pos_top_down(
        &self,
        pos: f32,
        total_items: usize,
        visible_items: usize,
    ) -> usize {
        if total_items <= visible_items {
            return 0;
        }
        let max_scroll = (total_items.saturating_sub(visible_items)) as f32;
        let (start, span, thumb_len) = match self.orientation {
            ScrollbarOrientation::Vertical => {
                let ratio = (visible_items as f32 / total_items as f32).clamp(0.05, 1.0);
                let th = (self.height * ratio)
                    .max(Self::MIN_THUMB_HEIGHT)
                    .min(self.height);
                (self.y, self.height, th)
            }
            ScrollbarOrientation::Horizontal => {
                let ratio = (visible_items as f32 / total_items as f32).clamp(0.05, 1.0);
                let tw = (self.width * ratio)
                    .max(Self::MIN_THUMB_HEIGHT)
                    .min(self.width);
                (self.x, self.width, tw)
            }
        };
        let travel = span - thumb_len;
        if travel <= 0.0 {
            return 0;
        }
        let norm = ((pos - start - thumb_len / 2.0) / travel).clamp(0.0, 1.0);
        (norm * max_scroll).round() as usize
    }

    pub fn thumb_color(&self, theme: &Theme) -> [f32; 4] {
        if self.is_dragging {
            [0.45, 0.45, 0.48, 1.0]
        } else if self.is_hovered {
            [0.35, 0.35, 0.38, 1.0]
        } else {
            theme.scrollbar_thumb
        }
    }

    // INFO: Returns the centralized theme background color for scrollbar track.
    pub fn track_color(&self, theme: &Theme) -> [f32; 4] {
        theme.scrollbar_track
    }
}
