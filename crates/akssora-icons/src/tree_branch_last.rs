// INFO: Box drawing branch corner glyph representing the final tree child connection.
pub struct TreeBranchLastIcon;

impl TreeBranchLastIcon {
    pub const GLYPH: &'static str = "\u{2514}\u{2500} ";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tree_branch_last_icon() {
        assert_eq!(TreeBranchLastIcon::GLYPH, "└─ ");
    }
}
