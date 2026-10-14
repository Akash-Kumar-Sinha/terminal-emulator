// INFO: Pin icon glyph representing pinned file tabs.
pub struct PinIcon;

impl PinIcon {
    pub const GLYPH: &'static str = "\u{22B9}";
    pub const GLYPH_SPACE: &'static str = "\u{22B9} ";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pin_icon() {
        assert_eq!(PinIcon::GLYPH, "\u{22B9}");
    }
}
