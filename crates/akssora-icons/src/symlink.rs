// INFO: Diagonal arrow glyph representing filesystem symlinks.
pub struct SymlinkIcon;

impl SymlinkIcon {
    pub const GLYPH: &'static str = "\u{2197}";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_symlink_icon() {
        assert_eq!(SymlinkIcon::GLYPH, "\u{2197}");
    }
}
