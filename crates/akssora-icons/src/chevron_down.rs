// INFO: Downward chevron / arrow icon for expanded trees, dropdowns, and context expansion.
pub struct ChevronDownIcon;

impl ChevronDownIcon {
    pub const GLYPH: &'static str = Self::GLYPH_SMALL;
    pub const GLYPH_SMALL: &'static str = "\u{25BE}";
    pub const GLYPH_LARGE: &'static str = "\u{25BC}";
    pub const CHAR_SMALL: char = '\u{25BE}';
    pub const CHAR_LARGE: char = '\u{25BC}';
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chevron_down_icon() {
        assert_eq!(ChevronDownIcon::GLYPH_SMALL, "\u{25BE}");
        assert_eq!(ChevronDownIcon::GLYPH_LARGE, "\u{25BC}");
    }
}
