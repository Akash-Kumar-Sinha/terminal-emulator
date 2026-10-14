// INFO: Single chevron prompt glyph representing Shell script files (.sh, .bash, .zsh).
pub struct FileShellIcon;

impl FileShellIcon {
    pub const GLYPH: &'static str = "\u{203A}";

    pub fn matches(lower_name: &str) -> bool {
        lower_name.ends_with(".sh") || lower_name.ends_with(".bash") || lower_name.ends_with(".zsh")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_shell_icon() {
        assert_eq!(FileShellIcon::GLYPH, "\u{203A}");
        assert!(FileShellIcon::matches("deploy.sh"));
    }
}
