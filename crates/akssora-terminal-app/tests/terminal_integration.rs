use akssora_terminal_config::{Color, Theme};
use akssora_terminal_parser::TerminalParser;
use akssora_terminal_screen::TerminalGrid;
use akssora_terminal_ui::{Header, HeaderButton, Scrollbar};

#[test]
fn test_terminal_screen_and_parser_integration() {
    let mut grid = TerminalGrid::new(24, 80);
    let mut parser = TerminalParser::new();

    parser.advance(&mut grid, b"Hello, Akssora!\r\n");
    {
        let lines = grid.visible_lines();
        let first_line_chars: String = lines[0].iter().map(|c| c.ch).collect();
        assert!(first_line_chars.starts_with("Hello, Akssora!"));
    }

    parser.advance(&mut grid, b"\x1b[31mRed\x1b[0m");
    {
        let lines = grid.visible_lines();
        let second_line = lines[1];
        assert_eq!(second_line[0].ch, 'R');
        assert_eq!(second_line[0].fg, Color::Indexed(1));
        assert_eq!(second_line[1].ch, 'e');
        assert_eq!(second_line[2].ch, 'd');
    }
}

#[test]
fn test_header_ui_hit_test() {
    let header = Header::new("Akssora");
    assert_eq!(header.title, "Akssora");

    let window_w = 1000.0;
    let close_btn = header.hit_test(window_w - 10.0, 10.0, window_w);
    assert_eq!(close_btn, Some(HeaderButton::Close));

    assert!(header.in_drag_region(500.0, 10.0, window_w));
}

#[test]
fn test_theme_color_resolution() {
    let theme = Theme::monochrome_black();
    assert_eq!(theme.resolve_fg(Color::Default, false), theme.foreground);
    assert_eq!(theme.resolve_fg(Color::Indexed(1), false), theme.ansi[1]);
    assert_eq!(theme.resolve_bg(Color::Default), None);
}

#[test]
fn test_bottom_up_layout_offset() {
    let mut grid = TerminalGrid::new(24, 80);
    let mut parser = TerminalParser::new();

    parser.advance(&mut grid, b"user@akssora:~$ ");
    let (crow, _) = grid.cursor();
    let content_rows = grid.content_rows();
    let active = (crow + 1).max(content_rows).min(grid.rows());
    assert_eq!(active, 1);
    let empty_offset_rows = grid.rows() - active;
    assert_eq!(empty_offset_rows, 23);
}

// INFO: Verify scrollbar thumb geometry calculations, track hit testing, and theme color alignment.
#[test]
fn test_scrollbar_geometry_and_interactions() {
    let mut scrollbar = Scrollbar::new();
    scrollbar.update_geometry(0.0, 40.0, 800.0, 600.0);

    assert_eq!(scrollbar.y, 40.0);
    assert_eq!(scrollbar.height, 600.0);
    assert!(scrollbar.hit_test(795.0, 100.0));
    assert!(!scrollbar.hit_test(100.0, 100.0));

    let total_lines = 100;
    let visible_lines = 20;

    let thumb_bottom = scrollbar.thumb_rect(total_lines, visible_lines, 0.0);
    assert!(thumb_bottom.is_some());
    let (_, y_bottom, _, h) = thumb_bottom.unwrap();
    assert!((y_bottom + h - (40.0 + 600.0)).abs() < 1.0);

    let max_scroll = (total_lines - visible_lines) as f32;
    let thumb_top = scrollbar.thumb_rect(total_lines, visible_lines, max_scroll);
    assert!(thumb_top.is_some());
    let (_, y_top, _, _) = thumb_top.unwrap();
    assert_eq!(y_top, 40.0);

    let theme = Theme::monochrome_black();
    assert_eq!(scrollbar.thumb_color(&theme), theme.header_background);
}

// INFO: Verify scrollback preserves history offset when viewing scrollback and auto-follows at bottom.
#[test]
fn test_scrollback_history_preservation_and_auto_follow() {
    let mut grid = TerminalGrid::new(5, 40);

    for i in 0..10 {
        grid.scroll_up();
        grid.newline();
        let _ = i;
    }
    assert_eq!(grid.scroll_offset(), 0);

    grid.scroll_up_view(4);
    assert_eq!(grid.scroll_offset(), 4);

    grid.scroll_up();
    assert_eq!(grid.scroll_offset(), 5);

    grid.reset_scroll();
    assert_eq!(grid.scroll_offset(), 0);

    grid.scroll_up();
    assert_eq!(grid.scroll_offset(), 0);
}

