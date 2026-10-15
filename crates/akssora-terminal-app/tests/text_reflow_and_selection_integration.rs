use akssora_terminal_app::App;
use akssora_terminal_parser::TerminalParser;
use akssora_terminal_screen::{SelectionRange, TerminalGrid, TerminalPoint};
use akssora_terminal_ui::{ContextMenuAction, SplitDirection};
use std::sync::Mutex;

static CLIPBOARD_TEST_MUTEX: Mutex<()> = Mutex::new(());

#[test]
fn test_ls_output_reflow_and_unwrapping() {
    let mut grid = TerminalGrid::new(24, 80);
    let mut parser = TerminalParser::new();

    // Simulate shell ls output with column-aligned filenames
    let ls_output = b"alpha_report.pdf    beta_specs.docx    gamma_archive.tar.gz\r\n\
                      delta_notes.txt     epsilon_data.csv   zeta_keys.pub\r\n\
                      user@akssora:~$ ";
    parser.advance(&mut grid, ls_output);

    // Initial 80-col check
    let initial_text = grid.all_text();
    assert!(initial_text.contains("alpha_report.pdf"));
    assert!(initial_text.contains("gamma_archive.tar.gz"));
    assert!(initial_text.contains("user@akssora:~$"));

    // 1. Narrow terminal (e.g. sidebar opened or split created: 80 -> 40 cols)
    grid.resize(24, 40);
    let narrowed_text = grid.all_text();
    assert!(narrowed_text.contains("alpha_report.pdf"));
    assert!(narrowed_text.contains("beta_specs.docx"));
    assert!(narrowed_text.contains("gamma_archive.tar.gz"));
    assert!(narrowed_text.contains("user@akssora:~$"));

    // 2. Expand terminal back (e.g. sidebar closed: 40 -> 80 cols)
    grid.resize(24, 80);
    let widened_text = grid.all_text();
    assert!(widened_text.contains("alpha_report.pdf"));
    assert!(widened_text.contains("gamma_archive.tar.gz"));
    assert!(widened_text.contains("user@akssora:~$"));

    // Check that prompt and cursor remained bottom-anchored and intact
    let (crow, _) = grid.cursor();
    assert!(crow < grid.rows());
    let vis = grid.visible_lines();
    let prompt_row: String = vis[crow].iter().map(|c| c.ch).collect();
    assert!(prompt_row.contains("user@akssora:~$"));
}

#[test]
fn test_mouse_text_selection_single_and_multiline() {
    let mut grid = TerminalGrid::new(24, 80);
    let mut parser = TerminalParser::new();

    parser.advance(
        &mut grid,
        b"First line content\r\nSecond line data\r\nThird line output",
    );

    let lines = grid.visible_lines();
    let owned_lines: Vec<Vec<akssora_terminal_screen::Cell>> = lines.into_iter().cloned().collect();
    let wrapped = grid.visible_line_wrapped();

    // 1. Single-line selection: "First line"
    let single_sel = SelectionRange::new(TerminalPoint::new(0, 0), TerminalPoint::new(0, 9));
    assert!(single_sel.contains(0, 0));
    assert!(single_sel.contains(0, 5));
    assert!(single_sel.contains(0, 9));
    assert!(!single_sel.contains(0, 10));
    assert!(!single_sel.contains(1, 0));

    let single_text = single_sel.extract_text(&owned_lines, &wrapped);
    assert_eq!(single_text, "First line");

    // 2. Multiline selection from line 0 to line 2
    let multi_sel = SelectionRange::new(TerminalPoint::new(0, 6), TerminalPoint::new(2, 4));
    assert!(multi_sel.contains(0, 6));
    assert!(multi_sel.contains(0, 15));
    assert!(multi_sel.contains(1, 0));
    assert!(multi_sel.contains(1, 10));
    assert!(multi_sel.contains(2, 4));
    assert!(!multi_sel.contains(2, 5));

    let multi_text = multi_sel.extract_text(&owned_lines, &wrapped);
    assert!(multi_text.contains("line content"));
    assert!(multi_text.contains("Second line data"));
    assert!(multi_text.contains("Third"));

    // 3. Normalized selection (reverse drag from bottom-right to top-left)
    let rev_sel = SelectionRange::new(TerminalPoint::new(2, 4), TerminalPoint::new(0, 6));
    let (s, e) = rev_sel.normalized();
    assert_eq!(s, TerminalPoint::new(0, 6));
    assert_eq!(e, TerminalPoint::new(2, 4));
    assert_eq!(rev_sel.extract_text(&owned_lines, &wrapped), multi_text);
}

