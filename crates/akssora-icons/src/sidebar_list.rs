// INFO: Trigram / horizontal list lines glyph representing Flat List view mode.
pub struct SidebarListIcon;

impl SidebarListIcon {
    pub const GLYPH: &'static str = "\u{2630}";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sidebar_list_icon() {
        assert_eq!(SidebarListIcon::GLYPH, "\u{2630}");
    }
}
