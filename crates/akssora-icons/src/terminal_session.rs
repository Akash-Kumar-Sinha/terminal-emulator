// INFO: Prompt greater-than glyph representing a Terminal session.
pub struct TerminalSessionIcon;

impl TerminalSessionIcon {
    pub const CHAR: char = '>';
    pub const GLYPH: &'static str = ">";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_terminal_session_icon() {
        assert_eq!(TerminalSessionIcon::CHAR, '>');
        assert_eq!(TerminalSessionIcon::GLYPH, ">");
    }
}
