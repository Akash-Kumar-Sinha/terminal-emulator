use akssora_terminal_app::App;
use akssora_terminal_ui::{Sidebar, SidebarRowKind, SidebarTarget, SplitDirection};

#[test]
fn test_sidebar_split_grouping_and_collapse() {
    let mut app = App::new();
    app.sidebar.is_expanded = true;
    app.sidebar.width = 240.0;
    let tab1_id = app.open_terminal_tab();
    assert_eq!(app.tab_count(), 1);

    // Single terminal session: normal tab row without grouping
    let rows_initial = app
        .sidebar
        .compute_rows(app.tab_manager.tabs(), &app.split_trees, 0.0);
    assert_eq!(rows_initial.len(), 1);
    assert!(matches!(rows_initial[0].kind, SidebarRowKind::NormalTab));

    // Create split view pane
    let pane2_id = app
        .split_active_pane(SplitDirection::Vertical)
        .expect("must create vertical split");

    // Multiple split panes in tab 1: displayed as grouped set under parent session
    let rows_split = app
        .sidebar
        .compute_rows(app.tab_manager.tabs(), &app.split_trees, 0.0);
    assert_eq!(rows_split.len(), 3);
    assert!(matches!(
        rows_split[0].kind,
        SidebarRowKind::GroupHeader {
            is_collapsed: false,
            split_count: 2
        }
    ));
    assert!(matches!(
        rows_split[1].kind,
        SidebarRowKind::SplitChild {
            child_index: 0,
            is_last: false,
            ..
        }
    ));
    assert!(matches!(
        rows_split[2].kind,
        SidebarRowKind::SplitChild {
            child_index: 1,
            is_last: true,
            ..
        }
    ));

    // Verify active pane focus and switching
    assert_eq!(app.split_manager.active_pane_id, Some(pane2_id));
    app.focus_next_split();
    assert_eq!(app.split_manager.active_pane_id, Some(tab1_id));
    app.focus_next_split();
    assert_eq!(app.split_manager.active_pane_id, Some(pane2_id));

    // Test collapsible grouping
    app.sidebar.toggle_tab_collapse(tab1_id);
    let rows_collapsed = app
        .sidebar
        .compute_rows(app.tab_manager.tabs(), &app.split_trees, 0.0);
    assert_eq!(rows_collapsed.len(), 1);
    assert!(matches!(
        rows_collapsed[0].kind,
        SidebarRowKind::GroupHeader {
            is_collapsed: true,
            split_count: 2
        }
    ));

    // Re-expand group
    app.sidebar.toggle_tab_collapse(tab1_id);
    let rows_expanded = app
        .sidebar
        .compute_rows(app.tab_manager.tabs(), &app.split_trees, 0.0);
    assert_eq!(rows_expanded.len(), 3);

    // Hit test split child
    let child_row_y = rows_expanded[2].y + 4.0;
    let target = app.sidebar.hit_test(
        50.0,
        child_row_y,
        app.tab_manager.tabs(),
        &app.split_trees,
        0.0,
    );
    assert_eq!(
        target,
        Some(SidebarTarget::SplitPane {
            tab_idx: 0,
            pane_id: pane2_id,
        })
    );

    // Close split pane: group automatically updates, reverting to single session without grouping
    app.close_split_pane(pane2_id);
    let rows_after_close = app
        .sidebar
        .compute_rows(app.tab_manager.tabs(), &app.split_trees, 0.0);
    assert_eq!(rows_after_close.len(), 1);
    assert!(matches!(
        rows_after_close[0].kind,
        SidebarRowKind::NormalTab
    ));
}

