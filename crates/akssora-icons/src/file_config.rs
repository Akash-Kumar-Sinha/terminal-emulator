// INFO: Diamond glyph representing configuration files (.toml, .yaml, .yml, .json).
pub struct FileConfigIcon;

impl FileConfigIcon {
    pub const GLYPH: &'static str = "\u{2756}";

    pub fn matches(lower_name: &str) -> bool {
        lower_name == "cargo.toml"
            || lower_name.ends_with(".toml")
            || lower_name.ends_with(".yaml")
            || lower_name.ends_with(".yml")
            || lower_name.ends_with(".json")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_config_icon() {
        assert_eq!(FileConfigIcon::GLYPH, "\u{2756}");
        assert!(FileConfigIcon::matches("cargo.toml"));
        assert!(FileConfigIcon::matches("config.yaml"));
    }
}
