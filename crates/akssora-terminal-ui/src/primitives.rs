// INFO: 2D geometry and colored rectangular primitives for window chrome and decorations.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ChromeRect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub color: [f32; 4],
}

// INFO: Styled text element specification for window titlebar labels.
#[derive(Clone, Debug, PartialEq)]
pub struct ChromeText {
    pub x: f32,
    pub y: f32,
    pub text: String,
    pub color: [f32; 4],
    pub bold: bool,
}
