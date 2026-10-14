// INFO: Search magnifying glass icon glyph for quick search and header.
pub struct SearchIcon;

impl SearchIcon {
    pub const GLYPH: &'static str = "\u{2315}";
    pub const HEADER_LABEL: &'static str = "\u{2315}  Search files";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_icon() {
        assert_eq!(SearchIcon::GLYPH, "\u{2315}");
    }
}
