// INFO: Vertical split rectangle glyph representing Split Preview.
pub struct SplitPreviewIcon;

impl SplitPreviewIcon {
    pub const GLYPH: &'static str = "\u{25EB}";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_preview_icon() {
        assert_eq!(SplitPreviewIcon::GLYPH, "\u{25EB}");
    }
}
