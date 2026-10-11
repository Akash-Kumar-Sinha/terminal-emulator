use std::path::Path;

use crate::diagnostic::Diagnostic;

// INFO: Language server protocol client interface defining editor synchronization and diagnostic retrieval.
pub trait LspClient: Send + Sync {
    fn initialize(&mut self, root_path: &Path) -> std::io::Result<()>;
    fn did_open(&mut self, file_path: &Path, content: &str) -> std::io::Result<()>;
    fn did_change(&mut self, file_path: &Path, content: &str) -> std::io::Result<()>;
    fn diagnostics(&self, file_path: &Path) -> Vec<Diagnostic>;
    fn shutdown(&mut self) -> std::io::Result<()>;
}
