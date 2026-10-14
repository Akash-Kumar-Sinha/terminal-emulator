// INFO: Triple bar glyph representing the Markdown Preview action.
pub struct PreviewIcon;

impl PreviewIcon {
    pub const GLYPH: &'static str = "\u{2261}";
    pub const CHAR: char = '\u{2261}';
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_preview_icon() {
        assert_eq!(PreviewIcon::GLYPH, "\u{2261}");
        assert_eq!(PreviewIcon::CHAR, '\u{2261}');
    }
}
