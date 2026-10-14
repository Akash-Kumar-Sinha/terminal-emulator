use std::collections::HashMap;

// INFO: Vim modal editing engine supporting Normal, Insert, Visual, VisualLine, Replace, and Command modes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VimMode {
    Normal,
    Insert,
    Visual,
    VisualLine,
    VisualBlock,
    Replace,
    Command,
}

impl VimMode {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Normal => "NORMAL",
            Self::Insert => "INSERT",
            Self::Visual => "VISUAL",
            Self::VisualLine => "V-LINE",
            Self::VisualBlock => "V-BLOCK",
            Self::Replace => "REPLACE",
            Self::Command => "COMMAND",
        }
    }
}

// INFO: In-line find character motion metadata for repeating via ';' or reverse repeating via ','.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FindCharMotion {
    pub target: char,
    pub forward: bool,
    pub till: bool, // true for 't' / 'T', false for 'f' / 'F'
}

// INFO: Rich Vim modal engine handling operators, counts, text objects, named registers, and showcmd status.
#[derive(Clone, Debug)]
pub struct VimEngine {
    pub mode: VimMode,
    pub register: String,
    pub registers: HashMap<char, (String, bool)>, // (content, linewise)
    pub is_linewise: bool,
    pub command_line: String,
    pub status_message: Option<String>,
    pub pending_operator: Option<char>,
    pub count_prefix: Option<usize>,
    pub showcmd: String,
    pub last_find_motion: Option<FindCharMotion>,
    pub active_register_target: Option<char>,
    pub last_action_for_dot: Option<String>,
    pub sync_system_clipboard: bool,
    pub show_status_bar: bool,
}

impl Default for VimEngine {
    fn default() -> Self {
        Self {
            mode: VimMode::Normal,
            register: String::new(),
            registers: HashMap::new(),
            is_linewise: false,
            command_line: String::new(),
            status_message: None,
            pending_operator: None,
            count_prefix: None,
            showcmd: String::new(),
            last_find_motion: None,
            active_register_target: None,
            last_action_for_dot: None,
            sync_system_clipboard: false,
            show_status_bar: true,
        }
    }
}

impl VimEngine {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_mode(&mut self, mode: VimMode) {
        self.mode = mode;
        self.pending_operator = None;
        self.count_prefix = None;
        self.showcmd.clear();
        self.active_register_target = None;
        if mode != VimMode::Command {
            self.command_line.clear();
        }
    }

    pub fn clear_operator_state(&mut self) {
        self.pending_operator = None;
        self.count_prefix = None;
        self.showcmd.clear();
        self.active_register_target = None;
    }

    pub fn effective_count(&self) -> usize {
        self.count_prefix.unwrap_or(1).max(1)
    }

    pub fn push_count_digit(&mut self, digit: usize) {
        let curr = self.count_prefix.unwrap_or(0);
        let next = curr * 10 + digit;
        self.count_prefix = Some(next);
        self.showcmd = format!("{next}");
    }

    pub fn set_register_content(&mut self, content: String, linewise: bool) {
        let target_reg = self.active_register_target.take();
        if let Some(r) = target_reg {
            self.registers
                .insert(r.to_ascii_lowercase(), (content.clone(), linewise));
        }
        self.register = content;
        self.is_linewise = linewise;
    }

    pub fn get_register_content(&self) -> (String, bool) {
        if let Some(r) = self.active_register_target
            && let Some((content, linewise)) = self.registers.get(&r.to_ascii_lowercase())
        {
            return (content.clone(), *linewise);
        }
        (self.register.clone(), self.is_linewise)
    }

    pub fn status_text(&self) -> String {
        if let Some(msg) = &self.status_message {
            msg.clone()
        } else if self.mode == VimMode::Command {
            format!(":{}", self.command_line)
        } else {
            let mode_str = format!("-- {} --", self.mode.label());
            if !self.showcmd.is_empty() {
                format!("{mode_str}  {}", self.showcmd)
            } else {
                mode_str
            }
        }
    }
}
