// INFO: Checked markdown checkbox glyph string ([✓] ).
pub struct CheckboxCheckedIcon;

impl CheckboxCheckedIcon {
    pub const GLYPH: &'static str = "[\u{2713}] ";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_checkbox_checked_icon() {
        assert_eq!(CheckboxCheckedIcon::GLYPH, "[✓] ");
    }
}
