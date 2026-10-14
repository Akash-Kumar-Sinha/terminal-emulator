// INFO: Counterclockwise circular arrow icon representing the Discard action.
pub struct ArrowUndoIcon;

impl ArrowUndoIcon {
    pub const GLYPH: &'static str = "\u{21BA}";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arrow_undo_icon() {
        assert_eq!(ArrowUndoIcon::GLYPH, "\u{21BA}");
    }
}
