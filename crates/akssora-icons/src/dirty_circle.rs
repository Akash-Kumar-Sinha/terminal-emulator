// INFO: Bullet / filled circle glyph representing unsaved file changes.
pub struct DirtyCircleIcon;

impl DirtyCircleIcon {
    pub const GLYPH: &'static str = "\u{2022}";
    pub const GLYPH_SPACE: &'static str = "\u{2022} ";
    pub const CHAR: char = '\u{2022}';
    pub const CIRCLE_GLYPH: &'static str = "●";

    pub fn format_dirty_tab(title: &str) -> String {
        format!("{} {title}", Self::GLYPH)
    }

    pub fn format_dirty_title(title: &str) -> String {
        format!("{} {title}", Self::CIRCLE_GLYPH)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dirty_circle_icon() {
        assert_eq!(DirtyCircleIcon::GLYPH, "\u{2022}");
        assert_eq!(DirtyCircleIcon::CIRCLE_GLYPH, "●");
    }
}
