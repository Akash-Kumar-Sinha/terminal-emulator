// INFO: Akssora application identity wordmark and branding emblem.
pub struct AppLogoIcon;

impl AppLogoIcon {
    pub const WORDMARK: &'static str = "AKSSORA";
    pub const SHORT_NAME: &'static str = "Akssora";
    pub const TERMINAL_PROMPT: char = '>';
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_logo() {
        assert_eq!(AppLogoIcon::WORDMARK, "AKSSORA");
        assert_eq!(AppLogoIcon::SHORT_NAME, "Akssora");
    }
}