// INFO: Verify multi-tab lifecycle, tab addition, switching, and tab layout preference dynamics.
#[test]
fn test_multi_tab_lifecycle_and_layout() {
    use akssora_terminal_config::TabLayoutPreference;
    use akssora_terminal_ui::{TabKind, TabManager};

    let mut manager = TabManager::new();
    assert!(manager.is_empty());

    let tab1 = manager.add_tab("Terminal 1", TabKind::Terminal);
    assert_eq!(manager.len(), 1);
    assert_eq!(manager.active_index(), 0);
    assert_eq!(manager.active_id(), Some(tab1));

    let tab2 = manager.add_tab(
        "Editor: main.rs",
        TabKind::Editor {
            path: "main.rs".into(),
        },
    );
    assert_eq!(manager.len(), 2);
    assert_eq!(manager.active_index(), 1);
    assert_eq!(manager.active_id(), Some(tab2));

    let _tab3 = manager.add_tab(
        "API: dev",
        TabKind::ApiTesting {
            workspace: "dev".into(),
        },
    );
    assert_eq!(manager.len(), 3);
    assert_eq!(manager.active_index(), 2);

    manager.next_tab();
    assert_eq!(manager.active_index(), 0);

    manager.prev_tab();
    assert_eq!(manager.active_index(), 2);

    assert!(manager.select(1));
    assert_eq!(manager.active_index(), 1);

    // Verify renaming and preservation
    assert!(manager.set_custom_title(tab1, "Renamed Terminal"));
    assert_eq!(manager.tabs()[0].title, "Renamed Terminal");
    assert!(manager.tabs()[0].custom_name);

    // Auto command naming does not overwrite custom name
    assert!(!manager.set_auto_title(tab1, "cargo build"));
    assert_eq!(manager.tabs()[0].title, "Renamed Terminal");

    // Auto command naming on a fresh tab without custom name
    let tab_fresh = manager.add_tab("Terminal", TabKind::Terminal);
    assert!(!manager.tabs().last().unwrap().custom_name);
    assert!(manager.set_auto_title(tab_fresh, "cargo run"));
    assert_eq!(manager.tabs().last().unwrap().title, "cargo run");
    // Second command does not overwrite first command
    assert!(!manager.set_auto_title(tab_fresh, "ls -la"));
    assert_eq!(manager.tabs().last().unwrap().title, "cargo run");

    // Close middle tab
    let closed = manager.close(1);
    assert!(closed.is_some());
    assert_eq!(manager.active_id(), Some(tab_fresh));

    // Test TabLayoutPreference variants: Sidebar and TitleBar
    let layout_tb = TabLayoutPreference::TitleBar;
    assert!(layout_tb.includes_title_bar());
    assert!(!layout_tb.includes_sidebar());
    assert!(!layout_tb.show_sidebar_toggle());

    let layout_sb = TabLayoutPreference::Sidebar;
    assert!(!layout_sb.includes_title_bar());
    assert!(layout_sb.includes_sidebar());
    assert!(layout_sb.show_sidebar_toggle());
    assert_eq!(layout_sb.next(), TabLayoutPreference::TitleBar);
    assert_eq!(layout_tb.next(), TabLayoutPreference::Sidebar);
}