#[test]
fn test_titlebar_split_indicator_and_grouping() {
    let mut app = App::new();
    let tab1_id = app.open_terminal_tab();

    // Initially 1 session: split count is 0 / 1 (no extra indicator)
    let initial_count = app
        .split_trees
        .get(&tab1_id)
        .map(|t| t.all_leaf_ids().len())
        .unwrap_or(0);
    assert_eq!(initial_count, 0);

    // Create split
    let pane2_id = app
        .split_active_pane(SplitDirection::Horizontal)
        .expect("must create split");
    let split_count = app
        .split_trees
        .get(&tab1_id)
        .map(|t| t.all_leaf_ids().len())
        .unwrap_or(0);
    assert_eq!(split_count, 2);

    let tab_title = app.tab_manager.tabs()[0].display_name().to_string();
    let titlebar_label = if split_count > 1 {
        format!("{tab_title} [{split_count}]")
    } else {
        tab_title.clone()
    };
    assert_eq!(titlebar_label, format!("{tab_title} [2]"));

    // Close split: indicator updates immediately
    app.close_split_pane(pane2_id);
    let updated_count = app
        .split_trees
        .get(&tab1_id)
        .map(|t| t.all_leaf_ids().len())
        .unwrap_or(0);
    assert_eq!(updated_count, 0);
    let updated_label = if updated_count > 1 {
        format!("{tab_title} [{updated_count}]")
    } else {
        tab_title.to_string()
    };
    assert_eq!(updated_label, tab_title);
}

#[test]
fn test_ctrl_w_and_ctrl_d_tab_closure() {
    let mut app = App::new();
    let _ = app.open_terminal_tab();
    let _ = app.open_terminal_tab();
    assert_eq!(app.tab_count(), 2);

    // Close one tab when multiple tabs exist
    app.close_active_tab_or_prompt();
    assert_eq!(app.tab_count(), 1);
    assert!(!app.is_exit_prompt_active());

    // When only 1 tab remains, closing prompts user to confirm before exit
    app.close_active_tab_or_prompt();
    assert_eq!(app.tab_count(), 1);
    assert!(app.is_exit_prompt_active());

    // Dismiss prompt
    app.dismiss_exit_prompt();
    assert!(!app.is_exit_prompt_active());
}

#[test]
fn test_sidebar_drag_resizing_and_persistence() {
    let mut sidebar = Sidebar::new();
    assert_eq!(sidebar.width, 240.0);
    assert_eq!(sidebar.saved_width, 240.0);

    // Simulate drag resize to 320px
    let new_w = 320.0f32.clamp(Sidebar::MIN_WIDTH, Sidebar::MAX_WIDTH);
    sidebar.width = new_w;
    sidebar.saved_width = new_w;
    assert_eq!(sidebar.width, 320.0);

    // Toggle collapse: width becomes 0.0, saved_width preserved
    sidebar.toggle();
    assert!(!sidebar.is_expanded);
    assert_eq!(sidebar.width, 0.0);
    assert_eq!(sidebar.saved_width, 320.0);

    // Toggle expand: restores saved_width
    sidebar.toggle();
    assert!(sidebar.is_expanded);
    assert_eq!(sidebar.width, 320.0);
}

#[test]
fn test_ctrl_d_active_split_closure() {
    let mut app = App::new();
    let tab1_id = app.open_terminal_tab();
    let pane2_id = app
        .split_active_pane(SplitDirection::Vertical)
        .expect("must create split");

    assert_eq!(
        app.split_trees.get(&tab1_id).unwrap().all_leaf_ids().len(),
        2
    );
    assert_eq!(app.active_pane_id(), Some(pane2_id));

    // When command buffer is empty, close split pane
    app.close_split_pane(pane2_id);
    assert!(!app.split_trees.contains_key(&tab1_id));
    assert_eq!(app.active_pane_id(), Some(tab1_id));

    // Closing the remaining session in the tab follows tab rules
    app.close_active_tab_or_prompt();
    assert!(app.is_exit_prompt_active());
}

