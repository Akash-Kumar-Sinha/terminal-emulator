use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

// INFO: Formatter tool selection.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub enum FormatterTool {
    #[default]
    Auto,
    Builtin,
    Dprint,
    Prettier,
}

impl FormatterTool {
    pub fn parse(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "dprint" => Self::Dprint,
            "prettier" => Self::Prettier,
            "builtin" => Self::Builtin,
            _ => Self::Auto,
        }
    }
}

// INFO: Formatter configuration options.
#[derive(Clone, Debug)]
pub struct FormatterConfig {
    pub tool: FormatterTool,
    pub tab_size: usize,
    pub format_on_save: bool,
}

impl Default for FormatterConfig {
    fn default() -> Self {
        Self {
            tool: FormatterTool::Auto,
            tab_size: 4,
            format_on_save: false,
        }
    }
}

// INFO: Error resulting from formatting failures.
#[derive(Debug)]
pub enum FormatterError {
    ExternalFailed(String),
    Io(std::io::Error),
}

impl std::fmt::Display for FormatterError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ExternalFailed(msg) => write!(f, "Formatter error: {msg}"),
            Self::Io(err) => write!(f, "I/O error: {err}"),
        }
    }
}

impl std::error::Error for FormatterError {}

impl From<std::io::Error> for FormatterError {
    fn from(err: std::io::Error) -> Self {
        Self::Io(err)
    }
}

// INFO: Inspects workspace root directory to locate external configuration files for dprint or prettier.
pub fn detect_workspace_formatter_config(
    workspace_root: &Path,
) -> Option<(FormatterTool, PathBuf)> {
    let dprint_candidates = ["dprint.json", ".dprint.json"];
    for name in &dprint_candidates {
        let p = workspace_root.join(name);
        if p.exists() {
            return Some((FormatterTool::Dprint, p));
        }
    }

    let prettier_candidates = [
        ".prettierrc",
        ".prettierrc.json",
        ".prettierrc.yml",
        ".prettierrc.yaml",
        ".prettierrc.js",
        "prettier.config.js",
    ];
    for name in &prettier_candidates {
        let p = workspace_root.join(name);
        if p.exists() {
            return Some((FormatterTool::Prettier, p));
        }
    }

    None
}

// INFO: Executes an external formatter binary, piping content via stdin and receiving formatted output.
pub fn run_external_formatter(
    cmd_name: &str,
    args: &[&str],
    input: &str,
    current_dir: Option<&Path>,
) -> Result<String, FormatterError> {
    let mut cmd = Command::new(cmd_name);
    cmd.args(args);
    if let Some(dir) = current_dir {
        cmd.current_dir(dir);
    }
    cmd.stdin(Stdio::piped());
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());

    let mut child = cmd.spawn()?;
    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(input.as_bytes())?;
    }

    let output = child.wait_with_output()?;
    if output.status.success() {
        let formatted = String::from_utf8_lossy(&output.stdout).to_string();
        Ok(formatted)
    } else {
        let err_msg = String::from_utf8_lossy(&output.stderr).to_string();
        Err(FormatterError::ExternalFailed(err_msg))
    }
}

// INFO: High-level Markdown formatter coordinating external tools and pure-Rust builtin engine.
pub struct MarkdownFormatter;

impl MarkdownFormatter {
    // INFO: Formats Markdown content with fallback to built-in rules if external tools are unavailable.
    pub fn format(
        content: &str,
        tool: FormatterTool,
        workspace_root: Option<&Path>,
        file_path: Option<&Path>,
    ) -> Result<String, FormatterError> {
        let effective_tool = match tool {
            FormatterTool::Auto => {
                if let Some(root) = workspace_root
                    && let Some((detected, _)) = detect_workspace_formatter_config(root)
                {
                    detected
                } else {
                    FormatterTool::Builtin
                }
            }
            other => other,
        };

        match effective_tool {
            FormatterTool::Dprint => {
                // Try dprint fmt --stdin md
                let res = run_external_formatter(
                    "dprint",
                    &["fmt", "--stdin", "md"],
                    content,
                    workspace_root,
                );
                match res {
                    Ok(formatted) => Ok(formatted),
                    Err(_) => {
                        // Fallback gracefully to builtin formatting if dprint binary is missing
                        Ok(Self::format_builtin(content))
                    }
                }
            }
            FormatterTool::Prettier => {
                let file_arg = file_path
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or_else(|| "document.md".to_string());
                let res = run_external_formatter(
                    "npx",
                    &["prettier", "--stdin-filepath", &file_arg],
                    content,
                    workspace_root,
                );
                match res {
                    Ok(formatted) => Ok(formatted),
                    Err(_) => Ok(Self::format_builtin(content)),
                }
            }
            FormatterTool::Builtin | FormatterTool::Auto => Ok(Self::format_builtin(content)),
        }
    }

