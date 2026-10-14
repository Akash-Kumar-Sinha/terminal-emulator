// INFO: Gear glyph representing Rust source files (.rs).
pub struct FileRustIcon;

impl FileRustIcon {
    pub const GLYPH: &'static str = "\u{2699}";
    pub const EXTENSION: &'static str = ".rs";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_rust_icon() {
        assert_eq!(FileRustIcon::GLYPH, "\u{2699}");
    }
}
