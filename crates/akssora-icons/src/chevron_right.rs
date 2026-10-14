// INFO: Rightward chevron / arrow icon for collapsed trees and accordions.
pub struct ChevronRightIcon;

impl ChevronRightIcon {
    pub const GLYPH_SMALL: &'static str = "\u{25B8}";
    pub const GLYPH_LARGE: &'static str = "\u{25B6}";
    pub const CHAR_SMALL: char = '\u{25B8}';
    pub const CHAR_LARGE: char = '\u{25B6}';
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chevron_right_icon() {
        assert_eq!(ChevronRightIcon::GLYPH_SMALL, "\u{25B8}");
        assert_eq!(ChevronRightIcon::GLYPH_LARGE, "\u{25B6}");
    }
}
