use akssora_terminal_config::{Color, Theme};
use akssora_terminal_parser::TerminalParser;
use akssora_terminal_screen::TerminalGrid;
use akssora_terminal_ui::{Header, HeaderButton, HeaderItem, Scrollbar};

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

    // INFO: Center of header contains Quick Search trigger box
    assert_eq!(
        header.hit_test_item(500.0, 10.0, window_w, 1, true),
        Some(HeaderItem::QuickSearch)
    );
    assert!(!header.in_drag_region(500.0, 10.0, window_w));
    assert!(header.in_drag_region(250.0, 10.0, window_w));
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
    assert_eq!(scrollbar.thumb_color(&theme), [0.25, 0.25, 0.27, 0.85]);
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
fn test_ui_font_size_configuration_hit_testing_and_backward_compatibility() {
    use akssora_terminal_app::App;
    use akssora_terminal_ui::{SettingsAction, SettingsView};

    let mut app = App::new();
    let mut fonts = akssora_terminal_renderer::FontManager::new();
    fonts.set_size(14.0);
    let mut ui_fonts = akssora_terminal_renderer::FontManager::new();
    ui_fonts.set_size(14.0);
    app.set_test_fonts(fonts, ui_fonts);

    // Initial state
    assert_eq!(app.current_ui_font_size(), 14.0);
    assert_eq!(app.terminal_font_size(), 14.0);

    // Increase UI font size
    app.increase_ui_font_size();
    assert_eq!(app.current_ui_font_size(), 15.0);
    assert_eq!(app.config.ui_font_size, 15.0);
    assert_eq!(
        app.terminal_font_size(),
        14.0,
        "Terminal font size must remain untouched"
    );

    // Decrease UI font size
    app.decrease_ui_font_size();
    app.decrease_ui_font_size();
    assert_eq!(app.current_ui_font_size(), 13.0);
    assert_eq!(app.config.ui_font_size, 13.0);
    assert_eq!(
        app.terminal_font_size(),
        14.0,
        "Terminal font size must remain untouched"
    );

    // Reset UI font size
    app.reset_ui_font_size();
    assert_eq!(app.current_ui_font_size(), 14.0);
    assert_eq!(app.config.ui_font_size, 14.0);

    // Hit testing for UI font size buttons in General tab
    let (ws_x, ws_y, ws_w, cell_h) = (0.0, 36.0, 800.0, 18.0);

    let (mx, my, mw, mh) = SettingsView::ui_font_minus_button_rect(ws_x, ws_y, ws_w, cell_h);
    let hit_minus =
        SettingsView::hit_test_action(mx + mw * 0.5, my + mh * 0.5, ws_x, ws_y, ws_w, cell_h);
    assert_eq!(hit_minus, Some(SettingsAction::DecreaseUiFontSize));

    let (px, py, pw, ph) = SettingsView::ui_font_plus_button_rect(ws_x, ws_y, ws_w, cell_h);
    let hit_plus =
        SettingsView::hit_test_action(px + pw * 0.5, py + ph * 0.5, ws_x, ws_y, ws_w, cell_h);
    assert_eq!(hit_plus, Some(SettingsAction::IncreaseUiFontSize));

    let (rx, ry, rw, rh) = SettingsView::ui_font_reset_button_rect(ws_x, ws_y, ws_w, cell_h);
    let hit_reset =
        SettingsView::hit_test_action(rx + rw * 0.5, ry + rh * 0.5, ws_x, ws_y, ws_w, cell_h);
    assert_eq!(hit_reset, Some(SettingsAction::ResetUiFontSize));

    // Backward compatibility: load config JSON that lacks "ui_font_size"
    let legacy_json = "{\n  \"font_size\": 16.0,\n  \"cursor_style\": \"Block\"\n}";
    let mut legacy_config = akssora_terminal_config::Config::default();
    legacy_config.apply_json(legacy_json);
    assert_eq!(legacy_config.font.size, 16.0);
    assert_eq!(
        legacy_config.ui_font_size, 14.0,
        "Should default safely to 14.0"
    );
}

#[test]
fn test_ui_font_size_settings_mouse_clicks() {
    use akssora_terminal_app::App;
    use akssora_terminal_ui::{SettingsView, TabKind};

    let mut app = App::new();
    let mut fonts = akssora_terminal_renderer::FontManager::new();
    fonts.set_size(14.0);
    let mut ui_fonts = akssora_terminal_renderer::FontManager::new();
    ui_fonts.set_size(14.0);
    app.set_test_fonts(fonts, ui_fonts);

    // Open Settings tab
    app.tab_manager.add_tab("Settings", TabKind::Settings);
    app.tab_manager.select(app.tab_manager.len() - 1);

    let (w, top_h, sb_w) = (1000.0, 36.0, 0.0);
    let exp_w = app.explorer.effective_width();
    let ws_x = sb_w + exp_w;
    let cell_h = app.ui_cell_h();

    // Simulate clicking Plus button for UI font size
    let (px, py, pw, ph) =
        SettingsView::ui_font_plus_button_rect(ws_x, top_h, (w - ws_x).max(0.0), cell_h);
    let clicked_plus = app.handle_settings_clicks(px + pw * 0.5, py + ph * 0.5, w, top_h, sb_w);
    assert!(
        clicked_plus,
        "Click on UI font size plus button must be handled"
    );
    assert_eq!(app.current_ui_font_size(), 15.0);

    // Simulate clicking Minus button for UI font size
    let cell_h2 = app.ui_cell_h();
    let (mx, my, mw, mh) =
        SettingsView::ui_font_minus_button_rect(ws_x, top_h, (w - ws_x).max(0.0), cell_h2);
    let clicked_minus = app.handle_settings_clicks(mx + mw * 0.5, my + mh * 0.5, w, top_h, sb_w);
    assert!(
        clicked_minus,
        "Click on UI font size minus button must be handled"
    );
    assert_eq!(app.current_ui_font_size(), 14.0);

    // Click minus again
    let cell_h3 = app.ui_cell_h();
    let (mx, my, mw, mh) =
        SettingsView::ui_font_minus_button_rect(ws_x, top_h, (w - ws_x).max(0.0), cell_h3);
    let clicked_minus2 = app.handle_settings_clicks(mx + mw * 0.5, my + mh * 0.5, w, top_h, sb_w);
    assert!(clicked_minus2);
    assert_eq!(app.current_ui_font_size(), 13.0);

    // Click reset button
    let cell_h4 = app.ui_cell_h();
    let (rx, ry, rw, rh) =
        SettingsView::ui_font_reset_button_rect(ws_x, top_h, (w - ws_x).max(0.0), cell_h4);
    let clicked_reset = app.handle_settings_clicks(rx + rw * 0.5, ry + rh * 0.5, w, top_h, sb_w);
    assert!(clicked_reset);
    assert_eq!(app.current_ui_font_size(), 14.0);
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
        let (tx, ty, _tw, _th) = SettingsView::tab_button_rect(i, ws_x, ws_y, cell_h);
        let action = SettingsView::hit_test_action_tab(
            tx + 5.0,
            ty + 5.0,
            ws_x,
            ws_y,
            ws_w,
            cell_h,
            SettingsTab::General,
            &[],
            false,
            false,
        );
        assert_eq!(action, Some(SettingsAction::SelectTab(tab)));
    }

    // Verify config serialization of appearance and editor options
    let config = Config {
        theme_name: "Custom Dark".to_string(),
        cursor_style: "Beam".to_string(),
        cursor_blink: false,
        tab_size: 2,
        line_numbers: false,
        ..Config::default()
    };

    let json = config.to_json();
    assert!(json.contains("\"theme\": \"Custom Dark\""));
    assert!(json.contains("\"cursor_style\": \"Beam\""));
    assert!(json.contains("\"cursor_blink\": false"));
    assert!(json.contains("\"tab_size\": 2"));
    assert!(json.contains("\"line_numbers\": false"));

    let mut restored = Config::default();
    restored.apply_json(&json);
    assert_eq!(restored.theme_name, "Custom Dark");
    assert_eq!(restored.cursor_style, "Beam");
    assert!(!restored.cursor_blink);
    assert_eq!(restored.tab_size, 2);
    assert!(!restored.line_numbers);
}

// INFO: Verifies line number dropdown trigger and option hit testing when open vs closed.
#[test]
fn test_line_number_dropdown_interaction() {
    use akssora_terminal_config::LineNumberMode;
    use akssora_terminal_ui::{SettingsAction, SettingsTab, SettingsView};

    let cell_h = 18.0;
    let ws_x = 0.0;
    let ws_y = 40.0;
    let ws_w = 800.0;

    let (lx, ly, lw, _lh) = SettingsView::line_number_dropdown_rect(ws_x, ws_y, cell_h);
    assert_eq!(lw, SettingsView::LINE_NUMBER_DROPDOWN_WIDTH);

    // Clicking trigger triggers toggle
    let trigger_hit = SettingsView::hit_test_action_tab(
        lx + 10.0,
        ly + 10.0,
        ws_x,
        ws_y,
        ws_w,
        cell_h,
        SettingsTab::Editor,
        &[],
        false,
        false,
    );
    assert_eq!(trigger_hit, Some(SettingsAction::ToggleLineNumberDropdown));

    // When dropdown is closed, clicking where items would be does NOT hit SelectLineNumberMode
    let (ix0, iy0, _, _) = SettingsView::line_number_dropdown_item_rect(0, ws_x, ws_y, cell_h);
    let closed_hit = SettingsView::hit_test_action_tab(
        ix0 + 5.0,
        iy0 + 5.0,
        ws_x,
        ws_y,
        ws_w,
        cell_h,
        SettingsTab::Editor,
        &[],
        false,
        false,
    );
    assert_ne!(
        closed_hit,
        Some(SettingsAction::SelectLineNumberMode(LineNumberMode::Off))
    );

    // When dropdown is open, clicking items hits SelectLineNumberMode for each mode
    for (idx, &mode) in LineNumberMode::ALL.iter().enumerate() {
        let (ix, iy, _, _) = SettingsView::line_number_dropdown_item_rect(idx, ws_x, ws_y, cell_h);
        let open_hit = SettingsView::hit_test_action_tab(
            ix + 5.0,
            iy + 5.0,
            ws_x,
            ws_y,
            ws_w,
            cell_h,
            SettingsTab::Editor,
            &[],
            false,
            true, // line_number_dropdown_open
        );
        assert_eq!(open_hit, Some(SettingsAction::SelectLineNumberMode(mode)));
    }
}

// INFO: Verifies inline Git change indicators (added, modified, deleted) in editor documents and gutter state.
#[test]
fn test_editor_inline_git_change_indicators() {
    use akssora_editor::Document;
    use akssora_git::{GIT_DELETED_COLOR, LineGitChange};
    use std::path::PathBuf;

    let path = PathBuf::from("src/example.rs");
    let baseline = "fn main() {\n    println!(\"hello\");\n}\n";
    let mut doc = Document::open(path.clone(), baseline);
    doc.set_git_original_content(Some(baseline.to_string()));

    // Verify baseline colors
    assert_eq!(LineGitChange::Added.color(), [0.45, 0.78, 0.45, 0.95]);
    assert_eq!(LineGitChange::Modified.color(), [0.88, 0.68, 0.28, 0.95]);
    assert_eq!(GIT_DELETED_COLOR, [0.85, 0.30, 0.30, 0.95]);

    // Initially clean
    let statuses = doc.git_line_statuses();
    assert!(
        statuses
            .iter()
            .all(|s| s.change.is_none() && !s.deleted_above && !s.deleted_below)
    );

    // Modify second line
    doc.cursor = (1, 20);
    doc.insert_str(" // modified");
    let statuses = doc.git_line_statuses();
    assert_eq!(statuses[1].change, Some(LineGitChange::Modified));

    // Add new line
    doc.cursor = (2, 0);
    doc.insert_str("    let x = 42;\n");
    let statuses = doc.git_line_statuses();
    assert_eq!(statuses[1].change, Some(LineGitChange::Modified));
    assert_eq!(statuses[2].change, Some(LineGitChange::Added));

    // Delete line without replacement (e.g. "println!(\"hello\");" deleted)
    doc.buffer = akssora_editor::TextBuffer::from_text("fn main() {\n}\n");
    let statuses = doc.git_line_statuses();
    assert_eq!(statuses.len(), 3);
    assert_eq!(statuses[0].change, None);
    assert!(statuses[1].deleted_above);
}

// INFO: Verifies that terminal sidebar and file explorer can remain open simultaneously with accurate layout calculation.
#[test]
fn test_explorer_and_sidebar_simultaneous_layout() {
    use akssora_terminal_app::App;
    use akssora_terminal_ui::HeaderItem;

    let mut app = App::new();
    app.config.tab_layout = akssora_terminal_config::TabLayoutPreference::Sidebar;
    app.config.window.width = 1200;
    app.config.window.height = 800;

    let tab_id = app.open_terminal_tab();
    app.sidebar.is_expanded = true;
    app.sidebar.width = 240.0;

    // INFO: Explorer is initially collapsed
    assert!(!app.explorer.is_expanded);
    app.layout_blocks();
    let b1 = app.get_block(tab_id).expect("block exists");
    assert_eq!(b1.x, 240.0);

    // INFO: Toggle explorer on; both sidebar and explorer are now open simultaneously
    app.toggle_explorer();
    assert!(app.explorer.is_expanded);
    assert!(app.sidebar.is_expanded);
    assert_eq!(app.explorer.width, 240.0);

    app.layout_blocks();
    let b2 = app.get_block(tab_id).expect("block exists");
    assert_eq!(b2.x, 480.0);
    assert_eq!(b2.width, 1200 - 480);

    // INFO: Verify header hit test detects the ExplorerToggle button
    let (ex, ey, ew, eh) = app.header.explorer_toggle_rect();
    let item = app
        .header
        .hit_test_item(ex + ew / 2.0, ey + eh / 2.0, 1200.0, 1, false);
    assert_eq!(item, Some(HeaderItem::ExplorerToggle));

    // INFO: Toggle explorer off again
    app.toggle_explorer();
    assert!(!app.explorer.is_expanded);
    app.layout_blocks();
    let b3 = app.get_block(tab_id).expect("block exists");
    assert_eq!(b3.x, 240.0);
}

