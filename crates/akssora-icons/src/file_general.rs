// INFO: Squared minus glyph representing generic fallback files.
pub struct FileGeneralIcon;

impl FileGeneralIcon {
    pub const GLYPH: &'static str = "\u{229F}";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_general_icon() {
        assert_eq!(FileGeneralIcon::GLYPH, "\u{229F}");
    }
}
