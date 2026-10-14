// INFO: Window box glyph representing Image files (.png, .jpg, .jpeg, .svg, .gif).
pub struct FileImageIcon;

impl FileImageIcon {
    pub const GLYPH: &'static str = "\u{25A3}";

    pub fn matches(lower_name: &str) -> bool {
        lower_name.ends_with(".png")
            || lower_name.ends_with(".jpg")
            || lower_name.ends_with(".jpeg")
            || lower_name.ends_with(".svg")
            || lower_name.ends_with(".gif")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_image_icon() {
        assert_eq!(FileImageIcon::GLYPH, "\u{25A3}");
        assert!(FileImageIcon::matches("photo.png"));
    }
}
