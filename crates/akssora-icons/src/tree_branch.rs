// INFO: Box drawing branch glyph representing tree child connections.
pub struct TreeBranchIcon;

impl TreeBranchIcon {
    pub const GLYPH: &'static str = "\u{251C}\u{2500} ";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tree_branch_icon() {
        assert_eq!(TreeBranchIcon::GLYPH, "├─ ");
    }
}
