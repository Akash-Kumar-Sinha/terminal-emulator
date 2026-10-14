// INFO: Hollow square glyph representing a collapsed sidebar panel.
pub struct PanelCollapsedIcon;

impl PanelCollapsedIcon {
    pub const GLYPH: &'static str = "\u{25A2}";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_panel_collapsed_icon() {
        assert_eq!(PanelCollapsedIcon::GLYPH, "\u{25A2}");
    }
}
