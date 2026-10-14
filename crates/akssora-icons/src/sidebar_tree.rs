// INFO: Trigram / grid glyph representing Tree view mode.
pub struct SidebarTreeIcon;

impl SidebarTreeIcon {
    pub const GLYPH: &'static str = "\u{2637}";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sidebar_tree_icon() {
        assert_eq!(SidebarTreeIcon::GLYPH, "\u{2637}");
    }
}
