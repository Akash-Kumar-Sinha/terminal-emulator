// INFO: Bidirectional arrow glyph representing an API Testing session tab.
pub struct ApiTestingSessionIcon;

impl ApiTestingSessionIcon {
    pub const CHAR: char = '\u{21C4}';
    pub const GLYPH: &'static str = "\u{21C4}";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_testing_session_icon() {
        assert_eq!(ApiTestingSessionIcon::CHAR, '\u{21C4}');
    }
}
