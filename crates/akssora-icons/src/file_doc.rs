// INFO: Triple bar glyph representing documentation and text files (.md, .txt, .log).
pub struct FileDocIcon;

impl FileDocIcon {
    pub const GLYPH: &'static str = "\u{2261}";

    pub fn matches(lower_name: &str) -> bool {
        lower_name.ends_with(".md") || lower_name.ends_with(".txt") || lower_name.ends_with(".log")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_doc_icon() {
        assert_eq!(FileDocIcon::GLYPH, "\u{2261}");
        assert!(FileDocIcon::matches("readme.md"));
    }
}
