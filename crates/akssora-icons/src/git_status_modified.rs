// INFO: Bullet glyph representing Git Modified status.
pub struct GitStatusModifiedIcon;

impl GitStatusModifiedIcon {
    pub const GLYPH: &'static str = "\u{2022}";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_git_status_modified_icon() {
        assert_eq!(GitStatusModifiedIcon::GLYPH, "\u{2022}");
    }
}
