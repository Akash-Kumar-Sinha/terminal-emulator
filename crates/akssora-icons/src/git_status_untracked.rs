// INFO: Question mark glyph representing Git Untracked status.
pub struct GitStatusUntrackedIcon;

impl GitStatusUntrackedIcon {
    pub const GLYPH: &'static str = "?";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_git_status_untracked_icon() {
        assert_eq!(GitStatusUntrackedIcon::GLYPH, "?");
    }
}