    // INFO: Native pure-Rust Markdown formatter normalizing headings, lists, tables, blockquotes, and spacing while preserving code fences.
    pub fn format_builtin(content: &str) -> String {
        if content.is_empty() {
            return String::new();
        }

        let lines: Vec<&str> = content.lines().collect();
        let mut formatted_lines: Vec<String> = Vec::with_capacity(lines.len());
        let mut in_code_block = false;
        let mut code_fence_char = '`';
        let mut code_fence_len = 0;
        let mut in_table = false;
        let mut table_rows: Vec<String> = Vec::new();

        let flush_table = |rows: &mut Vec<String>, out: &mut Vec<String>| {
            if rows.is_empty() {
                return;
            }
            let aligned = align_table(rows);
            out.extend(aligned);
            rows.clear();
        };

        for line in lines {
            let trimmed_start = line.trim_start();

            // Check code block fence transitions
            if trimmed_start.starts_with("```") || trimmed_start.starts_with("~~~") {
                let fence_ch = trimmed_start.chars().next().unwrap();
                let fence_cnt = trimmed_start.chars().take_while(|&c| c == fence_ch).count();

                if in_code_block {
                    if fence_ch == code_fence_char && fence_cnt >= code_fence_len {
                        in_code_block = false;
                        formatted_lines.push(line.to_string());
                        continue;
                    }
                } else {
                    flush_table(&mut table_rows, &mut formatted_lines);
                    in_table = false;
                    in_code_block = true;
                    code_fence_char = fence_ch;
                    code_fence_len = fence_cnt;
                    formatted_lines.push(line.to_string());
                    continue;
                }
            }

            // Inside code block: preserve lines completely untouched
            if in_code_block {
                formatted_lines.push(line.to_string());
                continue;
            }

            // Check table rows
            if trimmed_start.starts_with('|')
                || (trimmed_start.contains('|') && !trimmed_start.starts_with('#'))
            {
                // Potential table line
                if is_table_line(trimmed_start) {
                    in_table = true;
                    table_rows.push(line.to_string());
                    continue;
                }
            }

            // If we were in a table and this line is not part of the table, flush it
            if in_table {
                flush_table(&mut table_rows, &mut formatted_lines);
                in_table = false;
            }

            // Heading formatting: ensures space after # hashes
            if trimmed_start.starts_with('#') {
                let hash_count = trimmed_start.chars().take_while(|&c| c == '#').count();
                if hash_count <= 6 {
                    let rest = &trimmed_start[hash_count..];
                    if !rest.starts_with(' ') && !rest.is_empty() {
                        let hashes = "#".repeat(hash_count);
                        formatted_lines.push(format!("{hashes} {rest}"));
                        continue;
                    }
                }
            }

            // Blockquote formatting: ensures space after >
            if let Some(stripped) = trimmed_start.strip_prefix('>') {
                let rest = stripped.trim_start();
                let indent_len = line.len() - trimmed_start.len();
                let indent = &line[..indent_len];
                if rest.is_empty() {
                    formatted_lines.push(format!("{indent}>"));
                } else {
                    formatted_lines.push(format!("{indent}> {rest}"));
                }
                continue;
            }

            // List item normalization (task lists & bullet lists)
            if let Some(formatted_list) = format_list_item(line) {
                formatted_lines.push(formatted_list);
                continue;
            }

            // Clean trailing spaces unless it's a markdown hard break (2 spaces at end)
            let trimmed_end = line.trim_end_matches([' ', '\t']);
            if line.ends_with("  ") && !trimmed_end.is_empty() {
                formatted_lines.push(format!("{trimmed_end}  "));
            } else {
                formatted_lines.push(trimmed_end.to_string());
            }
        }

        if in_table {
            flush_table(&mut table_rows, &mut formatted_lines);
        }

        // Collapse excessive blank lines (max 1 consecutive blank line)
        let mut final_lines: Vec<String> = Vec::with_capacity(formatted_lines.len());
        let mut last_was_empty = false;

        for l in formatted_lines {
            let is_empty = l.trim().is_empty();
            if is_empty {
                if !last_was_empty && !final_lines.is_empty() {
                    final_lines.push(String::new());
                    last_was_empty = true;
                }
            } else {
                final_lines.push(l);
                last_was_empty = false;
            }
        }

        // Remove trailing blank lines
        while final_lines
            .last()
            .map(|s| s.trim().is_empty())
            .unwrap_or(false)
        {
            final_lines.pop();
        }

        let mut result = final_lines.join("\n");
        if !result.is_empty() {
            result.push('\n');
        }
        result
    }
}

