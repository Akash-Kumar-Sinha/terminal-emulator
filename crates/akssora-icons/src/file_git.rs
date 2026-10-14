// INFO: Star glyph representing Git files (.git, .gitignore).
pub struct FileGitIcon;

impl FileGitIcon {
    pub const GLYPH: &'static str = "\u{2605}";

    pub fn matches(lower_name: &str) -> bool {
        lower_name.starts_with(".git") || lower_name.ends_with(".gitignore")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_git_icon() {
        assert_eq!(FileGitIcon::GLYPH, "\u{2605}");
        assert!(FileGitIcon::matches(".gitignore"));
    }
}
