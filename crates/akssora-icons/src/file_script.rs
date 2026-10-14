// INFO: Sparkle glyph representing JavaScript and TypeScript script files (.js, .ts, .jsx, .tsx).
pub struct FileScriptIcon;

impl FileScriptIcon {
    pub const GLYPH: &'static str = "\u{2737}";

    pub fn matches(lower_name: &str) -> bool {
        lower_name.ends_with(".js")
            || lower_name.ends_with(".ts")
            || lower_name.ends_with(".tsx")
            || lower_name.ends_with(".jsx")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_script_icon() {
        assert_eq!(FileScriptIcon::GLYPH, "\u{2737}");
        assert!(FileScriptIcon::matches("app.tsx"));
    }
}
