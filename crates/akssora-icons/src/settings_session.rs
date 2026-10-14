// INFO: Gear glyph representing a Settings session tab.
pub struct SettingsSessionIcon;

impl SettingsSessionIcon {
    pub const CHAR: char = '\u{2699}';
    pub const GLYPH: &'static str = "\u{2699}";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_settings_session_icon() {
        assert_eq!(SettingsSessionIcon::CHAR, '\u{2699}');
    }
}
