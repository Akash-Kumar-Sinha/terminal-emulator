pub mod formatter;
pub mod parser;

pub use formatter::{FormatterConfig, FormatterError, FormatterTool, MarkdownFormatter};
pub use parser::{
    InlineSpan, MarkdownBlock, ParsedMarkdown, TableAlignment, is_safe_url, parse_markdown,
    resolve_relative_path, sanitize_html,
};

// INFO: Display view modes for Markdown documents in the integrated editor.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum MarkdownViewMode {
    #[default]
    Source,
    Split,
    Preview,
}

impl MarkdownViewMode {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Source => "Source",
            Self::Split => "Split Preview",
            Self::Preview => "Preview",
        }
    }

    pub fn is_preview_active(&self) -> bool {
        matches!(self, Self::Split | Self::Preview)
    }

    pub fn toggle_preview(&self) -> Self {
        match self {
            Self::Source => Self::Preview,
            Self::Preview => Self::Source,
            Self::Split => Self::Preview,
        }
    }

    pub fn toggle_split(&self) -> Self {
        match self {
            Self::Split => Self::Source,
            _ => Self::Split,
        }
    }
}