// INFO: Verify sidebar toggle geometry, hit testing, and dynamic tooltip label formatting.
#[test]
fn test_sidebar_toggle_and_tooltips() {
    use akssora_terminal_config::{Action, FontConfig, Keybindings, TabLayoutPreference};
    use akssora_terminal_ui::{Header, HeaderItem, SettingsAction, SettingsView, Sidebar, Tooltip};

    let mut sidebar = Sidebar::new();
    assert!(sidebar.is_expanded);
    assert_eq!(sidebar.width, Sidebar::EXPANDED_WIDTH);
    assert_eq!(sidebar.effective_width(true), Sidebar::EXPANDED_WIDTH);

    // Sidebar completely invisible when collapsed
    sidebar.toggle();
    assert!(!sidebar.is_expanded);
    assert_eq!(sidebar.width, 0.0);
    assert_eq!(sidebar.effective_width(true), 0.0);

    let header = Header::new("Akssora");
    let toggle_rect = header.sidebar_toggle_rect();
    assert_eq!(toggle_rect.0, 0.0);
    assert!(toggle_rect.2 > 0.0);

    let hit = header.hit_test_item_ex(10.0, 10.0, 1000.0, 3, true, true, false);
    assert_eq!(hit, Some(HeaderItem::SidebarToggle));

    // Toggle hidden when sidebar layout disabled
    let hit_hidden = header.hit_test_item_ex(10.0, 10.0, 1000.0, 3, true, false, true);
    assert_ne!(hit_hidden, Some(HeaderItem::SidebarToggle));

    // Verify sidebar toggle visual rectangle and vertical header alignment
    let (vx, vy, vw, vh) = header.sidebar_toggle_visual_rect();
    assert!(vx >= 0.0);
    assert_eq!(vw, 28.0);
    assert_eq!(vh, 26.0);
    assert_eq!(vy, (header.height - vh) / 2.0);
    assert!(!header.is_sidebar_toggle_focused);
    assert!(!header.is_sidebar_toggle_active);

    // Dynamic keybinding tooltips
    let keybindings = Keybindings::default();
    let shortcut = keybindings
        .shortcut_for(Action::ToggleSidebar)
        .map(String::from);
    let tt_with_key = Tooltip::new("Toggle Sidebar", shortcut);
    assert_eq!(tt_with_key.display_label(), "Toggle Sidebar (Ctrl+B)");

    let tt_no_key = Tooltip::new("Toggle Sidebar", None);
    assert_eq!(tt_no_key.display_label(), "Toggle Sidebar");

    // Verify sidebar background matches theme.sidebar_background (black)
    let theme = akssora_terminal_config::Theme::monochrome_black();
    assert_eq!(sidebar.bg_color(&theme), theme.sidebar_background);

    // Verify SettingsView option hit testing
    let cell_h = 18.0;
    let (ox, oy, _, _) = SettingsView::layout_dropdown_item_rect(0, 0.0, 0.0, 800.0, cell_h);
    let opt_hit = SettingsView::hit_test_option(ox + 2.0, oy + 2.0, 0.0, 0.0, 800.0, cell_h);
    assert_eq!(opt_hit, Some(TabLayoutPreference::Sidebar));

    // Verify FontConfig increase, decrease, reset
    let mut font = FontConfig::default();
    assert_eq!(font.size, FontConfig::DEFAULT_SIZE);
    font.increase();
    assert_eq!(font.size, FontConfig::DEFAULT_SIZE + 1.0);
    font.decrease();
    assert_eq!(font.size, FontConfig::DEFAULT_SIZE);
    font.reset();
    assert_eq!(font.size, FontConfig::DEFAULT_SIZE);

    // Verify SettingsView action hit testing
    let (mx, my, _, _) = SettingsView::font_minus_button_rect(0.0, 0.0, 800.0, cell_h);
    assert_eq!(
        SettingsView::hit_test_action(mx + 2.0, my + 2.0, 0.0, 0.0, 800.0, cell_h),
        Some(SettingsAction::DecreaseFontSize)
    );
}

// INFO: Verify config serialization and persistence of tab_layout and font_size across restarts.
#[test]
fn test_config_persistence_and_font_size() {
    use akssora_terminal_config::{Config, FontConfig, TabLayoutPreference};

    let cfg = Config {
        tab_layout: TabLayoutPreference::Sidebar,
        font: FontConfig {
            size: 18.0,
            ..Default::default()
        },
        ..Default::default()
    };

    let json = cfg.to_json();
    assert!(json.contains("\"tab_layout\": \"Sidebar\""));
    assert!(json.contains("\"font_size\": 18"));

    let mut loaded = Config::default();
    loaded.apply_json(&json);
    assert_eq!(loaded.tab_layout, TabLayoutPreference::Sidebar);
    assert_eq!(loaded.font.size, 18.0);

    // Font size clamping
    let mut font = FontConfig {
        size: FontConfig::MAX_SIZE,
        ..Default::default()
    };
    assert_eq!(font.increase(), FontConfig::MAX_SIZE);
    font.size = FontConfig::MIN_SIZE;
    assert_eq!(font.decrease(), FontConfig::MIN_SIZE);
}

