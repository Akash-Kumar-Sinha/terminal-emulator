use akssora_editor::Document;

#[test]
fn test_document_editing_and_cursor() {
    let mut doc = Document::new();
    assert_eq!(doc.cursor, (0, 0));
    assert_eq!(doc.buffer.line_count(), 1);

    // Type text
    doc.insert_str("Hello");
    assert_eq!(doc.cursor, (0, 5));
    assert_eq!(doc.buffer.get_line(0).as_deref(), Some("Hello"));

    // Enter newline
    doc.insert_newline();
    assert_eq!(doc.cursor, (1, 0));
    assert_eq!(doc.buffer.line_count(), 2);

    doc.insert_str("World");
    assert_eq!(doc.cursor, (1, 5));

    // Cursor movement
    doc.move_cursor_left(false);
    assert_eq!(doc.cursor, (1, 4));

    doc.move_to_line_start(false);
    assert_eq!(doc.cursor, (1, 0));

    doc.move_to_line_end(false);
    assert_eq!(doc.cursor, (1, 5));

    doc.move_cursor_up(false);
    assert_eq!(doc.cursor, (0, 5));

    // Backspace
    doc.backspace();
    assert_eq!(doc.buffer.get_line(0).as_deref(), Some("Hell"));
    assert_eq!(doc.cursor, (0, 4));

    // Undo
    doc.undo();
    assert_eq!(doc.buffer.get_line(0).as_deref(), Some("Hello"));
    assert_eq!(doc.cursor, (0, 5));

    // Redo
    doc.redo();
    assert_eq!(doc.buffer.get_line(0).as_deref(), Some("Hell"));
    assert_eq!(doc.cursor, (0, 4));
}

#[test]
fn test_document_selection_and_deletion() {
    let mut doc = Document::open(
        std::path::PathBuf::from("test.txt"),
        "First Line\nSecond Line",
    );
    doc.select_all();
    assert!(doc.has_selection());
    assert_eq!(
        doc.selected_text(),
        Some("First Line\nSecond Line".to_string())
    );

    doc.delete_selection();
    assert_eq!(doc.buffer.to_string_content(), "");
    assert_eq!(doc.cursor, (0, 0));
}

#[test]
fn test_document_file_save() {
    let temp_file = std::env::temp_dir().join(format!(
        "akssora_doc_save_test_{}.txt",
        rand::random::<u64>()
    ));
    let mut doc = Document::open(temp_file.clone(), "Hello World");
    assert!(!doc.dirty);

    doc.move_to_line_end(false);
    doc.insert_str("!");
    assert!(doc.dirty);

    doc.save().unwrap();
    assert!(!doc.dirty);

    let saved = std::fs::read_to_string(&temp_file).unwrap();
    assert_eq!(saved, "Hello World!");

    let _ = std::fs::remove_file(&temp_file);
}

// INFO: Verifies that inline git change indicators (added, modified, deleted) stay synchronized as buffer changes.
#[test]
fn test_document_git_change_indicators_synchronization() {
    use akssora_git::LineGitChange;

    let baseline = "alpha\nbeta\ngamma";
    let mut doc = Document::open(std::path::PathBuf::from("tracked.txt"), baseline);
    doc.set_git_original_content(Some(baseline.to_string()));

    // 1. Initially matching baseline: no changes
    let statuses = doc.git_line_statuses();
    assert_eq!(statuses.len(), 3);
    assert!(
        statuses
            .iter()
            .all(|s| s.change.is_none() && !s.deleted_above && !s.deleted_below)
    );

    // 2. Modify line 1 ("beta" -> "beta modified")
    doc.cursor = (1, 4);
    doc.insert_str(" modified");
    let statuses = doc.git_line_statuses();
    assert_eq!(statuses.len(), 3);
    assert_eq!(statuses[0].change, None);
    assert_eq!(statuses[1].change, Some(LineGitChange::Modified));
    assert_eq!(statuses[2].change, None);

    // 3. Add a new line at the end
    doc.cursor = (2, 5);
    doc.insert_newline();
    doc.insert_str("delta");
    let statuses = doc.git_line_statuses();
    assert_eq!(statuses.len(), 4);
    assert_eq!(statuses[1].change, Some(LineGitChange::Modified));
    assert_eq!(statuses[3].change, Some(LineGitChange::Added));

    // 4. Delete first line ("alpha")
    doc.cursor = (0, 0);
    doc.selection_anchor = Some((1, 0));
    doc.delete_selection();
    let statuses = doc.git_line_statuses();
    assert_eq!(statuses.len(), 3);
    // Line 0 is now "beta modified", and has deleted_above = true because "alpha" was deleted
    assert_eq!(statuses[0].change, Some(LineGitChange::Modified));
    assert!(statuses[0].deleted_above);

    // 5. Reset buffer to match original baseline -> all indicators clear
    doc.buffer = akssora_editor::TextBuffer::from_text(baseline);
    let statuses = doc.git_line_statuses();
    assert_eq!(statuses.len(), 3);
    assert!(
        statuses
            .iter()
            .all(|s| s.change.is_none() && !s.deleted_above && !s.deleted_below)
    );
}
