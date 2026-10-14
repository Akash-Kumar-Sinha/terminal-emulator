// INFO: Rightward arrow icon representing Renamed status.
pub struct ArrowRightIcon;

impl ArrowRightIcon {
    pub const GLYPH: &'static str = "\u{2192}";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arrow_right_icon() {
        assert_eq!(ArrowRightIcon::GLYPH, "\u{2192}");
    }
}