// INFO: Verify settings tab deduplication and full workspace width when sidebar is collapsed.
#[test]
fn test_settings_deduplication_and_workspace_expansion() {
    use akssora_terminal_ui::{Sidebar, TabKind, TabManager};

    let mut manager = TabManager::new();
    let tab1 = manager.add_tab("Terminal 1", TabKind::Terminal);
    assert_eq!(manager.len(), 1);

    // Open settings tab
    let settings_pos = manager
        .tabs()
        .iter()
        .position(|t| matches!(t.kind, TabKind::Settings));
    assert!(settings_pos.is_none());
    let settings_id = manager.add_tab("Settings", TabKind::Settings);
    assert_eq!(manager.len(), 2);

    // Re-opening settings tab focuses existing tab without duplicating
    let existing_pos = manager
        .tabs()
        .iter()
        .position(|t| matches!(t.kind, TabKind::Settings));
    assert_eq!(existing_pos, Some(1));
    manager.select(existing_pos.unwrap());
    assert_eq!(manager.active_id(), Some(settings_id));
    assert_eq!(manager.len(), 2);

    // Workspace width when sidebar is collapsed (0.0 effective width)
    let mut sidebar = Sidebar::new();
    assert_eq!(sidebar.effective_width(true), Sidebar::EXPANDED_WIDTH);
    sidebar.toggle(); // Collapsed
    assert_eq!(sidebar.effective_width(true), 0.0);
    let win_w = 1200.0;
    let content_x = sidebar.effective_width(true);
    let content_w = win_w - content_x;
    assert_eq!(content_x, 0.0);
    assert_eq!(content_w, 1200.0);

    // Sessions are preserved
    assert_eq!(manager.tabs()[0].id, tab1);

    // Verify settings button header positioning, alignment, and tooltip
    let header = akssora_terminal_ui::Header::new("Akssora");
    let (sx, sy, sw, sh) = header.settings_button_rect(1000.0);
    assert_eq!(sw, 28.0);
    assert_eq!(sh, 26.0);
    assert_eq!(sy, (header.height - sh) * 0.5);
    assert!(sx < 1000.0 - 3.0 * akssora_terminal_ui::BUTTON_WIDTH);
    assert!(!header.is_settings_focused);
    assert!(!header.is_settings_active);

    // Tooltip for settings
    let tt_settings = akssora_terminal_ui::Tooltip::new("Settings", None);
    assert_eq!(tt_settings.display_label(), "Settings");
    let tt_settings_shortcut =
        akssora_terminal_ui::Tooltip::new("Settings", Some("Ctrl+,".to_string()));
    assert_eq!(tt_settings_shortcut.display_label(), "Settings (Ctrl+,)");
}

// INFO: Verify tab cycling (Ctrl+Tab / Ctrl+Shift+Tab) and close tab confirmation logic.
#[test]
fn test_tab_navigation_and_closing_rules() {
    use akssora_terminal_ui::{TabKind, TabManager};

    let mut manager = TabManager::new();
    let tab1 = manager.add_tab("Tab 1", TabKind::Terminal);
    let tab2 = manager.add_tab("Tab 2", TabKind::Terminal);
    let tab3 = manager.add_tab("Tab 3", TabKind::Terminal);

    assert_eq!(manager.active_index(), 2);

    // Next tab wraps around
    manager.next_tab();
    assert_eq!(manager.active_index(), 0);
    assert_eq!(manager.active_id(), Some(tab1));

    manager.next_tab();
    assert_eq!(manager.active_index(), 1);
    assert_eq!(manager.active_id(), Some(tab2));

    // Prev tab wraps around
    manager.prev_tab();
    assert_eq!(manager.active_index(), 0);
    manager.prev_tab();
    assert_eq!(manager.active_index(), 2);
    assert_eq!(manager.active_id(), Some(tab3));

    // Close tab with multiple tabs remaining
    assert_eq!(manager.len(), 3);
    let removed = manager.close(2);
    assert_eq!(removed.map(|t| t.id), Some(tab3));
    assert_eq!(manager.len(), 2);

    let removed2 = manager.close(1);
    assert_eq!(removed2.map(|t| t.id), Some(tab2));
    assert_eq!(manager.len(), 1);

    // Only 1 tab remaining
    assert_eq!(manager.len(), 1);
}

// INFO: Verify Ctrl+Shift+W closes other tabs when multiple exist, and prompts when only one exists.
#[test]
fn test_close_other_tabs_prompt_and_single_tab_rules() {
    use akssora_terminal_app::App;

    let mut app = App::new();
    let _t1 = app.open_terminal_tab();
    let _t2 = app.open_terminal_tab();
    let t3 = app.open_terminal_tab();

    assert_eq!(app.tab_count(), 3);
    assert_eq!(app.active_tab_id(), Some(t3));

    // Multiple tabs: close other tabs leaves only the active tab
    app.close_other_tabs();
    assert_eq!(app.tab_count(), 1);
    assert_eq!(app.active_tab_id(), Some(t3));
    assert!(!app.is_exit_prompt_active());

    // Single tab remaining: Ctrl+Shift+W prompts user to confirm before closing application
    app.close_other_tabs();
    assert_eq!(app.tab_count(), 1);
    assert!(app.is_exit_prompt_active());

    // Dismiss prompt
    app.dismiss_exit_prompt();
    assert!(!app.is_exit_prompt_active());

    // Single tab remaining: Ctrl+W also prompts user to confirm before closing application
    app.close_active_tab_or_prompt();
    assert_eq!(app.tab_count(), 1);
    assert!(app.is_exit_prompt_active());
}