#[test]
fn test_copy_and_paste_clipboard_integration() {
    let _lock = CLIPBOARD_TEST_MUTEX
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    let mut app = App::new();
    let tab_id = app.open_terminal_tab();

    if let Some(block) = app.get_block_mut(tab_id) {
        let grid = block.shell.grid_mut();
        for ch in "selected command text".chars() {
            grid.put_char(ch);
        }
        // Set selection over "command"
        block.shell.set_selection(Some(SelectionRange::new(
            TerminalPoint::new(0, 9),
            TerminalPoint::new(0, 15),
        )));
    }

    // Copy selection
    let copied = app.copy_active_selection();
    assert!(copied);

    // Retrieve from clipboard
    let clip = akssora_terminal_input::get_clipboard_text();
    assert_eq!(clip.as_deref(), Some("command"));

    // Test safe-paste
    if let Some(block) = app.get_block_mut(tab_id) {
        block.shell.paste_text("echo safe_paste\n");
    }

    // Set and get multi-line text safely
    akssora_terminal_input::set_clipboard_text("line 1\nline 2\nline 3");
    let multi_clip = akssora_terminal_input::get_clipboard_text();
    assert_eq!(multi_clip.as_deref(), Some("line 1\nline 2\nline 3"));
}

#[test]
fn test_context_menu_interaction() {
    let _lock = CLIPBOARD_TEST_MUTEX
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    let mut app = App::new();
    let tab_id = app.open_terminal_tab();

    if let Some(block) = app.get_block_mut(tab_id) {
        block.shell.set_selection(Some(SelectionRange::new(
            TerminalPoint::new(0, 0),
            TerminalPoint::new(0, 5),
        )));
    }

    // Open context menu at (150.0, 200.0)
    app.open_context_menu(150.0, 200.0);
    assert!(app.context_menu.is_some());
    let menu = app.context_menu.as_ref().unwrap();
    assert!(menu.contains(155.0, 210.0));

    // Test context menu action: Copy
    app.handle_context_menu_action(ContextMenuAction::Copy);
    assert!(app.context_menu.is_none());

    // Test context menu action: Select All
    app.open_context_menu(200.0, 200.0);
    app.handle_context_menu_action(ContextMenuAction::SelectAll);
    let sel = app.get_block(tab_id).and_then(|b| b.shell.selection());
    assert!(sel.is_some());
    let s = sel.unwrap();
    assert_eq!(s.start, TerminalPoint::new(0, 0));
}

#[test]
fn test_split_view_selection_and_reflow_isolation() {
    let _lock = CLIPBOARD_TEST_MUTEX
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    let mut app = App::new();
    let tab1_id = app.open_terminal_tab();

    let split_id = app
        .split_active_pane(SplitDirection::Vertical)
        .expect("must create vertical split");

    // Write different content into pane 1 and pane 2
    if let Some(b1) = app.get_block_mut(tab1_id) {
        for ch in "pane 1 content".chars() {
            b1.shell.grid_mut().put_char(ch);
        }
        b1.shell.set_selection(Some(SelectionRange::new(
            TerminalPoint::new(0, 0),
            TerminalPoint::new(0, 5),
        )));
    }

    if let Some(b2) = app.get_block_mut(split_id) {
        for ch in "pane 2 independent text".chars() {
            b2.shell.grid_mut().put_char(ch);
        }
    }

    // Pane 1 has active selection; Pane 2 has no selection
    assert_eq!(
        app.get_block(tab1_id)
            .unwrap()
            .shell
            .selected_text()
            .as_deref(),
        Some("pane 1")
    );
    assert!(
        app.get_block(split_id)
            .unwrap()
            .shell
            .selected_text()
            .is_none()
    );

    // Focus pane 2 and verify isolation
    app.split_manager.focus_pane(split_id);
    assert_eq!(app.active_pane_id(), Some(split_id));
    assert!(!app.copy_active_selection()); // pane 2 has no selection to copy

    // Focus pane 1 and verify copy works
    app.split_manager.focus_pane(tab1_id);
    assert!(app.copy_active_selection());
    assert_eq!(
        app.get_block(tab1_id)
            .unwrap()
            .shell
            .selected_text()
            .as_deref(),
        Some("pane 1")
    );
}