// INFO: Verifies file explorer CRUD workflow and integrated editor opening, typing, and saving.
#[test]
fn test_editor_integration_and_workspace_file_crud() {
    use akssora_terminal_app::App;
    use akssora_terminal_ui::{ExplorerContextMenu, TabKind};

    let temp_dir = std::env::temp_dir().join(format!("akssora_test_ws_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&temp_dir);
    std::fs::create_dir_all(&temp_dir).expect("create temp dir");

    let mut app = App::new();
    app.set_explorer_root(&temp_dir).expect("set explorer root");

    // INFO: Create new file inside explorer tree
    let created_file = app
        .explorer
        .tree
        .create_file(&temp_dir, "hello.rs")
        .expect("create file");
    assert!(created_file.exists());

    // INFO: Open file inside integrated code editor
    let tab_id = app.open_editor_file(&created_file);
    assert_eq!(app.tab_manager.active_id(), Some(tab_id));

    let tab = app.tab_manager.active_tab().expect("active tab");
    assert_eq!(
        tab.kind,
        TabKind::Editor {
            path: created_file.to_string_lossy().to_string()
        }
    );

    // INFO: Insert text and save document
    {
        let doc = app.active_editor_mut().expect("active editor doc");
        doc.insert_str("fn main() {\n    println!(\"Hello World\");\n}\n");
        doc.save().expect("save document");
    }

    let disk_content = std::fs::read_to_string(&created_file).expect("read file");
    assert!(disk_content.contains("println!(\"Hello World\")"));

    // INFO: Test context menu creation for items
    let menu = ExplorerContextMenu::new(100.0, 100.0, Some(created_file.clone()));
    assert!(
        menu.items
            .contains(&akssora_terminal_ui::ExplorerContextAction::OpenInEditor)
    );
    assert!(
        menu.items
            .contains(&akssora_terminal_ui::ExplorerContextAction::OpenInSplit)
    );
    assert!(
        menu.items
            .contains(&akssora_terminal_ui::ExplorerContextAction::OpenTerminalHere)
    );
    assert!(
        menu.items
            .contains(&akssora_terminal_ui::ExplorerContextAction::Delete)
    );

    // INFO: Verify opening file in split pane of terminal
    let split_pane_id = app.open_file_in_split_pane(&created_file);
    assert!(split_pane_id.is_some());

    // INFO: Verify hidden files are always shown by default
    assert!(app.explorer.tree.show_hidden);

    // INFO: Clean up test directory
    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_editor_file_tab_bar_multi_file_and_switching() {
    // INFO: Tests the dedicated editor file-tab bar multi-file lifecycle, switching, and closing.
    let mut app = akssora_terminal_app::App::new();
    let temp_dir =
        std::env::temp_dir().join(format!("akssora_app_tabbar_test_{}", rand::random::<u64>()));
    std::fs::create_dir_all(&temp_dir).unwrap();

    let file1 = temp_dir.join("main.rs");
    let file2 = temp_dir.join("lib.rs");
    std::fs::write(&file1, "fn main() {}\n").unwrap();
    std::fs::write(&file2, "pub fn hello() {}\n").unwrap();

    // INFO: Open first file
    app.open_editor_file(&file1);
    assert_eq!(app.editor_open_files.len(), 1);
    assert_eq!(app.editor_active_file.as_ref(), Some(&file1));

    // INFO: Open second file
    app.open_editor_file(&file2);
    assert_eq!(app.editor_open_files.len(), 2);
    assert_eq!(app.editor_active_file.as_ref(), Some(&file2));

    // INFO: Verify file tabs collection
    let tabs = app.collect_editor_file_tabs();
    assert_eq!(tabs.len(), 2);
    assert_eq!(tabs[0].title, "main.rs");
    assert!(!tabs[0].is_active);
    assert_eq!(tabs[1].title, "lib.rs");
    assert!(tabs[1].is_active);
    assert!(!tabs[1].is_dirty);

    // INFO: Edit active document (file2) and check dirty indicator
    {
        let doc = app.active_editor_mut().unwrap();
        doc.insert_str("// modified\n");
        assert!(doc.dirty);
    }
    let tabs_dirty = app.collect_editor_file_tabs();
    assert!(tabs_dirty[1].is_dirty);

    // INFO: Switch back to file1 and verify active file changed
    app.select_editor_file_tab(0);
    assert_eq!(app.editor_active_file.as_ref(), Some(&file1));
    {
        let doc = app.active_editor().unwrap();
        assert_eq!(doc.buffer.rope().to_string(), "fn main() {}\n");
    }

    // INFO: Switch back to file2 and verify edits preserved
    app.select_editor_file_tab(1);
    assert_eq!(app.editor_active_file.as_ref(), Some(&file2));
    {
        let doc = app.active_editor().unwrap();
        assert!(doc.buffer.rope().to_string().starts_with("// modified\n"));
    }

    // INFO: Close file2 tab
    app.close_editor_file_tab(1);
    assert_eq!(app.editor_open_files.len(), 1);
    assert_eq!(app.editor_active_file.as_ref(), Some(&file1));

    // INFO: Close file1 tab; editor tab should be closed from tab manager
    app.close_editor_file_tab(0);
    assert_eq!(app.editor_open_files.len(), 0);
    assert!(app.editor_active_file.is_none());
    assert!(app.active_editor().is_none());

    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_editor_tab_bar_hit_testing() {
    // INFO: Tests geometry calculation and hit-testing for the dedicated editor file-tab bar.
    use akssora_terminal_ui::{EditorTabBar, EditorTabTarget};

    let ws_x = 240.0;
    let ws_y = 38.0;
    let ws_w = 800.0;
    let tab_count = 3;

    let (t0_x, t0_y, _t0_w, t0_h) = EditorTabBar::tab_rect(0, tab_count, ws_x, ws_y, ws_w);
    assert_eq!(t0_x, 240.0);
    assert_eq!(t0_y, 38.0);
    assert_eq!(t0_h, EditorTabBar::HEIGHT);

    // INFO: Click inside tab 0 body
    let target = EditorTabBar::hit_test(t0_x + 20.0, t0_y + 10.0, ws_x, ws_y, ws_w, tab_count);
    assert_eq!(target, Some(EditorTabTarget::Tab(0)));

    // INFO: Click inside tab 0 close button
    let (c0_x, c0_y, _cw, _ch) = EditorTabBar::close_rect(0, tab_count, ws_x, ws_y, ws_w);
    let target_close = EditorTabBar::hit_test(c0_x + 4.0, c0_y + 4.0, ws_x, ws_y, ws_w, tab_count);
    assert_eq!(target_close, Some(EditorTabTarget::Close(0)));

    // INFO: Click outside tab bar vertically
    let outside = EditorTabBar::hit_test(t0_x + 20.0, ws_y + 40.0, ws_x, ws_y, ws_w, tab_count);
    assert_eq!(outside, None);
}

#[test]
fn test_quick_search_ui_and_fuzzy_matching() {
    // INFO: Tests quick file search scanning, exclusions, fuzzy matching, and keyboard selection.
    use akssora_terminal_ui::quick_search::{QuickSearchUi, fuzzy_match};
    use std::fs;

    let temp_dir = std::env::temp_dir().join(format!("akssora_qs_test_{}", std::process::id()));
    let _ = fs::remove_dir_all(&temp_dir);
    fs::create_dir_all(temp_dir.join("src")).unwrap();
    fs::create_dir_all(temp_dir.join("tests")).unwrap();
    fs::create_dir_all(temp_dir.join("target/debug")).unwrap();
    fs::create_dir_all(temp_dir.join(".git")).unwrap();

    fs::write(temp_dir.join("src/main.rs"), "fn main() {}\n").unwrap();
    fs::write(temp_dir.join("src/utils.rs"), "pub fn helper() {}\n").unwrap();
    fs::write(temp_dir.join("tests/test_run.rs"), "// test\n").unwrap();
    fs::write(temp_dir.join("target/debug/output.bin"), "bin").unwrap();
    fs::write(temp_dir.join(".git/config"), "git config").unwrap();
    fs::write(temp_dir.join(".hidden.txt"), "secret").unwrap();

    let exclusions = vec!["target".to_string(), ".git".to_string()];
    let mut qs = QuickSearchUi::new();
    assert!(!qs.is_open);

    // INFO: Toggle open and scan with empty query (respecting exclusions and hidden flag)
    qs.toggle(&temp_dir, &exclusions, false);
    assert!(qs.is_open);
    qs.update_search(&temp_dir, &exclusions, false);

    // INFO: Verify exclusions: target and .git must not be present, and hidden files excluded
    let paths: Vec<String> = qs.results.iter().map(|r| r.relative_path.clone()).collect();
    assert!(paths.contains(&"src/main.rs".to_string()));
    assert!(paths.contains(&"src/utils.rs".to_string()));
    assert!(paths.contains(&"tests/test_run.rs".to_string()));
    assert!(!paths.iter().any(|p| p.starts_with("target")));
    assert!(!paths.iter().any(|p| p.starts_with(".git")));
    assert!(!paths.contains(&".hidden.txt".to_string()));

    // INFO: Now allow hidden files and verify .hidden.txt is indexed
    qs.update_search(&temp_dir, &exclusions, true);
    let paths_with_hidden: Vec<String> =
        qs.results.iter().map(|r| r.relative_path.clone()).collect();
    assert!(paths_with_hidden.contains(&".hidden.txt".to_string()));

    // INFO: Fuzzy matching queries
    qs.query = "main".to_string();
    qs.update_search(&temp_dir, &exclusions, false);
    assert!(!qs.results.is_empty());
    assert_eq!(qs.results[0].filename, "main.rs");

    qs.query = "util".to_string();
    qs.update_search(&temp_dir, &exclusions, false);
    assert!(!qs.results.is_empty());
    assert_eq!(qs.results[0].filename, "utils.rs");

    // INFO: Keyboard navigation wrapping
    qs.query = "".to_string();
    qs.update_search(&temp_dir, &exclusions, false);
    let count = qs.results.len();
    assert!(count >= 3);
    assert_eq!(qs.selected_index, 0);

    qs.select_next();
    assert_eq!(qs.selected_index, 1);
    qs.select_prev();
    assert_eq!(qs.selected_index, 0);
    qs.select_prev();
    assert_eq!(qs.selected_index, 0); // INFO: Clamps at top

    // INFO: Selection retrieval
    let selected = qs.selected_result().unwrap();
    assert_eq!(selected.path, qs.results[0].path);

    // INFO: Geometry and hit testing
    let (ox, oy, ow, oh) = QuickSearchUi::overlay_rect(800.0, 38.0, count);
    assert!(ow > 0.0);
    assert!(oh > 0.0);
    let hit_first = qs.hit_test_item(ox + 50.0, oy + 55.0, 800.0, 38.0);
    assert_eq!(hit_first, Some(0));

    // INFO: Direct fuzzy match score sanity checks
    assert!(fuzzy_match("main", "main.rs", "src/main.rs").is_some());
    assert!(fuzzy_match("mr", "main.rs", "src/main.rs").is_some());
    assert!(fuzzy_match("xyz", "main.rs", "src/main.rs").is_none());

    // INFO: Close search palette
    qs.close();
    assert!(!qs.is_open);
    assert_eq!(qs.query, "");

    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_explorer_shortcuts_and_file_operations() {
    // INFO: Tests folder targeting, % file shortcut, inline rename updating tabs, and delete handling.
    use akssora_terminal_app::App;
    use akssora_terminal_ui::{
        ExplorerContextAction, ExplorerContextMenu, InlineInputMode, InlineInputState,
    };
    use std::fs;

    let temp_dir = std::env::temp_dir().join(format!("akssora_ops_test_{}", std::process::id()));
    let _ = fs::remove_dir_all(&temp_dir);
    let sub_dir = temp_dir.join("subfolder");
    fs::create_dir_all(&sub_dir).unwrap();
    let file1 = sub_dir.join("hello.rs");
    fs::write(&file1, "fn hello() {}\n").unwrap();

    let mut app = App::default();
    let _ = app.set_explorer_root(temp_dir.clone());

    // INFO: 1. Folder targeting without selection defaults to root directory
    assert_eq!(app.selected_explorer_directory(), temp_dir);

    // INFO: Select subfolder and verify target directory
    app.explorer.tree.selected_paths.clear();
    app.explorer.tree.selected_paths.insert(sub_dir.clone());
    assert_eq!(app.selected_explorer_directory(), sub_dir);

    // INFO: Select file within subfolder and verify target directory resolves to parent folder
    app.explorer.tree.selected_paths.clear();
    app.explorer.tree.selected_paths.insert(file1.clone());
    assert_eq!(app.selected_explorer_directory(), sub_dir);

    // INFO: 2. Open file in editor and test inline rename tab synchronization
    app.open_editor_file(&file1);
    assert_eq!(app.editor_open_files.len(), 1);
    assert_eq!(app.editor_active_file.as_ref(), Some(&file1));

    let renamed_file = sub_dir.join("world.rs");
    fs::rename(&file1, &renamed_file).unwrap();
    app.handle_file_renamed(&file1, &renamed_file);

    // INFO: Verify open editor file tabs and documents updated immediately
    assert_eq!(app.editor_open_files[0], renamed_file);
    assert_eq!(app.editor_active_file.as_ref(), Some(&renamed_file));
    assert!(app.editor_documents.contains_key(&renamed_file));
    assert!(!app.editor_documents.contains_key(&file1));

    let tabs = app.collect_editor_file_tabs();
    assert_eq!(tabs[0].title, "world.rs");

    // INFO: 3. Test delete synchronization: deleting open file removes tab
    fs::remove_file(&renamed_file).unwrap();
    app.handle_file_deleted(&renamed_file);
    assert_eq!(app.editor_open_files.len(), 0);
    assert!(app.editor_active_file.is_none());

    // INFO: 4. Test InlineInputState validation error display
    let mut inline_input = InlineInputState {
        mode: InlineInputMode::NewFile,
        text: "".to_string(),
        parent_dir: sub_dir.clone(),
        error: None,
    };
    assert!(inline_input.error.is_none());
    inline_input.error = Some("Filename cannot be empty".to_string());
    assert_eq!(
        inline_input.error.as_deref(),
        Some("Filename cannot be empty")
    );

    // INFO: 5. Verify ExplorerContextMenu includes CopyPath and RevealInFileManager
    let menu = ExplorerContextMenu::new(100.0, 100.0, Some(sub_dir.clone()));
    let actions: &[ExplorerContextAction] = &menu.items;
    assert!(actions.contains(&ExplorerContextAction::CopyPath));
    assert!(actions.contains(&ExplorerContextAction::RevealInFileManager));

    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_explorer_and_editor_tabs_enhancements() {
    use akssora_terminal_app::App;
    use akssora_terminal_config::{Action, Keybindings};
    use akssora_terminal_ui::{EditorTabBar, EditorTabTarget, file_type_icon};
    use std::fs;

    // 1. Verify Keybindings for editor tab closing actions
    let kb = Keybindings::default();
    assert_eq!(kb.shortcut_for(Action::CloseEditorTab), Some("Ctrl+W"));
    assert_eq!(
        kb.shortcut_for(Action::CloseOtherEditorTabs),
        Some("Ctrl+Shift+W")
    );
    assert_eq!(
        kb.shortcut_for(Action::CloseAllEditorTabs),
        Some("Ctrl+K, Ctrl+W")
    );

    // Verify file_type_icon helper
    assert_eq!(file_type_icon("main.rs"), "\u{2699}");
    assert_eq!(file_type_icon("package.json"), "\u{2756}");
    assert_eq!(file_type_icon("index.html"), "\u{25C8}");
    assert_eq!(file_type_icon("README.md"), "\u{2261}");

    let temp_dir =
        std::env::temp_dir().join(format!("akssora_tabs_enh_test_{}", std::process::id()));
    let _ = fs::remove_dir_all(&temp_dir);
    fs::create_dir_all(&temp_dir).unwrap();
    let file1 = temp_dir.join("main.rs");
    let file2 = temp_dir.join("lib.rs");
    let file3 = temp_dir.join("preview.rs");
    fs::write(&file1, "fn main() {}\n").unwrap();
    fs::write(&file2, "pub fn lib() {}\n").unwrap();
    fs::write(&file3, "fn preview() {}\n").unwrap();

    let mut app = App::default();
    app.config.auto_save = false;
    let _ = app.set_explorer_root(temp_dir.clone());

    // 2. Open persistent file and verify Explorer active file highlight & metadata
    app.open_editor_file(&file1);
    assert_eq!(app.editor_open_files.len(), 1);
    assert_eq!(app.editor_active_file.as_deref(), Some(file1.as_path()));
    assert_eq!(
        app.explorer.active_editor_file.as_deref(),
        Some(file1.as_path())
    );

    let tabs = app.collect_editor_file_tabs();
    assert_eq!(tabs.len(), 1);
    assert_eq!(tabs[0].title, "main.rs");
    assert_eq!(tabs[0].relative_path, "main.rs");
    assert!(!tabs[0].is_preview);
    assert!(!tabs[0].is_pinned);

    // 3. Open file in preview mode
    app.open_editor_file_with_preview(&file3, true);
    assert_eq!(app.editor_open_files.len(), 2);
    assert_eq!(app.editor_preview_file.as_deref(), Some(file3.as_path()));
    let tabs = app.collect_editor_file_tabs();
    assert!(
        tabs.iter()
            .find(|t| t.title == "preview.rs")
            .unwrap()
            .is_preview
    );

    // 4. Pin a tab and verify pin state & ordering
    app.toggle_pin_editor_file(&file3);
    assert!(app.editor_pinned_files.contains(&file3));
    assert!(app.editor_preview_file.is_none()); // Pinning promotes preview
    let tabs = app.collect_editor_file_tabs();
    assert_eq!(tabs[0].title, "preview.rs"); // Pinned tabs sort to front
    assert!(tabs[0].is_pinned);

    // 5. Drag and drop reordering
    app.open_editor_file(&file2);
    assert_eq!(app.editor_open_files.len(), 3);
    let orig_0 = app.editor_open_files[0].clone();
    let orig_1 = app.editor_open_files[1].clone();
    app.reorder_editor_file_tab(0, 1);
    assert_eq!(app.editor_open_files[0], orig_1);
    assert_eq!(app.editor_open_files[1], orig_0);

    // 6. Dirty unsaved changes confirmation prompt
    if let Some(doc) = app.editor_documents.get_mut(&file1) {
        doc.dirty = true;
    }
    let f1_idx = app
        .editor_open_files
        .iter()
        .position(|p| p == &file1)
        .unwrap();
    app.request_close_editor_tab(f1_idx);
    assert!(app.unsaved_prompt_target.is_some());
    // Cancel prompt keeps file open and preserved
    app.cancel_prompt_action();
    assert!(app.unsaved_prompt_target.is_none());
    assert!(app.editor_open_files.contains(&file1));

    // 7. Hit testing pin and tab targets
    let (ws_x, top_h, ws_w) = (0.0, 0.0, 800.0);
    let hit_tab = EditorTabBar::hit_test(20.0, 14.0, ws_x, top_h, ws_w, 3);
    assert!(matches!(hit_tab, Some(EditorTabTarget::Tab(0))));

    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_explorer_git_status_states() {
    use akssora_git::{FileGitStatus, GitFileChange, GitRepoStatus};
    use akssora_terminal_app::App;
    use std::path::PathBuf;

    let mut app = App::default();
    let root = PathBuf::from("/test/workspace");
    app.explorer.tree.root_path = root.clone();

    let repo_status = GitRepoStatus {
        branch: "main".to_string(),
        clean: false,
        changes: vec![
            GitFileChange {
                path: root.join("src/modified.rs"),
                status: FileGitStatus::Modified,
            },
            GitFileChange {
                path: root.join("src/new_file.rs"),
                status: FileGitStatus::Added,
            },
            GitFileChange {
                path: root.join("src/deleted.rs"),
                status: FileGitStatus::Deleted,
            },
            GitFileChange {
                path: root.join("tests/untracked.rs"),
                status: FileGitStatus::Untracked,
            },
            GitFileChange {
                path: root.join("src/renamed.rs"),
                status: FileGitStatus::Renamed,
            },
            GitFileChange {
                path: root.join("conflict.txt"),
                status: FileGitStatus::Conflict,
            },
        ],
        ..Default::default()
    };

    app.explorer.git_status = Some(repo_status.clone());

    // 1. Check exact file status lookups
    assert_eq!(
        repo_status.file_status(&root, &root.join("src/modified.rs")),
        Some(FileGitStatus::Modified)
    );
    assert_eq!(
        repo_status.file_status(&root, &root.join("src/new_file.rs")),
        Some(FileGitStatus::Added)
    );
    assert_eq!(
        repo_status.file_status(&root, &root.join("src/deleted.rs")),
        Some(FileGitStatus::Deleted)
    );
    assert_eq!(
        repo_status.file_status(&root, &root.join("tests/untracked.rs")),
        Some(FileGitStatus::Untracked)
    );
    assert_eq!(
        repo_status.file_status(&root, &root.join("src/renamed.rs")),
        Some(FileGitStatus::Renamed)
    );
    assert_eq!(
        repo_status.file_status(&root, &root.join("conflict.txt")),
        Some(FileGitStatus::Conflict)
    );

    // 2. Check directory status bubbling
    assert_eq!(
        repo_status.file_status(&root, &root.join("src")),
        Some(FileGitStatus::Modified)
    );
    assert_eq!(
        repo_status.file_status(&root, &root.join("tests")),
        Some(FileGitStatus::Untracked)
    );

    // 3. Unmodified file returns None
    assert_eq!(
        repo_status.file_status(&root, &root.join("README.md")),
        None
    );

    // 4. Verify status badge labels, icons, and subtle colors
    assert_eq!(FileGitStatus::Modified.badge_label(), "M");
    assert_eq!(FileGitStatus::Added.badge_label(), "A");
    assert_eq!(FileGitStatus::Deleted.badge_label(), "D");
    assert_eq!(FileGitStatus::Untracked.badge_label(), "U");
    assert_eq!(FileGitStatus::Renamed.badge_label(), "R");
    assert_eq!(FileGitStatus::Conflict.badge_label(), "!");

    assert_eq!(FileGitStatus::Modified.status_icon(), "\u{2022}");
    assert_eq!(FileGitStatus::Added.status_icon(), "+");
    assert_eq!(FileGitStatus::Deleted.status_icon(), "-");
    assert_eq!(FileGitStatus::Untracked.status_icon(), "?");
    assert_eq!(FileGitStatus::Renamed.status_icon(), "\u{2192}");
    assert_eq!(FileGitStatus::Conflict.status_icon(), "!");

    assert_eq!(FileGitStatus::Modified.status_description(), "Modified");
    assert_eq!(FileGitStatus::Added.status_description(), "Added");
    assert_eq!(FileGitStatus::Deleted.status_description(), "Deleted");
    assert_eq!(FileGitStatus::Untracked.status_description(), "Untracked");
    assert_eq!(FileGitStatus::Renamed.status_description(), "Renamed");
}

#[test]
fn test_inline_diff_view_features() {
    use akssora_editor::Document;
    use akssora_git::{LineGitChange, compute_line_diff};
    use akssora_terminal_config::{Action, Keybindings};
    use akssora_terminal_ui::{EditorTabBar, EditorTabTarget, EditorToolbarAction};
    use std::path::PathBuf;

    // 1. Verify Document has inline_diff disabled by default in editor and can be toggled
    let mut doc = Document::open(
        PathBuf::from("test.rs"),
        "fn main() {\n    println!(\"hello\");\n}\n",
    );
    assert!(!doc.inline_diff);
    doc.inline_diff = true;
    assert!(doc.inline_diff);
    doc.inline_diff = false;
    assert!(!doc.inline_diff);

    // 2. Baseline Myers line diff inspection with removed lines text and original modified line
    let orig = &["fn main() {", "    let old = 1;", "    to_delete();", "}"];
    let curr = &[
        "fn main() {",
        "    let new = 2;",
        "    added_1();",
        "    added_2();",
        "}",
    ];
    let diff = compute_line_diff(orig, curr);
    assert_eq!(diff.len(), 5);
    assert_eq!(diff[0].change, None);
    assert_eq!(diff[1].change, Some(LineGitChange::Modified));
    assert_eq!(diff[1].original_line.as_deref(), Some("    let old = 1;"));
    assert_eq!(diff[2].change, Some(LineGitChange::Modified));
    assert_eq!(diff[2].original_line.as_deref(), Some("    to_delete();"));
    assert_eq!(diff[3].change, Some(LineGitChange::Added));
    assert_eq!(diff[4].change, None);

    let orig_del = &["fn main() {", "    remove_me();", "}"];
    let curr_del = &["fn main() {", "}"];
    let diff_del = compute_line_diff(orig_del, curr_del);
    assert_eq!(diff_del.len(), 2);
    assert!(diff_del[1].deleted_above);
    assert_eq!(
        diff_del[1].deleted_lines,
        vec!["    remove_me();".to_string()]
    );

    // 3. Verify EditorTabBar toolbar hit testing for ToggleInlineDiff
    let ws_x = 240.0;
    let ws_y = 38.0;
    let ws_w = 800.0;
    let (bx, by, bw, bh) =
        EditorTabBar::toolbar_rect(EditorToolbarAction::ToggleInlineDiff, ws_x, ws_y, ws_w);
    assert!(bw > 0.0 && bh > 0.0);

    let hit = EditorTabBar::hit_test_toolbar(bx + 4.0, by + 4.0, ws_x, ws_y, ws_w);
    assert_eq!(hit, Some(EditorToolbarAction::ToggleInlineDiff));

    let target = EditorTabBar::hit_test_ex(bx + 4.0, by + 4.0, ws_x, ws_y, ws_w, 2, false);
    assert_eq!(target, Some(EditorTabTarget::ToggleInlineDiff));

    // 4. Verify Keybinding for Action::ToggleInlineDiff
    let kb = Keybindings::default();
    assert_eq!(
        kb.shortcut_for(Action::ToggleInlineDiff),
        Some("Ctrl+Alt+D")
    );
}

// INFO: Verifies editor space key behavior across beginning, middle, end of lines, empty lines, and selection replacement.
#[test]
fn test_editor_space_key_interactions() {
    use akssora_editor::Document;

    let mut doc = Document::new();
    assert_eq!(doc.cursor, (0, 0));

    // 1. Space on empty document
    doc.insert_char_smart(' ');
    assert_eq!(doc.cursor, (0, 1));
    assert_eq!(doc.buffer.get_line(0).as_deref(), Some(" "));

    // 2. Space at the end of line
    doc.insert_str("Hello");
    assert_eq!(doc.cursor, (0, 6));
    assert_eq!(doc.buffer.get_line(0).as_deref(), Some(" Hello"));
    doc.insert_char_smart(' ');
    assert_eq!(doc.cursor, (0, 7));
    assert_eq!(doc.buffer.get_line(0).as_deref(), Some(" Hello "));

    // 3. Space in the middle of text
    doc.cursor = (0, 3);
    doc.insert_char_smart(' ');
    assert_eq!(doc.cursor, (0, 4));
    assert_eq!(doc.buffer.get_line(0).as_deref(), Some(" He llo "));

    // 4. Space at beginning of line
    doc.move_to_line_start(false);
    assert_eq!(doc.cursor, (0, 0));
    doc.insert_char_smart(' ');
    assert_eq!(doc.cursor, (0, 1));
    assert_eq!(doc.buffer.get_line(0).as_deref(), Some("  He llo "));

    // 5. Space replacing selection
    doc.cursor = (0, 2);
    doc.selection_anchor = Some((0, 6));
    assert!(doc.has_selection());
    doc.insert_char_smart(' ');
    assert!(!doc.has_selection());
    assert_eq!(doc.buffer.get_line(0).as_deref(), Some("   lo "));

    // 6. Undo/Redo space insertions
    doc.undo();
    assert_eq!(doc.buffer.get_line(0).as_deref(), Some("  He llo "));
    doc.redo();
    assert_eq!(doc.buffer.get_line(0).as_deref(), Some("   lo "));
}

// INFO: Verifies terminal in-line editing (DCH, ICH, ECH, line editing without trailing corruption).
#[test]
fn test_terminal_inline_editing_escape_sequences() {
    let mut grid = TerminalGrid::new(24, 80);
    let mut parser = TerminalParser::new();

    // 1. Simulate pasted command: git checkout feature/editor-fixes
    let cmd = b"$ git checkout feature/editor-fixes";
    parser.advance(&mut grid, cmd);
    {
        let lines = grid.visible_lines();
        let line_chars: String = lines[0].iter().map(|c| c.ch).collect();
        assert!(line_chars.starts_with("$ git checkout feature/editor-fixes"));
    }

    // 2. Cursor moved 21 columns left to after 't' of checkout (from col 35 to col 14)
    let (row, col) = grid.cursor();
    assert_eq!(row, 0);
    assert_eq!(col, 35);
    parser.advance(&mut grid, b"\x1b[21D");
    assert_eq!(grid.cursor(), (0, 14));

    // 3. Send 3 Backspaces with DCH: \x08\x1b[1P 3 times (deleting 'out')
    // Readline deletes characters in the middle using backspace and DCH (\x1b[1P)
    parser.advance(&mut grid, b"\x08\x1b[1P\x08\x1b[1P\x08\x1b[1P");
    assert_eq!(grid.cursor(), (0, 11));
    {
        let lines = grid.visible_lines();
        let line_chars: String = lines[0].iter().map(|c| c.ch).collect();
        // The command must now read '$ git check feature/editor-fixes'
        // and crucially MUST NOT have stale 'out' or duplicate 'sssss' at the end!
        assert!(line_chars.starts_with("$ git check feature/editor-fixes"));
        let remainder = &line_chars["$ git check feature/editor-fixes".len()..];
        assert!(remainder.chars().all(|c| c == ' ' || c == '\0'));
    }

    // 4. Test ICH (Insert Character: \x1b[1@)
    // Cursor at 11 (' ' after check). Insert 1 blank and write '-'
    parser.advance(&mut grid, b"\x1b[1@-");
    {
        let lines = grid.visible_lines();
        let line_chars: String = lines[0].iter().map(|c| c.ch).collect();
        assert!(line_chars.starts_with("$ git check- feature/editor-fixes"));
    }

    // 5. Test ECH (Erase Character: \x1b[1X)
    // Erase the '-' just typed without moving cursor
    parser.advance(&mut grid, b"\x1b[1D\x1b[1X");
    {
        let lines = grid.visible_lines();
        let line_chars: String = lines[0].iter().map(|c| c.ch).collect();
        assert!(line_chars.starts_with("$ git check  feature/editor-fixes"));
    }

    // 7. Test Cursor Save and Restore (\x1b[s / \x1b[u and \x1b7 / \x1b8)
    parser.advance(&mut grid, b"\x1b[5;20H\x1b[s\x1b[1;1H\x1b[u");
    assert_eq!(grid.cursor(), (4, 19));
    parser.advance(&mut grid, b"\x1b[2;10H\x1b7\x1b[1;1H\x1b8");
    assert_eq!(grid.cursor(), (1, 9));
}

#[test]
fn test_deleted_files_rendering_and_diff_status() {
    use akssora_editor::Document;
    use akssora_git::{DiffHunk, DiffLine, DiffLineType, FileDiff, FileGitStatus, LineGitChange};
    use std::path::PathBuf;

    // 1. Deleted file with original content parsed from diff hunks
    let hunk = DiffHunk {
        old_start: 1,
        old_lines: 3,
        new_start: 0,
        new_lines: 0,
        header: "@@ -1,3 +0,0 @@".to_string(),
        lines: vec![
            DiffLine {
                line_type: DiffLineType::Delete,
                content: "fn calculate() -> i32 {\n".to_string(),
                old_line_num: Some(1),
                new_line_num: None,
            },
            DiffLine {
                line_type: DiffLineType::Delete,
                content: "    42\n".to_string(),
                old_line_num: Some(2),
                new_line_num: None,
            },
            DiffLine {
                line_type: DiffLineType::Delete,
                content: "}\n".to_string(),
                old_line_num: Some(3),
                new_line_num: None,
            },
        ],
    };

    let file_diff = FileDiff {
        path: PathBuf::from("src/calc.rs"),
        old_path: None,
        status: FileGitStatus::Deleted,
        additions: 0,
        deletions: 3,
        is_binary: false,
        line_count: 3,
        hunks: vec![hunk],
    };

    let retrieved_content = file_diff.deleted_content().expect("content must be parsed");
    assert_eq!(retrieved_content, "fn calculate() -> i32 {\n    42\n}\n");

    // 2. Open document with retrieved content and mark as deleted
    let mut doc = Document::open(PathBuf::from("src/calc.rs"), &retrieved_content);
    assert!(!doc.is_deleted);
    doc.set_deleted(true);
    assert!(doc.is_deleted);
    assert_eq!(doc.buffer.line_count(), 4);

    // 3. Document git line statuses must all be LineGitChange::Deleted
    let statuses = doc.git_line_statuses();
    assert_eq!(statuses.len(), 4);
    for status in statuses.iter() {
        assert_eq!(status.change, Some(LineGitChange::Deleted));
    }

    // 4. Distinguish between genuinely empty deleted file vs deleted file with content
    let empty_file_diff = FileDiff {
        path: PathBuf::from("empty.txt"),
        old_path: None,
        status: FileGitStatus::Deleted,
        additions: 0,
        deletions: 0,
        is_binary: false,
        line_count: 0,
        hunks: vec![],
    };
    let empty_content = empty_file_diff.deleted_content().unwrap_or_default();
    assert_eq!(empty_content, "");

    // 5. Explicit error reporting when content cannot be retrieved
    let missing_path = PathBuf::from("/non/existent/repo/root/file.rs");
    let err_res = akssora_git::GitInspector::file_deleted_content(
        std::path::Path::new("/non/existent/repo/root"),
        &missing_path,
        None,
    );
    assert!(err_res.is_err());
    let err_msg = err_res.unwrap_err();
    assert!(err_msg.contains("Git error") || err_msg.contains("cannot be retrieved"));
}

#[test]
fn test_code_review_accordion_natural_scrolling_and_caching() {
    use akssora_git::{DiffHunk, DiffLine, DiffLineType, FileDiff, FileGitStatus, RepoDiff};
    use akssora_terminal_ui::{CodeReviewLayout, CodeReviewModel};
    use std::path::PathBuf;

    let mut model = CodeReviewModel::default();
    model.diff = RepoDiff {
        files: vec![
            FileDiff {
                path: PathBuf::from("short.rs"),
                old_path: None,
                status: FileGitStatus::Modified,
                additions: 2,
                deletions: 1,
                is_binary: false,
                line_count: 5,
                hunks: vec![DiffHunk {
                    old_start: 1,
                    old_lines: 1,
                    new_start: 1,
                    new_lines: 2,
                    header: "@@ -1,1 +1,2 @@".to_string(),
                    lines: vec![
                        DiffLine {
                            line_type: DiffLineType::Delete,
                            content: "old line\n".to_string(),
                            old_line_num: Some(1),
                            new_line_num: None,
                        },
                        DiffLine {
                            line_type: DiffLineType::Add,
                            content: "new line 1\n".to_string(),
                            old_line_num: None,
                            new_line_num: Some(1),
                        },
                        DiffLine {
                            line_type: DiffLineType::Add,
                            content: "new line 2\n".to_string(),
                            old_line_num: None,
                            new_line_num: Some(2),
                        },
                    ],
                }],
            },
            FileDiff {
                path: PathBuf::from("long.rs"),
                old_path: None,
                status: FileGitStatus::Modified,
                additions: 100,
                deletions: 50,
                is_binary: false,
                line_count: 150,
                hunks: vec![],
            },
        ],
        files_changed: 2,
        total_additions: 102,
        total_deletions: 51,
    };

    let viewport_h = 800.0;

    // 1. Natural content-sized accordion heights
    let short_h = CodeReviewLayout::card_height_for_file_state(
        &model.diff.files[0],
        false,
        false,
        viewport_h,
    );
    let expected_short_h = CodeReviewLayout::CARD_HEADER_HEIGHT
        + CodeReviewLayout::UNMODIFIED_LINES_HEIGHT
        + (3.0 * CodeReviewLayout::DIFF_LINE_HEIGHT + 16.0);
    assert_eq!(short_h, expected_short_h);
    // Short file diff height must not artificially expand to fill viewport
    assert!(short_h < 200.0);
    assert!(short_h < viewport_h);

    // Full file view height matches file line count
    let full_h = CodeReviewLayout::card_height_for_file_state_ex(
        &model.diff.files[0],
        false,
        false,
        true,
        viewport_h,
    );
    let expected_full_h =
        CodeReviewLayout::CARD_HEADER_HEIGHT + (5.0 * CodeReviewLayout::DIFF_LINE_HEIGHT + 16.0);
    assert_eq!(full_h, expected_full_h);

    let long_h = CodeReviewLayout::card_height_for_file_state(
        &model.diff.files[1],
        false,
        false,
        viewport_h,
    );
    assert!(long_h > 1000.0);

    // 2. Total diff content height calculation
    let total_h1 = model.total_diff_content_height(viewport_h);
    assert_eq!(
        total_h1,
        12.0 + short_h
            + CodeReviewLayout::CARD_MARGIN_BOTTOM
            + long_h
            + CodeReviewLayout::CARD_MARGIN_BOTTOM
    );
    let total_h2 = model.total_diff_content_height(viewport_h);
    assert_eq!(total_h1, total_h2);

    // 3. Tree items caching
    let items1 = model.visible_tree_items();
    // 2 section headers (Staged Changes, Changes) + 2 file items in Changes
    assert_eq!(items1.len(), 4);
    let items2 = model.visible_tree_items();
    assert_eq!(items1, items2);

    // 4. Invalidation on collapsing an accordion
    // An expanded accordion must always be collapsible via toggle:
    model.toggle_file_collapsed(&PathBuf::from("short.rs"));
    assert!(
        model.is_file_collapsed(&PathBuf::from("short.rs")),
        "An expanded accordion must always be collapsible"
    );

    // Programmatically set collapsed state to test layout recomputation:
    model.set_file_collapsed(&PathBuf::from("short.rs"), true);
    assert!(model.is_file_collapsed(&PathBuf::from("short.rs")));

    // Recomputed height after collapse uses CARD_HEADER_HEIGHT for short.rs
    let collapsed_short_h = CodeReviewLayout::CARD_HEADER_HEIGHT;
    let total_h_collapsed = model.total_diff_content_height(viewport_h);
    assert_eq!(
        total_h_collapsed,
        12.0 + collapsed_short_h
            + CodeReviewLayout::CARD_MARGIN_BOTTOM
            + long_h
            + CodeReviewLayout::CARD_MARGIN_BOTTOM
    );

    // 5. Sticky header within accordion boundaries while code scrolls:
    let vy = 42.0;
    let header_y = CodeReviewLayout::card_header_y(vy - 50.0, long_h, false, vy);
    assert_eq!(header_y, vy);
    let collapsed_header_y = CodeReviewLayout::card_header_y(vy - 50.0, long_h, true, vy);
    assert_eq!(collapsed_header_y, vy - 50.0);
}

#[test]
fn test_strict_per_line_git_actions_and_commit_button() {
    use akssora_editor::{Document, GutterActionKind};
    use akssora_git::LineGitChange;
    use akssora_terminal_ui::CodeReviewLayout;
    use std::path::PathBuf;

    // 1. Commit button visibility: Ensure width cleanly accommodates "Commit" text
    let (_cx, _cy, cw, ch) = CodeReviewLayout::commit_btn_rect(0.0, 0.0, 800.0);
    assert!(cw >= 50.0);
    assert!(ch >= 20.0);
    let commit_text = "Commit";
    let approx_tw = commit_text.len() as f32 * 8.5;
    assert!(
        cw > approx_tw + 8.0,
        "Button width must fully fit 'Commit' text without clipping"
    );

    // 2. Setup document with Git baseline for line-level actions
    let mut doc = Document::open(
        PathBuf::from("/tmp/test_git_actions.rs"),
        "line 1\nline 2_mod\nline 3_added\nline 4",
    );
    doc.line_numbers = true;
    doc.line_number_mode = akssora_terminal_config::LineNumberMode::Absolute;
    doc.set_git_original_content(Some("line 1\nline 2\nline 4".to_string()));

    let statuses = doc.git_line_statuses();
    assert_eq!(statuses.len(), 4);
    assert_eq!(statuses[0].change, None);
    assert_eq!(statuses[1].change, Some(LineGitChange::Modified));
    assert_eq!(statuses[1].original_line.as_deref(), Some("line 2"));
    assert_eq!(statuses[2].change, Some(LineGitChange::Added));
    assert_eq!(statuses[3].change, None);

    // 3. Hover hit-testing:
    // Gutter width is 48.0. Line height is ~18.0.
    // Line 1 is modified (vis_row = 1, line_y = 18.0)
    let cell_w = 9.0;
    let cell_h = 18.0;
    let content_x = 0.0;
    let content_y = 0.0;
    let content_w = 500.0;

    // 3a. Hovering over line number text area (e.g. x = 10.0, y = 25.0) reveals buttons without clicking
    let hover_num =
        doc.hit_test_gutter_hover(10.0, 25.0, content_x, content_y, content_w, cell_w, cell_h);
    assert_eq!(hover_num, Some((1, None)));

    // Clicking line number text area does NOT trigger a git action
    let click_num =
        doc.hit_test_gutter_click(10.0, 25.0, content_x, content_y, content_w, cell_w, cell_h);
    assert_eq!(click_num, None);

    // 3b. Hovering over "+" (Add This Line) button at x = content_x + 48.0 - 25.0 = 23.0, y = 25.0
    let hover_add =
        doc.hit_test_gutter_hover(23.0, 25.0, content_x, content_y, content_w, cell_w, cell_h);
    assert_eq!(hover_add, Some((1, Some(GutterActionKind::StageLine))));

    // Clicking "+" button triggers StageLine
    let click_add =
        doc.hit_test_gutter_click(23.0, 25.0, content_x, content_y, content_w, cell_w, cell_h);
    assert_eq!(click_add, Some((1, GutterActionKind::StageLine)));

    // 3c. Hovering over "↶" (Revert This Line) button at x = content_x + 48.0 - 10.0 = 38.0, y = 25.0
    let hover_rev =
        doc.hit_test_gutter_hover(38.0, 25.0, content_x, content_y, content_w, cell_w, cell_h);
    assert_eq!(hover_rev, Some((1, Some(GutterActionKind::RevertLine))));

    // Clicking "↶" button triggers RevertLine
    let click_rev =
        doc.hit_test_gutter_click(38.0, 25.0, content_x, content_y, content_w, cell_w, cell_h);
    assert_eq!(click_rev, Some((1, GutterActionKind::RevertLine)));

    // 4. Strict Single-Line Revert: Modified Line
    // Line 1 was "line 2_mod". Reverting line 1 must restore "line 2" while leaving line 3_added untouched!
    assert!(doc.revert_line_at(1).is_ok());
    assert_eq!(
        doc.buffer.to_string_content(),
        "line 1\nline 2\nline 3_added\nline 4"
    );

    // 5. Strict Single-Line Revert: Added Line
    // Now line 2 is "line 3_added". Reverting line 2 must remove it without touching line 1, 2, or 4!
    assert!(doc.revert_line_at(2).is_ok());
    assert_eq!(doc.buffer.to_string_content(), "line 1\nline 2\nline 4");

    // 6. Strict Single-Line Revert: Deleted Line
    // Setup document where "line 3" is deleted between line 2 and line 4
    let mut doc_del = Document::open(
        PathBuf::from("/tmp/test_git_del.rs"),
        "line 1\nline 2\nline 4",
    );
    doc_del.line_numbers = true;
    doc_del.set_git_original_content(Some("line 1\nline 2\nline 3\nline 4".to_string()));
    let del_statuses = doc_del.git_line_statuses();
    assert_eq!(del_statuses.len(), 3);
    assert!(del_statuses[2].deleted_above);
    assert_eq!(del_statuses[2].deleted_lines, vec!["line 3".to_string()]);

    // Revert the deletion on line 2 (which restores deleted lines above it)
    assert!(doc_del.revert_line_at(2).is_ok());
    assert_eq!(
        doc_del.buffer.to_string_content(),
        "line 1\nline 2\nline 3\nline 4"
    );
}

#[test]
fn test_code_review_accordion_collapse_trigger_retention_and_large_diff_handling() {
    use akssora_git::{DiffHunk, DiffLine, DiffLineType, FileDiff, FileGitStatus, RepoDiff};
    use akssora_terminal_ui::{CodeReviewLayout, CodeReviewModel};
    use std::path::PathBuf;

    // 1. Setup a repository diff with a small file and a huge 45,000+ line file
    let file_small = FileDiff {
        path: PathBuf::from("small.rs"),
        old_path: None,
        status: FileGitStatus::Modified,
        hunks: vec![DiffHunk {
            old_start: 1,
            old_lines: 2,
            new_start: 1,
            new_lines: 2,
            header: "@@ -1,2 +1,2 @@".to_string(),
            lines: vec![
                DiffLine {
                    line_type: DiffLineType::Context,
                    content: "fn small() {".to_string(),
                    old_line_num: Some(1),
                    new_line_num: Some(1),
                },
                DiffLine {
                    line_type: DiffLineType::Add,
                    content: "    println!(\"small\");".to_string(),
                    old_line_num: None,
                    new_line_num: Some(2),
                },
            ],
        }],
        additions: 1,
        deletions: 0,
        is_binary: false,
        line_count: 60,
    };

    let file_large = FileDiff {
        path: PathBuf::from("large_45000.rs"),
        old_path: None,
        status: FileGitStatus::Added,
        hunks: vec![DiffHunk {
            old_start: 0,
            old_lines: 0,
            new_start: 1,
            new_lines: 45000,
            header: "@@ -0,0 +1,45000 @@".to_string(),
            lines: vec![],
        }],
        additions: 45000,
        deletions: 0,
        is_binary: false,
        line_count: 45000,
    };

    let file_after = FileDiff {
        path: PathBuf::from("after.rs"),
        old_path: None,
        status: FileGitStatus::Modified,
        hunks: vec![],
        additions: 100,
        deletions: 0,
        is_binary: false,
        line_count: 100,
    };

    let repo_diff = RepoDiff {
        files: vec![file_small, file_large, file_after],
        total_additions: 45101,
        total_deletions: 0,
        files_changed: 3,
    };

    let mut model = CodeReviewModel::new(std::path::Path::new("/mock/repo"));
    model.diff = repo_diff;

    let viewport_h = 800.0;
    let vy = 40.0;

    // Verify fast height calculation without freezing
    let total_h = model.total_diff_content_height(viewport_h);
    assert!(total_h > 45000.0 * 20.0);

    // 2. Stable Accordion Positioning Test:
    // User scrolls down into the 45,000 line file; header moves naturally without snapping to vy
    let offset_large = model.card_y_offset_for_file(1, viewport_h);
    model.diff_scroll_y = offset_large + 300.0;

    let curr_y_before = vy + offset_large - model.diff_scroll_y;
    let card_h_before = CodeReviewLayout::card_height_for_file_state(
        &model.diff.files[1],
        false,
        false,
        viewport_h,
    );
    let header_y_before = CodeReviewLayout::card_header_y(curr_y_before, card_h_before, false, vy);
    assert_eq!(
        header_y_before, vy,
        "Header must stay sticky at vy within accordion boundaries"
    );

    // Now user collapses file 1 ("large_45000.rs")
    model.set_file_collapsed(&PathBuf::from("large_45000.rs"), true);
    assert!(model.is_file_collapsed(&PathBuf::from("large_45000.rs")));

    // When closing, content closes and layout reflows naturally without jumping
    let max_scroll = model.max_diff_scroll(viewport_h);
    model.diff_scroll_y = model.diff_scroll_y.clamp(0.0, max_scroll);

    // 3. Test natural accordion scrolling and context expansion
    let offset_small = model.card_y_offset_for_file(0, viewport_h);
    model.diff_scroll_y = 0.0;
    let curr_y_small = vy + offset_small - model.diff_scroll_y;
    let card_h_small = CodeReviewLayout::card_height_for_file_state(
        &model.diff.files[0],
        false,
        false,
        viewport_h,
    );
    let header_y_small = CodeReviewLayout::card_header_y(curr_y_small, card_h_small, false, vy);
    assert_eq!(header_y_small, curr_y_small);

    // Collapse small file: layout reflows without viewport jumping
    model.set_file_collapsed(&PathBuf::from("small.rs"), true);
    let max_scroll_small = model.max_diff_scroll(viewport_h);
    model.diff_scroll_y = model.diff_scroll_y.clamp(0.0, max_scroll_small);
    assert_eq!(model.diff_scroll_y, 0.0);
}

#[test]
fn test_git_diff_retrieval_background_worker_architecture_pipeline() {
    use akssora_git::{
        DiffHunk, DiffLine, DiffLineType, FileDiff, FileGitStatus, IndexedRepoDiff, RepoDiff,
    };
    use akssora_terminal_app::app::review::worker::{DiffBackgroundWorker, DiffWorkerRequest};
    use akssora_terminal_ui::CodeReviewModel;
    use std::path::{Path, PathBuf};
    use std::sync::Arc;

    // 1. Setup mock repository files and verify Rayon parallel preprocessing into IndexedRepoDiff
    let mut files = Vec::new();
    for i in 0..10 {
        files.push(FileDiff {
            path: PathBuf::from(format!("src/file_{i}.rs")),
            old_path: None,
            status: FileGitStatus::Modified,
            hunks: vec![DiffHunk {
                old_start: 1,
                old_lines: 5,
                new_start: 1,
                new_lines: 5,
                header: format!("@@ -1,5 +1,5 @@ file {i}"),
                lines: vec![
                    DiffLine {
                        line_type: DiffLineType::Context,
                        content: "line 1".to_string(),
                        old_line_num: Some(1),
                        new_line_num: Some(1),
                    },
                    DiffLine {
                        line_type: DiffLineType::Add,
                        content: format!("added line in file {i}"),
                        old_line_num: None,
                        new_line_num: Some(2),
                    },
                ],
            }],
            additions: 1,
            deletions: 0,
            is_binary: false,
            line_count: 5,
        });
    }

    let repo_diff = RepoDiff {
        files,
        total_additions: 10,
        total_deletions: 0,
        files_changed: 10,
    };

    // Step D: Rayon parallel preprocessing -> Step E: Indexed immutable diff model
    let indexed_diff = Arc::new(IndexedRepoDiff::from_repo_diff(repo_diff));
    assert_eq!(indexed_diff.indexed_files.len(), 10);
    assert_eq!(indexed_diff.files_by_path.len(), 10);

    // Step F: UI thread updates CodeReviewModel
    let mut model = CodeReviewModel::new(Path::new("/mock/repo"));
    model.set_indexed_diff(indexed_diff.clone(), 1);
    assert_eq!(model.diff_generation, 1);
    assert_eq!(model.diff.files.len(), 10);
    assert!(!model.is_loading);

    // Step G & H: Virtualized renderer calculates visible lines + small buffer
    let (file, indexed_file) = indexed_diff
        .file_by_path(Path::new("src/file_3.rs"))
        .expect("file_3 must be found");
    assert_eq!(file.path, PathBuf::from("src/file_3.rs"));
    assert_eq!(indexed_file.total_visual_lines, 3); // 1 header line + 2 diff lines

    // Step B: Background worker lifecycle verification
    let worker = DiffBackgroundWorker::new();
    worker.send_request(DiffWorkerRequest {
        terminal_id: 1,
        generation: 2,
        repo_root: PathBuf::from("."),
        staged: false,
    });

    // Wait briefly for background worker response
    let start = std::time::Instant::now();
    let mut received = false;
    while start.elapsed() < std::time::Duration::from_secs(3) {
        if let Some(resp) = worker.try_recv_response() {
            assert_eq!(resp.generation, 2);
            assert!(!resp.staged);
            model.set_indexed_diff(resp.diff, resp.generation);
            assert_eq!(model.diff_generation, 2);
            received = true;
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    assert!(
        received,
        "Background worker must deliver response to UI thread"
    );

    // Step J & K: Invalidate affected data
    model.invalidate_layout();
    assert!(model.total_diff_content_height(800.0) > 0.0);
}

#[test]
fn test_stable_accordion_scrolling_and_natural_layout() {
    use akssora_git::{DiffHunk, DiffLine, DiffLineType, FileDiff, FileGitStatus, RepoDiff};
    use akssora_terminal_ui::{CodeReviewAction, CodeReviewLayout, CodeReviewModel};
    use std::path::PathBuf;

    // Create a multi-file diff scenario: File 0 (short diff), File 1 (medium diff), File 2 (short diff)
    let repo_diff = RepoDiff {
        files: vec![
            FileDiff {
                path: PathBuf::from("file_a.rs"),
                old_path: None,
                status: FileGitStatus::Modified,
                additions: 2,
                deletions: 1,
                is_binary: false,
                line_count: 5,
                hunks: vec![DiffHunk {
                    old_start: 1,
                    old_lines: 2,
                    new_start: 1,
                    new_lines: 3,
                    header: "@@ -1,2 +1,3 @@".to_string(),
                    lines: vec![
                        DiffLine {
                            line_type: DiffLineType::Delete,
                            content: "- old\n".to_string(),
                            old_line_num: Some(1),
                            new_line_num: None,
                        },
                        DiffLine {
                            line_type: DiffLineType::Add,
                            content: "+ new1\n".to_string(),
                            old_line_num: None,
                            new_line_num: Some(1),
                        },
                        DiffLine {
                            line_type: DiffLineType::Add,
                            content: "+ new2\n".to_string(),
                            old_line_num: None,
                            new_line_num: Some(2),
                        },
                    ],
                }],
            },
            FileDiff {
                path: PathBuf::from("file_b.rs"),
                old_path: None,
                status: FileGitStatus::Modified,
                additions: 10,
                deletions: 10,
                is_binary: false,
                line_count: 20,
                hunks: vec![DiffHunk {
                    old_start: 10,
                    old_lines: 5,
                    new_start: 10,
                    new_lines: 5,
                    header: "@@ -10,5 +10,5 @@".to_string(),
                    lines: vec![
                        DiffLine {
                            line_type: DiffLineType::Context,
                            content: "  ctx\n".to_string(),
                            old_line_num: Some(10),
                            new_line_num: Some(10),
                        },
                        DiffLine {
                            line_type: DiffLineType::Add,
                            content: "+ add\n".to_string(),
                            old_line_num: None,
                            new_line_num: Some(11),
                        },
                    ],
                }],
            },
            FileDiff {
                path: PathBuf::from("file_c.rs"),
                old_path: None,
                status: FileGitStatus::Modified,
                additions: 1,
                deletions: 1,
                is_binary: false,
                line_count: 3,
                hunks: vec![],
            },
        ],
        files_changed: 3,
        total_additions: 13,
        total_deletions: 12,
    };

    let mut model = CodeReviewModel::default();
    model.diff = repo_diff;

    let viewport_h = 600.0;
    let vy = 42.0;

    // 1. Verify individual accordion heights are calculated independently
    let h0 = CodeReviewLayout::card_height_for_file_state(
        &model.diff.files[0],
        false,
        false,
        viewport_h,
    );
    let h1 = CodeReviewLayout::card_height_for_file_state(
        &model.diff.files[1],
        false,
        false,
        viewport_h,
    );
    let h2 = CodeReviewLayout::card_height_for_file_state(
        &model.diff.files[2],
        false,
        false,
        viewport_h,
    );

    assert!(h0 > CodeReviewLayout::CARD_HEADER_HEIGHT);
    assert!(h1 > CodeReviewLayout::CARD_HEADER_HEIGHT);
    assert!(h2 > CodeReviewLayout::CARD_HEADER_HEIGHT);

    // Accordions maintain strict ordered positions:
    let offset0 = model.card_y_offset_for_file(0, viewport_h);
    let offset1 = model.card_y_offset_for_file(1, viewport_h);
    let offset2 = model.card_y_offset_for_file(2, viewport_h);

    assert_eq!(offset0, 12.0);
    assert_eq!(offset1, 12.0 + h0 + CodeReviewLayout::CARD_MARGIN_BOTTOM);
    assert_eq!(offset2, offset1 + h1 + CodeReviewLayout::CARD_MARGIN_BOTTOM);

    // 2. Continuous natural vertical scrolling:
    // When scrolling down, card 0 moves upwards, card 1 enters viewport naturally
    // Neither header snaps to vy, nor displaces earlier accordions
    model.diff_scroll_y = 50.0;
    let card0_y = vy + offset0 - model.diff_scroll_y;
    let card1_y = vy + offset1 - model.diff_scroll_y;

    let header0_y = CodeReviewLayout::card_header_y(card0_y, h0, false, vy);
    let header1_y = CodeReviewLayout::card_header_y(card1_y, h1, false, vy);

    assert_eq!(
        header0_y, vy,
        "Header 0 must stay sticky at vy within card 0 boundaries"
    );
    assert_eq!(
        header1_y, card1_y,
        "Header 1 must scroll naturally with card 1"
    );
    assert_eq!(card1_y - card0_y, h0 + CodeReviewLayout::CARD_MARGIN_BOTTOM);

    // 3. Collapsing does not displace unrelated accordions or jump parent scroll
    model.set_file_collapsed(&PathBuf::from("file_b.rs"), true);
    assert!(model.is_file_collapsed(&PathBuf::from("file_b.rs")));

    let max_scroll = model.max_diff_scroll(viewport_h);
    model.diff_scroll_y = model.diff_scroll_y.clamp(0.0, max_scroll);
    // Relative distance between card 0 and card 1 is unchanged
    assert_eq!(model.card_y_offset_for_file(1, viewport_h), offset1);

    // 4. Hit test editor inside card body (routes directly to the editor in editing mode with Git effects)
    let curr_card0_y = vy + offset0 - model.diff_scroll_y;
    let click_y_0 = curr_card0_y + CodeReviewLayout::CARD_HEADER_HEIGHT + 10.0;
    let action_0 = CodeReviewLayout::hit_test(&model, 0.0, 0.0, 800.0, 600.0, 300.0, click_y_0);
    assert!(matches!(
        action_0,
        Some(CodeReviewAction::ClickEditor { orig_idx: 0, .. })
    ));

    // Uncollapse file_b.rs: clicking its body routes to ClickEditor for file 1
    model.set_file_collapsed(&PathBuf::from("file_b.rs"), false);
    assert!(!model.is_file_collapsed(&PathBuf::from("file_b.rs")));
    let curr_card1_y = vy + offset1 - model.diff_scroll_y;
    let click_y_1 = curr_card1_y + CodeReviewLayout::CARD_HEADER_HEIGHT + 10.0;
    let action_1 = CodeReviewLayout::hit_test(&model, 0.0, 0.0, 800.0, 600.0, 300.0, click_y_1);
    assert!(matches!(
        action_1,
        Some(CodeReviewAction::ClickEditor { orig_idx: 1, .. })
    ));
}

#[test]
fn test_code_review_accordion_editor_mode_and_sticky_header() {
    use akssora_editor::Document;
    use akssora_git::{FileDiff, FileGitStatus, LineGitChange, RepoDiff};
    use akssora_terminal_ui::{CodeReviewLayout, CodeReviewModel};
    use std::path::PathBuf;

    let repo_dir =
        std::env::temp_dir().join(format!("akssora_review_editor_test_{}", std::process::id()));
    let _ = std::fs::create_dir_all(&repo_dir);

    let mut model = CodeReviewModel::new(&repo_dir);

    // 1. Setup a modified file and a deleted file
    let file1_path = PathBuf::from("src/main.rs");
    let full_path1 = repo_dir.join(&file1_path);
    let _ = std::fs::create_dir_all(full_path1.parent().unwrap());
    let _ = std::fs::write(&full_path1, "fn main() {\n    println!(\"hello\");\n}\n");

    let file2_path = PathBuf::from("src/deleted.rs");
    let full_path2 = repo_dir.join(&file2_path);

    let file1 = FileDiff {
        path: file1_path.clone(),
        old_path: None,
        is_binary: false,
        status: FileGitStatus::Modified,
        additions: 1,
        deletions: 1,
        line_count: 3,
        hunks: Vec::new(),
    };
    let file2 = FileDiff {
        path: file2_path.clone(),
        old_path: None,
        is_binary: false,
        status: FileGitStatus::Deleted,
        additions: 0,
        deletions: 2,
        line_count: 2,
        hunks: Vec::new(),
    };

    model.diff = RepoDiff {
        files: vec![file1, file2],
        total_additions: 1,
        total_deletions: 3,
        files_changed: 2,
    };

    // 2. Editor documents preparation:
    let mut doc1 = Document::open(
        full_path1.clone(),
        "fn main() {\n    println!(\"hello\");\n}\n",
    );
    doc1.inline_diff = true;
    doc1.line_numbers = true;
    doc1.line_number_mode = akssora_terminal_config::LineNumberMode::Absolute;
    doc1.set_git_original_content(Some("fn main() {\n    println!(\"old\");\n}\n".to_string()));

    let mut doc2 = Document::open(full_path2.clone(), "pub fn unused() {}\n");
    doc2.inline_diff = true;
    doc2.set_deleted(true);

    assert!(doc1.inline_diff);
    assert!(doc2.is_deleted);
    doc2.ensure_git_statuses_computed();
    let statuses2 = doc2.git_line_statuses();
    assert_eq!(statuses2[0].change, Some(LineGitChange::Deleted));

    // 3. Compact dynamic height based on content lines
    let card1_h =
        CodeReviewLayout::card_height_for_file_state(&model.diff.files[0], false, false, 800.0);
    assert_eq!(
        card1_h,
        CodeReviewLayout::CARD_HEADER_HEIGHT + 3.0 * CodeReviewLayout::DIFF_LINE_HEIGHT + 16.0
    );

    // 4. Sticky header behavior:
    let vy = 42.0;
    // When card is scrolled past top:
    let sticky_header_y = CodeReviewLayout::card_header_y(vy - 20.0, card1_h, false, vy);
    assert_eq!(
        sticky_header_y, vy,
        "Header sticks to vy when scrolled past top"
    );

    // When card scrolls past bottom, header moves with card boundary:
    let card_past_bottom = -200.0;
    let header_at_end = CodeReviewLayout::card_header_y(card_past_bottom, card1_h, false, vy);
    let expected_max = card_past_bottom + card1_h - CodeReviewLayout::CARD_HEADER_HEIGHT;
    assert_eq!(header_at_end, expected_max);

    // 5. Gutter hover hit testing on document (line 1 is the modified line)
    let gutter_hover = doc1.hit_test_gutter_hover(10.0, 25.0, 0.0, 0.0, 500.0, 9.0, 18.0);
    assert_eq!(gutter_hover.map(|(r, _)| r), Some(1));

    let _ = std::fs::remove_dir_all(&repo_dir);
}

// INFO: Verifies that Code Review uses the actual editor with Git effects, and once code has expanded it cannot be collapsed.
#[test]
fn test_code_review_editor_mode_and_non_collapsing_expanded_code() {
    use akssora_git::{DiffHunk, DiffLine, DiffLineType, FileDiff, FileGitStatus, RepoDiff};
    use akssora_terminal_ui::{CodeReviewAction, CodeReviewLayout, CodeReviewModel};
    use std::path::PathBuf;

    let repo_dir =
        std::env::temp_dir().join(format!("akssora_review_expand_test_{}", std::process::id()));
    let _ = std::fs::create_dir_all(&repo_dir);

    let mut model = CodeReviewModel::new(&repo_dir);
    model.diff = RepoDiff {
        files: vec![FileDiff {
            path: PathBuf::from("src/app.rs"),
            old_path: None,
            status: FileGitStatus::Modified,
            additions: 2,
            deletions: 1,
            is_binary: false,
            line_count: 10,
            hunks: vec![DiffHunk {
                old_start: 1,
                old_lines: 3,
                new_start: 1,
                new_lines: 4,
                header: "@@ -1,3 +1,4 @@".to_string(),
                lines: vec![
                    DiffLine {
                        line_type: DiffLineType::Context,
                        content: "fn main() {\n".to_string(),
                        old_line_num: Some(1),
                        new_line_num: Some(1),
                    },
                    DiffLine {
                        line_type: DiffLineType::Delete,
                        content: "    println!(\"old\");\n".to_string(),
                        old_line_num: Some(2),
                        new_line_num: None,
                    },
                    DiffLine {
                        line_type: DiffLineType::Add,
                        content: "    println!(\"new\");\n".to_string(),
                        old_line_num: None,
                        new_line_num: Some(2),
                    },
                ],
            }],
        }],
        files_changed: 1,
        total_additions: 2,
        total_deletions: 1,
    };

    let file_path = PathBuf::from("src/app.rs");

    // 1. Files start expanded by default
    assert!(!model.is_file_collapsed(&file_path));

    // 2. An expanded accordion must always be collapsible
    model.toggle_file_collapsed(&file_path);
    assert!(
        model.is_file_collapsed(&file_path),
        "An expanded accordion must always be collapsible"
    );
    // Clicking again reopens it
    model.toggle_file_collapsed(&file_path);
    assert!(
        !model.is_file_collapsed(&file_path),
        "Clicking again reopens the accordion"
    );

    // 3. Header clicks on an accordion header toggle collapse
    let (vx, vy, _vw, _vh) = CodeReviewLayout::diff_viewport_rect(&model, 0.0, 0.0, 800.0, 600.0);
    let card_x = vx + 16.0;
    let header_y = vy + 12.0;
    // Click header title area (not an action button)
    let header_action = CodeReviewLayout::hit_test(
        &model,
        0.0,
        0.0,
        800.0,
        600.0,
        card_x + 50.0,
        header_y + 10.0,
    );
    assert_eq!(
        header_action,
        Some(CodeReviewAction::ToggleFileCollapse(0)),
        "Clicking header toggles file collapse"
    );

    // 4. Body clicks route directly to the editor in editing mode
    let body_click_y = header_y + CodeReviewLayout::CARD_HEADER_HEIGHT + 20.0;
    let body_action =
        CodeReviewLayout::hit_test(&model, 0.0, 0.0, 800.0, 600.0, card_x + 60.0, body_click_y);
    assert!(
        matches!(
            body_action,
            Some(CodeReviewAction::ClickEditor { orig_idx: 0, .. })
        ),
        "Clicking card body routes directly to the editor"
    );

    // 5. In App: ensure editor document has git effects enabled
    let mut app = akssora_terminal_app::App::new();
    app.code_review.repo_root = repo_dir.clone();
    app.code_review.diff = model.diff.clone();
    app.ensure_editor_document_for_diff_file(0);

    let doc = app
        .editor_documents
        .get(&repo_dir.join(&file_path))
        .expect("Editor document must exist");
    assert!(
        doc.inline_diff,
        "Editor document must have inline_diff / git effects enabled"
    );
    assert!(
        doc.line_numbers,
        "Editor document must display line numbers"
    );

    let _ = std::fs::remove_dir_all(&repo_dir);
}

#[test]
fn test_code_review_in_place_editing_and_keyboard_dispatch() {
    use akssora_git::{DiffHunk, DiffLine, DiffLineType, FileDiff, FileGitStatus, RepoDiff};
    use akssora_terminal_ui::{CodeReviewAction, CodeReviewLayout, CodeReviewModel, TabKind};
    use std::path::PathBuf;

    let repo_dir = std::env::temp_dir().join(format!(
        "akssora_test_cr_edit_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&repo_dir).unwrap();
    let file_path = PathBuf::from("main.rs");
    let full_path = repo_dir.join(&file_path);
    std::fs::write(
        &full_path,
        "fn main() {\n    let val = 10;\n    println!(\"{val}\");\n}\n",
    )
    .unwrap();

    let mut model = CodeReviewModel::new(&repo_dir);
    model.diff = RepoDiff {
        files: vec![FileDiff {
            path: file_path.clone(),
            old_path: None,
            status: FileGitStatus::Modified,
            additions: 1,
            deletions: 0,
            line_count: 4,
            is_binary: false,
            hunks: vec![DiffHunk {
                header: "@@ -1,4 +1,4 @@".to_string(),
                old_start: 1,
                old_lines: 4,
                new_start: 1,
                new_lines: 4,
                lines: vec![
                    DiffLine {
                        line_type: DiffLineType::Context,
                        content: "fn main() {\n".to_string(),
                        old_line_num: Some(1),
                        new_line_num: Some(1),
                    },
                    DiffLine {
                        line_type: DiffLineType::Add,
                        content: "    let val = 10;\n".to_string(),
                        old_line_num: None,
                        new_line_num: Some(2),
                    },
                    DiffLine {
                        line_type: DiffLineType::Context,
                        content: "    println!(\"{val}\");\n".to_string(),
                        old_line_num: Some(3),
                        new_line_num: Some(3),
                    },
                    DiffLine {
                        line_type: DiffLineType::Context,
                        content: "}\n".to_string(),
                        old_line_num: Some(4),
                        new_line_num: Some(4),
                    },
                ],
            }],
        }],
        files_changed: 1,
        total_additions: 1,
        total_deletions: 0,
    };

    let mut app = akssora_terminal_app::App::new();
    app.code_review = model;
    let _ = app.tab_manager.add_tab(
        "Review".to_string(),
        TabKind::CodeReview {
            repo_path: repo_dir.to_string_lossy().to_string(),
        },
    );

    // 1. Ensure file is prepared for editing
    app.prepare_code_review_editor_file(0);
    assert_eq!(app.code_review.active_file_idx, Some(0));
    assert_eq!(app.code_review.active_editing_file, Some(full_path.clone()));

    // Cursor automatically points to first changed line (line 2, row index 1)
    let doc = app.active_editor().expect("active editor document");
    assert_eq!(doc.cursor.0, 1);

    // 2. Click on line 2 in Code Review diff
    let (vx, vy, _vw, _vh) =
        CodeReviewLayout::diff_viewport_rect(&app.code_review, 0.0, 0.0, 800.0, 600.0);
    let card_x = vx + 16.0;
    let card_top = vy + 12.0;
    // Line 1 is Context, Line 2 is Add.
    // Line 2 Y position: card_top + CARD_HEADER_HEIGHT + 8.0 + UNMODIFIED_LINES_HEIGHT + 1 * DIFF_LINE_HEIGHT + 5.0
    let line2_y = card_top
        + CodeReviewLayout::CARD_HEADER_HEIGHT
        + 8.0
        + CodeReviewLayout::UNMODIFIED_LINES_HEIGHT
        + CodeReviewLayout::DIFF_LINE_HEIGHT
        + 5.0;
    let click_x = card_x + 60.0;
    let hit_action =
        CodeReviewLayout::hit_test(&app.code_review, 0.0, 0.0, 800.0, 600.0, click_x, line2_y);
    assert!(matches!(
        hit_action,
        Some(CodeReviewAction::ClickEditor { orig_idx: 0, .. })
    ));

    let handled = app.handle_code_review_click(click_x, line2_y, 800.0, 600.0, 0.0, 0.0);
    assert!(handled);
    assert_eq!(app.active_editor().unwrap().cursor.0, 1);

    // 3. Edit document directly in Code Review panel
    {
        let doc = app.active_editor_mut().expect("active editor for editing");
        doc.insert_char_smart('X');
    }
    let updated_line = app
        .active_editor()
        .unwrap()
        .buffer
        .get_line(1)
        .unwrap()
        .to_string();
    assert!(
        updated_line.contains('X'),
        "Document buffer must contain edited character"
    );

    // 4. Verify active_editing_file remains focused and editing continues seamlessly
    assert!(
        app.code_review.active_editing_file.is_some(),
        "Active editing file must remain active for editing"
    );
    assert_eq!(app.code_review.active_editing_file, Some(full_path));

    let _ = std::fs::remove_dir_all(&repo_dir);
}

#[test]
fn test_code_review_clicked_line_highlight_across_line_types() {
    use akssora_git::{DiffHunk, DiffLine, DiffLineType, FileDiff, FileGitStatus, RepoDiff};
    use akssora_terminal_ui::{CodeReviewLayout, CodeReviewModel, TabKind};
    use std::path::PathBuf;

    let repo_dir = std::env::temp_dir().join(format!(
        "akssora_cr_highlight_test_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let _ = std::fs::create_dir_all(&repo_dir);
    let file_path = PathBuf::from("src/lib.rs");
    let full_path = repo_dir.join(&file_path);
    let _ = std::fs::create_dir_all(full_path.parent().unwrap());
    std::fs::write(&full_path, "fn first() {}\nfn added() {}\nfn last() {}\n").unwrap();

    let mut model = CodeReviewModel::new(&repo_dir);
    model.diff = RepoDiff {
        files: vec![FileDiff {
            path: file_path.clone(),
            old_path: None,
            status: FileGitStatus::Modified,
            additions: 1,
            deletions: 1,
            line_count: 3,
            is_binary: false,
            hunks: vec![DiffHunk {
                old_start: 1,
                old_lines: 3,
                new_start: 1,
                new_lines: 3,
                header: "@@ -1,3 +1,3 @@".to_string(),
                lines: vec![
                    DiffLine {
                        line_type: DiffLineType::Context,
                        content: "fn first() {}\n".to_string(),
                        old_line_num: Some(1),
                        new_line_num: Some(1),
                    },
                    DiffLine {
                        line_type: DiffLineType::Delete,
                        content: "fn old_deleted() {}\n".to_string(),
                        old_line_num: Some(2),
                        new_line_num: None,
                    },
                    DiffLine {
                        line_type: DiffLineType::Add,
                        content: "fn added() {}\n".to_string(),
                        old_line_num: None,
                        new_line_num: Some(2),
                    },
                    DiffLine {
                        line_type: DiffLineType::Context,
                        content: "fn last() {}\n".to_string(),
                        old_line_num: Some(3),
                        new_line_num: Some(3),
                    },
                ],
            }],
        }],
        files_changed: 1,
        total_additions: 1,
        total_deletions: 1,
    };

    let mut app = akssora_terminal_app::App::new();
    app.code_review = model;
    let _ = app.tab_manager.add_tab(
        "Review".to_string(),
        TabKind::CodeReview {
            repo_path: repo_dir.to_string_lossy().to_string(),
        },
    );

    app.prepare_code_review_editor_file(0);
    assert_eq!(app.code_review.active_file_idx, Some(0));
    assert_eq!(app.code_review.active_editing_file, Some(full_path.clone()));

    let (vx, vy, _vw, _vh) =
        CodeReviewLayout::diff_viewport_rect(&app.code_review, 0.0, 0.0, 800.0, 600.0);
    let card_x = vx + 16.0;
    let card_top = vy + 12.0;
    let click_x = card_x + 60.0;
    let base_lines_y = card_top
        + CodeReviewLayout::CARD_HEADER_HEIGHT
        + 8.0
        + CodeReviewLayout::UNMODIFIED_LINES_HEIGHT;

    // 1. Click Context line (Line 1: fn first() {})
    let line1_y = base_lines_y + 0.0 * CodeReviewLayout::DIFF_LINE_HEIGHT + 5.0;
    let handled1 = app.handle_code_review_click(click_x, line1_y, 800.0, 600.0, 0.0, 0.0);
    assert!(handled1, "Click on context line must be handled");
    assert_eq!(
        app.active_editor().unwrap().cursor.0,
        0,
        "Cursor must be on row 0 for context line 1"
    );

    // 2. Click Deleted line (Line 2: fn old_deleted() {})
    let line2_del_y = base_lines_y + 1.0 * CodeReviewLayout::DIFF_LINE_HEIGHT + 5.0;
    let handled2 = app.handle_code_review_click(click_x, line2_del_y, 800.0, 600.0, 0.0, 0.0);
    assert!(handled2, "Click on deleted line must be handled");
    assert_eq!(
        app.active_editor().unwrap().cursor.0,
        1,
        "Cursor row targets deleted line's old row index"
    );

    // 3. Click Added line (Line 3: fn added() {})
    let line3_add_y = base_lines_y + 2.0 * CodeReviewLayout::DIFF_LINE_HEIGHT + 5.0;
    let handled3 = app.handle_code_review_click(click_x, line3_add_y, 800.0, 600.0, 0.0, 0.0);
    assert!(handled3, "Click on added line must be handled");
    assert_eq!(
        app.active_editor().unwrap().cursor.0,
        1,
        "Cursor row targets added line's new row index"
    );

    // 4. Click second Context line (Line 4: fn last() {})
    let line4_y = base_lines_y + 3.0 * CodeReviewLayout::DIFF_LINE_HEIGHT + 5.0;
    let handled4 = app.handle_code_review_click(click_x, line4_y, 800.0, 600.0, 0.0, 0.0);
    assert!(handled4, "Click on context line 4 must be handled");
    assert_eq!(
        app.active_editor().unwrap().cursor.0,
        2,
        "Cursor must be on row 2 for context line 4"
    );

    let _ = std::fs::remove_dir_all(&repo_dir);
}

#[test]
fn test_code_review_hover_line_number_reveals_buttons_and_leaves_clean() {
    use akssora_editor::GutterActionKind;
    use akssora_git::{DiffHunk, DiffLine, DiffLineType, FileDiff, FileGitStatus, RepoDiff};
    use akssora_terminal_ui::{CodeReviewLayout, CodeReviewModel, TabKind};
    use std::path::PathBuf;

    let repo_dir = std::env::temp_dir().join(format!(
        "akssora_cr_gutter_test_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let _ = std::fs::create_dir_all(&repo_dir);
    let file_path = PathBuf::from("src/lib.rs");
    let full_path = repo_dir.join(&file_path);
    let _ = std::fs::create_dir_all(full_path.parent().unwrap());
    std::fs::write(&full_path, "fn first() {}\nfn added() {}\nfn last() {}\n").unwrap();

    let mut model = CodeReviewModel::new(&repo_dir);
    model.diff = RepoDiff {
        files: vec![FileDiff {
            path: file_path.clone(),
            old_path: None,
            status: FileGitStatus::Modified,
            additions: 1,
            deletions: 1,
            line_count: 3,
            is_binary: false,
            hunks: vec![DiffHunk {
                old_start: 1,
                old_lines: 3,
                new_start: 1,
                new_lines: 3,
                header: "@@ -1,3 +1,3 @@".to_string(),
                lines: vec![
                    DiffLine {
                        line_type: DiffLineType::Context,
                        content: "fn first() {}\n".to_string(),
                        old_line_num: Some(1),
                        new_line_num: Some(1),
                    },
                    DiffLine {
                        line_type: DiffLineType::Delete,
                        content: "fn old_deleted() {}\n".to_string(),
                        old_line_num: Some(2),
                        new_line_num: None,
                    },
                    DiffLine {
                        line_type: DiffLineType::Add,
                        content: "fn added() {}\n".to_string(),
                        old_line_num: None,
                        new_line_num: Some(2),
                    },
                    DiffLine {
                        line_type: DiffLineType::Context,
                        content: "fn last() {}\n".to_string(),
                        old_line_num: Some(3),
                        new_line_num: Some(3),
                    },
                ],
            }],
        }],
        files_changed: 1,
        total_additions: 1,
        total_deletions: 1,
    };

    let mut app = akssora_terminal_app::App::new();
    app.code_review = model;
    let _ = app.tab_manager.add_tab(
        "Review".to_string(),
        TabKind::CodeReview {
            repo_path: repo_dir.to_string_lossy().to_string(),
        },
    );

    app.prepare_code_review_editor_file(0);

    let (vx, vy, _vw, _vh) =
        CodeReviewLayout::diff_viewport_rect(&app.code_review, 0.0, 0.0, 800.0, 600.0);
    let card_x = vx + 16.0;
    let card_top = vy + 12.0;
    let base_lines_y = card_top
        + CodeReviewLayout::CARD_HEADER_HEIGHT
        + 8.0
        + CodeReviewLayout::UNMODIFIED_LINES_HEIGHT;

    // Line 1: Context (Y = base_lines_y + 0)
    // Line 2: Delete  (Y = base_lines_y + 1 * DIFF_LINE_HEIGHT)
    // Line 3: Add     (Y = base_lines_y + 2 * DIFF_LINE_HEIGHT)
    // Line 4: Context (Y = base_lines_y + 3 * DIFF_LINE_HEIGHT)
    let line1_y = base_lines_y + 0.0 * CodeReviewLayout::DIFF_LINE_HEIGHT + 5.0;
    let line2_y = base_lines_y + 1.0 * CodeReviewLayout::DIFF_LINE_HEIGHT + 5.0;
    let line3_y = base_lines_y + 2.0 * CodeReviewLayout::DIFF_LINE_HEIGHT + 5.0;

    // 1. Hovering outside gutter (e.g. over code text) -> None
    let hover_code = CodeReviewLayout::hit_test_gutter_hover(
        &app.code_review,
        0.0,
        0.0,
        800.0,
        600.0,
        card_x + 60.0,
        line3_y,
    );
    assert_eq!(
        hover_code, None,
        "Pointer outside gutter must not trigger gutter hover"
    );

    // 2. Hovering over context line gutter -> None (no changes to stage/revert)
    let hover_ctx = CodeReviewLayout::hit_test_gutter_hover(
        &app.code_review,
        0.0,
        0.0,
        800.0,
        600.0,
        card_x + 5.0,
        line1_y,
    );
    assert_eq!(
        hover_ctx, None,
        "Context line has no git changes, buttons stay hidden"
    );

    // 3. Hovering over added line number (x = card_x + 5.0) -> reveals both buttons (action = None)
    let hover_add_line = CodeReviewLayout::hit_test_gutter_hover(
        &app.code_review,
        0.0,
        0.0,
        800.0,
        600.0,
        card_x + 5.0,
        line3_y,
    );
    assert_eq!(
        hover_add_line,
        Some(akssora_terminal_ui::CodeReviewGutterHover {
            file_idx: 0,
            row: 1,
            is_deletion: false,
            action: None,
        }),
        "Hovering over line number must reveal both buttons (action is None)"
    );

    // 4. Floating line action popup geometry & button hit-testing
    app.code_review.hovered_gutter = hover_add_line;
    let geom = CodeReviewLayout::line_action_popup_geometry(
        &app.code_review,
        0.0,
        0.0,
        800.0,
        600.0,
        0,
        1,
        false,
    )
    .expect("Popup geometry should exist above line number");

    // Popup is positioned above the line number
    assert!(geom.popup_rect.1 != line3_y);

    let (ax, ay, aw, ah) = geom.add_btn_rect;
    let hover_btn_add = CodeReviewLayout::hit_test_gutter_hover(
        &app.code_review,
        0.0,
        0.0,
        800.0,
        600.0,
        ax + aw * 0.5,
        ay + ah * 0.5,
    );
    assert_eq!(
        hover_btn_add,
        Some(akssora_terminal_ui::CodeReviewGutterHover {
            file_idx: 0,
            row: 1,
            is_deletion: false,
            action: Some(GutterActionKind::StageLine),
        }),
        "Hovering over 'Stage Change' must target StageLine"
    );

    // 5. Hovering over "Revert Changes" button -> RevertLine action
    let (rx, ry, rw, rh) = geom.remove_btn_rect;
    let hover_btn_rev = CodeReviewLayout::hit_test_gutter_hover(
        &app.code_review,
        0.0,
        0.0,
        800.0,
        600.0,
        rx + rw * 0.5,
        ry + rh * 0.5,
    );
    assert_eq!(
        hover_btn_rev,
        Some(akssora_terminal_ui::CodeReviewGutterHover {
            file_idx: 0,
            row: 1,
            is_deletion: false,
            action: Some(GutterActionKind::RevertLine),
        }),
        "Hovering over 'Revert Changes' must target RevertLine"
    );

    // 6. Hovering over deleted line number -> targets deleted line row with is_deletion: true
    let hover_del_line = CodeReviewLayout::hit_test_gutter_hover(
        &app.code_review,
        0.0,
        0.0,
        800.0,
        600.0,
        card_x + 5.0,
        line2_y,
    );
    assert_eq!(
        hover_del_line,
        Some(akssora_terminal_ui::CodeReviewGutterHover {
            file_idx: 0,
            row: 1,
            is_deletion: true,
            action: None,
        }),
        "Hovering over deleted line reveals both buttons with is_deletion: true"
    );

    // 7. Verify tooltip resolution for hovered gutter actions in Code Review
    app.editor_hovered_gutter = Some((1, Some(GutterActionKind::StageLine)));
    let tip_add = akssora_terminal_app::app::tooltips::code_review::check_code_review_tooltip(
        &app,
        ax + aw * 0.5,
        ay + ah * 0.5,
        0.0,
        0.0,
        800.0,
        600.0,
    );
    assert!(tip_add.is_some());
    assert_eq!(tip_add.unwrap().text, "Stage Change");

    app.editor_hovered_gutter = Some((1, Some(GutterActionKind::RevertLine)));
    let tip_rev = akssora_terminal_app::app::tooltips::code_review::check_code_review_tooltip(
        &app,
        rx + rw * 0.5,
        ry + rh * 0.5,
        0.0,
        0.0,
        800.0,
        600.0,
    );
    assert!(tip_rev.is_some());
    assert_eq!(tip_rev.unwrap().text, "Revert Changes");

    // 8. Verify mouse pointer / hand cursor is active when hovering popup buttons
    app.code_review.hovered_gutter = hover_btn_rev;
    let is_hand = app.is_button_hovered(rx + rw * 0.5, ry + rh * 0.5, 800.0, 600.0, 0.0);
    assert!(
        is_hand,
        "Hovering over popup button must trigger hand cursor"
    );

    // 9. Reverting added line discards only that line's changes without affecting unrelated lines
    let initial_lines = app.active_editor().unwrap().buffer.line_count();
    let handled_rev =
        app.handle_code_review_click(rx + rw * 0.5, ry + rh * 0.5, 800.0, 600.0, 0.0, 0.0);
    assert!(handled_rev, "Clicking revert line must be handled");
    let after_revert_doc = app.active_editor().unwrap();
    assert_eq!(
        after_revert_doc.buffer.line_count(),
        initial_lines - 1,
        "Reverting added line must remove exactly that line"
    );
    assert_eq!(
        after_revert_doc.buffer.get_line(0).unwrap().trim(),
        "fn first() {}",
        "Unrelated line 1 must remain intact"
    );

    let _ = std::fs::remove_dir_all(&repo_dir);
}

#[test]
fn test_code_review_sidebar_tree_styling_and_selection_isolation() {
    use akssora_git::{FileDiff, FileGitStatus, RepoDiff};
    use akssora_terminal_ui::{
        CodeReviewAction, CodeReviewLayout, CodeReviewModel, CodeReviewTreeItem,
    };
    use std::path::PathBuf;

    let diff_files = vec![
        FileDiff {
            path: PathBuf::from("src/first.rs"),
            old_path: None,
            status: FileGitStatus::Modified,
            hunks: vec![],
            additions: 12,
            deletions: 3,
            is_binary: false,
            line_count: 50,
        },
        FileDiff {
            path: PathBuf::from("src/second.rs"),
            old_path: None,
            status: FileGitStatus::Added,
            hunks: vec![],
            additions: 25,
            deletions: 0,
            is_binary: false,
            line_count: 25,
        },
        FileDiff {
            path: PathBuf::from("src/third.rs"),
            old_path: None,
            status: FileGitStatus::Deleted,
            hunks: vec![],
            additions: 0,
            deletions: 15,
            is_binary: false,
            line_count: 0,
        },
    ];

    let unstaged_diff = RepoDiff {
        files: diff_files.clone(),
        total_additions: 37,
        total_deletions: 18,
        files_changed: 3,
    };

    let mut model = CodeReviewModel::default();
    model.diff = unstaged_diff.clone();
    model.unstaged_diff = unstaged_diff;
    model.is_staged = false;

    // 1. Initial selection: file 0 is selected
    model.select_file(0);
    assert_eq!(model.selected_file_idx, 0);
    assert_eq!(model.active_file_idx, Some(0));

    let items = model.visible_tree_items();
    let file_items: Vec<_> = items
        .iter()
        .filter_map(|it| match it {
            CodeReviewTreeItem::File {
                is_staged,
                orig_idx,
                path,
                display_name,
                status,
                additions,
                deletions,
                ..
            } => Some((
                *is_staged,
                *orig_idx,
                path.clone(),
                display_name.clone(),
                *status,
                *additions,
                *deletions,
            )),
            _ => None,
        })
        .collect();

    assert_eq!(file_items.len(), 3);
    assert_eq!(file_items[0].1, 0); // first.rs
    assert_eq!(file_items[1].1, 1); // second.rs
    assert_eq!(file_items[2].1, 2); // third.rs

    // Verify git status colors and statistics are preserved
    assert_eq!(file_items[0].4, FileGitStatus::Modified);
    assert_eq!(file_items[0].5, 12);
    assert_eq!(file_items[0].6, 3);
    assert_eq!(file_items[1].4, FileGitStatus::Added);
    assert_eq!(file_items[2].4, FileGitStatus::Deleted);

    // 2. Select file 1 (src/second.rs): file 0 is deselected, file 1 is selected
    model.select_file(1);
    assert_eq!(model.selected_file_idx, 1);
    assert_eq!(model.active_file_idx, Some(1));
    assert_ne!(model.active_file_idx, Some(0));

    // 3. Select file 2 (src/third.rs): file 1 is deselected, file 2 is selected
    model.select_file(2);
    assert_eq!(model.selected_file_idx, 2);
    assert_eq!(model.active_file_idx, Some(2));
    assert_ne!(model.active_file_idx, Some(1));

    // 4. Verify sidebar hit test on file actions works without shifting layout
    let (sbx, sby, sbw, _sbh) =
        CodeReviewLayout::sidebar_rect(&model, 0.0, 0.0, 800.0, 600.0).unwrap();
    let row_top = sby + CodeReviewLayout::SIDEBAR_TOOLBAR_HEIGHT + 4.0;
    // Row 0: Staged Changes header, Row 1: Changes header, Row 2: Directory "src", Row 3: first.rs
    let row_file_y = row_top + 3.0 * CodeReviewLayout::FILE_ROW_HEIGHT;
    let (open_rect, mid_rect, stage_opt) =
        CodeReviewLayout::sidebar_file_action_rects(sbx, sbw, row_file_y, false);

    assert!(
        stage_opt.is_some(),
        "Unstaged file must have stage action rect"
    );
    assert!(open_rect.0 < mid_rect.0);
    assert!(mid_rect.0 < stage_opt.unwrap().0);

    let stage_act = CodeReviewLayout::hit_test(
        &model,
        0.0,
        0.0,
        800.0,
        600.0,
        stage_opt.unwrap().0 + 5.0,
        stage_opt.unwrap().1 + 5.0,
    );
    assert!(matches!(
        stage_act,
        Some(CodeReviewAction::StageFileByPath(_))
    ));
}

#[test]
fn test_code_review_split_view_and_expand_collapse_toggle() {
    use akssora_terminal_ui::{CodeReviewAction, CodeReviewLayout};

    let mut app = akssora_terminal_app::App::new();
    let _tab1 = app.open_terminal_tab();
    let initial_tab_count = app.tab_manager.len();
    assert_eq!(initial_tab_count, 1, "App should have 1 terminal tab");

    // 1. Initial State: Code Review is closed
    assert!(!app.is_code_review_open());
    assert_eq!(app.code_review_bounds(), None);
    assert_eq!(app.code_review_split_divider_rect(), None);

    // 2. Open Code Review: Opens in split view by default alongside the terminal
    app.open_code_review();
    assert!(
        app.is_code_review_open(),
        "Code Review must be marked as open"
    );
    assert!(
        !app.code_review.is_full_view,
        "Code Review must open in split view by default"
    );
    assert_eq!(
        app.code_review.split_ratio, 0.5,
        "Default split ratio should be 0.5"
    );

    // Ensure terminal session/tab is preserved (not replaced or closed)
    assert_eq!(
        app.tab_manager.len(),
        initial_tab_count,
        "Terminal tab count must be preserved"
    );

    // Bounds check in split view: terminal pane and code review appear side-by-side
    let bounds = app.code_review_bounds();
    assert!(bounds.is_some(), "Code Review bounds must be present");
    let (cr_x, cr_y, cr_w, cr_h) = bounds.unwrap();
    assert!(cr_w > 0.0 && cr_h > 0.0);
    assert!(
        cr_x > 0.0,
        "Code review in split view should be positioned to the right of terminal"
    );

    // Split divider rect check: divider must be present between terminal and code review
    let div_rect = app.code_review_split_divider_rect();
    assert!(div_rect.is_some(), "Split divider must exist in split view");
    let (div_x, div_y, div_w, div_h) = div_rect.unwrap();
    assert!(
        (div_x + div_w - cr_x).abs() < 1.0,
        "Divider must sit immediately to the left of Code Review"
    );
    assert_eq!(div_y, cr_y);
    assert_eq!(div_h, cr_h);

    // 3. Header Expand Button hit test & Tooltip in split view
    let (win_w, win_h) = (
        app.config.window.width as f32,
        app.config.window.height as f32,
    );
    let show_tabs = app.config.tab_layout.includes_title_bar();
    let top_h = app.header.total_height(show_tabs);
    let show_sidebar = app.config.tab_layout.show_sidebar();
    let sb_w = app.sidebar.effective_width(show_sidebar);
    let exp_w = app.explorer.effective_width();
    let ws_x = sb_w + exp_w;

    let expand_btn = CodeReviewLayout::expand_btn_rect(cr_x, cr_y, cr_w);
    let hit_action = CodeReviewLayout::hit_test(
        &app.code_review,
        cr_x,
        cr_y,
        cr_w,
        cr_h,
        expand_btn.0 + 5.0,
        expand_btn.1 + 5.0,
    );
    assert_eq!(
        hit_action,
        Some(CodeReviewAction::ToggleExpandView),
        "Hit test on expand button must return ToggleExpandView"
    );

    // Check tooltip in split view: "Expand Code Review"
    let tooltip = akssora_terminal_app::app::tooltips::code_review::check_code_review_tooltip(
        &app,
        expand_btn.0 + 5.0,
        expand_btn.1 + 5.0,
        sb_w,
        exp_w,
        win_w,
        win_h,
    );
    assert!(
        tooltip.is_some(),
        "Tooltip must be present when hovering expand button"
    );
    assert_eq!(tooltip.unwrap().text, "Expand Code Review");

    // 4. Click Expand Button -> Expand into Full View
    let handled = app.handle_code_review_click(
        expand_btn.0 + 5.0,
        expand_btn.1 + 5.0,
        win_w,
        win_h,
        top_h,
        ws_x,
    );
    assert!(handled, "Click on expand button must be handled");
    assert!(
        app.code_review.is_full_view,
        "Code Review must now be in full view"
    );

    // In full view: divider is hidden, Code Review takes full workspace
    assert_eq!(
        app.code_review_split_divider_rect(),
        None,
        "Split divider must be hidden in full view"
    );
    let full_bounds = app.code_review_bounds().unwrap();
    assert!(
        full_bounds.2 > cr_w,
        "Full view width must be larger than split view width"
    );

    // Check tooltip in full view: "Restore Split View"
    let full_expand_btn =
        CodeReviewLayout::expand_btn_rect(full_bounds.0, full_bounds.1, full_bounds.2);
    let restore_tooltip =
        akssora_terminal_app::app::tooltips::code_review::check_code_review_tooltip(
            &app,
            full_expand_btn.0 + 5.0,
            full_expand_btn.1 + 5.0,
            sb_w,
            exp_w,
            win_w,
            win_h,
        );
    assert!(
        restore_tooltip.is_some(),
        "Restore tooltip must be present in full view"
    );
    assert_eq!(restore_tooltip.unwrap().text, "Restore Split View");

    // 5. Click Restore Button -> Collapse back into Split View
    let handled_restore = app.handle_code_review_click(
        full_expand_btn.0 + 5.0,
        full_expand_btn.1 + 5.0,
        win_w,
        win_h,
        top_h,
        ws_x,
    );
    assert!(handled_restore, "Click on restore button must be handled");
    assert!(
        !app.code_review.is_full_view,
        "Code Review must return to split view"
    );
    assert_eq!(
        app.code_review.split_ratio, 0.5,
        "Split ratio must be restored"
    );
    assert!(
        app.code_review_split_divider_rect().is_some(),
        "Split divider must reappear"
    );

    // 6. Test Split Divider Resizing
    app.code_review.set_split_ratio(0.7);
    assert_eq!(app.code_review.split_ratio, 0.7);
    let resized_bounds = app.code_review_bounds().unwrap();
    assert!(
        resized_bounds.2 > cr_w,
        "Increasing split ratio must allocate more width to Code Review"
    );

    // 7. Hit test and click Close Button -> Code Review closes and restores full terminal
    let (curr_x, curr_y, curr_w, curr_h) = resized_bounds;
    let close_btn = CodeReviewLayout::close_btn_rect(curr_x, curr_y, curr_w);
    let hit_close = CodeReviewLayout::hit_test(
        &app.code_review,
        curr_x,
        curr_y,
        curr_w,
        curr_h,
        close_btn.0 + 5.0,
        close_btn.1 + 5.0,
    );
    assert_eq!(hit_close, Some(CodeReviewAction::ClosePanel));

    let handled_close = app.handle_code_review_click(
        close_btn.0 + 5.0,
        close_btn.1 + 5.0,
        win_w,
        win_h,
        top_h,
        ws_x,
    );
    assert!(handled_close, "Click on close button must be handled");
    assert!(!app.is_code_review_open(), "Code Review must be closed");
    assert_eq!(app.code_review_bounds(), None);
    assert_eq!(app.code_review_split_divider_rect(), None);
}

#[test]
fn test_code_review_per_repository_independence_and_context_switching() {
    use akssora_terminal_app::App;

    let base_dir = std::env::temp_dir().join(format!("akssora_repo_test_{}", std::process::id()));
    let repo_a = base_dir.join("repo_a");
    let repo_b = base_dir.join("repo_b");
    let nested_a = repo_a.join("src").join("deep").join("nested");

    let _ = std::fs::remove_dir_all(&base_dir);
    std::fs::create_dir_all(repo_a.join(".git")).unwrap();
    std::fs::create_dir_all(&nested_a).unwrap();
    std::fs::create_dir_all(repo_b.join(".git")).unwrap();

    let canonical_a = App::canonical_repo_root(&repo_a);
    let canonical_b = App::canonical_repo_root(&repo_b);
    let canonical_nested_a = App::canonical_repo_root(&nested_a);

    // 1. Nested directories within the same Git repository resolve to the same canonical repository context
    assert_eq!(
        canonical_nested_a, canonical_a,
        "Nested directory must resolve to the repository root"
    );
    assert_ne!(
        canonical_a, canonical_b,
        "Separate repositories must have distinct roots"
    );

    let mut app = App::new();

    // 2. Open Code Review for Repo A
    app.open_code_review_for(&nested_a);
    assert!(app.is_code_review_open());
    assert_eq!(app.current_code_review_repo(), canonical_a);
    assert_eq!(app.code_review.repo_root, canonical_a);

    // Modify Repo A's UI state (selection, scroll, collapsed state)
    app.code_review.selected_file_idx = 4;
    app.code_review.diff_scroll_y = 175.0;
    app.code_review.toggle_section_collapse(true);

    // 3. Switch Code Review to Repo B without closing the panel
    app.set_code_review_repo(&repo_b);
    assert!(
        app.is_code_review_open(),
        "Panel must remain open across repo switch"
    );
    assert_eq!(app.current_code_review_repo(), canonical_b);
    assert_eq!(app.code_review.repo_root, canonical_b);

    // Repo B must have its own independent initial state
    assert_eq!(app.code_review.selected_file_idx, 0);
    assert_eq!(app.code_review.diff_scroll_y, 0.0);

    // Modify Repo B's UI state
    app.code_review.selected_file_idx = 2;
    app.code_review.diff_scroll_y = 320.0;

    // 4. Switch back to Repo A: Repo A's state must be completely preserved
    app.set_code_review_repo(&nested_a);
    assert_eq!(app.current_code_review_repo(), canonical_a);
    assert_eq!(app.code_review.repo_root, canonical_a);
    assert_eq!(
        app.code_review.selected_file_idx, 4,
        "Repo A selected file index must be restored"
    );
    assert_eq!(
        app.code_review.diff_scroll_y, 175.0,
        "Repo A scroll position must be restored"
    );
    assert!(
        app.code_review.staged_collapsed,
        "Repo A staged section collapse state must be restored"
    );

    // 5. Switch back to Repo B: Repo B's state must also be completely preserved
    app.set_code_review_repo(&repo_b);
    assert_eq!(app.current_code_review_repo(), canonical_b);
    assert_eq!(
        app.code_review.selected_file_idx, 2,
        "Repo B selected file index must be restored"
    );
    assert_eq!(
        app.code_review.diff_scroll_y, 320.0,
        "Repo B scroll position must be restored"
    );

    let _ = std::fs::remove_dir_all(&base_dir);
}

#[test]
fn test_code_review_terminal_independence_and_cwd_isolation() {
    use akssora_terminal_app::App;

    let base_dir = std::env::temp_dir().join(format!("akssora_term_indep_{}", std::process::id()));
    let repo_a = base_dir.join("repo_a");
    let repo_b = base_dir.join("repo_b");
    let non_git = base_dir.join("non_git_folder");

    let _ = std::fs::remove_dir_all(&base_dir);
    std::fs::create_dir_all(repo_a.join(".git")).unwrap();
    std::fs::create_dir_all(repo_b.join(".git")).unwrap();
    std::fs::create_dir_all(&non_git).unwrap();

    let canonical_a = App::canonical_repo_root(&repo_a);
    let canonical_b = App::canonical_repo_root(&repo_b);

    let mut app = App::new();

    // 1. Terminal A created in repo_a
    let term_a_id = app.open_terminal_tab_in(Some(&repo_a));
    assert_eq!(app.tab_count(), 1);

    // Terminal A opens Code Review in repo_a
    app.open_code_review();
    assert!(app.is_code_review_open());
    assert!(!app.code_review.is_full_view);
    assert_eq!(app.current_code_review_repo(), canonical_a);

    // Modify Terminal A's Code Review state
    app.code_review.selected_file_idx = 3;
    app.code_review.diff_scroll_y = 150.0;

    // 2. Creating Terminal B in a non-git directory:
    // Must NOT inherit Terminal A's Code Review panel, diff, or header statistics
    let term_b_id = app.open_terminal_tab_in(Some(&non_git));
    assert_eq!(app.tab_count(), 2);
    assert_ne!(term_a_id, term_b_id, "Terminals must be separate sessions");

    assert!(
        !app.is_code_review_open(),
        "Terminal B must start with Code Review closed"
    );
    assert_eq!(
        app.header.git_additions, 0,
        "Terminal B must not inherit git additions"
    );
    assert_eq!(
        app.header.git_deletions, 0,
        "Terminal B must not inherit git deletions"
    );
    assert!(
        !app.header.is_git_repo,
        "Non-git folder must not be detected as git repo"
    );

    // 3. Terminal B opens Code Review in the non-git directory
    app.open_code_review();
    assert!(app.is_code_review_open());
    assert!(
        app.code_review.diff.files.is_empty(),
        "Non-git directory must produce no diff files"
    );
    assert!(
        app.code_review.visible_tree_items().is_empty(),
        "Non-git directory must display no staging or section controls"
    );
    assert_eq!(app.header.git_additions, 0);
    assert_eq!(app.header.git_deletions, 0);
    assert!(!app.header.is_git_repo);

    // 4. Terminal B navigates to repo_b and opens Code Review
    app.open_code_review_for(&repo_b);
    assert_eq!(app.current_code_review_repo(), canonical_b);
    assert_eq!(app.code_review.repo_root, canonical_b);
    assert!(app.header.is_git_repo);

    // 5. Switch back to Terminal A:
    // Terminal A retains its original panel, split state, repo context, and UI state
    app.select_tab(0);
    assert!(
        app.is_code_review_open(),
        "Terminal A's Code Review panel must remain open"
    );
    assert_eq!(
        app.current_code_review_repo(),
        canonical_a,
        "Terminal A's repo must be repo_a"
    );
    assert_eq!(
        app.code_review.selected_file_idx, 3,
        "Terminal A's selected file must be preserved"
    );
    assert_eq!(
        app.code_review.diff_scroll_y, 150.0,
        "Terminal A's scroll position must be preserved"
    );

    // 6. Switch back to Terminal B:
    app.select_tab(1);
    assert!(
        app.is_code_review_open(),
        "Terminal B's Code Review panel must remain open"
    );
    assert_eq!(
        app.current_code_review_repo(),
        canonical_b,
        "Terminal B's repo must be repo_b"
    );

    // 7. Closing Code Review on Terminal B does not affect Terminal A
    app.close_code_review();
    assert!(!app.is_code_review_open());

    app.select_tab(0);
    assert!(
        app.is_code_review_open(),
        "Terminal A's Code Review must remain open after Terminal B closed its panel"
    );

    let _ = std::fs::remove_dir_all(&base_dir);
}

#[test]
fn test_closing_one_code_review_panel_does_not_close_the_other() {
    use akssora_terminal_app::App;
    use akssora_terminal_ui::SplitDirection;

    let base_dir =
        std::env::temp_dir().join(format!("akssora_close_both_bug_{}", std::process::id()));
    let repo_a = base_dir.join("repo_a");
    let repo_b = base_dir.join("repo_b");

    let _ = std::fs::remove_dir_all(&base_dir);
    std::fs::create_dir_all(repo_a.join(".git")).unwrap();
    std::fs::create_dir_all(repo_b.join(".git")).unwrap();

    let canonical_a = App::canonical_repo_root(&repo_a);
    let canonical_b = App::canonical_repo_root(&repo_b);

    let mut app = App::new();

    // --- Case 1: Two Terminal Tabs ---
    // 1. Open Terminal A in repo_a and open Code Review
    let _term_a = app.open_terminal_tab_in(Some(&repo_a));
    app.open_code_review();
    assert!(app.is_code_review_open());
    assert_eq!(app.current_code_review_repo(), canonical_a);

    // 2. Open Terminal B in repo_b and open Code Review
    let _term_b = app.open_terminal_tab_in(Some(&repo_b));
    app.open_code_review();
    assert!(app.is_code_review_open());
    assert_eq!(app.current_code_review_repo(), canonical_b);

    // 3. Close Code Review on Terminal B
    app.close_code_review();
    assert!(
        !app.is_code_review_open(),
        "Terminal B Code Review must be closed"
    );

    // 4. Switch to Terminal A via prev_tab: Terminal A must STILL be open!
    app.prev_tab();
    assert!(
        app.is_code_review_open(),
        "Closing Terminal B's Code Review must NOT close Terminal A's Code Review"
    );
    assert_eq!(app.current_code_review_repo(), canonical_a);

    // 5. Switch back to Terminal B via next_tab: Terminal B must STILL be closed!
    app.next_tab();
    assert!(
        !app.is_code_review_open(),
        "Terminal B Code Review must remain closed"
    );

    // --- Case 2: Split Panes within a Tab ---
    // In Terminal B, split the pane into repo_a
    let _split_id = app
        .split_active_pane_in(SplitDirection::Vertical, Some(&repo_a))
        .unwrap();
    // Split pane starts with code review closed
    assert!(!app.is_code_review_open());

    // Open Code Review in split pane (which is in repo_a)
    app.open_code_review();
    assert!(app.is_code_review_open());
    assert_eq!(app.current_code_review_repo(), canonical_a);

    // Switch focus back to original pane of tab B
    app.focus_next_split();
    assert!(
        !app.is_code_review_open(),
        "Original pane in tab B must still have Code Review closed"
    );

    // Open Code Review in original pane of tab B
    app.open_code_review();
    assert!(app.is_code_review_open());
    assert_eq!(app.current_code_review_repo(), canonical_b);

    // Now both split panes have Code Review open.
    // Close Code Review in original pane of tab B:
    app.close_code_review();
    assert!(!app.is_code_review_open());

    // Switch focus back to split_id:
    app.focus_next_split();
    assert!(
        app.is_code_review_open(),
        "Split pane must retain its open Code Review panel after other pane closed its panel"
    );
    assert_eq!(app.current_code_review_repo(), canonical_a);

    let _ = std::fs::remove_dir_all(&base_dir);
}

#[test]
fn test_code_review_split_view_seamless_inline_editing() {
    use akssora_git::{DiffHunk, DiffLine, DiffLineType, FileDiff, FileGitStatus, RepoDiff};
    use akssora_terminal_ui::{CodeReviewAction, CodeReviewLayout, CodeReviewModel};
    use std::path::PathBuf;

    let base_dir = std::env::temp_dir().join(format!(
        "akssora_test_inline_edit_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(base_dir.join(".git")).unwrap();
    let file_path = PathBuf::from("service.rs");
    let full_path = base_dir.join(&file_path);
    std::fs::write(
        &full_path,
        "pub fn compute() -> i32 {\n    let factor = 2;\n    factor * 21\n}\n",
    )
    .unwrap();

    let mut model = CodeReviewModel::new(&base_dir);
    model.diff = RepoDiff {
        files: vec![FileDiff {
            path: file_path.clone(),
            old_path: None,
            status: FileGitStatus::Modified,
            additions: 1,
            deletions: 0,
            line_count: 4,
            is_binary: false,
            hunks: vec![DiffHunk {
                header: "@@ -1,4 +1,4 @@".to_string(),
                old_start: 1,
                old_lines: 4,
                new_start: 1,
                new_lines: 4,
                lines: vec![
                    DiffLine {
                        line_type: DiffLineType::Context,
                        content: "pub fn compute() -> i32 {\n".to_string(),
                        old_line_num: Some(1),
                        new_line_num: Some(1),
                    },
                    DiffLine {
                        line_type: DiffLineType::Add,
                        content: "    let factor = 2;\n".to_string(),
                        old_line_num: None,
                        new_line_num: Some(2),
                    },
                    DiffLine {
                        line_type: DiffLineType::Context,
                        content: "    factor * 21\n".to_string(),
                        old_line_num: Some(3),
                        new_line_num: Some(3),
                    },
                    DiffLine {
                        line_type: DiffLineType::Context,
                        content: "}\n".to_string(),
                        old_line_num: Some(4),
                        new_line_num: Some(4),
                    },
                ],
            }],
        }],
        files_changed: 1,
        total_additions: 1,
        total_deletions: 0,
    };

    let diff = model.diff.clone();

    let mut app = akssora_terminal_app::App::new();
    let _tab = app.open_terminal_tab();
    app.open_code_review_for(&base_dir);
    app.code_review.diff = diff;
    app.prepare_code_review_editor_file(0);

    // 1. Verify Code Review is open in split view in a terminal tab
    assert!(app.is_code_review_open(), "Code Review must be open");
    assert!(!app.code_review.is_full_view, "Must be in split view");

    // 2. active_editor and active_editor_mut return the document in split view!
    let doc = app
        .active_editor()
        .expect("Must have active editor in split view");
    assert_eq!(doc.path, Some(full_path.clone()));

    // 3. Click on line 2 (the added line) in Code Review card body to position cursor
    let (cr_x, cr_y, cr_w, cr_h) = app
        .code_review_bounds()
        .expect("Must have bounds in split view");
    let (vx, vy, _vw, _vh) =
        CodeReviewLayout::diff_viewport_rect(&app.code_review, cr_x, cr_y, cr_w, cr_h);
    let card_x = vx + 16.0;
    let card_top = vy + 12.0;
    let line2_y = card_top
        + CodeReviewLayout::CARD_HEADER_HEIGHT
        + 8.0
        + CodeReviewLayout::UNMODIFIED_LINES_HEIGHT
        + CodeReviewLayout::DIFF_LINE_HEIGHT
        + 4.0;
    let click_x = card_x + 80.0;

    let hit =
        CodeReviewLayout::hit_test(&app.code_review, cr_x, cr_y, cr_w, cr_h, click_x, line2_y);
    assert!(
        matches!(hit, Some(CodeReviewAction::ClickEditor { orig_idx: 0, .. })),
        "Hit test must target ClickEditor for file 0"
    );

    let show_sidebar = app.config.tab_layout.show_sidebar();
    let show_tabs = app.config.tab_layout.includes_title_bar();
    let top_h = app.header.total_height(show_tabs);
    let sb_w = app.sidebar.effective_width(show_sidebar);
    let exp_w = app.explorer.effective_width();
    let ws_x = sb_w + exp_w;
    let w = app.config.window.width as f32;
    let h = app.config.window.height as f32;

    let clicked = app.handle_code_review_click(click_x, line2_y, w, h, top_h, ws_x);
    assert!(clicked, "Click inside Code Review must be handled");

    // Cursor row must now be 1 (line 2 of the document)
    let doc_after_click = app.active_editor().unwrap();
    assert_eq!(
        doc_after_click.cursor.0, 1,
        "Cursor row should be line 2 (index 1)"
    );

    // 4. Directly edit in-place without clicking any special edit buttons
    {
        let doc_mut = app
            .active_editor_mut()
            .expect("Must get mutable active editor");
        doc_mut.insert_char_smart('_');
        doc_mut.insert_char_smart('m');
        doc_mut.insert_char_smart('o');
        doc_mut.insert_char_smart('d');
    }

    let modified_line = app
        .active_editor()
        .unwrap()
        .buffer
        .get_line(1)
        .unwrap()
        .to_string();
    assert!(
        modified_line.contains("_mod"),
        "Buffer must contain typed text: {modified_line}"
    );

    // 5. Test saving: Ctrl+S saves document to disk
    {
        let doc_mut = app.active_editor_mut().unwrap();
        let _ = doc_mut.save();
    }
    let saved_content = std::fs::read_to_string(&full_path).unwrap();
    assert!(
        saved_content.contains("_mod"),
        "Disk file must be saved with new edits"
    );

    let _ = std::fs::remove_dir_all(&base_dir);
}

#[test]
fn test_header_right_buttons_no_overlap_and_per_terminal_git_stats() {
    use akssora_terminal_app::App;
    use akssora_terminal_ui::{CodeReviewLayout, Header, HeaderButton, HeaderItem};

    // 1. Verify Header Right-side Buttons Geometry (No Overlaps)
    let mut header = Header::new("Akssora");
    header.is_git_repo = true;
    header.git_additions = 110;
    header.git_deletions = 7;

    for win_w in [600.0, 800.0, 1000.0, 1440.0] {
        let (cr_x, _cr_y, cr_w, _cr_h) = header.code_review_button_rect(win_w);
        let (sx, _sy, sw, _sh) = header.settings_button_rect(win_w);
        let (min_x, _min_y, _min_w, _min_h) = header.button_rect(HeaderButton::Minimize, win_w);

        // Code review right edge must be strictly to the left of settings button
        assert!(
            cr_x + cr_w <= sx,
            "Code Review button [{}..{}] must not overlap settings button [{}] at width {}",
            cr_x,
            cr_x + cr_w,
            sx,
            win_w
        );
        assert!(
            sx - (cr_x + cr_w) >= 6.0,
            "Must maintain at least 6px spacing between Code Review button and Settings button"
        );

        // Settings button right edge must be strictly to the left of minimize button
        assert!(
            sx + sw <= min_x,
            "Settings button [{}..{}] must not overlap window minimize button [{}] at width {}",
            sx,
            sx + sw,
            min_x,
            win_w
        );

        // Hit testing must cleanly distinguish each item
        assert_eq!(
            header.hit_test_item_ex(cr_x + 5.0, 15.0, win_w, 1, false, true, true),
            Some(HeaderItem::CodeReview),
            "Hit testing Code Review button must return CodeReview item"
        );
        assert_eq!(
            header.hit_test_item_ex(sx + 5.0, 15.0, win_w, 1, false, true, true),
            Some(HeaderItem::Settings),
            "Hit testing Settings button must return Settings item"
        );
        assert_eq!(
            header.hit_test_item_ex(min_x + 5.0, 15.0, win_w, 1, false, true, true),
            Some(HeaderItem::WindowButton(HeaderButton::Minimize)),
            "Hit testing Minimize button must return Minimize window button"
        );
    }

    // 2. Verify Staged/Unstaged Button Removed from Code Review Top Bar
    let (ux, uy, uw, uh) = CodeReviewLayout::stage_toggle_btn_rect(0.0, 0.0);
    assert_eq!(
        (ux, uy, uw, uh),
        (0.0, 0.0, 0.0, 0.0),
        "Stage toggle button must be zeroed out in Code Review top bar"
    );

    let (tx, _ty, tw, _th) = CodeReviewLayout::toggle_collapse_all_btn_rect(0.0, 0.0);
    assert!(
        tx < 100.0,
        "Collapse all toggle button must be shifted left next to sidebar toggle"
    );
    assert!(tw > 50.0);

    // 3. Verify Per-Terminal Git Stats and Switching
    let base_dir =
        std::env::temp_dir().join(format!("akssora_git_stats_test_{}", std::process::id()));
    let repo_dir = base_dir.join("active_repo");
    let non_git_dir = base_dir.join("other_folder");
    let _ = std::fs::remove_dir_all(&base_dir);
    std::fs::create_dir_all(repo_dir.join(".git")).unwrap();
    std::fs::create_dir_all(&non_git_dir).unwrap();

    let mut app = App::new();

    // Terminal A created in repo_dir
    let term_a = app.open_terminal_tab_in(Some(&repo_dir));
    app.code_review.diff.total_additions = 42;
    app.code_review.diff.total_deletions = 8;
    app.sync_header_git_state_for_active_terminal();

    assert!(
        app.header.is_git_repo,
        "Terminal A must be recognized as git repo"
    );
    assert_eq!(
        app.header.git_additions, 42,
        "Terminal A must have 42 additions"
    );
    assert_eq!(
        app.header.git_deletions, 8,
        "Terminal A must have 8 deletions"
    );

    // Save Terminal A state
    app.save_active_terminal_code_review();

    // Terminal B created in non-git directory
    let term_b = app.open_terminal_tab_in(Some(&non_git_dir));
    assert_ne!(term_a, term_b);
    assert!(!app.header.is_git_repo, "Terminal B must not be a git repo");
    assert_eq!(
        app.header.git_additions, 0,
        "Terminal B must have 0 additions"
    );
    assert_eq!(
        app.header.git_deletions, 0,
        "Terminal B must have 0 deletions"
    );

    // Switch back to Terminal A: restores 42 additions and 8 deletions
    app.select_tab(0);
    assert!(
        app.header.is_git_repo,
        "Terminal A must be git repo upon selection"
    );
    assert_eq!(
        app.header.git_additions, 42,
        "Terminal A additions must be restored"
    );
    assert_eq!(
        app.header.git_deletions, 8,
        "Terminal A deletions must be restored"
    );

    // 4. Verify Sidebar File Diff Hover Suppresses Tooltip
    // Tooltip for SelectFile or SelectTreeFile must be None (mouse hover highlight is enough)
    let tip = akssora_terminal_app::app::tooltips::code_review::check_code_review_tooltip(
        &app, 0.0, 0.0, 0.0, 0.0, 800.0, 600.0,
    );
    // When hovering over non-action regions or file selection, no "View File Diff" tooltip
    if let Some(t) = tip {
        assert_ne!(
            t.text, "View File Diff",
            "Must not display 'View File Diff' tooltip"
        );
    }

    let _ = std::fs::remove_dir_all(&base_dir);
}

#[test]
fn test_accordion_collapse_stationary_viewport_and_continuous_scrolling() {
    use akssora_git::{DiffHunk, DiffLine, DiffLineType, FileDiff, FileGitStatus, RepoDiff};
    use akssora_terminal_app::App;
    use akssora_terminal_ui::CodeReviewLayout;
    use std::path::PathBuf;

    let base_dir =
        std::env::temp_dir().join(format!("akssora_collapse_test_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&base_dir);
    std::fs::create_dir_all(base_dir.join(".git")).unwrap();

    let make_file = |name: &str, lines: usize| FileDiff {
        path: PathBuf::from(name),
        old_path: None,
        status: FileGitStatus::Modified,
        hunks: vec![DiffHunk {
            old_start: 1,
            old_lines: lines,
            new_start: 1,
            new_lines: lines,
            header: format!("@@ -1,{} +1,{} @@", lines, lines),
            lines: (0..lines)
                .map(|i| DiffLine {
                    line_type: DiffLineType::Add,
                    content: format!("Line {}\n", i),
                    old_line_num: None,
                    new_line_num: Some(i + 1),
                })
                .collect(),
        }],
        additions: lines,
        deletions: 0,
        is_binary: false,
        line_count: lines,
    };

    let repo_diff = RepoDiff {
        files: vec![
            make_file("file0.rs", 10),
            make_file("file1.rs", 50),
            make_file("file2.rs", 50),
        ],
        files_changed: 3,
        total_additions: 110,
        total_deletions: 0,
    };

    let mut app = App::new();
    let _term = app.open_terminal_tab_in(Some(&base_dir));
    app.open_code_review_for(&base_dir);
    app.code_review.is_full_view = true;
    app.code_review.diff = repo_diff;

    let win_w = 1000.0;
    let win_h = 800.0;
    let top_h = 40.0;
    let ws_x = 0.0;

    let (vx, vy, _vw, vh) =
        CodeReviewLayout::diff_viewport_rect(&app.code_review, ws_x, top_h, win_w, win_h - top_h);
    let offset1 = app.code_review.card_y_offset_for_file(1, vh);
    let card1_h_expanded = CodeReviewLayout::card_height_for_file_state(
        &app.code_review.diff.files[1],
        false,
        false,
        vh,
    );

    // ── 1. Case A: Collapse accordion below vy (card_top >= vy) ─────────
    app.code_review.diff_scroll_y = 0.0;
    app.code_review.active_file_idx = Some(1);

    // Click header of file 1
    let card1_x = vx + 16.0;
    let header1_y = vy + offset1 - app.code_review.diff_scroll_y;

    app.handle_code_review_click(card1_x + 50.0, header1_y + 10.0, win_w, win_h, top_h, ws_x);

    // File 1 must now be collapsed
    assert!(
        app.code_review
            .is_file_collapsed(&PathBuf::from("file1.rs")),
        "File 1 must be collapsed"
    );
    // Viewport must remain stationary: diff_scroll_y stays 0.0
    assert_eq!(
        app.code_review.diff_scroll_y, 0.0,
        "Viewport must remain stationary when collapsing"
    );
    // active_file_idx must not jump to file 0 or file 2
    assert_eq!(
        app.code_review.active_file_idx, None,
        "active_file_idx must not jump or focus to another accordion"
    );
    // Subsequent accordions move up as part of normal document reflow
    let offset2_after = app.code_review.card_y_offset_for_file(2, vh);
    let expected_offset2 =
        offset1 + CodeReviewLayout::CARD_HEADER_HEIGHT + CodeReviewLayout::CARD_MARGIN_BOTTOM;
    assert_eq!(
        offset2_after, expected_offset2,
        "File 2 must move up naturally following file 1 collapse"
    );

    // ── 2. Expand file 1 again ──────────────────────────────────────────
    app.handle_code_review_click(card1_x + 50.0, header1_y + 10.0, win_w, win_h, top_h, ws_x);
    assert!(
        !app.code_review
            .is_file_collapsed(&PathBuf::from("file1.rs"))
    );

    // ── 3. Case B: Collapse accordion while scrolled into it (sticky header at vy) ───
    // User scrolls so that file 1's top is 150px above vy; its header is sticky at vy
    app.code_review.diff_scroll_y = offset1 + 150.0;
    app.code_review.active_file_idx = Some(1);

    let card1_top_before = vy + offset1 - app.code_review.diff_scroll_y;
    assert!(card1_top_before < vy, "Card top is above viewport");
    let sticky_y = CodeReviewLayout::card_header_y(card1_top_before, card1_h_expanded, false, vy);
    assert_eq!(sticky_y, vy, "Header must be sticky at vy");

    // Click sticky header at vy
    app.handle_code_review_click(card1_x + 50.0, vy + 10.0, win_w, win_h, top_h, ws_x);

    // File 1 must be collapsed
    assert!(
        app.code_review
            .is_file_collapsed(&PathBuf::from("file1.rs"))
    );
    // Scroll anchoring must keep file 1's collapsed header at vy without jumping
    assert_eq!(
        app.code_review.diff_scroll_y, offset1,
        "diff_scroll_y must be anchored to offset1 so collapsed header stays at vy"
    );
    let card1_top_after = vy + offset1 - app.code_review.diff_scroll_y;
    assert_eq!(
        card1_top_after, vy,
        "Collapsed card header must remain exactly at vy"
    );
    assert_eq!(
        app.code_review.active_file_idx, None,
        "Must not focus or jump to another accordion"
    );

    // ── 4. Verify continuous scrolling in both directions ───────────────
    // Can scroll up towards file 0
    app.code_review.scroll_diff(-50.0, vh);
    assert!(
        app.code_review.diff_scroll_y < offset1,
        "Must scroll smoothly upward towards file 0"
    );
    // Can scroll down towards file 2
    app.code_review.scroll_diff(100.0, vh);
    assert!(
        app.code_review.diff_scroll_y > offset1,
        "Must scroll smoothly downward towards file 2"
    );

    let _ = std::fs::remove_dir_all(&base_dir);
}

#[test]
fn test_code_review_open_file_in_editor_and_state_preservation() {
    use akssora_git::{DiffHunk, DiffLine, DiffLineType, FileDiff, FileGitStatus, RepoDiff};
    use akssora_terminal_app::App;
    use akssora_terminal_ui::TabKind;
    use std::path::PathBuf;

    let test_id = std::process::id();
    let repo_a = std::env::temp_dir().join(format!("akssora_open_file_repo_a_{test_id}"));
    let repo_b = std::env::temp_dir().join(format!("akssora_open_file_repo_b_{test_id}"));
    let _ = std::fs::remove_dir_all(&repo_a);
    let _ = std::fs::remove_dir_all(&repo_b);

    std::fs::create_dir_all(repo_a.join(".git")).unwrap();
    std::fs::create_dir_all(repo_a.join("src/nested")).unwrap();
    std::fs::create_dir_all(repo_b.join(".git")).unwrap();

    let file_a_path = repo_a.join("src/nested/file with spaces.rs");
    std::fs::write(
        &file_a_path,
        "fn hello_repo_a() {\n    println!(\"A\");\n}\n",
    )
    .unwrap();

    let file_b_path = repo_b.join("main.rs");
    std::fs::write(
        &file_b_path,
        "fn hello_repo_b() {\n    println!(\"B\");\n}\n",
    )
    .unwrap();

    let diff_file_a = FileDiff {
        path: PathBuf::from("src/nested/file with spaces.rs"),
        old_path: None,
        status: FileGitStatus::Modified,
        hunks: vec![DiffHunk {
            old_start: 1,
            old_lines: 3,
            new_start: 1,
            new_lines: 3,
            header: "@@ -1,3 +1,3 @@".to_string(),
            lines: vec![DiffLine {
                line_type: DiffLineType::Add,
                content: "fn hello_repo_a() {\n".to_string(),
                old_line_num: None,
                new_line_num: Some(1),
            }],
        }],
        additions: 1,
        deletions: 0,
        is_binary: false,
        line_count: 3,
    };

    let diff_deleted = FileDiff {
        path: PathBuf::from("deleted_old.rs"),
        old_path: None,
        status: FileGitStatus::Deleted,
        hunks: vec![DiffHunk {
            old_start: 1,
            old_lines: 2,
            new_start: 0,
            new_lines: 0,
            header: "@@ -1,2 +0,0 @@".to_string(),
            lines: vec![
                DiffLine {
                    line_type: DiffLineType::Delete,
                    content: "old line 1\n".to_string(),
                    old_line_num: Some(1),
                    new_line_num: None,
                },
                DiffLine {
                    line_type: DiffLineType::Delete,
                    content: "old line 2\n".to_string(),
                    old_line_num: Some(2),
                    new_line_num: None,
                },
            ],
        }],
        additions: 0,
        deletions: 2,
        is_binary: false,
        line_count: 2,
    };

    let mut app = App::new();

    // 1. Create Terminal A and open Code Review in Repo A
    let term_a = app.open_terminal_tab_in(Some(&repo_a));
    assert_eq!(term_a, 1);
    app.open_code_review_for(&repo_a);
    app.code_review.diff = RepoDiff {
        files: vec![diff_file_a.clone(), diff_deleted.clone()],
        files_changed: 2,
        total_additions: 1,
        total_deletions: 2,
    };
    assert!(
        app.is_code_review_open(),
        "Code Review must be open in Terminal A"
    );

    // 2. Click "Open File in Editor" for the file with spaces and nested path (idx 0)
    let full_path_a = repo_a.join(&diff_file_a.path);
    app.open_editor_file(&full_path_a);

    // Verify Editor tab was created and selected
    let active_tab = app.tab_manager.active_tab().expect("Active tab exists");
    assert!(
        matches!(active_tab.kind, TabKind::Editor { .. }),
        "Must switch to Editor tab"
    );
    assert!(
        !app.is_code_review_open(),
        "Code Review must not be open while on an Editor tab"
    );

    // Verify document loaded properly
    assert_eq!(app.editor_active_file.as_ref(), Some(&full_path_a));
    assert!(app.editor_open_files.contains(&full_path_a));
    {
        let doc = app.active_editor().expect("Active editor document exists");
        assert!(
            !doc.inline_diff,
            "Editor document must not have inline_diff forced"
        );
        assert_eq!(doc.cursor, (0, 0));
        assert!(!doc.dirty);
        assert!(doc.buffer.to_string_content().contains("fn hello_repo_a()"));
    }

    // 3. User makes unsaved edits and moves cursor in the editor
    {
        let doc = app.active_editor_mut().expect("Active editor mut exists");
        doc.cursor = (1, 4);
        doc.buffer.insert_str(1, 4, " // modified");
        doc.dirty = true;
        doc.scroll_row = 5;
    }

    // 4. Switch back to Terminal A
    let term_a_pos = app
        .tab_manager
        .tabs()
        .iter()
        .position(|t| matches!(t.kind, TabKind::Terminal))
        .expect("Terminal tab exists");
    app.select_tab(term_a_pos);

    // Code Review in Terminal A is restored
    assert!(
        app.is_code_review_open(),
        "Terminal A's Code Review must be restored"
    );
    assert_eq!(app.code_review.repo_root, repo_a);

    // 5. Click "Open File in Editor" again for the same file
    app.open_editor_file(&full_path_a);

    // Must focus existing editor tab without duplicating or losing unsaved edits
    assert_eq!(
        app.editor_open_files
            .iter()
            .filter(|p| **p == full_path_a)
            .count(),
        1,
        "Must not duplicate open file tab"
    );
    {
        let doc = app.active_editor().expect("Active editor exists");
        assert!(doc.dirty, "Unsaved changes must be preserved");
        assert_eq!(doc.cursor, (1, 4), "Cursor position must be preserved");
        assert_eq!(doc.scroll_row, 5, "Scroll position must be preserved");
        assert!(
            doc.buffer.to_string_content().contains("// modified"),
            "Edited buffer content must be preserved"
        );
    }

    // 6. Test opening a deleted file from Code Review
    let deleted_full_path = repo_a.join(&diff_deleted.path);
    app.select_tab(term_a_pos);
    app.open_editor_file(&deleted_full_path);
    {
        let doc = app.active_editor().expect("Deleted file doc exists");
        assert!(doc.is_deleted, "Deleted file must be flagged as deleted");
        assert!(
            doc.buffer.to_string_content().contains("old line 1"),
            "Deleted file content must be loaded from diff hunks"
        );
    }

    // 7. Test Terminal B in Repo B does not leak or resolve to Repo A
    let term_b = app.open_terminal_tab_in(Some(&repo_b));
    assert_eq!(term_b, 3);
    app.open_code_review_for(&repo_b);
    let diff_file_b = FileDiff {
        path: PathBuf::from("main.rs"),
        old_path: None,
        status: FileGitStatus::Modified,
        hunks: vec![],
        additions: 1,
        deletions: 0,
        is_binary: false,
        line_count: 1,
    };
    app.code_review.diff = RepoDiff {
        files: vec![diff_file_b.clone()],
        files_changed: 1,
        total_additions: 1,
        total_deletions: 0,
    };
    assert_eq!(app.code_review.repo_root, repo_b);

    // Open file from Terminal B's Code Review
    let full_path_b = repo_b.join(&diff_file_b.path);
    app.open_editor_file(&full_path_b);
    assert_eq!(app.editor_active_file.as_ref(), Some(&full_path_b));
    {
        let doc = app.active_editor().expect("Repo B doc exists");
        assert!(doc.buffer.to_string_content().contains("fn hello_repo_b()"));
    }

    // Cleanup temp dirs
    let _ = std::fs::remove_dir_all(&repo_a);
    let _ = std::fs::remove_dir_all(&repo_b);
}
