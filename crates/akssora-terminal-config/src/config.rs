use crate::font::FontConfig;
use crate::theme::Theme;
use crate::window::WindowConfig;

// INFO: Root configuration model aggregating theme, font, and window settings.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Config {
    pub theme: Theme,
    pub font: FontConfig,
    pub window: WindowConfig,
}

impl Config {
    pub fn new() -> Self {
        Self::default()
    }
}