#[test]
fn test_cursor_navigation_and_click_editing() {
    let mut app = App::new();
    let tab_id = app.open_terminal_tab();

    if let Some(block) = app.get_block_mut(tab_id) {
        let grid = block.shell.grid_mut();
        // Simulate prompt: "user@akssora:~$ ls -la"
        for ch in "user@akssora:~$ ls -la".chars() {
            grid.put_char(ch);
        }
    }

    let b = app.get_block(tab_id).unwrap();
    let (crow, ccol) = b.shell.cursor();
    assert_eq!(crow, 0);
    assert_eq!(ccol, 22);

    // Clicking at column 16 (on active cursor row) should send 6 Left Arrows to the shell
    let cell_w = 9.0;
    let cell_h = 18.0;
    let margin = 2.0;

    let target_col = 16.0;
    let click_x = b.x + margin + target_col * cell_w + 2.0;
    // Calculate click y accounting for bottom_up offset
    let lines_count = b.shell.grid().visible_lines().len();
    let y_offset = (lines_count - 1) as f32 * cell_h;
    let click_y = b.y + margin + y_offset + 5.0;

    app.handle_terminal_click_in_pane(tab_id, click_x, click_y);
    // Selection is cleared after click
    assert!(app.get_block(tab_id).unwrap().shell.selection().is_none());
}

#[test]
fn test_continuous_output_streaming_and_scrolling_isolation() {
    let mut app = App::new();
    let tab_id = app.open_terminal_tab();

    // 1. Output streaming when viewport is at the bottom (scroll_offset == 0)
    let block = app.get_block_mut(tab_id).unwrap();
    let grid = block.shell.grid_mut();
    for i in 0..30 {
        let line = format!("Streaming log entry line {:03}\r\n", i);
        for ch in line.chars() {
            if ch == '\n' {
                grid.newline();
            } else if ch != '\r' {
                grid.put_char(ch);
            }
        }
    }

    // Auto-follow maintains offset 0
    assert_eq!(grid.scroll_offset(), 0);
    assert!(grid.scrollback_len() > 0);

    // 2. User scrolls upward to inspect history
    grid.scroll_up_view(10);
    assert_eq!(grid.scroll_offset(), 10);

    // 3. Continuous high-volume output arrives while inspecting history
    for i in 30..60 {
        let line = format!("Incoming background output line {:03}\r\n", i);
        for ch in line.chars() {
            if ch == '\n' {
                grid.newline();
            } else if ch != '\r' {
                grid.put_char(ch);
            }
        }
    }

    // Scroll position must be preserved (not reset to 0 by incoming characters)
    // and incremented with newlines pushed to scrollback
    assert!(grid.scroll_offset() >= 10);

    // 4. Returning to bottom resumes auto-scrolling
    grid.reset_scroll();
    assert_eq!(grid.scroll_offset(), 0);

    let next_line = "Final output entry\r\n";
    for ch in next_line.chars() {
        if ch == '\n' {
            grid.newline();
        } else if ch != '\r' {
            grid.put_char(ch);
        }
    }
    assert_eq!(grid.scroll_offset(), 0);
}

#[test]
fn test_split_pane_text_selection_vs_drag_repositioning() {
    let mut app = App::new();
    let tab_id = app.open_terminal_tab();
    let right_pane = app
        .split_active_pane(SplitDirection::Vertical)
        .expect("must create vertical split");

    // Verify two split panes exist
    assert_eq!(app.active_tab_blocks().len(), 2);

    let left_block = app.get_block_mut(tab_id).unwrap();
    let left_grid = left_block.shell.grid_mut();
    for ch in "Left split pane command output".chars() {
        left_grid.put_char(ch);
    }

    let right_block = app.get_block_mut(right_pane).unwrap();
    let right_grid = right_block.shell.grid_mut();
    for ch in "Right split pane command output".chars() {
        right_grid.put_char(ch);
    }

    // 1. Ordinary mouse drag inside the text area of the right pane must NOT trigger pane_drag
    let (rb_x, rb_y) = {
        let b = app.get_block(right_pane).unwrap();
        (b.x, b.y)
    };
    let text_click_x = rb_x + 30.0;
    let text_click_y = rb_y + 60.0;

    app.set_test_cursor_pos(text_click_x, text_click_y);
    app.handle_test_mouse_press();
    // At this point without Ctrl and below header, pane_drag must be None
    assert!(
        app.split_manager.pane_drag.is_none(),
        "Ordinary click in split terminal text area must not initiate pane_drag"
    );

    // 2. Ctrl-click or click in top header drag handle DOES trigger pane_drag
    app.set_test_modifiers(winit::keyboard::ModifiersState::CONTROL);
    app.set_test_cursor_pos(text_click_x, text_click_y);
    app.handle_test_mouse_press();
    assert!(
        app.split_manager.pane_drag.is_some(),
        "Ctrl-click should trigger pane drag-and-drop rearrangement"
    );

    // Release mouse
    app.handle_test_mouse_release();
    assert!(app.split_manager.pane_drag.is_none());

    // 3. Header zone click (y <= block.y + 24.0) without Ctrl triggers pane_drag
    app.set_test_modifiers(winit::keyboard::ModifiersState::empty());
    let header_click_y = rb_y + 10.0;
    app.set_test_cursor_pos(text_click_x, header_click_y);
    app.handle_test_mouse_press();
    assert!(
        app.split_manager.pane_drag.is_some(),
        "Clicking top pane drag handle should trigger pane drag-and-drop rearrangement"
    );

    app.handle_test_mouse_release();
    assert!(app.split_manager.pane_drag.is_none());
}

