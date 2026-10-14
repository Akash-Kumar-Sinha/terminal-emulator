use crate::app::App;
use winit::event::KeyEvent;
use winit::keyboard::Key;

// INFO: Handles global editor chords, function shortcuts, and document-level key commands.
pub(crate) fn handle_editor_shortcuts(
    app: &mut App,
    event: &KeyEvent,
    visible_rows: usize,
    visible_cols: usize,
) -> bool {
    let ctrl = app.modifiers.control_key();
    let shift = app.modifiers.shift_key();
    let alt = app.modifiers.alt_key();

    // Check pending chord first
    if let Some((chord, inst)) = app.pending_chord.take()
        && inst.elapsed() < std::time::Duration::from_millis(1500)
        && chord == "Ctrl+K"
    {
        if ctrl
            && !shift
            && matches!(&event.logical_key, Key::Character(s) if s.eq_ignore_ascii_case("w"))
        {
            app.request_close_all_editor_tabs();
            return true;
        }
        // Ctrl+K V chord: Toggle Markdown split preview
        if !ctrl && matches!(&event.logical_key, Key::Character(s) if s.eq_ignore_ascii_case("v")) {
            app.toggle_active_markdown_split_preview();
            return true;
        }
        if ctrl && matches!(&event.logical_key, Key::Character(s) if s.eq_ignore_ascii_case("v")) {
            app.toggle_active_markdown_split_preview();
            return true;
        }
    }

    // Detect start of Ctrl+K chord
    if ctrl
        && !shift
        && matches!(&event.logical_key, Key::Character(s) if s.eq_ignore_ascii_case("k"))
    {
        app.pending_chord = Some(("Ctrl+K".to_string(), std::time::Instant::now()));
        return true;
    }

    // Format Document: Alt+Shift+F or Ctrl+Shift+I
    if ((alt
        && shift
        && matches!(&event.logical_key, Key::Character(s) if s.eq_ignore_ascii_case("f")))
        || (ctrl
            && shift
            && matches!(&event.logical_key, Key::Character(s) if s.eq_ignore_ascii_case("i"))))
        && app.format_active_editor()
    {
        return true;
    }

    // Toggle Markdown Preview: Ctrl+Shift+V
    if ctrl
        && shift
        && matches!(&event.logical_key, Key::Character(s) if s.eq_ignore_ascii_case("v"))
        && let Some(doc) = app.active_editor()
        && doc.is_markdown()
    {
        app.toggle_active_markdown_preview();
        return true;
    }

    // Toggle Inline Diff: Ctrl+Alt+D
    if ctrl
        && alt
        && matches!(&event.logical_key, Key::Character(s) if s.eq_ignore_ascii_case("d"))
        && let Some(doc) = app.active_editor_mut()
    {
        doc.inline_diff = !doc.inline_diff;
        if let Some(win) = app.window.as_ref() {
            win.request_redraw();
        }
        return true;
    }

    // Close other editor tabs: Ctrl+Shift+W
    if ctrl
        && shift
        && matches!(&event.logical_key, Key::Character(s) if s.eq_ignore_ascii_case("w"))
    {
        app.request_close_other_editor_tabs();
        return true;
    }

    // Close current editor tab: Ctrl+W
    if ctrl
        && !shift
        && matches!(&event.logical_key, Key::Character(s) if s.eq_ignore_ascii_case("w"))
    {
        if let Some(pos) = app
            .editor_active_file
            .as_ref()
            .and_then(|p| app.editor_open_files.iter().position(|f| f == p))
        {
            app.request_close_editor_tab(pos);
        } else if !app.editor_open_files.is_empty() {
            app.request_close_editor_tab(0);
        } else {
            app.close_active_tab_or_prompt();
        }
        return true;
    }

    // Save: Ctrl+S (with configurable format-on-save)
    if ctrl
        && !shift
        && matches!(&event.logical_key, Key::Character(s) if s.eq_ignore_ascii_case("s"))
    {
        if app.config.format_on_save {
            let _ = app.format_active_editor();
        }
        if let Some(doc) = app.active_editor_mut() {
            let _ = doc.save();
        }
        let is_code_review = app
            .tab_manager
            .active_tab()
            .map(|t| matches!(t.kind, akssora_terminal_ui::TabKind::CodeReview { .. }))
            .unwrap_or(false);
        if is_code_review {
            app.request_code_review_diff_reload();
        }
        app.refresh_git_status();
        if let Some(win) = app.window.as_ref() {
            win.request_redraw();
        }
        return true;
    }

    if let Some(doc) = app.active_editor_mut() {
        // Undo: Ctrl+Z
        if ctrl
            && !shift
            && matches!(&event.logical_key, Key::Character(s) if s.eq_ignore_ascii_case("z"))
        {
            doc.undo();
            if let Some(win) = app.window.as_ref() {
                win.request_redraw();
            }
            return true;
        }
        // Redo: Ctrl+Y or Ctrl+Shift+Z
        if (ctrl
            && !shift
            && matches!(&event.logical_key, Key::Character(s) if s.eq_ignore_ascii_case("y")))
            || (ctrl
                && shift
                && matches!(&event.logical_key, Key::Character(s) if s.eq_ignore_ascii_case("z")))
        {
            doc.redo();
            if let Some(win) = app.window.as_ref() {
                win.request_redraw();
            }
            return true;
        }
        // Select all: Ctrl+A
        if ctrl
            && !shift
            && matches!(&event.logical_key, Key::Character(s) if s.eq_ignore_ascii_case("a"))
        {
            doc.select_all();
            if let Some(win) = app.window.as_ref() {
                win.request_redraw();
            }
            return true;
        }

        // Toggle Line Wrapping: Alt+Z
        if alt
            && !ctrl
            && matches!(&event.logical_key, Key::Character(s) if s.eq_ignore_ascii_case("z"))
        {
            let new_wrap = !doc.line_wrap;
            doc.line_wrap = new_wrap;
            doc.scroll_to_cursor(visible_rows.max(1), visible_cols.max(1));
            app.config.line_wrap = new_wrap;
            let _ = app.config.save();
            if let Some(win) = app.window.as_ref() {
                win.request_redraw();
            }
            return true;
        }
    }

    false
}
