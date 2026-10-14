// INFO: Bidirectional horizontal arrow icon representing Staged/Unstaged toggle and API testing.
pub struct ArrowLeftRightIcon;

impl ArrowLeftRightIcon {
    pub const GLYPH: &'static str = "\u{21C4}";
    pub const CHAR: char = '\u{21C4}';
    pub const STAGED_LABEL: &'static str = "\u{21C4}  Staged changes";
    pub const UNSTAGED_LABEL: &'static str = "\u{21C4}  Unstaged changes";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arrow_left_right_icon() {
        assert_eq!(ArrowLeftRightIcon::GLYPH, "\u{21C4}");
        assert_eq!(ArrowLeftRightIcon::CHAR, '\u{21C4}');
    }
}
