// INFO: Split rectangle glyph representing an expanded sidebar panel.
pub struct PanelExpandedIcon;

impl PanelExpandedIcon {
    pub const GLYPH: &'static str = "\u{25EB}";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_panel_expanded_icon() {
        assert_eq!(PanelExpandedIcon::GLYPH, "\u{25EB}");
    }
}
