pub mod clipboard;
pub mod key;
pub mod mouse;

pub use clipboard::{get_clipboard_text, set_clipboard_text};
pub use key::encode_key;
pub use mouse::{encode_sgr_mouse, scroll_delta_to_lines};
