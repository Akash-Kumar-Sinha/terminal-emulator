// INFO: Plus icon glyph representing Add, Stage, or New Tab.
pub struct AddIcon;

impl AddIcon {
    pub const GLYPH: &'static str = "+";
    pub const CHAR: char = '+';
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_icon() {
        assert_eq!(AddIcon::GLYPH, "+");
        assert_eq!(AddIcon::CHAR, '+');
    }
}