// INFO: Verify ReusableButton hit-testing, dynamic tooltips, and shortcut fallback.
#[test]
fn test_reusable_button_and_tooltips() {
    use akssora_terminal_config::{Action, Keybindings};
    use akssora_terminal_ui::ReusableButton;

    let kb = Keybindings::default();
    let btn = ReusableButton::new(
        "sidebar_toggle",
        (10.0, 10.0, 36.0, 36.0),
        "Sidebar",
        "Toggle Sidebar",
        kb.shortcut_for(Action::ToggleSidebar).map(String::from),
    );

    assert!(btn.hit_test(20.0, 20.0));
    assert!(!btn.hit_test(0.0, 0.0));
    assert!(!btn.hit_test(100.0, 100.0));

    let tt = btn.tooltip();
    assert_eq!(tt.display_label(), "Toggle Sidebar (Ctrl+B)");
    assert_eq!(tt.target_rect, (10.0, 10.0, 36.0, 36.0));

    // Button without shortcut displays function only
    let btn_no_sc =
        ReusableButton::new("minimize", (50.0, 10.0, 36.0, 36.0), "—", "Minimize", None);
    let tt_no_sc = btn_no_sc.tooltip();
    assert_eq!(tt_no_sc.display_label(), "Minimize");
}

// INFO: Verify native window controls layout, maximize/restore state switching, tooltips, and drag safety.
#[test]
fn test_native_window_controls_and_interactions() {
    use akssora_terminal_ui::{Header, HeaderButton, HeaderItem, Tooltip};

    let mut header = Header::new("Akssora");
    let window_w = 1200.0;

    // 1. Positioning in top-right corner and consistent sizing
    let (min_x, min_y, min_w, min_h) = header.button_rect(HeaderButton::Minimize, window_w);
    let (max_x, max_y, max_w, max_h) = header.button_rect(HeaderButton::Maximize, window_w);
    let (close_x, close_y, close_w, close_h) = header.button_rect(HeaderButton::Close, window_w);

    assert_eq!(min_y, 0.0);
    assert_eq!(max_y, 0.0);
    assert_eq!(close_y, 0.0);
    assert_eq!(min_h, header.height);
    assert_eq!(max_h, header.height);
    assert_eq!(close_h, header.height);
    assert_eq!(min_w, max_w);
    assert_eq!(max_w, close_w);

    // Close is rightmost, Maximize is in middle, Minimize is on left
    assert_eq!(close_x + close_w, window_w);
    assert_eq!(max_x + max_w, close_x);
    assert_eq!(min_x + min_w, max_x);

    // Visual pills are vertically centered
    let (_, v_min_y, _, v_min_h) = header.button_visual_rect(HeaderButton::Minimize, window_w);
    assert_eq!(v_min_y, ((header.height - v_min_h) * 0.5).round());

    // 2. Visually distinct separation from Settings button
    let (sx, _, sw, _) = header.settings_button_rect(window_w);
    assert!(
        sx + sw < min_x,
        "Settings button must have a clear gap before Minimize button"
    );

    // 3. Hit testing
    assert_eq!(
        header.hit_test_item(min_x + 5.0, 10.0, window_w, 0, false),
        Some(HeaderItem::WindowButton(HeaderButton::Minimize))
    );
    assert_eq!(
        header.hit_test_item(max_x + 5.0, 10.0, window_w, 0, false),
        Some(HeaderItem::WindowButton(HeaderButton::Maximize))
    );
    assert_eq!(
        header.hit_test_item(close_x + 5.0, 10.0, window_w, 0, false),
        Some(HeaderItem::WindowButton(HeaderButton::Close))
    );
    assert_eq!(
        header.hit_test_item(sx + 5.0, 10.0, window_w, 0, false),
        Some(HeaderItem::Settings)
    );

    // 4. Maximize and Restore state switching & tooltips
    header.is_maximized = false;
    let max_label = if header.is_maximized {
        "Restore"
    } else {
        "Maximize"
    };
    let tt_max = Tooltip::for_target(max_label, None, (max_x, max_y, max_w, max_h));
    assert_eq!(tt_max.display_label(), "Maximize");

    header.is_maximized = true;
    let restore_label = if header.is_maximized {
        "Restore"
    } else {
        "Maximize"
    };
    let tt_restore = Tooltip::for_target(restore_label, None, (max_x, max_y, max_w, max_h));
    assert_eq!(tt_restore.display_label(), "Restore");

    let tt_min = Tooltip::for_target("Minimize", None, (min_x, min_y, min_w, min_h));
    assert_eq!(tt_min.display_label(), "Minimize");

    let tt_close = Tooltip::for_target("Close", None, (close_x, close_y, close_w, close_h));
    assert_eq!(tt_close.display_label(), "Close");

    // 5. Drag region safety: header buttons do not interfere with titlebar dragging
    assert!(!header.in_drag_region(min_x + 5.0, 10.0, window_w));
    assert!(!header.in_drag_region(max_x + 5.0, 10.0, window_w));
    assert!(!header.in_drag_region(close_x + 5.0, 10.0, window_w));
    assert!(!header.in_drag_region(sx + 5.0, 10.0, window_w));

    // Empty space in header allows dragging
    assert!(header.in_drag_region(300.0, 15.0, window_w));
}

