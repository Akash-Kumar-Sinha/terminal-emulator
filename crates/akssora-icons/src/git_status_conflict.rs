// INFO: Exclamation mark glyph representing Git Conflict status.
pub struct GitStatusConflictIcon;

impl GitStatusConflictIcon {
    pub const GLYPH: &'static str = "!";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_git_status_conflict_icon() {
        assert_eq!(GitStatusConflictIcon::GLYPH, "!");
    }
}
