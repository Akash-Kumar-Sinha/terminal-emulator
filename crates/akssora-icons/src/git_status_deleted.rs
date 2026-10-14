// INFO: Minus glyph representing Git Deleted status.
pub struct GitStatusDeletedIcon;

impl GitStatusDeletedIcon {
    pub const GLYPH: &'static str = "-";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_git_status_deleted_icon() {
        assert_eq!(GitStatusDeletedIcon::GLYPH, "-");
    }
}
