use crate::color::{Color, rgb, srgb_to_linear};

// INFO: Visual theme specification defining colors for chrome, editor blocks, and text output.
#[derive(Clone, Debug, PartialEq)]
pub struct Theme {
    pub name: String,
    pub background: [f32; 4],
    pub foreground: [f32; 4],
    pub border: [f32; 4],
    pub cursor: [f32; 4],
    pub cursor_text: [f32; 4],
    pub selection: [f32; 4],
    pub header_bg: [f32; 4],
    pub header_title: [f32; 4],
    pub header_icon: [f32; 4],
    pub header_button_hover: [f32; 4],
    pub header_close_hover: [f32; 4],
    pub ansi: [[f32; 4]; 16],
}

impl Default for Theme {
    fn default() -> Self {
        Self::modern_dark()
    }
}

impl Theme {
    pub fn modern_dark() -> Self {
        Self {
            name: "Akssora Modern Dark".to_string(),
            background: rgb(0, 0, 0),
            border: rgb(0, 0, 0),
            foreground: rgb(236, 236, 236),
            cursor: rgb(9, 106, 185),
            cursor_text: rgb(30, 33, 36),
            selection: rgb(46, 51, 59),
            header_bg: rgb(1, 1, 1),
            header_title: rgb(236, 236, 236),
            header_icon: rgb(236, 236, 236),
            header_button_hover: rgb(5, 5, 5),
            header_close_hover: rgb(184, 11, 19),
            ansi: [
                rgb(43, 43, 43),
                rgb(213, 63, 61),
                rgb(91, 189, 78),
                rgb(220, 173, 60),
                rgb(40, 110, 230),
                rgb(196, 96, 209),
                rgb(38, 178, 178),
                rgb(200, 200, 205),
                rgb(108, 112, 122),
                rgb(232, 84, 74),
                rgb(110, 208, 92),
                rgb(233, 190, 90),
                rgb(58, 138, 247),
                rgb(214, 128, 226),
                rgb(58, 196, 196),
                rgb(255, 255, 255),
            ],
        }
    }

    pub fn clear_color_linear(&self) -> (f64, f64, f64, f64) {
        let [r, g, b, a] = self.background;
        (
            srgb_to_linear(r) as f64,
            srgb_to_linear(g) as f64,
            srgb_to_linear(b) as f64,
            a as f64,
        )
    }

    pub fn ansi_256_rgba(&self, idx: u8) -> [f32; 4] {
        match idx {
            0..=15 => self.ansi[idx as usize],
            16..=231 => {
                let i = idx - 16;
                let level = |c: u8| -> u8 { if c == 0 { 0 } else { 55 + c * 40 } };
                rgb(level(i / 36), level((i % 36) / 6), level(i % 6))
            }
            232..=255 => {
                let v = 8 + (idx - 232) * 10;
                rgb(v, v, v)
            }
        }
    }

    pub fn resolve_fg(&self, color: Color, bold: bool) -> [f32; 4] {
        match color {
            Color::Default => self.foreground,
            Color::Indexed(idx) => {
                if bold && idx < 8 {
                    self.ansi[idx as usize + 8]
                } else {
                    self.ansi_256_rgba(idx)
                }
            }
            Color::Rgb(r, g, b) => rgb(r, g, b),
        }
    }

    pub fn resolve_bg(&self, color: Color) -> Option<[f32; 4]> {
        match color {
            Color::Default => None,
            Color::Indexed(idx) => Some(self.ansi_256_rgba(idx)),
            Color::Rgb(r, g, b) => Some(rgb(r, g, b)),
        }
    }
}
