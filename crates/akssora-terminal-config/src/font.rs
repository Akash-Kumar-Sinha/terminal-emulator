// INFO: Font configuration for typeface selection, metrics, and sizing.
#[derive(Clone, Debug, PartialEq)]
pub struct FontConfig {
    pub size: f32,
    pub line_height_ratio: f32,
    pub family: String,
}

impl FontConfig {
    pub const MIN_SIZE: f32 = 8.0;
    pub const MAX_SIZE: f32 = 32.0;
    pub const DEFAULT_SIZE: f32 = 14.0;
    pub const STEP: f32 = 1.0;

    pub fn increase(&mut self) -> f32 {
        self.size = (self.size + Self::STEP).min(Self::MAX_SIZE);
        self.size
    }

    pub fn decrease(&mut self) -> f32 {
        self.size = (self.size - Self::STEP).max(Self::MIN_SIZE);
        self.size
    }

    pub fn reset(&mut self) -> f32 {
        self.size = Self::DEFAULT_SIZE;
        self.size
    }
}

impl Default for FontConfig {
    fn default() -> Self {
        Self {
            size: Self::DEFAULT_SIZE,
            line_height_ratio: 1.2,
            family: "Roboto Mono".to_string(),
        }
    }
}