// INFO: Checks if a line resembles a Markdown table row.
fn is_table_line(line: &str) -> bool {
    let t = line.trim();
    if t.starts_with('|') && t.ends_with('|') {
        return true;
    }
    // Also row with multiple pipes
    t.matches('|').count() >= 2
}

// INFO: Aligns Markdown pipe table columns neatly.
fn align_table(rows: &[String]) -> Vec<String> {
    if rows.is_empty() {
        return Vec::new();
    }

    let parsed_rows: Vec<Vec<String>> = rows
        .iter()
        .map(|r| {
            let trimmed = r.trim().trim_matches('|');
            trimmed
                .split('|')
                .map(|cell| cell.trim().to_string())
                .collect()
        })
        .collect();

    let col_count = parsed_rows.iter().map(|r| r.len()).max().unwrap_or(0);
    if col_count == 0 {
        return rows.to_vec();
    }

    let mut col_widths = vec![3usize; col_count];
    for (row_idx, r) in parsed_rows.iter().enumerate() {
        let is_separator =
            row_idx == 1 && r.iter().all(|c| c.chars().all(|ch| ch == '-' || ch == ':'));
        if !is_separator {
            for (col_idx, cell) in r.iter().enumerate() {
                if col_idx < col_count {
                    col_widths[col_idx] = col_widths[col_idx].max(cell.len());
                }
            }
        }
    }

    let mut aligned = Vec::with_capacity(rows.len());
    for (row_idx, row) in parsed_rows.iter().enumerate() {
        let is_separator = row_idx == 1
            && row
                .iter()
                .all(|c| c.chars().all(|ch| ch == '-' || ch == ':'));

        let mut line = String::from("| ");
        for (col_idx, &width) in col_widths.iter().enumerate().take(col_count) {
            let cell = row.get(col_idx).map(|s| s.as_str()).unwrap_or("");
            if is_separator {
                let left_align = cell.starts_with(':');
                let right_align = cell.ends_with(':');
                let dashes = "-".repeat(
                    width
                        .saturating_sub(
                            if left_align { 1 } else { 0 } + if right_align { 1 } else { 0 },
                        )
                        .max(1),
                );
                let sep = match (left_align, right_align) {
                    (true, true) => format!(":{dashes}:"),
                    (true, false) => format!(":{dashes}-"),
                    (false, true) => format!("-{dashes}:"),
                    (false, false) => format!("-{dashes}-"),
                };
                line.push_str(&sep);
            } else {
                let pad = " ".repeat(width.saturating_sub(cell.len()));
                line.push_str(cell);
                line.push_str(&pad);
            }
            if col_idx + 1 < col_count {
                line.push_str(" | ");
            } else {
                line.push_str(" |");
            }
        }
        aligned.push(line);
    }

    aligned
}

// INFO: Normalizes list items, bullet points, and task lists.
fn format_list_item(line: &str) -> Option<String> {
    let indent_len = line.len() - line.trim_start().len();
    let indent = &line[..indent_len];
    let trimmed = line.trim_start();

    // Task list: - [ ] or - [x]
    if trimmed.starts_with("- [ ]")
        || trimmed.starts_with("- [x]")
        || trimmed.starts_with("- [X]")
        || trimmed.starts_with("* [ ]")
        || trimmed.starts_with("* [x]")
        || trimmed.starts_with("* [X]")
    {
        let is_checked = trimmed.contains("[x]") || trimmed.contains("[X]");
        let check_str = if is_checked { "[x]" } else { "[ ]" };
        let rest = trimmed[5..].trim_start();
        return Some(format!("{indent}- {check_str} {rest}"));
    }

    // Unordered bullet item: -, *, +
    if trimmed.starts_with("- ") || trimmed.starts_with("* ") || trimmed.starts_with("+ ") {
        let rest = trimmed[2..].trim_start();
        return Some(format!("{indent}- {rest}"));
    }

    // Ordered list item: 1. or 1)
    if let Some(dot_idx) = trimmed.find('.') {
        let num_part = &trimmed[..dot_idx];
        if !num_part.is_empty() && num_part.chars().all(|c| c.is_ascii_digit()) {
            let rest = trimmed[dot_idx + 1..].trim_start();
            return Some(format!("{indent}{num_part}. {rest}"));
        }
    }

    None
}
