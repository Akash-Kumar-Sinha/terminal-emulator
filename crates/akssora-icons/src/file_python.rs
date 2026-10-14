// INFO: Pi glyph representing Python source files (.py).
pub struct FilePythonIcon;

impl FilePythonIcon {
    pub const GLYPH: &'static str = "\u{03C0}";
    pub const EXTENSION: &'static str = ".py";

    pub fn matches(lower_name: &str) -> bool {
        lower_name.ends_with(".py")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_python_icon() {
        assert_eq!(FilePythonIcon::GLYPH, "\u{03C0}");
        assert!(FilePythonIcon::matches("script.py"));
    }
}
