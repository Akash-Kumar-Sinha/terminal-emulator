use pulldown_cmark::{Alignment, Event, HeadingLevel, Options, Parser, Tag, TagEnd};
use std::path::{Path, PathBuf};

// INFO: Inline formatted span within a Markdown text block.
#[derive(Clone, Debug, PartialEq)]
pub enum InlineSpan {
    Text(String),
    Bold(String),
    Italic(String),
    BoldItalic(String),
    Code(String),
    Link {
        text: String,
        url: String,
        safe: bool,
    },
    Image {
        alt: String,
        url: String,
        resolved_path: Option<PathBuf>,
        safe: bool,
    },
    Strikethrough(String),
}

// INFO: Table column alignment specification.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TableAlignment {
    Left,
    Center,
    Right,
    None,
}

impl From<Alignment> for TableAlignment {
    fn from(a: Alignment) -> Self {
        match a {
            Alignment::Left => TableAlignment::Left,
            Alignment::Center => TableAlignment::Center,
            Alignment::Right => TableAlignment::Right,
            Alignment::None => TableAlignment::None,
        }
    }
}

// INFO: Structural block element in rendered Markdown document.
#[derive(Clone, Debug, PartialEq)]
pub enum MarkdownBlock {
    Heading {
        level: u8,
        spans: Vec<InlineSpan>,
    },
    Paragraph(Vec<InlineSpan>),
    Blockquote(Vec<MarkdownBlock>),
    List {
        ordered: bool,
        start: u64,
        items: Vec<Vec<InlineSpan>>,
    },
    TaskList {
        items: Vec<(bool, Vec<InlineSpan>)>,
    },
    CodeBlock {
        language: Option<String>,
        code: Vec<String>,
    },
    Table {
        headers: Vec<String>,
        alignments: Vec<TableAlignment>,
        rows: Vec<Vec<String>>,
    },
    ThematicBreak,
    RawHtml(String),
}

// INFO: Parsed Markdown document tree with diagnostics and error tracking.
#[derive(Clone, Debug, Default)]
pub struct ParsedMarkdown {
    pub blocks: Vec<MarkdownBlock>,
    pub error: Option<String>,
    pub word_count: usize,
    pub line_count: usize,
}

// INFO: Checks if a URL contains dangerous executable schemes.
pub fn is_safe_url(url: &str) -> bool {
    let trimmed = url.trim();
    let lower = trimmed.to_lowercase();
    if lower.starts_with("javascript:")
        || lower.starts_with("vbscript:")
        || lower.starts_with("data:")
    {
        return false;
    }
    true
}

// INFO: Sanitizes raw HTML strings to strip dangerous scripts, event handlers, and active content.
pub fn sanitize_html(raw: &str) -> String {
    let lower = raw.to_lowercase();
    if lower.contains("<script")
        || lower.contains("</script>")
        || lower.contains("<iframe")
        || lower.contains("<object")
        || lower.contains("<embed")
        || lower.contains("javascript:")
        || lower.contains("onerror=")
        || lower.contains("onload=")
        || lower.contains("onclick=")
    {
        // Strip out dangerous HTML content
        let mut clean = String::new();
        let mut in_tag = false;
        let mut tag_buf = String::new();

        for ch in raw.chars() {
            if ch == '<' {
                in_tag = true;
                tag_buf.clear();
                tag_buf.push(ch);
            } else if ch == '>' {
                in_tag = false;
                tag_buf.push(ch);
                let tag_lower = tag_buf.to_lowercase();
                if !tag_lower.starts_with("<script")
                    && !tag_lower.starts_with("</script")
                    && !tag_lower.starts_with("<iframe")
                    && !tag_lower.starts_with("<object")
                    && !tag_lower.starts_with("<embed")
                    && !tag_lower.contains("onerror")
                    && !tag_lower.contains("onload")
                    && !tag_lower.contains("onclick")
                {
                    clean.push_str(&tag_buf);
                }
            } else if in_tag {
                tag_buf.push(ch);
            } else {
                clean.push(ch);
            }
        }
        clean
    } else {
        raw.to_string()
    }
}

