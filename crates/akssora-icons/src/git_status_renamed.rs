// INFO: Rightward arrow glyph representing Git Renamed status.
pub struct GitStatusRenamedIcon;

impl GitStatusRenamedIcon {
    pub const GLYPH: &'static str = "\u{2192}";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_git_status_renamed_icon() {
        assert_eq!(GitStatusRenamedIcon::GLYPH, "\u{2192}");
    }
}