#[test]
fn test_terminal_auto_scroll_to_bottom_on_input_and_paste() {
    let _lock = CLIPBOARD_TEST_MUTEX
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    let mut app = App::new();
    let tab_id = app.open_terminal_tab();

    // Fill grid with 40 lines of text so scrollback is populated
    if let Some(block) = app.get_block_mut(tab_id) {
        let grid = block.shell.grid_mut();
        for i in 0..40 {
            let line = format!("Scrollback history entry #{:02}\r\n", i);
            for ch in line.chars() {
                if ch == '\n' {
                    grid.newline();
                } else if ch != '\r' {
                    grid.put_char(ch);
                }
            }
        }
        assert!(grid.scrollback_len() > 0);
        assert_eq!(grid.scroll_offset(), 0);
    }

    // 1. User scrolls upward to inspect history
    let block = app.get_block_mut(tab_id).unwrap();
    block.shell.scroll_up(15);
    assert_eq!(block.shell.scroll_offset(), 15);

    // 2. Terminal reset_scroll directly returns viewport to bottom
    app.get_block_mut(tab_id).unwrap().shell.reset_scroll();
    assert_eq!(
        app.get_block(tab_id).unwrap().shell.scroll_offset(),
        0,
        "reset_scroll must return scroll offset to 0"
    );

    // 3. Scrolling up again, then pasting text via block.shell.paste_text
    app.get_block_mut(tab_id).unwrap().shell.scroll_up(12);
    assert_eq!(app.get_block(tab_id).unwrap().shell.scroll_offset(), 12);
    app.get_block_mut(tab_id)
        .unwrap()
        .shell
        .paste_text("cargo test\n");
    assert_eq!(
        app.get_block(tab_id).unwrap().shell.scroll_offset(),
        0,
        "Terminal::paste_text must auto-scroll viewport to bottom"
    );

    // 4. Scrolling up again, then pasting via paste_into_active
    app.get_block_mut(tab_id).unwrap().shell.scroll_up(14);
    assert_eq!(app.get_block(tab_id).unwrap().shell.scroll_offset(), 14);
    akssora_terminal_input::set_clipboard_text("echo auto_scroll");
    app.paste_into_active();
    assert_eq!(
        app.get_block(tab_id).unwrap().shell.scroll_offset(),
        0,
        "paste_into_active must auto-scroll viewport to bottom"
    );

    // 5. Normal scrollback navigation is preserved when only scrolling
    app.get_block_mut(tab_id).unwrap().shell.scroll_up(10);
    assert_eq!(app.get_block(tab_id).unwrap().shell.scroll_offset(), 10);
    app.get_block_mut(tab_id).unwrap().shell.scroll_down(4);
    assert_eq!(
        app.get_block(tab_id).unwrap().shell.scroll_offset(),
        6,
        "Normal scrollback navigation must not be reset without user input"
    );

    // 6. Split view isolation: scrolling one pane does not alter other pane's scroll position
    let split_id = app
        .split_active_pane(SplitDirection::Vertical)
        .expect("must create vertical split");

    // Scroll pane 1 up
    app.get_block_mut(tab_id).unwrap().shell.scroll_up(9);
    assert_eq!(app.get_block(tab_id).unwrap().shell.scroll_offset(), 15);

    // Populate split pane with lines so it has scrollback
    if let Some(block) = app.get_block_mut(split_id) {
        let grid = block.shell.grid_mut();
        for i in 0..30 {
            let line = format!("Split pane history entry #{:02}\r\n", i);
            for ch in line.chars() {
                if ch == '\n' {
                    grid.newline();
                } else if ch != '\r' {
                    grid.put_char(ch);
                }
            }
        }
    }

    // Scroll split pane (pane 2) up
    app.get_block_mut(split_id).unwrap().shell.scroll_up(7);
    assert_eq!(app.get_block(split_id).unwrap().shell.scroll_offset(), 7);

    // Reset split pane scroll (e.g. on input)
    app.get_block_mut(split_id).unwrap().shell.reset_scroll();

    // Active split pane (pane 2) scrolled to bottom, while pane 1 preserved its scroll offset
    assert_eq!(
        app.get_block(split_id).unwrap().shell.scroll_offset(),
        0,
        "Active split pane must auto-scroll to bottom on reset"
    );
    assert_eq!(
        app.get_block(tab_id).unwrap().shell.scroll_offset(),
        15,
        "Inactive pane's scroll position must remain preserved"
    );
}
