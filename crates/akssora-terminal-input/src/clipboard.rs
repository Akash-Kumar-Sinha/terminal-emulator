use std::io::Write;
use std::process::{Command, Stdio};
use std::sync::Mutex;

static CLIPBOARD_LOCK: Mutex<()> = Mutex::new(());
static INTERNAL_CLIPBOARD: Mutex<String> = Mutex::new(String::new());

// INFO: Retrieves plain text content from the system clipboard or in-memory fallback.
pub fn get_clipboard_text() -> Option<String> {
    let _guard = CLIPBOARD_LOCK.lock().ok();

    // 1. Try Wayland clipboard (wl-paste)
    if let Ok(output) = Command::new("wl-paste")
        .arg("--no-newline")
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output()
        && output.status.success()
        && let Ok(s) = String::from_utf8(output.stdout)
        && !s.is_empty()
    {
        return Some(s);
    }

    // 2. Try X11 clipboard (xclip)
    if let Ok(output) = Command::new("xclip")
        .args(["-selection", "clipboard", "-o"])
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output()
        && output.status.success()
        && let Ok(s) = String::from_utf8(output.stdout)
        && !s.is_empty()
    {
        return Some(s);
    }

    // 3. Fallback to in-memory clipboard
    let lock = INTERNAL_CLIPBOARD.lock().ok()?;
    if lock.is_empty() {
        None
    } else {
        Some(lock.clone())
    }
}

// INFO: Stores plain text content into the system clipboard and in-memory fallback.
pub fn set_clipboard_text(text: &str) {
    let _guard = CLIPBOARD_LOCK.lock().ok();

    if let Ok(mut lock) = INTERNAL_CLIPBOARD.lock() {
        *lock = text.to_string();
    }

    // 1. Try Wayland wl-copy
    if let Ok(mut child) = Command::new("wl-copy")
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
    {
        if let Some(mut stdin) = child.stdin.take() {
            let _ = stdin.write_all(text.as_bytes());
        }
        let _ = child.wait();
        return;
    }

    // 2. Try X11 xclip
    if let Ok(mut child) = Command::new("xclip")
        .args(["-selection", "clipboard"])
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
    {
        if let Some(mut stdin) = child.stdin.take() {
            let _ = stdin.write_all(text.as_bytes());
        }
        let _ = child.wait();
    }
}
