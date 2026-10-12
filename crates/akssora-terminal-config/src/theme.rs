use crate::color::{Color, rgb, srgb_to_linear};

// INFO: Visual theme specification defining colors for application chrome,
// editor surfaces, text output, and terminal rendering.
#[derive(Clone, Debug, PartialEq)]
pub struct Theme {
    pub name: String,

    // Core backgrounds
    pub background: [f32; 4],
    pub sidebar_background: [f32; 4],
    pub header_background: [f32; 4],
    pub title_bar_background: [f32; 4],

    // Session backgrounds
    pub session_background: [f32; 4],
    pub session_selected_background: [f32; 4],
    pub session_hover_background: [f32; 4],

    // Tab backgrounds
    pub tab_background: [f32; 4],
    pub tab_selected_background: [f32; 4],
    pub tab_hover_background: [f32; 4],

    // Button backgrounds
    pub button_hover_background: [f32; 4],
    pub button_active_background: [f32; 4],
    pub close_button_hover_background: [f32; 4],

    // Borders
    pub border: [f32; 4],
    pub border_subtle: [f32; 4],

    // Text and foreground colors
    pub foreground: [f32; 4],
    pub text_primary: [f32; 4],
    pub text_secondary: [f32; 4],
    pub text_muted: [f32; 4],

    // Header foreground and interaction colors
    pub header_foreground: [f32; 4],
    pub header_icon: [f32; 4],
    pub header_button_hover_background: [f32; 4],

    // Accent colors
    pub accent: [f32; 4],
    pub accent_subtle: [f32; 4],
    pub accent_hover: [f32; 4],

    // Close button foreground
    pub close_button_foreground: [f32; 4],

    // Tooltip colors
    pub tooltip_background: [f32; 4],
    pub tooltip_border: [f32; 4],
    pub tooltip_foreground: [f32; 4],
    pub tooltip_shortcut_foreground: [f32; 4],

    // Terminal cursor and selection
    pub cursor: [f32; 4],
    pub cursor_foreground: [f32; 4],
    pub selection_background: [f32; 4],
    pub hover_background: [f32; 4],

    // ANSI terminal palette (preserves terminal colors)
    pub ansi: [[f32; 4]; 16],
}

impl Default for Theme {
    fn default() -> Self {
        Self::monochrome_black()
    }
}

impl Theme {
    pub fn monochrome_black() -> Self {
        let black = rgb(0, 0, 0);
        let near_black = rgb(1, 1, 1);

        let white = rgb(255, 255, 255);
        let text_secondary = rgb(220, 220, 220);
        let text_muted = rgb(112, 112, 112);

        let subtle_hover = rgb(18, 18, 18);

        Self {
            name: "Akssora Pure Black".to_string(),

            // Core backgrounds
            background: black,
            sidebar_background: near_black,
            header_background: near_black,
            title_bar_background: black,

            // Session backgrounds
            session_background: near_black,
            session_selected_background: rgb(3, 3, 3),
            session_hover_background: rgb(6, 6, 6),

            // Tab backgrounds
            tab_background: rgb(3, 3, 3),
            tab_selected_background: rgb(6, 6, 6),
            tab_hover_background: rgb(5, 5, 5),

            // Button backgrounds
            button_hover_background: rgb(6, 6, 6),
            button_active_background: rgb(3, 3, 3),
            close_button_hover_background: rgb(5, 5, 5),

            // Borders
            border: black,
            border_subtle: rgb(24, 24, 27),

            // Text and foreground colors
            foreground: white,
            text_primary: white,
            text_secondary,
            text_muted,

            // Header foreground and interaction colors
            header_foreground: white,
            header_icon: white,
            header_button_hover_background: subtle_hover,

            // Accent colors (monochrome)
            accent: white,
            accent_subtle: rgb(180, 180, 180),
            accent_hover: rgb(220, 220, 220),

            // Close button foreground
            close_button_foreground: white,

            // Tooltip colors
            tooltip_background: black,
            tooltip_border: rgb(2, 2, 2),
            tooltip_foreground: white,
            tooltip_shortcut_foreground: text_muted,

            // Terminal cursor and selection
            cursor: white,
            cursor_foreground: black,
            selection_background: rgb(45, 45, 45),
            hover_background: rgb(15, 15, 15),

            // ANSI colors
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

    pub fn slate_dark() -> Self {
        let bg = rgb(15, 23, 42); // slate-900
        let surface = rgb(30, 41, 59); // slate-800
        let surface_hover = rgb(51, 65, 85); // slate-700
        let white = rgb(248, 250, 252); // slate-50
        let text_sec = rgb(203, 213, 225); // slate-300
        let text_mut = rgb(148, 163, 184); // slate-400

        Self {
            name: "Slate Dark".to_string(),

            // Core backgrounds
            background: bg,
            sidebar_background: surface,
            header_background: surface,
            title_bar_background: bg,

            // Session backgrounds
            session_background: surface,
            session_selected_background: rgb(51, 65, 85),
            session_hover_background: surface_hover,

            // Tab backgrounds
            tab_background: surface,
            tab_selected_background: rgb(51, 65, 85),
            tab_hover_background: surface_hover,

            // Button backgrounds
            button_hover_background: surface_hover,
            button_active_background: rgb(71, 85, 105),
            close_button_hover_background: rgb(71, 85, 105),

            // Borders
            border: bg,
            border_subtle: rgb(51, 65, 85),

            // Text and foreground colors
            foreground: white,
            text_primary: white,
            text_secondary: text_sec,
            text_muted: text_mut,

            // Header foreground and interaction colors
            header_foreground: white,
            header_icon: white,
            header_button_hover_background: surface_hover,

            // Accent colors
            accent: rgb(56, 189, 248), // sky-400
            accent_subtle: rgb(148, 163, 184),
            accent_hover: rgb(125, 211, 252),

            // Close button foreground
            close_button_foreground: white,

            // Tooltip colors
            tooltip_background: bg,
            tooltip_border: rgb(51, 65, 85),
            tooltip_foreground: white,
            tooltip_shortcut_foreground: text_mut,

            // Terminal cursor and selection
            cursor: rgb(56, 189, 248),
            cursor_foreground: bg,
            selection_background: rgb(51, 65, 85),
            hover_background: surface_hover,

            // ANSI colors
            ansi: [
                rgb(15, 23, 42),
                rgb(239, 68, 68),
                rgb(34, 197, 94),
                rgb(234, 179, 8),
                rgb(59, 130, 246),
                rgb(168, 85, 247),
                rgb(6, 182, 212),
                rgb(226, 232, 240),
                rgb(100, 116, 139),
                rgb(248, 113, 113),
                rgb(74, 222, 128),
                rgb(250, 204, 21),
                rgb(96, 165, 250),
                rgb(192, 132, 252),
                rgb(34, 211, 238),
                rgb(255, 255, 255),
            ],
        }
    }

    pub fn from_name(name: &str) -> Self {
        match name.to_lowercase().as_str() {
            "slate dark" | "slate" => Self::slate_dark(),
            _ => Self::monochrome_black(),
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
                let value = 8 + (idx - 232) * 10;
                rgb(value, value, value)
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