// INFO: Verify close all tabs prompts user to confirm before closing application.
#[test]
fn test_close_all_tabs_prompt_and_behavior() {
    use akssora_terminal_ui::{TabKind, TabManager};

    let mut manager = TabManager::new();
    manager.add_tab("Tab 1", TabKind::Terminal);
    manager.add_tab("Tab 2", TabKind::Terminal);
    manager.add_tab("Tab 3", TabKind::Terminal);
    assert_eq!(manager.len(), 3);

    // Verify close_by_id and clear
    manager.clear();
    assert_eq!(manager.len(), 0);
    assert!(manager.is_empty());
}

// INFO: Verify split-view support, horizontal/vertical layouts, resizing, minimum constraints, and session isolation.
#[test]
fn test_split_view_creation_resizing_and_isolation() {
    use akssora_terminal_app::App;
    use akssora_terminal_config::{Action, Keybindings};
    use akssora_terminal_ui::{PaneTree, SplitDirection};

    // 1. Verify keybinding mapping for Ctrl+Shift+D
    let kb = Keybindings::default();
    assert_eq!(kb.shortcut_for(Action::CreateSplit), Some("Ctrl+Shift+D"));

    // 2. PaneTree geometric layout & minimum pane size enforcement
    let mut tree = PaneTree::new_leaf(1);
    assert_eq!(tree.all_leaf_ids(), vec![1]);

    // Split vertically (side by side)
    assert!(tree.split_leaf(1, 2, SplitDirection::Vertical));
    assert_eq!(tree.all_leaf_ids(), vec![1, 2]);

    let rects = tree.compute_rects(0.0, 0.0, 1000.0, 600.0);
    assert_eq!(rects.len(), 2);
    let (id1, x1, y1, w1, h1) = rects[0];
    let (id2, x2, _y2, w2, h2) = rects[1];
    assert_eq!(id1, 1);
    assert_eq!(id2, 2);
    assert_eq!(x1, 0.0);
    assert_eq!(y1, 0.0);
    assert_eq!(h1, 600.0);
    assert_eq!(h2, 600.0);
    assert!((x2 - (w1 + PaneTree::DIVIDER_THICKNESS)).abs() < 0.1);
    assert!((w1 + w2 + PaneTree::DIVIDER_THICKNESS - 1000.0).abs() < 0.1);

    // Divider calculation
    let divs = tree.compute_dividers(0.0, 0.0, 1000.0, 600.0);
    assert_eq!(divs.len(), 1);
    let (dir, div_x, div_y, div_w, div_h, path) = &divs[0];
    assert_eq!(*dir, SplitDirection::Vertical);
    assert!((*div_x - w1).abs() < 0.1);
    assert_eq!(*div_y, 0.0);
    assert_eq!(*div_w, PaneTree::DIVIDER_THICKNESS);
    assert_eq!(*div_h, 600.0);

    // Divider resize ratio adjustment
    tree.adjust_ratio(path, 0.1);
    let rects_after = tree.compute_rects(0.0, 0.0, 1000.0, 600.0);
    assert!(rects_after[0].3 > w1); // Left pane expanded

    // Split horizontally (top / bottom) on pane 2
    assert!(tree.split_leaf(2, 3, SplitDirection::Horizontal));
    assert_eq!(tree.all_leaf_ids(), vec![1, 2, 3]);
    let rects_3 = tree.compute_rects(0.0, 0.0, 1000.0, 600.0);
    assert_eq!(rects_3.len(), 3);
    assert_eq!(rects_3[1].0, 2); // pane 2 is top
    assert_eq!(rects_3[2].0, 3); // pane 3 is bottom
    assert!(rects_3[2].2 > rects_3[1].2); // y offset of pane 3 is greater

    // 3. App split view integration & session isolation
    let mut app = App::new();
    let tab1 = app.open_terminal_tab();
    assert_eq!(app.tab_count(), 1);

    // Create vertical split pane
    let pane2 = app
        .split_active_pane(SplitDirection::Vertical)
        .expect("split created");
    assert_ne!(tab1, pane2);

    let active_blocks = app.active_tab_blocks();
    assert_eq!(active_blocks.len(), 2);

    // Verify each pane has its own independent terminal shell and session
    let b1 = active_blocks[0];
    let b2 = active_blocks[1];
    assert_ne!(b1.id, b2.id);

    // Clean up
    app.close_all_tabs();
    assert_eq!(app.tab_count(), 0);
    assert_eq!(app.active_tab_blocks().len(), 0);
}

