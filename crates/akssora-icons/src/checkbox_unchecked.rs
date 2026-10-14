// INFO: Unchecked markdown checkbox glyph string ([ ] ).
pub struct CheckboxUncheckedIcon;

impl CheckboxUncheckedIcon {
    pub const GLYPH: &'static str = "[ ] ";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_checkbox_unchecked_icon() {
        assert_eq!(CheckboxUncheckedIcon::GLYPH, "[ ] ");
    }
}
