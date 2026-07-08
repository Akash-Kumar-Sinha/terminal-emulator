use crate::theme;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum HeaderButton {
    Minimize,
    Maximize,
    Close,
}

const BUTTON_WIDTH: f32 = 46.0;

pub struct Header {
    pub height: f32,
    pub bg_color: [f32; 4],
    pub title: String,
    pub title_color: [f32; 4],
    pub icon_color: [f32; 4],
    pub button_hover: [f32; 4],
    pub close_hover: [f32; 4],
    pub hovered: Option<HeaderButton>,
}

impl Default for Header {
    fn default() -> Self {
        Self::new("AKS emulator")
    }
}

impl Header {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            height: 36.0,
            bg_color: theme::Theme::default().header_bg,
            title: title.into(),
            title_color: theme::Theme::default().header_title,
            icon_color: theme::Theme::default().header_icon,
            button_hover: theme::Theme::default().header_button_hover,
            close_hover: theme::Theme::default().header_close_hover,
            hovered: None,
        }
    }

    pub fn button_rect(&self, btn: HeaderButton, window_w: f32) -> (f32, f32, f32, f32) {
        let slot = match btn {
            HeaderButton::Close => 0.0,
            HeaderButton::Maximize => 1.0,
            HeaderButton::Minimize => 2.0,
        };
        let x = window_w - (slot + 1.0) * BUTTON_WIDTH;
        (x, 0.0, BUTTON_WIDTH, self.height)
    }

    pub fn hit_test(&self, x: f32, y: f32, window_w: f32) -> Option<HeaderButton> {
        if y < 0.0 || y > self.height {
            return None;
        }
        for btn in [
            HeaderButton::Minimize,
            HeaderButton::Maximize,
            HeaderButton::Close,
        ] {
            let (bx, _, bw, _) = self.button_rect(btn, window_w);
            if x >= bx && x < bx + bw {
                return Some(btn);
            }
        }
        None
    }

    pub fn in_drag_region(&self, x: f32, y: f32, window_w: f32) -> bool {
        y >= 0.0 && y <= self.height && self.hit_test(x, y, window_w).is_none()
    }

    pub fn icon(btn: HeaderButton) -> char {
        match btn {
            HeaderButton::Minimize => '\u{2212}',
            HeaderButton::Maximize => '\u{25A1}',
            HeaderButton::Close => '\u{00D7}',
        }
    }
}
