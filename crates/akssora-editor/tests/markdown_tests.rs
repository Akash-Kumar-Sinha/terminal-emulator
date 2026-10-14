use akssora_editor::{
    Document, FormatterTool, MarkdownBlock, MarkdownFormatter, MarkdownViewMode, is_safe_url,
    parse_markdown, resolve_relative_path, sanitize_html,
};
use std::path::PathBuf;

#[test]
fn test_is_markdown_file_extension() {
    let doc_md = Document::open(PathBuf::from("README.md"), "# Heading");
    assert!(doc_md.is_markdown());

    let doc_markdown = Document::open(PathBuf::from("docs/guide.markdown"), "Guide text");
    assert!(doc_markdown.is_markdown());

    let doc_rs = Document::open(PathBuf::from("src/main.rs"), "fn main() {}");
    assert!(!doc_rs.is_markdown());

    let doc_unnamed = Document::new();
    assert!(!doc_unnamed.is_markdown());
}

#[test]
fn test_markdown_view_mode_transitions() {
    let mut mode = MarkdownViewMode::Source;

    // Toggle Preview
    mode = mode.toggle_preview();
    assert_eq!(mode, MarkdownViewMode::Preview);
    mode = mode.toggle_preview();
    assert_eq!(mode, MarkdownViewMode::Source);

    // Toggle Split
    mode = mode.toggle_split();
    assert_eq!(mode, MarkdownViewMode::Split);
    mode = mode.toggle_split();
    assert_eq!(mode, MarkdownViewMode::Source);

    // Direct transitions preserve document properties
    let mut doc = Document::open(
        PathBuf::from("test.md"),
        "# Title\n\nSome paragraph text here.\n",
    );
    doc.cursor = (1, 3);
    doc.scroll_row = 5;
    doc.preview_scroll_row = 12;

    doc.markdown_mode = MarkdownViewMode::Split;
    assert_eq!(doc.cursor, (1, 3));
    assert_eq!(doc.scroll_row, 5);
    assert_eq!(doc.preview_scroll_row, 12);
    assert_eq!(
        doc.buffer.to_string_content(),
        "# Title\n\nSome paragraph text here.\n"
    );

    doc.markdown_mode = MarkdownViewMode::Preview;
    assert_eq!(doc.cursor, (1, 3));
    assert_eq!(doc.scroll_row, 5);
    assert_eq!(doc.preview_scroll_row, 12);
}

