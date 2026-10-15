use std::path::PathBuf;
use std::sync::Arc;
use std::sync::mpsc::{Receiver, Sender, channel};
use std::thread;

use akssora_git::{IndexedRepoDiff, load_repo_diff_indexed};

// INFO: Background worker request message dispatched from the UI thread.
#[derive(Clone, Debug)]
pub struct DiffWorkerRequest {
    pub terminal_id: u64,
    pub generation: u64,
    pub repo_root: PathBuf,
    pub staged: bool,
}

// INFO: Preprocessed immutable diff model response delivered back to the UI thread.
pub struct DiffWorkerResponse {
    pub terminal_id: u64,
    pub generation: u64,
    pub repo_root: PathBuf,
    pub staged: bool,
    pub staged_diff: Arc<IndexedRepoDiff>,
    pub unstaged_diff: Arc<IndexedRepoDiff>,
    pub diff: Arc<IndexedRepoDiff>,
}

// INFO: Dedicated background worker thread isolating Git I/O, diff parsing, and Rayon preprocessing.
pub struct DiffBackgroundWorker {
    request_tx: Sender<DiffWorkerRequest>,
    response_rx: Receiver<DiffWorkerResponse>,
}

impl DiffBackgroundWorker {
    pub fn new() -> Self {
        let (req_tx, req_rx) = channel::<DiffWorkerRequest>();
        let (res_tx, res_rx) = channel::<DiffWorkerResponse>();

        thread::Builder::new()
            .name("akssora-git-diff-worker".to_string())
            .spawn(move || {
                while let Ok(mut req) = req_rx.recv() {
                    // Drain any queued requests in channel to coalesce rapid bursts into latest state
                    while let Ok(newer_req) = req_rx.try_recv() {
                        req = newer_req;
                    }

                    // Step A: Git diff retrieval -> Step C: Parse diff -> Step D: Rayon parallel preprocessing
                    let staged_indexed = load_repo_diff_indexed(&req.repo_root, true);
                    let unstaged_indexed = load_repo_diff_indexed(&req.repo_root, false);
                    let active = if req.staged {
                        staged_indexed.clone()
                    } else {
                        unstaged_indexed.clone()
                    };
                    let response = DiffWorkerResponse {
                        terminal_id: req.terminal_id,
                        generation: req.generation,
                        repo_root: req.repo_root,
                        staged: req.staged,
                        staged_diff: Arc::new(staged_indexed),
                        unstaged_diff: Arc::new(unstaged_indexed),
                        diff: Arc::new(active),
                    };

                    // Step E: Send indexed immutable diff model to Step F: UI thread
                    if res_tx.send(response).is_err() {
                        break;
                    }
                }
            })
            .expect("Failed to spawn git diff background worker thread");

        Self {
            request_tx: req_tx,
            response_rx: res_rx,
        }
    }

    pub fn send_request(&self, request: DiffWorkerRequest) {
        let _ = self.request_tx.send(request);
    }

    pub fn try_recv_response(&self) -> Option<DiffWorkerResponse> {
        self.response_rx.try_recv().ok()
    }
}

impl Default for DiffBackgroundWorker {
    fn default() -> Self {
        Self::new()
    }
}
