// INFO: Window configuration parameters for geometry and initial presentation.
#[derive(Clone, Debug, PartialEq)]
pub struct WindowConfig {
    pub title: String,
    pub width: u32,
    pub height: u32,
    pub decorations: bool,
    pub margin: f32,
    pub min_block_height: f32,
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            title: "Akssora".to_string(),
            width: 1200,
            height: 800,
            decorations: false,
            margin: 2.0,
            min_block_height: 100.0,
        }
    }
}
