// INFO: Upward chevron / arrow icon for open dropdowns and context expansion top.
pub struct ChevronUpIcon;

impl ChevronUpIcon {
    pub const GLYPH: &'static str = Self::GLYPH_SMALL;
    pub const GLYPH_SMALL: &'static str = "\u{25B4}";
    pub const GLYPH_LARGE: &'static str = "\u{25B2}";
    pub const CHAR_SMALL: char = '\u{25B4}';
    pub const CHAR_LARGE: char = '\u{25B2}';
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chevron_up_icon() {
        assert_eq!(ChevronUpIcon::GLYPH_SMALL, "\u{25B4}");
        assert_eq!(ChevronUpIcon::GLYPH_LARGE, "\u{25B2}");
    }
}
