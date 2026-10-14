// INFO: Plus glyph representing Git Added status.
pub struct GitStatusAddedIcon;

impl GitStatusAddedIcon {
    pub const GLYPH: &'static str = "+";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_git_status_added_icon() {
        assert_eq!(GitStatusAddedIcon::GLYPH, "+");
    }
}
