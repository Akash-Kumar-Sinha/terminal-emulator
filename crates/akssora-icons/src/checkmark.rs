// INFO: Checkmark glyph representing checked, selected, or active state.
pub struct CheckmarkIcon;

impl CheckmarkIcon {
    pub const GLYPH: &'static str = "\u{2713}";
    pub const GLYPH_SPACE: &'static str = "\u{2713} ";
    pub const BRANCH_CURRENT: &'static str = "\u{2713} ";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_checkmark_icon() {
        assert_eq!(CheckmarkIcon::GLYPH, "\u{2713}");
    }
}
