// INFO: Minus / hyphen glyph representing Remove, Unstage, or font size decrease.
pub struct RemoveIcon;

impl RemoveIcon {
    pub const GLYPH: &'static str = "-";
    pub const MINUS_GLYPH: &'static str = "\u{2212}";
    pub const CHAR: char = '-';
    pub const MINUS_CHAR: char = '\u{2212}';
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_remove_icon() {
        assert_eq!(RemoveIcon::GLYPH, "-");
        assert_eq!(RemoveIcon::MINUS_GLYPH, "−");
        assert_eq!(RemoveIcon::MINUS_CHAR, '−');
    }
}
