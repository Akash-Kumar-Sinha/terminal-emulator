use super::{Header, TAB_HEIGHT, TAB_MAX_WIDTH, TAB_MIN_WIDTH};

// INFO: Titlebar horizontal tab bar positioning and layout rectangle calculations.
impl Header {
    pub fn tab_bar_rect(&self, window_w: f32) -> (f32, f32, f32, f32) {
        (0.0, self.height, window_w, Self::TAB_BAR_HEIGHT)
    }

    pub fn tabs_start_x(&self) -> f32 {
        8.0
    }

    pub fn tabs_start_x_ex(&self, _show_toggle: bool) -> f32 {
        8.0
    }

    pub fn tabs_available_width(&self, window_w: f32) -> f32 {
        (window_w - 50.0).max(0.0)
    }

    pub fn tabs_available_width_ex(
        &self,
        window_w: f32,
        _show_toggle: bool,
        _show_header_settings: bool,
    ) -> f32 {
        (window_w - 50.0).max(0.0)
    }

    pub fn tab_rect(
        &self,
        idx: usize,
        total_tabs: usize,
        window_w: f32,
    ) -> Option<(f32, f32, f32, f32)> {
        self.tab_rect_ex(idx, total_tabs, window_w, false, false)
    }

    pub fn tab_rect_ex(
        &self,
        idx: usize,
        total_tabs: usize,
        window_w: f32,
        _show_toggle: bool,
        _show_header_settings: bool,
    ) -> Option<(f32, f32, f32, f32)> {
        if total_tabs == 0 {
            return None;
        }
        let avail_w = self.tabs_available_width(window_w);
        let item_w = (avail_w / total_tabs as f32).clamp(TAB_MIN_WIDTH, TAB_MAX_WIDTH);
        let x = self.tabs_start_x() + idx as f32 * (item_w + 3.0);
        let y = self.height + (Self::TAB_BAR_HEIGHT - TAB_HEIGHT) / 2.0;
        Some((x, y, item_w, TAB_HEIGHT))
    }

    pub fn tab_close_rect(
        &self,
        idx: usize,
        total_tabs: usize,
        window_w: f32,
    ) -> Option<(f32, f32, f32, f32)> {
        self.tab_close_rect_ex(idx, total_tabs, window_w, false, false)
    }

    pub fn tab_close_rect_ex(
        &self,
        idx: usize,
        total_tabs: usize,
        window_w: f32,
        _show_toggle: bool,
        _show_header_settings: bool,
    ) -> Option<(f32, f32, f32, f32)> {
        let (tx, ty, tw, th) = self.tab_rect_ex(idx, total_tabs, window_w, false, false)?;
        let btn_size = 18.0;
        let bx = tx + tw - btn_size - 6.0;
        let by = ty + (th - btn_size) / 2.0;
        Some((bx, by, btn_size, btn_size))
    }

    pub fn new_tab_rect(&self, total_tabs: usize, window_w: f32) -> (f32, f32, f32, f32) {
        self.new_tab_rect_ex(total_tabs, window_w, false, false)
    }

    pub fn new_tab_rect_ex(
        &self,
        total_tabs: usize,
        window_w: f32,
        _show_toggle: bool,
        _show_header_settings: bool,
    ) -> (f32, f32, f32, f32) {
        let last_x = if total_tabs == 0 {
            self.tabs_start_x()
        } else if let Some((tx, _, tw, _)) = self.tab_rect(total_tabs - 1, total_tabs, window_w) {
            tx + tw + 4.0
        } else {
            self.tabs_start_x()
        };
        let y = self.height + (Self::TAB_BAR_HEIGHT - 22.0) / 2.0;
        (last_x, y, 22.0, 22.0)
    }
}
