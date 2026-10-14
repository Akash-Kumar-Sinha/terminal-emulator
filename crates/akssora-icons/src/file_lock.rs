// INFO: Diamond glyph representing Lockfiles (.lock).
pub struct FileLockIcon;

impl FileLockIcon {
    pub const GLYPH: &'static str = "\u{25C6}";

    pub fn matches(lower_name: &str) -> bool {
        lower_name.ends_with(".lock")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_lock_icon() {
        assert_eq!(FileLockIcon::GLYPH, "\u{25C6}");
        assert!(FileLockIcon::matches("Cargo.lock"));
    }
}
