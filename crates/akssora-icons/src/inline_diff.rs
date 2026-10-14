// INFO: Plus-minus sign glyph representing Toggle Inline Diff.
pub struct InlineDiffIcon;

impl InlineDiffIcon {
    pub const GLYPH: &'static str = "\u{00B1}";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inline_diff_icon() {
        assert_eq!(InlineDiffIcon::GLYPH, "\u{00B1}");
    }
}
