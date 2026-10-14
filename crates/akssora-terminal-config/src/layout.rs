// INFO: Tab layout placement preference for the Akssora user interface.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum TabLayoutPreference {
    Sidebar,
    #[default]
    TitleBar,
}

impl TabLayoutPreference {
    #[allow(non_upper_case_globals)]
    pub const TitleBarOnly: Self = Self::TitleBar;
    #[allow(non_upper_case_globals)]
    pub const SidebarOnly: Self = Self::Sidebar;

    pub fn includes_sidebar(&self) -> bool {
        matches!(self, Self::Sidebar)
    }

    pub fn includes_title_bar(&self) -> bool {
        matches!(self, Self::TitleBar)
    }

    pub fn show_title_bar(&self) -> bool {
        self.includes_title_bar()
    }

    pub fn show_sidebar(&self) -> bool {
        self.includes_sidebar()
    }

    pub fn show_sidebar_toggle(&self) -> bool {
        self.includes_sidebar()
    }

    pub fn next(&self) -> Self {
        match self {
            Self::TitleBar => Self::Sidebar,
            Self::Sidebar => Self::TitleBar,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::Sidebar => "Sidebar",
            Self::TitleBar => "Title Bar",
        }
    }

    pub fn description(&self) -> &'static str {
        self.label()
    }

    pub fn parse(s: &str) -> Self {
        match s.trim().to_ascii_lowercase().as_str() {
            "sidebar" | "sidebaronly" => Self::Sidebar,
            _ => Self::TitleBar,
        }
    }
}
