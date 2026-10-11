// INFO: Font configuration for typeface selection, metrics, and sizing.
#[derive(Clone, Debug, PartialEq)]
pub struct FontConfig {
    pub size: f32,
    pub line_height_ratio: f32,
    pub family: String,
}

impl Default for FontConfig {
    fn default() -> Self {
        Self {
            size: 14.0,
            line_height_ratio: 1.2,
            family: "Roboto Mono".to_string(),
        }
    }
}
