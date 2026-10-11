// INFO: Asynchronous event channel types emitted by background PTY reader threads.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PtyEvent {
    Data(Vec<u8>),
    Closed,
}
