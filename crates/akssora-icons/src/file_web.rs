// INFO: Diamond in square glyph representing Web stylesheets and markup (.html, .css, .scss).
pub struct FileWebIcon;

impl FileWebIcon {
    pub const GLYPH: &'static str = "\u{25C8}";

    pub fn matches(lower_name: &str) -> bool {
        lower_name.ends_with(".html")
            || lower_name.ends_with(".css")
            || lower_name.ends_with(".scss")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_web_icon() {
        assert_eq!(FileWebIcon::GLYPH, "\u{25C8}");
        assert!(FileWebIcon::matches("index.html"));
    }
}