#[test]
fn test_horizontal_and_vertical_splits_with_state_preservation() {
    let mut app = App::new();
    let tab1_id = app.open_terminal_tab();

    // Create vertical split
    let pane_v_id = app
        .split_active_pane(SplitDirection::Vertical)
        .expect("must create vertical split");

    // Create horizontal split from the active pane
    let pane_h_id = app
        .split_active_pane(SplitDirection::Horizontal)
        .expect("must create horizontal split");

    let tree = app.split_trees.get(&tab1_id).expect("must have split tree");
    let leaf_ids = tree.all_leaf_ids();
    assert_eq!(leaf_ids.len(), 3);
    assert!(leaf_ids.contains(&tab1_id));
    assert!(leaf_ids.contains(&pane_v_id));
    assert!(leaf_ids.contains(&pane_h_id));

    // Each pane has its own independent Block with separate shell/grid
    let blocks = app.active_tab_blocks();
    assert_eq!(blocks.len(), 3);
    let block_pane_ids: Vec<u64> = blocks.iter().filter_map(|b| b.pane_id).collect();
    assert!(block_pane_ids.contains(&tab1_id));
    assert!(block_pane_ids.contains(&pane_v_id));
    assert!(block_pane_ids.contains(&pane_h_id));
}

#[test]
fn test_pane_drag_repositioning_and_swap() {
    use akssora_terminal_ui::DropZone;

    let mut app = App::new();
    let tab1_id = app.open_terminal_tab();
    let pane2_id = app
        .split_active_pane(SplitDirection::Vertical)
        .expect("must create vertical split");

    // Test DropZone calculations
    let (px, py, pw, ph) = (100.0, 50.0, 400.0, 300.0);
    assert_eq!(
        DropZone::from_point(110.0, 200.0, px, py, pw, ph),
        DropZone::Left
    );
    assert_eq!(
        DropZone::from_point(490.0, 200.0, px, py, pw, ph),
        DropZone::Right
    );
    assert_eq!(
        DropZone::from_point(300.0, 60.0, px, py, pw, ph),
        DropZone::Top
    );
    assert_eq!(
        DropZone::from_point(300.0, 340.0, px, py, pw, ph),
        DropZone::Bottom
    );
    assert_eq!(
        DropZone::from_point(300.0, 200.0, px, py, pw, ph),
        DropZone::Center
    );

    // Verify preview rect matches expected geometry
    let (left_rx, _, left_rw, _) = DropZone::Left.preview_rect(px, py, pw, ph);
    assert_eq!(left_rx, px);
    assert_eq!(left_rw, pw * 0.5);

    // Swap panes via Center zone
    let swapped = app.reposition_pane(tab1_id, pane2_id, DropZone::Center);
    assert!(swapped);
    let leaves = app.split_trees.get(&tab1_id).unwrap().all_leaf_ids();
    assert_eq!(leaves, vec![pane2_id, tab1_id]);

    // Reposition to top split
    let split_top = app.reposition_pane(tab1_id, pane2_id, DropZone::Top);
    assert!(split_top);
    assert_eq!(app.split_manager.active_pane_id, Some(tab1_id));

    // Ensure session instances remain intact in active tab blocks
    let blocks_after = app.active_tab_blocks();
    assert_eq!(blocks_after.len(), 2);
}

#[test]
fn test_divider_minimum_size_enforcement() {
    use akssora_terminal_ui::{PaneTree, SplitDirection};

    let mut tree = PaneTree::new_leaf(1);
    tree.split_leaf(1, 2, SplitDirection::Vertical);

    // Layout on 300px total width
    let rects = tree.compute_rects(0.0, 0.0, 300.0, 200.0);
    assert_eq!(rects.len(), 2);

    // Divider at 50% gives ~148px each (> MIN_PANE_WIDTH = 120px)
    assert!(rects[0].3 >= PaneTree::MIN_PANE_WIDTH);
    assert!(rects[1].3 >= PaneTree::MIN_PANE_WIDTH);

    // Drag divider far to the left
    let dividers = tree.compute_dividers(0.0, 0.0, 300.0, 200.0);
    assert_eq!(dividers.len(), 1);
    tree.adjust_ratio(&dividers[0].5, -0.45);
    let clamped_rects = tree.compute_rects(0.0, 0.0, 300.0, 200.0);

    // Left pane cannot shrink below MIN_PANE_WIDTH or relative minimum
    assert!(clamped_rects[0].3 >= PaneTree::MIN_PANE_WIDTH.min(300.0 * 0.45) - 1.0);
}