// INFO: Resolves relative asset links against document file path and workspace root.
pub fn resolve_relative_path(
    base_file: Option<&Path>,
    workspace_root: Option<&Path>,
    target: &str,
) -> Option<PathBuf> {
    if !is_safe_url(target) {
        return None;
    }

    let lower = target.to_lowercase();
    if lower.starts_with("http://")
        || lower.starts_with("https://")
        || lower.starts_with("mailto:")
        || lower.starts_with('#')
    {
        return None;
    }

    let clean = target.split('?').next().unwrap_or(target);
    let clean = clean.split('#').next().unwrap_or(clean);

    let target_path = Path::new(clean);
    if target_path.is_absolute() {
        if let Some(ws) = workspace_root {
            let stripped = clean.trim_start_matches('/');
            return Some(ws.join(stripped));
        }
        return Some(target_path.to_path_buf());
    }

    let base_dir = base_file
        .and_then(|p| p.parent())
        .or(workspace_root)
        .unwrap_or(Path::new("."));

    Some(base_dir.join(target_path))
}

// INFO: Parses Markdown source text into structured document blocks using pulldown-cmark.
pub fn parse_markdown(
    markdown: &str,
    base_file: Option<&Path>,
    workspace_root: Option<&Path>,
) -> ParsedMarkdown {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_TASKLISTS);
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_HEADING_ATTRIBUTES);
    options.insert(Options::ENABLE_FOOTNOTES);

    let parser = Parser::new_ext(markdown, options);

    let mut blocks = Vec::new();
    let mut current_spans: Vec<InlineSpan> = Vec::new();
    let mut bold = false;
    let mut italic = false;
    let mut strikethrough = false;
    let mut current_link: Option<(String, String)> = None; // (url, title)
    let mut current_image: Option<(String, String)> = None; // (url, alt)

    // Code block state
    let mut in_code_block = false;
    let mut code_lang = None;
    let mut code_lines = Vec::new();

    // List state
    let mut in_list = false;
    let mut list_ordered = false;
    let mut list_start = 1;
    let mut list_items: Vec<Vec<InlineSpan>> = Vec::new();
    let mut current_list_item: Vec<InlineSpan> = Vec::new();
    let mut is_task_list = false;
    let mut task_items: Vec<(bool, Vec<InlineSpan>)> = Vec::new();
    let mut current_task_checked = false;

    // Blockquote state
    let mut in_blockquote = false;
    let mut blockquote_blocks = Vec::new();

    // Table state
    let mut in_table = false;
    let mut table_headers: Vec<String> = Vec::new();
    let mut table_alignments: Vec<TableAlignment> = Vec::new();
    let mut table_rows: Vec<Vec<String>> = Vec::new();
    let mut current_table_row: Vec<String> = Vec::new();
    let mut in_table_head = false;
    let mut current_cell_text = String::new();

    for event in parser {
        match event {
            Event::Start(tag) => match tag {
                Tag::Paragraph => {
                    current_spans.clear();
                }
                Tag::Heading { level, .. } => {
                    current_spans.clear();
                    let lvl = match level {
                        HeadingLevel::H1 => 1,
                        HeadingLevel::H2 => 2,
                        HeadingLevel::H3 => 3,
                        HeadingLevel::H4 => 4,
                        HeadingLevel::H5 => 5,
                        HeadingLevel::H6 => 6,
                    };
                    // Tag carries heading level
                    let _ = lvl;
                }
                Tag::BlockQuote(_) => {
                    in_blockquote = true;
                    blockquote_blocks.clear();
                }
                Tag::CodeBlock(kind) => {
                    in_code_block = true;
                    code_lines.clear();
                    code_lang = match kind {
                        pulldown_cmark::CodeBlockKind::Fenced(lang) => {
                            let s = lang.as_ref().trim().to_string();
                            if s.is_empty() { None } else { Some(s) }
                        }
                        pulldown_cmark::CodeBlockKind::Indented => None,
                    };
                }
                Tag::List(start_num) => {
                    in_list = true;
                    list_ordered = start_num.is_some();
                    list_start = start_num.unwrap_or(1);
                    list_items.clear();
                    task_items.clear();
                    is_task_list = false;
                }
                Tag::Item => {
                    current_list_item.clear();
                }
                Tag::Table(aligns) => {
                    in_table = true;
                    table_headers.clear();
                    table_alignments = aligns.into_iter().map(TableAlignment::from).collect();
                    table_rows.clear();
                }
                Tag::TableHead => {
                    in_table_head = true;
                    current_table_row.clear();
                }
                Tag::TableRow => {
                    current_table_row.clear();
                }
                Tag::TableCell => {
                    current_cell_text.clear();
                }
                Tag::Emphasis => italic = true,
                Tag::Strong => bold = true,
                Tag::Strikethrough => strikethrough = true,
                Tag::Link {
                    dest_url, title, ..
                } => {
                    current_link = Some((dest_url.to_string(), title.to_string()));
                }
                Tag::Image {
                    dest_url, title, ..
                } => {
                    current_image = Some((dest_url.to_string(), title.to_string()));
                }
                _ => {}
            },
            Event::End(tag_end) => match tag_end {
                TagEnd::Paragraph => {
                    if in_blockquote {
                        blockquote_blocks
                            .push(MarkdownBlock::Paragraph(std::mem::take(&mut current_spans)));
                    } else if in_list {
                        // Paragraph inside list item
                        current_list_item.append(&mut current_spans);
                    } else {
                        blocks.push(MarkdownBlock::Paragraph(std::mem::take(&mut current_spans)));
                    }
                }
                TagEnd::Heading(level) => {
                    let lvl = match level {
                        HeadingLevel::H1 => 1,
                        HeadingLevel::H2 => 2,
                        HeadingLevel::H3 => 3,
                        HeadingLevel::H4 => 4,
                        HeadingLevel::H5 => 5,
                        HeadingLevel::H6 => 6,
                    };
                    let b = MarkdownBlock::Heading {
                        level: lvl,
                        spans: std::mem::take(&mut current_spans),
                    };
                    if in_blockquote {
                        blockquote_blocks.push(b);
                    } else {
                        blocks.push(b);
                    }
                }
                TagEnd::BlockQuote(_) => {
                    in_blockquote = false;
                    blocks.push(MarkdownBlock::Blockquote(std::mem::take(
                        &mut blockquote_blocks,
                    )));
                }
                TagEnd::CodeBlock => {
                    in_code_block = false;
                    let b = MarkdownBlock::CodeBlock {
                        language: code_lang.take(),
                        code: std::mem::take(&mut code_lines),
                    };
                    if in_blockquote {
                        blockquote_blocks.push(b);
                    } else {
                        blocks.push(b);
                    }
                }
                TagEnd::List(_) => {
                    in_list = false;
                    if is_task_list {
                        let b = MarkdownBlock::TaskList {
                            items: std::mem::take(&mut task_items),
                        };
                        if in_blockquote {
                            blockquote_blocks.push(b);
                        } else {
                            blocks.push(b);
                        }
                    } else {
                        let b = MarkdownBlock::List {
                            ordered: list_ordered,
                            start: list_start,
                            items: std::mem::take(&mut list_items),
                        };
                        if in_blockquote {
                            blockquote_blocks.push(b);
                        } else {
                            blocks.push(b);
                        }
                    }
                }
                TagEnd::Item => {
                    if is_task_list {
                        task_items
                            .push((current_task_checked, std::mem::take(&mut current_list_item)));
                    } else {
                        list_items.push(std::mem::take(&mut current_list_item));
                    }
                }
                TagEnd::Table => {
                    in_table = false;
                    let b = MarkdownBlock::Table {
                        headers: std::mem::take(&mut table_headers),
                        alignments: std::mem::take(&mut table_alignments),
                        rows: std::mem::take(&mut table_rows),
                    };
                    if in_blockquote {
                        blockquote_blocks.push(b);
                    } else {
                        blocks.push(b);
                    }
                }
                TagEnd::TableHead => {
                    in_table_head = false;
                    table_headers = std::mem::take(&mut current_table_row);
                }
                TagEnd::TableRow => {
                    if !in_table_head {
                        table_rows.push(std::mem::take(&mut current_table_row));
                    }
                }
                TagEnd::TableCell => {
                    current_table_row.push(std::mem::take(&mut current_cell_text));
                }
                TagEnd::Emphasis => italic = false,
                TagEnd::Strong => bold = false,
                TagEnd::Strikethrough => strikethrough = false,
                TagEnd::Link => {
                    current_link = None;
                }
                TagEnd::Image => {
                    current_image = None;
                }
                _ => {}
            },
            Event::Text(t) => {
                if in_code_block {
                    for line in t.lines() {
                        code_lines.push(line.to_string());
                    }
                } else if in_table {
                    current_cell_text.push_str(&t);
                } else if let Some((url, _)) = &current_link {
                    let safe = is_safe_url(url);
                    let span = InlineSpan::Link {
                        text: t.to_string(),
                        url: url.clone(),
                        safe,
                    };
                    if in_list {
                        current_list_item.push(span);
                    } else {
                        current_spans.push(span);
                    }
                } else if let Some((url, alt)) = &current_image {
                    let safe = is_safe_url(url);
                    let resolved = resolve_relative_path(base_file, workspace_root, url);
                    let display_alt = if t.is_empty() {
                        alt.clone()
                    } else {
                        t.to_string()
                    };
                    let span = InlineSpan::Image {
                        alt: display_alt,
                        url: url.clone(),
                        resolved_path: resolved,
                        safe,
                    };
                    if in_list {
                        current_list_item.push(span);
                    } else {
                        current_spans.push(span);
                    }
                } else {
                    let text = t.to_string();
                    let span = if bold && italic {
                        InlineSpan::BoldItalic(text)
                    } else if bold {
                        InlineSpan::Bold(text)
                    } else if italic {
                        InlineSpan::Italic(text)
                    } else if strikethrough {
                        InlineSpan::Strikethrough(text)
                    } else {
                        InlineSpan::Text(text)
                    };
                    if in_list {
                        current_list_item.push(span);
                    } else {
                        current_spans.push(span);
                    }
                }
            }
            Event::Code(c) => {
                let span = InlineSpan::Code(c.to_string());
                if in_table {
                    current_cell_text.push_str(&format!("`{c}`"));
                } else if in_list {
                    current_list_item.push(span);
                } else {
                    current_spans.push(span);
                }
            }
            Event::Html(h) => {
                let sanitized = sanitize_html(&h);
                if !sanitized.is_empty() {
                    blocks.push(MarkdownBlock::RawHtml(sanitized));
                }
            }
            Event::InlineHtml(ih) => {
                let sanitized = sanitize_html(&ih);
                if !sanitized.is_empty() {
                    let span = InlineSpan::Text(sanitized);
                    if in_list {
                        current_list_item.push(span);
                    } else {
                        current_spans.push(span);
                    }
                }
            }
            Event::Rule => {
                blocks.push(MarkdownBlock::ThematicBreak);
            }
            Event::TaskListMarker(checked) => {
                is_task_list = true;
                current_task_checked = checked;
            }
            Event::SoftBreak | Event::HardBreak => {
                let span = InlineSpan::Text(" ".to_string());
                if in_list {
                    current_list_item.push(span);
                } else {
                    current_spans.push(span);
                }
            }
            _ => {}
        }
    }

    let word_count = markdown.split_whitespace().count();
    let line_count = markdown.lines().count();

    ParsedMarkdown {
        blocks,
        error: None,
        word_count,
        line_count,
    }
}
