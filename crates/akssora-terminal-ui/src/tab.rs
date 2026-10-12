// INFO: Supported tab resource types within Akssora.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TabKind {
    Terminal,
    Editor { path: String },
    ApiTesting { workspace: String },
    PluginManagement,
    Settings,
}

impl TabKind {
    pub fn icon(&self) -> char {
        match self {
            Self::Terminal => '>',
            Self::Editor { .. } => '\u{2261}',
            Self::ApiTesting { .. } => '\u{21C4}',
            Self::PluginManagement => '\u{29C9}',
            Self::Settings => '\u{2699}',
        }
    }
}

// INFO: Tab model representing an individual session or resource.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Tab {
    pub id: u64,
    pub title: String,
    pub kind: TabKind,
    pub custom_name: bool,
    pub first_command: Option<String>,
}

impl Tab {
    pub fn new_terminal(id: u64, title: impl Into<String>) -> Self {
        Self {
            id,
            title: title.into(),
            kind: TabKind::Terminal,
            custom_name: false,
            first_command: None,
        }
    }

    pub fn new_editor(id: u64, path: impl Into<String>, title: impl Into<String>) -> Self {
        Self {
            id,
            title: title.into(),
            kind: TabKind::Editor { path: path.into() },
            custom_name: false,
            first_command: None,
        }
    }

    pub fn new_api(id: u64, workspace: impl Into<String>, title: impl Into<String>) -> Self {
        Self {
            id,
            title: title.into(),
            kind: TabKind::ApiTesting {
                workspace: workspace.into(),
            },
            custom_name: false,
            first_command: None,
        }
    }

    pub fn new_plugin_management(id: u64) -> Self {
        Self {
            id,
            title: "Plugin Management".to_string(),
            kind: TabKind::PluginManagement,
            custom_name: true,
            first_command: None,
        }
    }

    pub fn new_settings(id: u64) -> Self {
        Self {
            id,
            title: "Settings".to_string(),
            kind: TabKind::Settings,
            custom_name: true,
            first_command: None,
        }
    }

    pub fn display_name(&self) -> &str {
        &self.title
    }
}

// INFO: Tab manager managing the ordered list of tabs, active index, and lifecycle events.
#[derive(Clone, Debug, Default)]
pub struct TabManager {
    tabs: Vec<Tab>,
    active_idx: usize,
    next_id: u64,
}

impl TabManager {
    pub fn new() -> Self {
        Self {
            tabs: Vec::new(),
            active_idx: 0,
            next_id: 1,
        }
    }

    pub fn tabs(&self) -> &[Tab] {
        &self.tabs
    }

    pub fn active_index(&self) -> usize {
        self.active_idx
    }

    pub fn active_tab(&self) -> Option<&Tab> {
        self.tabs.get(self.active_idx)
    }

    pub fn active_id(&self) -> Option<u64> {
        self.active_tab().map(|t| t.id)
    }

    pub fn is_empty(&self) -> bool {
        self.tabs.is_empty()
    }

    pub fn len(&self) -> usize {
        self.tabs.len()
    }

    pub fn add_tab(&mut self, title: impl Into<String>, kind: TabKind) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        let is_custom = matches!(kind, TabKind::Settings | TabKind::PluginManagement);
        let tab = Tab {
            id,
            title: title.into(),
            kind,
            custom_name: is_custom,
            first_command: None,
        };
        self.tabs.push(tab);
        self.active_idx = self.tabs.len() - 1;
        id
    }

    pub fn select(&mut self, idx: usize) -> bool {
        if idx < self.tabs.len() {
            self.active_idx = idx;
            true
        } else {
            false
        }
    }

    pub fn select_by_id(&mut self, id: u64) -> bool {
        if let Some(pos) = self.tabs.iter().position(|t| t.id == id) {
            self.active_idx = pos;
            true
        } else {
            false
        }
    }

    pub fn next_tab(&mut self) {
        if !self.tabs.is_empty() {
            self.active_idx = (self.active_idx + 1) % self.tabs.len();
        }
    }

    pub fn prev_tab(&mut self) {
        if !self.tabs.is_empty() {
            self.active_idx = (self.active_idx + self.tabs.len() - 1) % self.tabs.len();
        }
    }

    pub fn rename(&mut self, id: u64, new_title: impl Into<String>) -> bool {
        self.set_custom_title(id, new_title)
    }

    pub fn set_custom_title(&mut self, id: u64, new_title: impl Into<String>) -> bool {
        if let Some(tab) = self.tabs.iter_mut().find(|t| t.id == id) {
            tab.title = new_title.into();
            tab.custom_name = true;
            true
        } else {
            false
        }
    }

    pub fn set_auto_title(&mut self, id: u64, command: &str) -> bool {
        if let Some(tab) = self.tabs.iter_mut().find(|t| t.id == id)
            && !tab.custom_name
            && tab.first_command.is_none()
        {
            let cmd = command.trim();
            if !cmd.is_empty() {
                tab.title = cmd.to_string();
                tab.first_command = Some(cmd.to_string());
                return true;
            }
        }
        false
    }

    // INFO: Updates tab identifier during split consolidation to preserve session tracking.
    pub fn update_tab_id(&mut self, old_id: u64, new_id: u64) -> bool {
        if let Some(tab) = self.tabs.iter_mut().find(|t| t.id == old_id) {
            tab.id = new_id;
            true
        } else {
            false
        }
    }

    pub fn close(&mut self, idx: usize) -> Option<Tab> {
        if idx >= self.tabs.len() {
            return None;
        }
        let removed = self.tabs.remove(idx);
        if self.tabs.is_empty() {
            self.active_idx = 0;
        } else if self.active_idx >= self.tabs.len() {
            self.active_idx = self.tabs.len() - 1;
        }
        Some(removed)
    }

    pub fn close_by_id(&mut self, id: u64) -> Option<Tab> {
        let pos = self.tabs.iter().position(|t| t.id == id)?;
        self.close(pos)
    }

    pub fn clear(&mut self) {
        self.tabs.clear();
        self.active_idx = 0;
    }
}
