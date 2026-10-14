// INFO: Two joined squares glyph representing Toggle Full File view.
pub struct FullFileIcon;

impl FullFileIcon {
    pub const GLYPH: &'static str = "\u{29C9}";
    pub const CHAR: char = '\u{29C9}';
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_full_file_icon() {
        assert_eq!(FullFileIcon::GLYPH, "\u{29C9}");
    }
}