#[test]
fn test_close_first_split_pane_recovering_remaining_terminal() {
    let mut app = App::new();
    app.config.tab_layout = akssora_terminal_config::TabLayoutPreference::Sidebar;
    app.sidebar.is_expanded = true;
    app.sidebar.width = 240.0;
    let tab1_id = app.open_terminal_tab();
    let pane2_id = app
        .split_active_pane(SplitDirection::Vertical)
        .expect("must create vertical split");

    // Rename pane 2 to test custom name preservation
    app.rename_pane(pane2_id, "Worker Pane");

    // Close the original tab1 pane instead of the split pane
    app.close_split_pane(tab1_id);

    // Remaining terminal must be pane2_id
    assert_eq!(app.active_pane_id(), Some(pane2_id));
    assert_eq!(app.split_manager.active_pane_id, Some(pane2_id));
    assert_eq!(app.tab_manager.active_id(), Some(pane2_id));

    // Split tree should be removed
    assert!(!app.split_trees.contains_key(&pane2_id));

    // Remaining terminal must be in active blocks
    let blocks = app.active_tab_blocks();
    assert_eq!(blocks.len(), 1);
    assert_eq!(blocks[0].pane_id, Some(pane2_id));

    // Geometry must occupy full content space
    let (win_w, win_h) = (app.config.window.width, app.config.window.height);
    let top_h = app
        .header
        .total_height(app.config.tab_layout.includes_title_bar());
    let expected_w = (win_w as f32 - app.sidebar.width) as u32;
    let expected_h = (win_h as f32 - top_h) as u32;
    assert_eq!(blocks[0].width, expected_w);
    assert_eq!(blocks[0].height, expected_h);

    // Sidebar rows should revert to single independent entry without grouping
    let rows = app
        .sidebar
        .compute_rows(app.tab_manager.tabs(), &app.split_trees, 0.0);
    assert_eq!(rows.len(), 1);
    assert!(matches!(rows[0].kind, SidebarRowKind::NormalTab));

    let items = app.collect_session_items();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].pane_id, pane2_id);
    assert_eq!(items[0].name, "Worker Pane");

    let session_rows = app.sidebar.compute_session_rows(&items, 0.0);
    assert_eq!(session_rows.len(), 1);
    assert!(matches!(
        session_rows[0].kind,
        SidebarRowKind::Session {
            is_split: false,
            ..
        }
    ));
}

#[test]
fn test_multi_split_closing_resize_and_state_preservation() {
    let mut app = App::new();
    app.config.tab_layout = akssora_terminal_config::TabLayoutPreference::Sidebar;
    app.sidebar.is_expanded = true;
    app.sidebar.width = 240.0;
    let tab1_id = app.open_terminal_tab();

    // 1. Create vertical split
    let pane_v = app
        .split_active_pane(SplitDirection::Vertical)
        .expect("must create vertical split");

    // 2. Create horizontal split from active pane
    let pane_h = app
        .split_active_pane(SplitDirection::Horizontal)
        .expect("must create horizontal split");

    assert_eq!(app.active_tab_blocks().len(), 3);
    assert_eq!(app.active_pane_id(), Some(pane_h));

    // Close horizontal split: layout resizes to 2 panes
    app.close_split_pane(pane_h);
    let blocks_2 = app.active_tab_blocks();
    assert_eq!(blocks_2.len(), 2);
    assert!(blocks_2.iter().any(|b| b.pane_id == Some(tab1_id)));
    assert!(blocks_2.iter().any(|b| b.pane_id == Some(pane_v)));
    assert!(app.active_pane_id() == Some(tab1_id) || app.active_pane_id() == Some(pane_v));

    // Close vertical split: single remaining terminal recovers full workspace
    app.close_split_pane(pane_v);
    let blocks_1 = app.active_tab_blocks();
    assert_eq!(blocks_1.len(), 1);
    assert_eq!(blocks_1[0].pane_id, Some(tab1_id));

    let (win_w, win_h) = (app.config.window.width, app.config.window.height);
    let top_h = app
        .header
        .total_height(app.config.tab_layout.includes_title_bar());
    let expected_w = (win_w as f32 - app.sidebar.width) as u32;
    let expected_h = (win_h as f32 - top_h) as u32;
    assert_eq!(blocks_1[0].width, expected_w);
    assert_eq!(blocks_1[0].height, expected_h);

    // Grouping must be cleaned up
    assert!(!app.split_trees.contains_key(&tab1_id));
    let rows = app
        .sidebar
        .compute_rows(app.tab_manager.tabs(), &app.split_trees, 0.0);
    assert_eq!(rows.len(), 1);
    assert!(matches!(rows[0].kind, SidebarRowKind::NormalTab));
}

