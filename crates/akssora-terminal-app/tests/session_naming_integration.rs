use akssora_terminal_app::App;
use akssora_terminal_ui::{SidebarRowKind, SidebarTarget, SplitDirection};

#[test]
fn test_dynamic_session_naming_and_project_grouping() {
    let mut app = App::new();
    app.sidebar.is_expanded = true;
    let tab1 = app.open_terminal_tab();
    let _tab2 = app.open_terminal_tab();

    let items = app.collect_session_items();
    assert_eq!(items.len(), 2);

    // Sidebar session rows outside split view should remain independent entries (no project grouping)
    let rows = app.sidebar.compute_session_rows(&items, 0.0);
    assert_eq!(rows.len(), 2);

    // Should NOT have project header or group header
    let has_project_header = rows
        .iter()
        .any(|r| matches!(&r.kind, SidebarRowKind::ProjectHeader { .. }));
    assert!(
        !has_project_header,
        "Terminals outside split view must not have ProjectHeader"
    );

    // Both should be independent Session entries (is_split: false)
    for r in &rows {
        assert!(matches!(
            &r.kind,
            SidebarRowKind::Session {
                is_split: false,
                ..
            }
        ));
    }

    // Now test split grouping: split tab1
    app.tab_manager.select_by_id(tab1);
    let split_pane = app
        .split_active_pane(SplitDirection::Vertical)
        .expect("split created");
    let items_split = app.collect_session_items();
    let rows_split = app.sidebar.compute_session_rows(&items_split, 0.0);

    // Tab 1 is now split, so it should have a GroupHeader and 2 split child sessions
    let has_group_header = rows_split
        .iter()
        .any(|r| matches!(&r.kind, SidebarRowKind::GroupHeader { split_count: 2, .. }));
    assert!(has_group_header, "Expected GroupHeader for split view");

    // Closing split pane should automatically remove the group header
    app.close_split_pane(split_pane);
    let items_after = app.collect_session_items();
    let rows_after = app.sidebar.compute_session_rows(&items_after, 0.0);
    let has_group_after = rows_after
        .iter()
        .any(|r| matches!(&r.kind, SidebarRowKind::GroupHeader { .. }));
    assert!(
        !has_group_after,
        "GroupHeader must be removed when reduced to 1 terminal"
    );
}

#[test]
fn test_manual_session_renaming_preservation() {
    let mut app = App::new();
    let tab1 = app.open_terminal_tab();

    // Verify initial item
    let items_before = app.collect_session_items();
    assert_eq!(items_before.len(), 1);

    // Manually rename pane
    app.rename_pane(tab1, "Custom Server Runner");

    let items_after = app.collect_session_items();
    assert_eq!(items_after[0].name, "Custom Server Runner");

    // Renaming preserves custom name even if resolve is called again
    let rows = app.sidebar.compute_session_rows(&items_after, 0.0);
    let session_row = rows
        .iter()
        .find(|r| matches!(&r.kind, SidebarRowKind::Session { pane_id, .. } if *pane_id == tab1));
    assert!(session_row.is_some());
    if let Some(r) = session_row
        && let SidebarRowKind::Session { session_name, .. } = &r.kind
    {
        assert_eq!(session_name, "Custom Server Runner");
    }
}

#[test]
fn test_split_session_sidebar_independent_listing() {
    let mut app = App::new();
    let tab1 = app.open_terminal_tab();
    let split_pane = app
        .split_active_pane(SplitDirection::Vertical)
        .expect("split created");

    let items = app.collect_session_items();
    assert_eq!(items.len(), 2);
    assert!(items.iter().any(|s| s.pane_id == tab1));
    assert!(items.iter().any(|s| s.pane_id == split_pane));

    // Clicking session activates corresponding pane without losing state
    let target = SidebarTarget::Session {
        tab_idx: 0,
        pane_id: tab1,
    };
    app.handle_sidebar_target(target);
    assert_eq!(app.split_manager.active_pane_id, Some(tab1));

    let target2 = SidebarTarget::Session {
        tab_idx: 0,
        pane_id: split_pane,
    };
    app.handle_sidebar_target(target2);
    assert_eq!(app.split_manager.active_pane_id, Some(split_pane));
}

#[test]
fn test_session_state_persistence_file() {
    let mut app = App::new();
    let temp_file = std::env::temp_dir().join(format!(
        "akssora_test_session_{}.json",
        rand::random::<u64>()
    ));
    app.session_file = Some(temp_file.clone());

    let tab1 = app.open_terminal_tab();
    app.rename_pane(tab1, "Persistent Build Job");

    // Save session
    app.save_session_state();

    // Verify state was saved to isolated file
    assert!(temp_file.exists());
    let content = std::fs::read_to_string(&temp_file).expect("read session.json");
    assert!(content.contains("Persistent Build Job"));
    let _ = std::fs::remove_file(&temp_file);
}

