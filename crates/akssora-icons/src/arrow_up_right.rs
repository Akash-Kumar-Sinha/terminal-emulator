// INFO: Diagonal upper-right arrow icon representing external open or symlink.
pub struct ArrowUpRightIcon;

impl ArrowUpRightIcon {
    pub const GLYPH: &'static str = "\u{2197}";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arrow_up_right_icon() {
        assert_eq!(ArrowUpRightIcon::GLYPH, "\u{2197}");
    }
}
