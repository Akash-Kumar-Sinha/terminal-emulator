// INFO: Triple bar glyph representing an Editor session tab.
pub struct EditorSessionIcon;

impl EditorSessionIcon {
    pub const CHAR: char = '\u{2261}';
    pub const GLYPH: &'static str = "\u{2261}";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_editor_session_icon() {
        assert_eq!(EditorSessionIcon::CHAR, '\u{2261}');
    }
}