#[test]
fn test_markdown_parser_ast_nodes() {
    let markdown_text = r#"# Main Title

This is a **bold** and *italic* paragraph with `code` and [link](https://example.com).

- [ ] Task incomplete
- [x] Task complete

- Normal item

> Quote block

```rust
fn hello() {
    println!("hi");
}
```

| Col1 | Col2 |
| --- | --- |
| Val1 | Val2 |
"#;

    let parsed = parse_markdown(markdown_text, None, None);
    assert!(!parsed.blocks.is_empty());

    let has_heading = parsed
        .blocks
        .iter()
        .any(|b| matches!(b, MarkdownBlock::Heading { level: 1, .. }));
    assert!(has_heading, "Heading block should be present");

    let has_task_list = parsed
        .blocks
        .iter()
        .any(|b| matches!(b, MarkdownBlock::TaskList { .. }));
    assert!(has_task_list, "TaskList block should be present");

    let has_code_block = parsed.blocks.iter().any(
        |b| matches!(b, MarkdownBlock::CodeBlock { language: Some(lang), .. } if lang == "rust"),
    );
    assert!(has_code_block, "Rust code block should be present");

    let has_table = parsed
        .blocks
        .iter()
        .any(|b| matches!(b, MarkdownBlock::Table { .. }));
    assert!(has_table, "Table block should be present");

    let has_quote = parsed
        .blocks
        .iter()
        .any(|b| matches!(b, MarkdownBlock::Blockquote(_)));
    assert!(has_quote, "Blockquote block should be present");
}

#[test]
fn test_url_security_and_scheme_validation() {
    assert!(is_safe_url("https://example.com/image.png"));
    assert!(is_safe_url("http://example.com"));
    assert!(is_safe_url("relative/path/to/img.png"));
    assert!(is_safe_url("./local_file.md"));
    assert!(is_safe_url("../parent/doc.md"));
    assert!(is_safe_url("mailto:dev@example.com"));

    // Dangerous schemes must be rejected
    assert!(!is_safe_url("javascript:alert(1)"));
    assert!(!is_safe_url("JAVASCRIPT:alert(document.cookie)"));
    assert!(!is_safe_url("vbscript:msgbox(1)"));
    assert!(!is_safe_url("data:text/html,<script>alert(1)</script>"));
}

#[test]
fn test_html_sanitization() {
    let clean = sanitize_html("<p>Hello world</p>");
    assert_eq!(clean, "<p>Hello world</p>");

    let malicious_script = "<script>alert('pwned')</script>Safe text";
    let sanitized = sanitize_html(malicious_script);
    assert!(!sanitized.contains("<script>"));
    assert!(!sanitized.contains("</script>"));
    assert!(sanitized.contains("Safe text"));

    let malicious_handler = r#"<img src="foo.png" onerror="alert(1)" onload="alert(2)" />"#;
    let sanitized_handler = sanitize_html(malicious_handler);
    assert!(!sanitized_handler.contains("onerror"));
    assert!(!sanitized_handler.contains("onload"));
}

#[test]
fn test_relative_asset_path_resolution() {
    let doc_path = PathBuf::from("/workspace/docs/guide.md");
    let ws_root = PathBuf::from("/workspace");

    // Relative to doc dir
    let resolved = resolve_relative_path(Some(&doc_path), Some(&ws_root), "images/arch.png");
    assert_eq!(
        resolved,
        Some(PathBuf::from("/workspace/docs/images/arch.png"))
    );

    // Parent relative
    let resolved_parent =
        resolve_relative_path(Some(&doc_path), Some(&ws_root), "../assets/logo.svg");
    assert!(resolved_parent.is_some());

    // Root relative
    let resolved_root = resolve_relative_path(Some(&doc_path), Some(&ws_root), "/assets/icon.png");
    assert_eq!(
        resolved_root,
        Some(PathBuf::from("/workspace/assets/icon.png"))
    );
}

#[test]
fn test_markdown_formatter_pure_rust_builtin() {
    let unformatted = r#"#Heading Without Space
- item 1
-  item 2

```rust
// code must be strictly preserved
fn foo()   {   }
```

| Col1 |Col2|
|---|---|
|A|B|
"#;

    let formatted = MarkdownFormatter::format(unformatted, FormatterTool::Builtin, None, None)
        .expect("Formatting should succeed");

    // Heading has space
    assert!(formatted.contains("# Heading Without Space"));
    // Code block inside preserved exactly
    assert!(formatted.contains("fn foo()   {   }"));
    // Table padded
    assert!(formatted.contains("| Col1 | Col2 |"));

    // Idempotent formatting: running formatting a second time produces identical output
    let formatted_twice = MarkdownFormatter::format(&formatted, FormatterTool::Builtin, None, None)
        .expect("Formatting second pass should succeed");
    assert_eq!(formatted, formatted_twice, "Formatter must be idempotent");
}

#[test]
fn test_document_format_markdown_integration() {
    let mut doc = Document::open(PathBuf::from("test.md"), "#Title\nSome   content.\n");

    let changed = doc
        .format_markdown(FormatterTool::Builtin, None)
        .expect("Format succeeds");
    assert!(changed);
    assert!(doc.buffer.to_string_content().contains("# Title"));
    assert!(doc.dirty);

    // Format again when already formatted -> returns changed = false
    let changed_second = doc
        .format_markdown(FormatterTool::Builtin, None)
        .expect("Format succeeds");
    assert!(!changed_second);
}

#[test]
fn test_markdown_cache_invalidation_on_edits() {
    let mut doc = Document::open(PathBuf::from("notes.md"), "# Title 1\n");
    doc.cursor = (0, 9);
    let preview1 = doc.parse_markdown_preview(None);
    assert_eq!(preview1.blocks.len(), 1);

    // Insert new character
    doc.insert_str(" Updated");
    let preview2 = doc.parse_markdown_preview(None);
    assert_eq!(preview2.blocks.len(), 1);

    // Check heading contents
    if let MarkdownBlock::Heading { spans, .. } = &preview2.blocks[0] {
        let text: String = spans
            .iter()
            .map(|s| match s {
                akssora_editor::InlineSpan::Text(t) => t.as_str(),
                _ => "",
            })
            .collect();
        assert!(text.contains("Updated"));
    } else {
        panic!("Expected Heading block, got: {:?}", preview2.blocks);
    }

    // Undo restores previous
    doc.undo();
    let preview3 = doc.parse_markdown_preview(None);
    if let MarkdownBlock::Heading { spans, .. } = &preview3.blocks[0] {
        let text: String = spans
            .iter()
            .map(|s| match s {
                akssora_editor::InlineSpan::Text(t) => t.as_str(),
                _ => "",
            })
            .collect();
        assert!(!text.contains("Updated"));
    }
}
