// INFO: Tooltip configuration and dynamic layout calculator for toolbar buttons.
#[derive(Clone, Debug, PartialEq)]
pub struct Tooltip {
    pub text: String,
    pub shortcut: Option<String>,
    pub target_rect: (f32, f32, f32, f32),
}

impl Tooltip {
    pub fn new(text: impl Into<String>, shortcut: Option<String>) -> Self {
        Self {
            text: text.into(),
            shortcut,
            target_rect: (0.0, 0.0, 0.0, 0.0),
        }
    }

    pub fn for_target(
        text: impl Into<String>,
        shortcut: Option<String>,
        target_rect: (f32, f32, f32, f32),
    ) -> Self {
        Self {
            text: text.into(),
            shortcut,
            target_rect,
        }
    }

    pub fn display_label(&self) -> String {
        if let Some(sc) = &self.shortcut
            && !sc.trim().is_empty()
        {
            return format!("{} ({})", self.text, sc);
        }
        self.text.clone()
    }

    #[allow(clippy::too_many_arguments)]
    pub fn bounds_for_target(
        &self,
        target_x: f32,
        target_y: f32,
        target_w: f32,
        target_h: f32,
        char_w: f32,
        char_h: f32,
        screen_w: f32,
    ) -> (f32, f32, f32, f32) {
        let label = self.display_label();
        let padding_x = 8.0;
        let padding_y = 4.0;
        let width = label.len() as f32 * char_w + padding_x * 2.0;
        let height = char_h + padding_y * 2.0;

        let mut x = target_x + (target_w - width) / 2.0;
        if x < 4.0 {
            x = 4.0;
        } else if x + width > screen_w - 4.0 {
            x = (screen_w - 4.0 - width).max(4.0);
        }

        let y = target_y + target_h + 4.0;
        (x, y, width, height)
    }

    pub fn bounds(
        &self,
        char_w: f32,
        char_h: f32,
        screen_w: f32,
        screen_h: f32,
    ) -> (f32, f32, f32, f32) {
        let (tx, ty, tw, th) = self.target_rect;
        let label = self.display_label();
        let padding_x = 8.0;
        let padding_y = 4.0;
        let width = label.len() as f32 * char_w + padding_x * 2.0;
        let height = char_h + padding_y * 2.0;

        let mut x = tx + (tw - width) / 2.0;
        if x < 4.0 {
            x = 4.0;
        } else if x + width > screen_w - 4.0 {
            x = (screen_w - 4.0 - width).max(4.0);
        }

        let y = if ty + th + height + 4.0 <= screen_h {
            ty + th + 4.0
        } else {
            (ty - height - 4.0).max(4.0)
        };

        (x, y, width, height)
    }
}
