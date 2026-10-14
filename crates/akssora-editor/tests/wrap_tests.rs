use std::path::PathBuf;

use akssora_editor::Document;
use akssora_editor::wrap::wrap_line;

#[test]
fn test_wrap_empty_lines() {
    let rows = wrap_line(0, "", 40);
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].logical_row, 0);
    assert_eq!(rows[0].char_start, 0);
    assert_eq!(rows[0].char_end, 0);
    assert!(!rows[0].is_continuation);

    let doc = Document::open(PathBuf::from("empty.txt"), "\n\n");
    let layout = doc.wrap_layout(40);
    assert_eq!(layout.total_visual_rows(), 3);
    for r in 0..3 {
        let vr = layout.get_visual_row(r).unwrap();
        assert_eq!(vr.logical_row, r);
        assert_eq!(vr.char_start, 0);
        assert_eq!(vr.char_end, 0);
        assert!(!vr.is_continuation);
    }
}

#[test]
fn test_wrap_long_lines_at_word_boundaries() {
    let line = "The quick brown fox jumps over the lazy dog";
    let rows = wrap_line(0, line, 16);
    // Should break cleanly at words
    assert!(rows.len() >= 3);
    assert_eq!(rows[0].char_start, 0);
    assert!(!rows[0].is_continuation);
    for r in &rows[1..] {
        assert!(r.is_continuation);
    }

    // Invariant: all characters must be covered without overlap or gap
    let mut covered = 0;
    for vr in &rows {
        assert_eq!(vr.char_start, covered);
        covered = vr.char_end;
    }
    assert_eq!(covered, line.chars().count());
}

#[test]
fn test_wrap_unbroken_long_token() {
    let line = "https://example.com/a/very/long/url/that/does/not/contain/any/spaces/at/all";
    let rows = wrap_line(0, line, 20);
    assert!(rows.len() > 1);

    // Verify all chars covered
    let mut covered = 0;
    for vr in &rows {
        assert_eq!(vr.char_start, covered);
        covered = vr.char_end;
    }
    assert_eq!(covered, line.chars().count());
}

#[test]
fn test_wrap_unicode_and_wide_characters() {
    let line = "你好世界，这是一个测试！🦀 Hello Rust programming 🚀";
    let char_count = line.chars().count();
    let rows = wrap_line(0, line, 12);
    assert!(rows.len() > 1);

    let mut covered = 0;
    for vr in &rows {
        assert_eq!(vr.char_start, covered);
        covered = vr.char_end;
    }
    assert_eq!(covered, char_count);

    let mut doc = Document::open(PathBuf::from("unicode.txt"), line);
    doc.line_wrap = true;
    let layout = doc.wrap_layout(12);
    assert!(layout.total_visual_rows() > 1);

    // Moving through all characters preserves valid logical coordinates
    for col in 0..char_count {
        let (vis_row, vis_col) = layout.logical_to_visual(0, col);
        let (back_row, back_col) = layout.visual_to_logical(vis_row, vis_col);
        assert_eq!(back_row, 0);
        assert_eq!(back_col, col);
    }
}

#[test]
fn test_document_content_preservation() {
    let original = "A very long line that will wrap when displayed but must not change on disk or in the buffer";
    let mut doc = Document::open(PathBuf::from("wrap.txt"), original);
    doc.line_wrap = true;

    let layout = doc.wrap_layout(25);
    assert!(layout.total_visual_rows() > 1);

    // Buffer content and line count must remain completely untouched
    assert_eq!(doc.buffer.line_count(), 1);
    assert_eq!(doc.buffer.to_string_content(), original);
    assert!(!doc.dirty);
}

#[test]
fn test_viewport_resizing_dynamic_recalculation() {
    let text = "Line one with quite a lot of text to test wrap recalculation on resize\nLine two with even more text here";
    let mut doc = Document::open(PathBuf::from("resize.txt"), text);
    doc.line_wrap = true;

    let layout_wide = doc.wrap_layout(80);
    let rows_wide = layout_wide.total_visual_rows();

    let layout_narrow = doc.wrap_layout(20);
    let rows_narrow = layout_narrow.total_visual_rows();

    assert!(rows_narrow > rows_wide);

    // Resizing back to wide produces identical wide layout
    let layout_wide_again = doc.wrap_layout(80);
    assert_eq!(layout_wide_again.total_visual_rows(), rows_wide);
}

