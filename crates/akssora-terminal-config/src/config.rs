use std::path::PathBuf;

use crate::font::FontConfig;
use crate::keybinding::Keybindings;
use crate::layout::TabLayoutPreference;
use crate::theme::Theme;
use crate::window::WindowConfig;

// INFO: Root configuration model aggregating theme, font, window, tab layout, and keybindings.
#[derive(Clone, Debug, PartialEq)]
pub struct Config {
    pub theme: Theme,
    pub theme_name: String,
    pub font: FontConfig,
    pub ui_font_size: f32,
    pub window: WindowConfig,
    pub tab_layout: TabLayoutPreference,
    pub keybindings: Keybindings,
    pub sidebar_width: f32,
    pub cursor_style: String, // "Block", "Beam", "Underline"
    pub cursor_blink: bool,
    pub tab_size: usize,
    pub line_numbers: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            theme: Theme::default(),
            theme_name: "Akssora Pure Black".to_string(),
            font: FontConfig::default(),
            ui_font_size: 14.0,
            window: WindowConfig::default(),
            tab_layout: TabLayoutPreference::default(),
            keybindings: Keybindings::default(),
            sidebar_width: 240.0,
            cursor_style: "Block".to_string(),
            cursor_blink: true,
            tab_size: 4,
            line_numbers: true,
        }
    }
}

impl Config {
    pub fn new() -> Self {
        Self::load()
    }

    pub fn config_path() -> PathBuf {
        if let Ok(dir) = std::env::var("AKSSORA_CONFIG_DIR") {
            PathBuf::from(dir).join("config.json")
        } else if let Ok(home) = std::env::var("HOME") {
            PathBuf::from(home)
                .join(".config")
                .join("akssora")
                .join("config.json")
        } else {
            PathBuf::from(".config").join("akssora").join("config.json")
        }
    }

    pub fn load() -> Self {
        let mut cfg = Self::default();
        let path = Self::config_path();
        if path.exists()
            && let Ok(data) = std::fs::read_to_string(&path)
        {
            cfg.apply_json(&data);
        }
        cfg
    }

    pub fn save(&self) -> std::io::Result<()> {
        let path = Self::config_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let json = self.to_json();
        std::fs::write(path, json)?;
        Ok(())
    }

    pub fn apply_json(&mut self, json_str: &str) {
        for line in json_str.lines() {
            let line = line.trim();
            if line.starts_with("\"tab_layout\":") || line.starts_with("\"tab_layout\" :") {
                if let Some(val) = line.split(':').nth(1) {
                    let cleaned = val
                        .trim()
                        .trim_matches(|c| c == ',' || c == '"' || c == ' ');
                    self.tab_layout = TabLayoutPreference::parse(cleaned);
                }
            } else if line.starts_with("\"font_size\":") || line.starts_with("\"font_size\" :") {
                if let Some(val) = line.split(':').nth(1) {
                    let cleaned = val
                        .trim()
                        .trim_matches(|c| c == ',' || c == '"' || c == ' ');
                    if let Ok(sz) = cleaned.parse::<f32>() {
                        self.font.size = sz.clamp(FontConfig::MIN_SIZE, FontConfig::MAX_SIZE);
                    }
                }
            } else if line.starts_with("\"ui_font_size\":")
                || line.starts_with("\"ui_font_size\" :")
            {
                if let Some(val) = line.split(':').nth(1) {
                    let cleaned = val
                        .trim()
                        .trim_matches(|c| c == ',' || c == '"' || c == ' ');
                    if let Ok(sz) = cleaned.parse::<f32>() {
                        self.ui_font_size = sz.clamp(FontConfig::MIN_SIZE, FontConfig::MAX_SIZE);
                    }
                }
            } else if line.starts_with("\"window_width\":")
                || line.starts_with("\"window_width\" :")
            {
                if let Some(val) = line.split(':').nth(1) {
                    let cleaned = val
                        .trim()
                        .trim_matches(|c| c == ',' || c == '"' || c == ' ');
                    if let Ok(w) = cleaned.parse::<u32>()
                        && w >= 400
                    {
                        self.window.width = w;
                    }
                }
            } else if line.starts_with("\"window_height\":")
                || line.starts_with("\"window_height\" :")
            {
                if let Some(val) = line.split(':').nth(1) {
                    let cleaned = val
                        .trim()
                        .trim_matches(|c| c == ',' || c == '"' || c == ' ');
                    if let Ok(h) = cleaned.parse::<u32>()
                        && h >= 300
                    {
                        self.window.height = h;
                    }
                }
            } else if (line.starts_with("\"sidebar_width\":")
                || line.starts_with("\"sidebar_width\" :"))
                && let Some(val) = line.split(':').nth(1)
            {
                let cleaned = val
                    .trim()
                    .trim_matches(|c| c == ',' || c == '"' || c == ' ');
                if let Ok(sw) = cleaned.parse::<f32>() {
                    self.sidebar_width = sw.clamp(160.0, 480.0);
                }
            } else if (line.starts_with("\"theme\":") || line.starts_with("\"theme\" :"))
                && let Some(val) = line.split(':').nth(1)
            {
                let cleaned = val
                    .trim()
                    .trim_matches(|c| c == ',' || c == '"' || c == ' ');
                self.theme_name = cleaned.to_string();
                self.theme = Theme::from_name(cleaned);
            } else if (line.starts_with("\"cursor_style\":")
                || line.starts_with("\"cursor_style\" :"))
                && let Some(val) = line.split(':').nth(1)
            {
                let cleaned = val
                    .trim()
                    .trim_matches(|c| c == ',' || c == '"' || c == ' ');
                self.cursor_style = cleaned.to_string();
            } else if (line.starts_with("\"cursor_blink\":")
                || line.starts_with("\"cursor_blink\" :"))
                && let Some(val) = line.split(':').nth(1)
            {
                let cleaned = val
                    .trim()
                    .trim_matches(|c| c == ',' || c == '"' || c == ' ');
                self.cursor_blink = cleaned == "true";
            } else if (line.starts_with("\"tab_size\":") || line.starts_with("\"tab_size\" :"))
                && let Some(val) = line.split(':').nth(1)
            {
                let cleaned = val
                    .trim()
                    .trim_matches(|c| c == ',' || c == '"' || c == ' ');
                if let Ok(ts) = cleaned.parse::<usize>() {
                    self.tab_size = ts.clamp(2, 8);
                }
            } else if (line.starts_with("\"line_numbers\":")
                || line.starts_with("\"line_numbers\" :"))
                && let Some(val) = line.split(':').nth(1)
            {
                let cleaned = val
                    .trim()
                    .trim_matches(|c| c == ',' || c == '"' || c == ' ');
                self.line_numbers = cleaned == "true";
            }
        }
    }

    pub fn to_json(&self) -> String {
        format!(
            "{{\n  \"tab_layout\": \"{}\",\n  \"font_size\": {},\n  \"ui_font_size\": {},\n  \"window_width\": {},\n  \"window_height\": {},\n  \"sidebar_width\": {},\n  \"theme\": \"{}\",\n  \"cursor_style\": \"{}\",\n  \"cursor_blink\": {},\n  \"tab_size\": {},\n  \"line_numbers\": {}\n}}\n",
            self.tab_layout.label(),
            self.font.size,
            self.ui_font_size,
            self.window.width,
            self.window.height,
            self.sidebar_width,
            self.theme_name,
            self.cursor_style,
            self.cursor_blink,
            self.tab_size,
            self.line_numbers
        )
    }
}
