use winit::event::KeyEvent;
use winit::keyboard::{Key, ModifiersState, NamedKey};

// INFO: Maps raw window key events and active modifiers into standard ANSI/VT terminal byte sequences.
pub fn encode_key(event: &KeyEvent, mods: ModifiersState) -> Option<Vec<u8>> {
    if mods.control_key()
        && let Key::Character(s) = &event.logical_key
        && let Some(c) = s.chars().next()
    {
        let up = c.to_ascii_uppercase();
        if up.is_ascii_alphabetic() {
            return Some(vec![(up as u8) & 0x1f]);
        }
        match c {
            ' ' => return Some(vec![0]),
            '[' => return Some(vec![0x1b]),
            '\\' => return Some(vec![0x1c]),
            ']' => return Some(vec![0x1d]),
            _ => {}
        }
    }

    match &event.logical_key {
        Key::Named(named) => match named {
            NamedKey::Enter => Some(b"\r".to_vec()),
            NamedKey::Backspace => Some(vec![0x7f]),
            NamedKey::Tab => Some(b"\t".to_vec()),
            NamedKey::Escape => Some(vec![0x1b]),
            NamedKey::ArrowUp => Some(b"\x1b[A".to_vec()),
            NamedKey::ArrowDown => Some(b"\x1b[B".to_vec()),
            NamedKey::ArrowRight => Some(b"\x1b[C".to_vec()),
            NamedKey::ArrowLeft => Some(b"\x1b[D".to_vec()),
            NamedKey::Home => Some(b"\x1b[H".to_vec()),
            NamedKey::End => Some(b"\x1b[F".to_vec()),
            NamedKey::Delete => Some(b"\x1b[3~".to_vec()),
            NamedKey::PageUp => Some(b"\x1b[5~".to_vec()),
            NamedKey::PageDown => Some(b"\x1b[6~".to_vec()),
            NamedKey::Space => Some(b" ".to_vec()),
            _ => None,
        },
        _ => event.text.as_ref().map(|t| t.as_bytes().to_vec()),
    }
}