#[test]
fn test_cursor_movement_across_wrapped_lines() {
    let text = "First line is intentionally made very long so that it wraps into multiple visual lines when displayed\nSecond line is short";
    let mut doc = Document::open(PathBuf::from("nav.txt"), text);
    doc.line_wrap = true;
    let wrap_w = 25;

    let layout = doc.wrap_layout(wrap_w);
    let vis_rows_line_0 = layout.line_rows[0].len();
    assert!(vis_rows_line_0 >= 3);

    // Cursor starts at (0, 0)
    assert_eq!(doc.cursor, (0, 0));

    // Moving down visual should stay on logical line 0, but advance visual rows!
    doc.move_cursor_down_visual(false, wrap_w);
    assert_eq!(doc.cursor.0, 0);
    assert!(doc.cursor.1 > 0);
    let (v_row, _) = doc.logical_to_visual(doc.cursor.0, doc.cursor.1, wrap_w);
    assert_eq!(v_row, 1);

    // Move down visual again
    doc.move_cursor_down_visual(false, wrap_w);
    assert_eq!(doc.cursor.0, 0);
    let (v_row2, _) = doc.logical_to_visual(doc.cursor.0, doc.cursor.1, wrap_w);
    assert_eq!(v_row2, 2);

    // Move up visual
    doc.move_cursor_up_visual(false, wrap_w);
    let (v_row_up, _) = doc.logical_to_visual(doc.cursor.0, doc.cursor.1, wrap_w);
    assert_eq!(v_row_up, 1);

    // Move to visual line end
    doc.move_to_line_end_visual(false, wrap_w);
    let vr = layout.get_visual_row(1).unwrap();
    assert_eq!(doc.cursor.1, vr.char_end);

    // Move to visual line start
    doc.move_to_line_start_visual(false, wrap_w);
    assert_eq!(doc.cursor.1, vr.char_start);
}

#[test]
fn test_selection_across_wrapped_lines() {
    let text =
        "Line 0 wraps into multiple segments when viewport is narrow\nLine 1 is another line";
    let mut doc = Document::open(PathBuf::from("sel.txt"), text);
    doc.line_wrap = true;

    // Anchor at start of line 0
    doc.cursor = (0, 0);
    doc.move_cursor_down_visual(true, 20);
    assert!(doc.has_selection());

    let (anchor, cursor) = doc.selection_range().unwrap();
    assert_eq!(anchor, (0, 0));
    assert_eq!(cursor, doc.cursor);
    assert!(cursor.1 > 0);

    let sel_text = doc.selected_text().unwrap();
    assert!(!sel_text.is_empty());
}

#[test]
fn test_scrolling_with_wrapped_lines() {
    let text = "A very long line that spans several visual rows across the screen when line wrapping is enabled in the editor\nShort 1\nShort 2\nShort 3";
    let mut doc = Document::open(PathBuf::from("scroll.txt"), text);
    doc.line_wrap = true;
    let wrap_w = 20;

    let layout = doc.wrap_layout(wrap_w);
    let line_0_vis_count = layout.line_rows[0].len();
    assert!(line_0_vis_count >= 4);

    // Place cursor at the end of line 0
    let end_col = doc.buffer.line_len(0);
    doc.cursor = (0, end_col);

    // Scroll to cursor in a viewport with 2 visible rows
    doc.scroll_to_cursor(2, wrap_w);
    // scroll_row should be adjusted so the cursor's visual row is visible
    let (cursor_vis, _) = doc.logical_to_visual(0, end_col, wrap_w);
    assert!(doc.scroll_row <= cursor_vis);
    assert!(doc.scroll_row + 2 > cursor_vis);
    assert_eq!(doc.scroll_col, 0);
}

#[test]
fn test_wrap_toggle() {
    let text = "A line that exceeds forty columns in length and wraps when enabled";
    let mut doc = Document::open(PathBuf::from("toggle.txt"), text);

    doc.line_wrap = false;
    assert_eq!(doc.total_visual_rows(30), 1);

    doc.line_wrap = true;
    assert!(doc.total_visual_rows(30) > 1);

    doc.line_wrap = false;
    assert_eq!(doc.total_visual_rows(30), 1);
}