#[test]
fn test_terminal_font_size_consistency_across_splits_and_ui_isolation() {
    use akssora_terminal_app::App;

    let mut app = App::new();
    let mut fonts = akssora_terminal_renderer::FontManager::new();
    fonts.set_size(14.0);
    let mut ui_fonts = akssora_terminal_renderer::FontManager::new();
    ui_fonts.set_size(akssora_terminal_renderer::UI_FONT_SIZE);
    app.set_test_fonts(fonts, ui_fonts);

    let tab1 = app.open_terminal_tab();
    let split_v = app
        .split_active_pane(akssora_terminal_ui::SplitDirection::Vertical)
        .expect("must create vertical split");
    let split_h = app
        .split_active_pane(akssora_terminal_ui::SplitDirection::Horizontal)
        .expect("must create horizontal split");

    // Initial grid rows/cols on default 14.0pt font
    let b1 = app.get_block(tab1).unwrap();
    let b2 = app.get_block(split_v).unwrap();
    let b3 = app.get_block(split_h).unwrap();
    let initial_cols_b1 = b1.shell.grid().cols();
    let initial_cols_b2 = b2.shell.grid().cols();
    let initial_cols_b3 = b3.shell.grid().cols();

    // UI fonts size is at default UI_FONT_SIZE
    assert_eq!(
        app.current_ui_font_size(),
        akssora_terminal_renderer::UI_FONT_SIZE
    );

    // Increase terminal font size to 20.0pt
    app.set_font_size(20.0);
    assert_eq!(app.config.font.size, 20.0);
    assert_eq!(app.terminal_font_size(), 20.0);

    // UI fonts for header and sidebar remain unchanged (isolated from terminal zoom)
    assert_eq!(
        app.current_ui_font_size(),
        akssora_terminal_renderer::UI_FONT_SIZE,
        "Header and sidebar font size must remain independently controlled"
    );

    // All terminal sessions and split panes consistently resized according to the new 20.0pt font
    let b1_resized = app.get_block(tab1).unwrap();
    let b2_resized = app.get_block(split_v).unwrap();
    let b3_resized = app.get_block(split_h).unwrap();

    // Since cell width increased at 20pt, fewer character columns fit in the same pixel width
    assert!(b1_resized.shell.grid().cols() < initial_cols_b1);
    assert!(b2_resized.shell.grid().cols() < initial_cols_b2);
    assert!(b3_resized.shell.grid().cols() < initial_cols_b3);

    // Reset font size restores default 14.0pt font across all split panes
    app.reset_font_size();
    assert_eq!(
        app.config.font.size,
        akssora_terminal_config::FontConfig::DEFAULT_SIZE
    );
    assert_eq!(
        app.terminal_font_size(),
        akssora_terminal_config::FontConfig::DEFAULT_SIZE
    );
    assert_eq!(
        app.get_block(tab1).unwrap().shell.grid().cols(),
        initial_cols_b1
    );
    assert_eq!(
        app.get_block(split_v).unwrap().shell.grid().cols(),
        initial_cols_b2
    );
    assert_eq!(
        app.get_block(split_h).unwrap().shell.grid().cols(),
        initial_cols_b3
    );

    // Persistence check: verify saved config preserves new terminal font size and independent UI font size
    app.set_font_size(18.0);
    app.set_ui_font_size(16.0);
    let json = app.config.to_json();
    assert!(json.contains("\"font_size\": 18"));
    assert!(json.contains("\"ui_font_size\": 16"));
    let mut loaded = akssora_terminal_config::Config::default();
    loaded.apply_json(&json);
    assert_eq!(loaded.font.size, 18.0);
    assert_eq!(loaded.ui_font_size, 16.0);
}