#[test]
fn test_closed_terminals_removed_from_persistence() {
    let mut app = App::new();
    let temp_file = std::env::temp_dir().join(format!(
        "akssora_test_session_{}.json",
        rand::random::<u64>()
    ));
    app.session_file = Some(temp_file.clone());

    let tab1_id = app.open_terminal_tab();
    app.rename_pane(tab1_id, "Permanent Main");

    let split_id = app
        .split_active_pane(SplitDirection::Vertical)
        .expect("must create split");
    app.rename_pane(split_id, "Ephemeral Worker");

    // Close ephemeral worker split
    app.close_split_pane(split_id);

    // Verify session persistence file does not contain Ephemeral Worker
    assert!(temp_file.exists());
    let content = std::fs::read_to_string(&temp_file).expect("read temp session file");
    assert!(
        !content.contains("Ephemeral Worker"),
        "Closed terminal must not be in persisted state"
    );
    assert!(
        content.contains("Permanent Main"),
        "Active terminal must remain in persisted state"
    );
    let _ = std::fs::remove_file(&temp_file);
}

#[test]
fn test_sidebar_toggle_terminal_resize_and_reflow() {
    let mut app = App::new();
    app.config.tab_layout = akssora_terminal_config::TabLayoutPreference::Sidebar;
    app.config.window.width = 1200;
    app.config.window.height = 800;
    app.sidebar.is_expanded = true;
    app.sidebar.width = 240.0;

    let tab_id = app.open_terminal_tab();
    app.layout_blocks();

    // Verify initial single block layout with sidebar expanded
    let block1 = app.get_block(tab_id).expect("block exists");
    assert_eq!(block1.x, 240.0);
    assert_eq!(block1.width, 960);
    let initial_cols = block1.shell.grid().cols();

    // Write text that spans the width into the terminal grid
    if let Some(block) = app.get_block_mut(tab_id) {
        let grid = block.shell.grid_mut();
        for ch in "hello world long text command reflow test".chars() {
            grid.put_char(ch);
        }
    }

    // Toggle sidebar closed: terminal workspace widens to full window width
    app.sidebar.is_expanded = false;
    app.layout_blocks();

    let block_closed = app.get_block(tab_id).expect("block exists");
    assert_eq!(block_closed.x, 0.0);
    assert_eq!(block_closed.width, 1200);
    assert!(block_closed.shell.grid().cols() >= initial_cols);

    // Verify text content was preserved during widening
    let lines = block_closed.shell.grid().lines();
    let text: String = lines[0].iter().map(|c| c.ch).collect();
    assert!(text.contains("hello world long text"));

    // Create a vertical split view
    let split_id = app
        .split_active_pane(SplitDirection::Vertical)
        .expect("must create split");
    app.layout_blocks();

    // Verify both panes share the 1200px workspace minus the 4px divider
    let p1 = app.get_block(tab_id).expect("pane 1 exists");
    let p2 = app.get_block(split_id).expect("pane 2 exists");
    assert_eq!(p1.x, 0.0);
    assert_eq!(p1.width, 598);
    assert_eq!(p2.x, 602.0);
    assert_eq!(p2.width, 598);

    // Re-open sidebar to 300px: workspace shrinks to 900px, splits resize to (900-4)/2 = 448px each
    app.sidebar.is_expanded = true;
    app.sidebar.width = 300.0;
    app.layout_blocks();

    let p1_shrunk = app.get_block(tab_id).expect("pane 1 exists");
    let p2_shrunk = app.get_block(split_id).expect("pane 2 exists");
    assert_eq!(p1_shrunk.x, 300.0);
    assert_eq!(p1_shrunk.width, 448);
    assert_eq!(p2_shrunk.x, 752.0);
    assert_eq!(p2_shrunk.width, 448);

    // Verify text reflowed and characters remain intact without clipping
    let p1_lines = p1_shrunk.shell.grid().lines();
    let p1_text: String = p1_lines
        .iter()
        .flat_map(|r| r.iter().map(|c| c.ch))
        .collect();
    assert!(p1_text.contains("hello world"));
}

