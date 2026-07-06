const fn rgb(r: u8, g: u8, b: u8) -> [f32; 4] {
    [r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0, 1.0]
}

fn srgb_to_linear(c: f32) -> f32 {
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Color {
    #[default]
    Default,
    Indexed(u8),
    Rgb(u8, u8, u8),
}

pub struct Theme {
    pub background: [f32; 4],
    pub foreground: [f32; 4],
    pub cursor: [f32; 4],
    pub cursor_text: [f32; 4],
    pub selection: [f32; 4],
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
            background: rgb(0, 0, 0),
            foreground: rgb(236, 236, 236),
            cursor: rgb(9, 106, 185),
            cursor_text: rgb(30, 33, 36),
            selection: rgb(46, 51, 59),
            ansi: [
                rgb(46, 52, 54),
                rgb(222, 32, 48),
                rgb(97, 203, 22),
                rgb(225, 152, 16),
                rgb(9, 136, 240),
                rgb(170, 6, 220),
                rgb(5, 179, 202),
                rgb(197, 200, 204),
                rgb(3, 59, 164),
                rgb(240, 7, 30),
                rgb(103, 228, 7),
                rgb(233, 153, 6),
                rgb(4, 138, 241),
                rgb(180, 5, 239),
                rgb(7, 197, 222),
                rgb(255, 255, 255),
            ],
        }
    }

    pub fn background_rgba(&self) -> [f32; 4] {
        self.background
    }

    pub fn clear_color(&self) -> wgpu::Color {
        let [r, g, b, a] = self.background;
        wgpu::Color {
            r: srgb_to_linear(r) as f64,
            g: srgb_to_linear(g) as f64,
            b: srgb_to_linear(b) as f64,
            a: a as f64,
        }
    }

    pub fn foreground_rgba(&self) -> [f32; 4] {
        self.foreground
    }

    pub fn cursor_rgba(&self) -> [f32; 4] {
        self.cursor
    }

    pub fn cursor_text_rgba(&self) -> [f32; 4] {
        self.cursor_text
    }

    pub fn selection_rgba(&self) -> [f32; 4] {
        self.selection
    }

    pub fn ansi_rgba(&self, idx: usize) -> [f32; 4] {
        self.ansi[idx]
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
