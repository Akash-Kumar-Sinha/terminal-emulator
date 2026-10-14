// INFO: Two joined squares glyph representing a Plugin Management session tab.
pub struct PluginSessionIcon;

impl PluginSessionIcon {
    pub const CHAR: char = '\u{29C9}';
    pub const GLYPH: &'static str = "\u{29C9}";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_session_icon() {
        assert_eq!(PluginSessionIcon::CHAR, '\u{29C9}');
    }
}