#[test]
fn test_ctrl_drag_pane_rearrangement_in_splits() {
    use akssora_terminal_ui::DropZone;

    let mut app = App::new();
    let tab_id = app.open_terminal_tab();

    // 1. Create a vertical split
    let pane_v_id = app
        .split_active_pane(SplitDirection::Vertical)
        .expect("must create vertical split");

    // 2. Create a nested horizontal split inside pane_v
    let pane_h_id = app
        .split_active_pane(SplitDirection::Horizontal)
        .expect("must create nested horizontal split");

    // Write commands and data to verify session preservation
    if let Some(b1) = app.get_block_mut(tab_id) {
        for ch in "session 1 data\n".chars() {
            b1.shell.grid_mut().put_char(ch);
        }
    }
    if let Some(b2) = app.get_block_mut(pane_v_id) {
        for ch in "session 2 data\n".chars() {
            b2.shell.grid_mut().put_char(ch);
        }
    }
    if let Some(b3) = app.get_block_mut(pane_h_id) {
        for ch in "session 3 data\n".chars() {
            b3.shell.grid_mut().put_char(ch);
        }
    }

    // Verify 3 distinct panes in active tab
    assert_eq!(app.active_tab_blocks().len(), 3);

    // Test drag cancellation: if drag distance < 10.0 or no hover target, nothing moves
    let b1_pos = {
        let b = app.get_block(tab_id).unwrap();
        (b.x + 40.0, b.y + 40.0)
    };
    app.set_test_modifiers(winit::keyboard::ModifiersState::CONTROL);
    app.set_test_cursor_pos(b1_pos.0, b1_pos.1);
    app.handle_test_mouse_press();
    assert!(app.split_manager.pane_drag.is_some());
    // Cancel release without moving
    app.handle_test_mouse_release();
    assert!(app.split_manager.pane_drag.is_none());

    // Rearrange nested pane (pane_h_id) to the Left of tab_id
    let rearranged = app.reposition_pane(pane_h_id, tab_id, DropZone::Left);
    assert!(rearranged);

    // Verify all 3 panes still exist and active pane received focus
    assert_eq!(app.active_tab_blocks().len(), 3);
    assert_eq!(app.active_pane_id(), Some(pane_h_id));

    // Verify sessions, PTYs, and text content were completely preserved
    let b1_text = app.get_block(tab_id).unwrap().shell.grid().all_text();
    let b2_text = app.get_block(pane_v_id).unwrap().shell.grid().all_text();
    let b3_text = app.get_block(pane_h_id).unwrap().shell.grid().all_text();
    assert!(b1_text.contains("session 1 data"));
    assert!(b2_text.contains("session 2 data"));
    assert!(b3_text.contains("session 3 data"));

    // Verify each pane can continue running commands after rearrangement
    if let Some(b3) = app.get_block_mut(pane_h_id) {
        for ch in "post-rearrange command\n".chars() {
            b3.shell.grid_mut().put_char(ch);
        }
    }
    assert!(
        app.get_block(pane_h_id)
            .unwrap()
            .shell
            .grid()
            .all_text()
            .contains("post-rearrange command")
    );
}
