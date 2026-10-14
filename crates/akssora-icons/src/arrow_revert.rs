// INFO: Curved return arrow icon representing line revert in gutter.
pub struct ArrowRevertIcon;

impl ArrowRevertIcon {
    pub const GLYPH: &'static str = "\u{21B6}";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arrow_revert_icon() {
        assert_eq!(ArrowRevertIcon::GLYPH, "\u{21B6}");
    }
}
