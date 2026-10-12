use std::path::{Path, PathBuf};

// INFO: Serialized session state for application restoration across restarts.
#[derive(Clone, Debug, PartialEq)]
pub struct PersistedPane {
    pub pane_id: u64,
    pub tab_id: u64,
    pub name: String,
    pub custom_name: bool,
    pub cwd: PathBuf,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PersistedState {
    pub panes: Vec<PersistedPane>,
    pub active_tab_id: Option<u64>,
}

impl PersistedState {
    pub fn default_path() -> PathBuf {
        if let Ok(dir) = std::env::var("AKSSORA_CONFIG_DIR") {
            PathBuf::from(dir).join("session.json")
        } else if let Ok(home) = std::env::var("HOME") {
            PathBuf::from(home)
                .join(".config")
                .join("akssora")
                .join("session.json")
        } else {
            PathBuf::from(".config")
                .join("akssora")
                .join("session.json")
        }
    }

    pub fn save_to_file(&self, path: &Path) -> std::io::Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let mut lines = Vec::new();
        lines.push("{".to_string());
        lines.push(format!(
            "  \"active_tab_id\": {},",
            self.active_tab_id.unwrap_or(0)
        ));
        lines.push("  \"panes\": [".to_string());

        for (i, pane) in self.panes.iter().enumerate() {
            let comma = if i + 1 < self.panes.len() { "," } else { "" };
            let cwd_escaped = pane.cwd.display().to_string().replace('\\', "\\\\");
            let name_escaped = pane.name.replace('"', "\\\"");
            lines.push(format!(
                "    {{\"pane_id\": {}, \"tab_id\": {}, \"name\": \"{}\", \"custom_name\": {}, \"cwd\": \"{}\"}}{}",
                pane.pane_id, pane.tab_id, name_escaped, pane.custom_name, cwd_escaped, comma
            ));
        }

        lines.push("  ]".to_string());
        lines.push("}".to_string());

        let json = lines.join("\n");
        std::fs::write(path, json)?;
        Ok(())
    }

    pub fn load_from_file(path: &Path) -> Option<Self> {
        let content = std::fs::read_to_string(path).ok()?;
        let mut active_tab_id = None;
        let mut panes = Vec::new();

        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("\"active_tab_id\":") {
                if let Some(val) = trimmed.split(':').nth(1) {
                    let cleaned = val.trim().trim_matches(|c| c == ',' || c == ' ');
                    if let Ok(id) = cleaned.parse::<u64>()
                        && id > 0
                    {
                        active_tab_id = Some(id);
                    }
                }
            } else if trimmed.starts_with("{\"pane_id\":")
                && let Some(pane) = parse_persisted_pane(trimmed)
            {
                panes.push(pane);
            }
        }

        if panes.is_empty() {
            None
        } else {
            Some(Self {
                panes,
                active_tab_id,
            })
        }
    }
}

fn parse_persisted_pane(line: &str) -> Option<PersistedPane> {
    let pane_id = extract_field(line, "\"pane_id\":")?.parse::<u64>().ok()?;
    let tab_id = extract_field(line, "\"tab_id\":")?.parse::<u64>().ok()?;
    let name = extract_string_field(line, "\"name\":")?;
    let custom_name = extract_field(line, "\"custom_name\":")? == "true";
    let cwd_str = extract_string_field(line, "\"cwd\":")?;

    Some(PersistedPane {
        pane_id,
        tab_id,
        name,
        custom_name,
        cwd: PathBuf::from(cwd_str),
    })
}

fn extract_field<'a>(line: &'a str, key: &str) -> Option<&'a str> {
    let idx = line.find(key)?;
    let rem = &line[idx + key.len()..];
    let end = rem.find([',', '}'])?;
    Some(rem[..end].trim())
}

fn extract_string_field(line: &str, key: &str) -> Option<String> {
    let idx = line.find(key)?;
    let rem = &line[idx + key.len()..];
    let first_quote = rem.find('"')?;
    let rem_after_quote = &rem[first_quote + 1..];
    let second_quote = rem_after_quote.find('"')?;
    Some(rem_after_quote[..second_quote].to_string())
}
