// INFO: Branch glyph representing a Code Review session tab.
pub struct CodeReviewSessionIcon;

impl CodeReviewSessionIcon {
    pub const CHAR: char = '\u{2387}';
    pub const GLYPH: &'static str = "\u{2387}";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_code_review_session_icon() {
        assert_eq!(CodeReviewSessionIcon::CHAR, '\u{2387}');
    }
}