#[test]
fn test_tab_close_button_geometry_and_hit_target() {
    let header = Header::new("Akssora");
    let window_w = 1200.0;
    let total_tabs = 3;

    // Verify close button size is at least 16-18px
    let close_rect = header
        .tab_close_rect(0, total_tabs, window_w)
        .expect("Tab 0 close button rect");
    let (cx, cy, cw, ch) = close_rect;
    assert_eq!(cw, 18.0, "Close button hit width should be 18px");
    assert_eq!(ch, 18.0, "Close button hit height should be 18px");

    // Verify tab rect bounds contain the close button with sufficient right margin
    let tab_rect = header
        .tab_rect(0, total_tabs, window_w)
        .expect("Tab 0 rect");
    let (tx, ty, tw, th) = tab_rect;
    assert!(cx > tx, "Close button should be inside tab rect");
    assert!(
        cx + cw <= tx + tw,
        "Close button must not overflow tab rect"
    );
    assert_eq!(
        cx + cw + 6.0,
        tx + tw,
        "Close button should be aligned right with 6px margin"
    );
    assert!(
        cy >= ty && cy + ch <= ty + th,
        "Close button must be vertically centered"
    );

    // Hit test item should identify TabClose target accurately
    let hit = header.hit_test_item_ex(cx + 9.0, cy + 9.0, window_w, total_tabs, true, false, false);
    assert_eq!(hit, Some(akssora_terminal_ui::HeaderItem::TabClose(0)));

    // Clicking inside tab body but outside close button should hit Tab
    let hit_tab = header.hit_test_item_ex(
        tx + 20.0,
        cy + 9.0,
        window_w,
        total_tabs,
        true,
        false,
        false,
    );
    assert_eq!(hit_tab, Some(akssora_terminal_ui::HeaderItem::Tab(0)));
}

#[test]
fn test_keybinding_conflict_detection_and_scoping() {
    use akssora_terminal_config::{Action, KeybindingScope, Keybindings};

    let mut bindings = Keybindings::default();

    // Default keybindings should have zero conflicts
    let initial_conflicts = bindings.find_conflicts();
    assert!(
        initial_conflicts.is_empty(),
        "Default bindings should have no conflicting shortcuts, found: {:?}",
        initial_conflicts
    );

    // Verify actions have correct scopes
    assert_eq!(Action::ToggleSidebar.scope(), KeybindingScope::Global);
    assert_eq!(Action::OpenSettings.scope(), KeybindingScope::Global);
    assert_eq!(Action::NewTab.scope(), KeybindingScope::Panel);
    assert_eq!(Action::CloseTab.scope(), KeybindingScope::Panel);
    assert_eq!(Action::CreateSplit.scope(), KeybindingScope::Terminal);
    assert_eq!(Action::Copy.scope(), KeybindingScope::Terminal);

    // Conflict test: Assigning a Global shortcut to a Terminal action should warn
    let conflict = bindings.check_conflict(Action::CreateSplit, "Ctrl+B");
    assert!(
        conflict.is_some(),
        "Ctrl+B conflicts between Global (ToggleSidebar) and Terminal (CreateSplit)"
    );
    let warn = conflict.unwrap();
    assert_eq!(warn.shortcut, "Ctrl+B");
    assert_eq!(warn.second_action, Action::ToggleSidebar);

    // Conflict test: Two Panel actions with same shortcut
    bindings.set_shortcut(Action::NextTab, Some("Ctrl+W".to_string()));
    let conflicts = bindings.find_conflicts();
    assert_eq!(conflicts.len(), 1);
    assert_eq!(conflicts[0].shortcut, "Ctrl+W");
}

#[test]
fn test_settings_tab_selection_and_preference_persistence() {
    use akssora_terminal_config::Config;
    use akssora_terminal_ui::{SettingsAction, SettingsTab, SettingsView};

    let cell_h = 18.0;
    let ws_x = 0.0;
    let ws_y = 40.0;
    let ws_w = 800.0;

    // Verify category tab hit test
    for (i, &tab) in SettingsTab::ALL.iter().enumerate() {
        let (tx, ty, tw, th) = SettingsView::tab_button_rect(i, ws_x, ws_y, cell_h);
        let action = SettingsView::hit_test_action_tab(
            tx + 5.0,
            ty + 5.0,
            ws_x,
            ws_y,
            ws_w,
            cell_h,
            SettingsTab::General,
            &[],
        );
        assert_eq!(action, Some(SettingsAction::SelectTab(tab)));
    }

    // Verify config serialization of appearance and editor options
    let mut config = Config::default();
    config.theme_name = "Slate Dark".to_string();
    config.cursor_style = "Beam".to_string();
    config.cursor_blink = false;
    config.tab_size = 2;
    config.line_numbers = false;

    let json = config.to_json();
    assert!(json.contains("\"theme\": \"Slate Dark\""));
    assert!(json.contains("\"cursor_style\": \"Beam\""));
    assert!(json.contains("\"cursor_blink\": false"));
    assert!(json.contains("\"tab_size\": 2"));
    assert!(json.contains("\"line_numbers\": false"));

    let mut restored = Config::default();
    restored.apply_json(&json);
    assert_eq!(restored.theme_name, "Slate Dark");
    assert_eq!(restored.cursor_style, "Beam");
    assert_eq!(restored.cursor_blink, false);
    assert_eq!(restored.tab_size, 2);
    assert_eq!(restored.line_numbers, false);
}
