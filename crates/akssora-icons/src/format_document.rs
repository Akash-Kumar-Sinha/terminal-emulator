// INFO: Icon and action identifier for document formatting.
pub struct FormatDocumentIcon;

impl FormatDocumentIcon {
    pub const LABEL: &'static str = "Format Document";
    pub const GLYPH: &'static str = "\u{2261}";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_document_icon() {
        assert_eq!(FormatDocumentIcon::LABEL, "Format Document");
    }
}
