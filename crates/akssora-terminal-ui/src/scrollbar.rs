use akssora_terminal_config::Theme;

// INFO: Terminal output area scrollbar with dynamic thumb sizing, hover states, and drag tracking.
#[derive(Clone, Debug)]
pub struct Scrollbar {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub is_hovered: bool,
    pub is_dragging: bool,
    pub drag_start_y: f32,
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
            is_hovered: false,
            is_dragging: false,
            drag_start_y: 0.0,
            drag_start_offset: 0.0,
        }
    }

    pub fn update_geometry(&mut self, block_x: f32, block_y: f32, block_w: f32, output_h: f32) {
        self.x = (block_x + block_w - Self::VISIBLE_WIDTH - 2.0).max(0.0);
        self.y = block_y;
        self.height = output_h.max(0.0);
    }

    pub fn hit_test(&self, mx: f32, my: f32) -> bool {
        if self.height <= 0.0 {
            return false;
        }
        let hit_x0 = self.x + self.width - Self::HIT_WIDTH;
        let hit_x1 = self.x + self.width + 4.0;
        mx >= hit_x0 && mx <= hit_x1 && my >= self.y && my <= self.y + self.height
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

    pub fn thumb_color(&self, theme: &Theme) -> [f32; 4] {
        if self.is_dragging || self.is_hovered {
            theme.session_selected_background
        } else {
            theme.header_background
        }
    }

    pub fn track_color(&self, theme: &Theme) -> [f32; 4] {
        [
            theme.background[0],
            theme.background[1],
            theme.background[2],
            0.0,
        ]
    }
}