// INFO: Verifies that pages (Settings, Editor, API Testing, Plugin Management) appear in the sidebar with appropriate icons, support switching without losing state, and prevent singleton duplicates.
#[test]
fn test_sidebar_open_pages_and_singleton_deduplication() {
    let mut app = App::new();
    app.sidebar.is_expanded = true;

    // Initial state: 1 terminal
    let term1_id = app.open_terminal_tab();

    // Open various pages and views
    let settings_id = app.open_settings_tab();
    let editor_id = app.open_editor_tab("/path/to/my_code.rs");
    let api_id = app.open_api_tab("payment_gateway");
    let plugin_id = app.open_plugin_management_tab();

    // Collect sidebar session items
    let items = app.collect_session_items();
    assert_eq!(items.len(), 5);

    // Verify icons and titles
    let term_item = items.iter().find(|i| i.pane_id == term1_id).unwrap();
    assert_eq!(term_item.icon, '>');

    let settings_item = items.iter().find(|i| i.pane_id == settings_id).unwrap();
    assert_eq!(settings_item.name, "Settings");
    assert_eq!(settings_item.icon, '\u{2699}');

    let editor_item = items.iter().find(|i| i.pane_id == editor_id).unwrap();
    assert_eq!(editor_item.name, "my_code.rs");
    assert_eq!(editor_item.icon, '\u{2261}');

    let api_item = items.iter().find(|i| i.pane_id == api_id).unwrap();
    assert_eq!(api_item.name, "API: payment_gateway");
    assert_eq!(api_item.icon, '\u{21C4}');

    let plugin_item = items.iter().find(|i| i.pane_id == plugin_id).unwrap();
    assert_eq!(plugin_item.name, "Plugin Management");
    assert_eq!(plugin_item.icon, '\u{29C9}');

    // Verify sidebar rows rendering
    let rows = app.sidebar.compute_session_rows(&items, 0.0);
    assert_eq!(rows.len(), 5);
    for row in &rows {
        assert!(matches!(
            &row.kind,
            SidebarRowKind::Session {
                is_split: false,
                ..
            }
        ));
    }

    // Verify switching between open pages without losing state
    assert_eq!(app.active_tab_id(), Some(plugin_id));

    // Switch to Settings via sidebar target
    let settings_idx = app
        .tab_manager
        .tabs()
        .iter()
        .position(|t| t.id == settings_id)
        .unwrap();
    app.handle_sidebar_target(SidebarTarget::Session {
        tab_idx: settings_idx,
        pane_id: settings_id,
    });
    assert_eq!(app.active_tab_id(), Some(settings_id));

    // Switch to Terminal
    let term_idx = app
        .tab_manager
        .tabs()
        .iter()
        .position(|t| t.id == term1_id)
        .unwrap();
    app.handle_sidebar_target(SidebarTarget::Session {
        tab_idx: term_idx,
        pane_id: term1_id,
    });
    assert_eq!(app.active_tab_id(), Some(term1_id));

    // Verify singleton deduplication: opening Settings again activates existing tab
    let reopen_settings_id = app.open_settings_tab();
    assert_eq!(reopen_settings_id, settings_id);
    assert_eq!(app.active_tab_id(), Some(settings_id));
    assert_eq!(app.tab_manager.len(), 5);
    assert_eq!(app.collect_session_items().len(), 5);

    // Verify singleton deduplication for Plugin Management
    let reopen_plugin_id = app.open_plugin_management_tab();
    assert_eq!(reopen_plugin_id, plugin_id);
    assert_eq!(app.active_tab_id(), Some(plugin_id));
    assert_eq!(app.tab_manager.len(), 5);

    // Verify singleton deduplication for Editor with same path
    let reopen_editor_id = app.open_editor_tab("/path/to/my_code.rs");
    assert_eq!(reopen_editor_id, editor_id);
    assert_eq!(app.active_tab_id(), Some(editor_id));
    assert_eq!(app.tab_manager.len(), 5);

    // Verify closing an open page removes it from the sidebar
    app.handle_sidebar_target(SidebarTarget::SessionClose {
        tab_idx: settings_idx,
        pane_id: settings_id,
    });
    assert_eq!(app.tab_manager.len(), 4);
    let items_after_close = app.collect_session_items();
    assert_eq!(items_after_close.len(), 4);
    assert!(!items_after_close.iter().any(|i| i.pane_id == settings_id));

    // Verify remaining tabs are valid and can be switched with next_tab / prev_tab
    let initial_active = app.active_tab_id();
    app.next_tab();
    assert_ne!(app.active_tab_id(), initial_active);
}
