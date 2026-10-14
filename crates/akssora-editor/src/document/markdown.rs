use std::path::Path;

use crate::buffer::TextBuffer;
use crate::markdown::{
    FormatterError, FormatterTool, MarkdownFormatter, ParsedMarkdown, parse_markdown,
};

use super::Document;

impl Document {
    // ── Markdown Formatting & Preview ─────────────────────────────────────────

    pub fn is_markdown(&self) -> bool {
        if let Some(path) = &self.path
            && let Some(ext) = path.extension()
        {
            let e = ext.to_string_lossy().to_lowercase();
            if e == "md" || e == "markdown" {
                return true;
            }
        }
        let t = self.title().to_lowercase();
        t.ends_with(".md") || t.ends_with(".markdown")
    }

    pub fn invalidate_markdown_cache(&self) {
        self.cached_parsed_markdown.borrow_mut().take();
        self.cached_markdown_hash.set(0);
    }

    pub fn format_markdown(
        &mut self,
        tool: FormatterTool,
        workspace_root: Option<&Path>,
    ) -> Result<bool, FormatterError> {
        let content = self.buffer.to_string_content();
        let formatted =
            MarkdownFormatter::format(&content, tool, workspace_root, self.path.as_deref())?;

        if formatted == content {
            return Ok(false);
        }

        self.record_undo();
        self.buffer = TextBuffer::from_text(&formatted);
        self.cached_max_line_len.set(None);
        self.invalidate_markdown_cache();
        self.clamp_cursor();
        Ok(true)
    }

    pub fn parse_markdown_preview(&self, workspace_root: Option<&Path>) -> ParsedMarkdown {
        let content = self.buffer.to_string_content();
        let mut hash: u64 = 0xcbf29ce484222325;
        for byte in content.as_bytes() {
            hash ^= *byte as u64;
            hash = hash.wrapping_mul(0x100000001b3);
        }

        if self.cached_markdown_hash.get() == hash
            && let Some(cached) = self.cached_parsed_markdown.borrow().as_ref()
        {
            return cached.clone();
        }

        let parsed = parse_markdown(&content, self.path.as_deref(), workspace_root);
        *self.cached_parsed_markdown.borrow_mut() = Some(parsed.clone());
        self.cached_markdown_hash.set(hash);
        parsed
    }
}
