// INFO: Folder glyphs representing directories in file trees.
pub struct FolderIcon;

impl FolderIcon {
    pub const GLYPH: &'static str = "\u{25A0}";
    pub const FOLDER_EMOJI: &'static str = "\u{1F4C1}";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_folder_icon() {
        assert_eq!(FolderIcon::GLYPH, "\u{25A0}");
        assert_eq!(FolderIcon::FOLDER_EMOJI, "\u{1F4C1}");
    }
}
