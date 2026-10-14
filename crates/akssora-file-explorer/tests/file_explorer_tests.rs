use akssora_file_explorer::{EntryKind, FileTree};
use std::fs;

#[test]
fn test_file_explorer_crud_and_tree_navigation() {
    let temp_dir =
        std::env::temp_dir().join(format!("akssora_explorer_test_{}", rand::random::<u64>()));
    fs::create_dir_all(&temp_dir).unwrap();

    let mut tree = FileTree::new(&temp_dir).unwrap();
    assert_eq!(tree.flatten_rows().len(), 0);

    // 1. Create file
    let file1 = tree.create_file(&temp_dir, "test1.rs").unwrap();
    assert!(file1.exists());
    let rows = tree.flatten_rows();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].name, "test1.rs");
    assert_eq!(rows[0].kind, EntryKind::File);

    // 2. Create directory
    let sub_dir = tree.create_dir(&temp_dir, "src").unwrap();
    assert!(sub_dir.exists());

    // 3. Create file inside sub directory
    let file2 = tree.create_file(&sub_dir, "main.rs").unwrap();
    assert!(file2.exists());

    // Directory expansion
    tree.expand(&sub_dir);
    let rows_expanded = tree.flatten_rows();
    assert_eq!(rows_expanded.len(), 3); // src, main.rs (depth 1), test1.rs (depth 0)

    // Directory collapse
    tree.collapse(&sub_dir);
    let rows_collapsed = tree.flatten_rows();
    assert_eq!(rows_collapsed.len(), 2);

    // 4. Rename
    let renamed = tree.rename(&file1, "renamed.rs").unwrap();
    assert!(renamed.exists());
    assert!(!file1.exists());

    // 5. Copy
    let copied = tree.copy_entry(&renamed, &sub_dir).unwrap();
    assert!(copied.exists());
    assert_eq!(copied.file_name().unwrap(), "renamed.rs");

    // 6. Move
    let moved = tree.move_entry(&renamed, &sub_dir).unwrap();
    assert!(moved.exists());
    assert!(!renamed.exists());

    // 7. Delete
    tree.delete(&moved).unwrap();
    assert!(!moved.exists());

    // Cleanup
    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_file_explorer_hidden_and_exclusions() {
    let temp_dir = std::env::temp_dir().join(format!(
        "akssora_explorer_hidden_test_{}",
        rand::random::<u64>()
    ));
    fs::create_dir_all(&temp_dir).unwrap();

    let mut tree = FileTree::new(&temp_dir).unwrap();
    let _hidden = tree.create_file(&temp_dir, ".hidden_file").unwrap();
    let _target = tree.create_dir(&temp_dir, "target").unwrap();
    let _normal = tree.create_file(&temp_dir, "normal.txt").unwrap();

    // Default: hidden false, target excluded
    tree.set_show_hidden(false);
    let rows = tree.flatten_rows();
    let names: Vec<String> = rows.iter().map(|r| r.name.clone()).collect();
    assert!(names.contains(&"normal.txt".to_string()));
    assert!(!names.contains(&".hidden_file".to_string()));
    assert!(!names.contains(&"target".to_string()));

    // Toggle hidden: .hidden_file appears
    tree.set_show_hidden(true);
    let rows_hidden = tree.flatten_rows();
    let names_hidden: Vec<String> = rows_hidden.iter().map(|r| r.name.clone()).collect();
    assert!(names_hidden.contains(&".hidden_file".to_string()));
    assert!(!names_hidden.contains(&"target".to_string()));

    // Clear exclusions: target appears
    tree.set_exclusions(vec![]);
    let rows_all = tree.flatten_rows();
    let names_all: Vec<String> = rows_all.iter().map(|r| r.name.clone()).collect();
    assert!(names_all.contains(&"target".to_string()));

    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_file_explorer_multi_selection_and_drag_drop() {
    let temp_dir = std::env::temp_dir().join(format!(
        "akssora_explorer_drag_test_{}",
        rand::random::<u64>()
    ));
    fs::create_dir_all(&temp_dir).unwrap();

    let mut tree = FileTree::new(&temp_dir).unwrap();
    let f1 = tree.create_file(&temp_dir, "a.txt").unwrap();
    let f2 = tree.create_file(&temp_dir, "b.txt").unwrap();
    let f3 = tree.create_file(&temp_dir, "c.txt").unwrap();
    let dest_dir = tree.create_dir(&temp_dir, "dest").unwrap();

    // Multi-selection
    tree.select_single(f1.clone());
    tree.toggle_select(f2.clone());
    assert!(tree.is_selected(&f1));
    assert!(tree.is_selected(&f2));
    assert!(!tree.is_selected(&f3));

    // Drag and drop into dest_dir
    tree.start_drag(f1.clone());
    tree.update_drop_target(Some(dest_dir.clone()));
    let moved = tree.finish_drag().unwrap();
    assert_eq!(moved.len(), 2);
    assert!(dest_dir.join("a.txt").exists());
    assert!(dest_dir.join("b.txt").exists());
    assert!(!f1.exists());
    assert!(!f2.exists());

    let _ = fs::remove_dir_all(&temp_dir);
}
