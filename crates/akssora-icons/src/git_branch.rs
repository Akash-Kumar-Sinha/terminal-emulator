// INFO: Git branch icon representing current branch in header and sidebar sessions.
pub struct GitBranchIcon;

impl GitBranchIcon {
    pub const GLYPH: &'static str = "\u{2387}";
    pub const CHAR: char = '\u{2387}';

    pub fn format_branch(branch: &str) -> String {
        format!("{} {branch}", Self::GLYPH)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_git_branch_icon() {
        assert_eq!(GitBranchIcon::GLYPH, "\u{2387}");
        assert_eq!(GitBranchIcon::format_branch("main"), "⎇ main");
    }
}
