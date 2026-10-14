// INFO: Multiplication sign cross glyph representing the Close button.
pub struct CloseIcon;

impl CloseIcon {
    pub const GLYPH: &'static str = "\u{00D7}";
    pub const CHAR: char = '\u{00D7}';
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_close_icon() {
        assert_eq!(CloseIcon::GLYPH, "\u{00D7}");
        assert_eq!(CloseIcon::CHAR, '\u{00D7}');
    }
}
