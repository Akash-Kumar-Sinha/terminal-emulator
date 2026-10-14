// INFO: Two joined squares glyph representing the Copy action.
pub struct CopyIcon;

impl CopyIcon {
    pub const GLYPH: &'static str = "\u{29C9}";
    pub const CHAR: char = '\u{29C9}';
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_copy_icon() {
        assert_eq!(CopyIcon::GLYPH, "\u{29C9}");
        assert_eq!(CopyIcon::CHAR, '\u{29C9}');
    }
}
