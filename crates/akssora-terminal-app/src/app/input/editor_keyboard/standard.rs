use akssora_editor::Document;
use winit::event::KeyEvent;
use winit::keyboard::{Key, NamedKey};

// INFO: Result of processing a standard non-Vim editor key event.
#[derive(Debug, Default)]
pub(crate) struct StandardKeyResult {}

// INFO: Standard non-Vim editor navigation, selection, and character input dispatch.
pub(crate) fn handle_standard_key(
    doc: &mut Document,
    event: &KeyEvent,
    shift: bool,
    ctrl: bool,
    visible_rows: usize,
    visible_cols: usize,
    _is_code_review: bool,
) -> StandardKeyResult {
    let result = StandardKeyResult::default();

    match &event.logical_key {
        Key::Named(NamedKey::ArrowLeft) => doc.move_cursor_left(shift),
        Key::Named(NamedKey::ArrowRight) => doc.move_cursor_right(shift),
        Key::Named(NamedKey::ArrowUp) => doc.move_cursor_up_visual(shift, visible_cols),
        Key::Named(NamedKey::ArrowDown) => doc.move_cursor_down_visual(shift, visible_cols),
        Key::Named(NamedKey::Home) => doc.move_to_line_start_visual(shift, visible_cols),
        Key::Named(NamedKey::End) => doc.move_to_line_end_visual(shift, visible_cols),
        Key::Named(NamedKey::PageUp) => {
            for _ in 0..visible_rows.max(1) {
                doc.move_cursor_up_visual(shift, visible_cols);
            }
        }
        Key::Named(NamedKey::PageDown) => {
            for _ in 0..visible_rows.max(1) {
                doc.move_cursor_down_visual(shift, visible_cols);
            }
        }
        Key::Named(NamedKey::Backspace) => doc.backspace(),
        Key::Named(NamedKey::Delete) => doc.delete(),
        Key::Named(NamedKey::Enter) => doc.insert_newline(),
        Key::Named(NamedKey::Tab) => doc.insert_tab(),
        Key::Named(NamedKey::Space) if !ctrl => doc.insert_char_smart(' '),
        Key::Named(NamedKey::Escape) => {
            doc.clear_selection();
        }
        Key::Character(s) if !ctrl => {
            for ch in s.chars() {
                doc.insert_char_smart(ch);
            }
        }
        _ => {}
    }

    result
}
