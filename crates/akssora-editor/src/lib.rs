pub mod buffer;
pub mod document;
pub mod markdown;
pub mod matching_brackets;
pub mod paragraph_iterator;
pub mod search;
pub mod symbols;
pub mod vim;
pub mod word_iterator;
pub mod wrap;

pub use buffer::TextBuffer;
pub use document::{Document, GutterActionKind};
pub use markdown::{
    FormatterConfig, FormatterError, FormatterTool, InlineSpan, MarkdownBlock, MarkdownFormatter,
    MarkdownViewMode, ParsedMarkdown, TableAlignment, is_safe_url, parse_markdown,
    resolve_relative_path, sanitize_html,
};
pub use matching_brackets::find_matching_bracket;
pub use paragraph_iterator::{next_paragraph, prev_paragraph};
pub use search::{SearchMatch, SearchModel, SearchQuery};
pub use symbols::{autocomplete_pair, is_closing_symbol, matching_closer, matching_opener};
pub use vim::{FindCharMotion, VimEngine, VimMode};
pub use word_iterator::{
    WordType, next_word_end, next_word_start, prev_word_start, word_text_object_range,
};
pub use wrap::{VisualRow, WrapLayout, wrap_line};
