//! macOS main-thread dispatcher for native `rfd` pickers.
//!
//! `rfd` requires the process main thread when `NSApplication` is not running
//! (headless HTTP host). Tokio `spawn_blocking` workers are not main-thread, so
//! picker routes enqueue work here and the CLI pumps it while awaiting results.

use std::path::{Path, PathBuf};
use std::sync::{mpsc, OnceLock};
use std::thread::ThreadId;
use std::time::Duration;

use crate::picker::{pick_file_rfd, pick_folder_rfd};

enum PickerRequest {
    File {
        filter: Vec<String>,
        multiple: bool,
        start_path: Option<PathBuf>,
        reply: mpsc::Sender<Option<Vec<PathBuf>>>,
    },
    Folder {
        start_path: Option<PathBuf>,
        reply: mpsc::Sender<Option<PathBuf>>,
    },
}

static DISPATCH_TX: OnceLock<mpsc::Sender<PickerRequest>> = OnceLock::new();
static MAIN_THREAD_ID: OnceLock<ThreadId> = OnceLock::new();

/// Pump handle created on the process main thread before starting the HTTP host.
pub struct MacosPickerPump {
    rx: mpsc::Receiver<PickerRequest>,
}

impl MacosPickerPump {
    /// Install the dispatcher and return a pump for the main thread to drain.
    pub fn install() -> Self {
        let (tx, rx) = mpsc::channel();
        let _ = MAIN_THREAD_ID.set(std::thread::current().id());
        let _ = DISPATCH_TX.set(tx);
        Self { rx }
    }

    /// Run at most one queued picker on the main thread, waiting up to `timeout`.
    pub fn drain(&self, timeout: Duration) {
        match self.rx.recv_timeout(timeout) {
            Ok(req) => execute(req),
            Err(mpsc::RecvTimeoutError::Timeout | mpsc::RecvTimeoutError::Disconnected) => {}
        }
    }
}

/// Whether the current thread is the process main thread (set at [`MacosPickerPump::install`]).
pub fn is_main_thread() -> bool {
    MAIN_THREAD_ID
        .get()
        .is_some_and(|id| *id == std::thread::current().id())
}

fn execute(req: PickerRequest) {
    debug_assert!(is_main_thread());
    match req {
        PickerRequest::File {
            filter,
            multiple,
            start_path,
            reply,
        } => {
            let start = start_path.as_deref();
            let _ = reply.send(pick_file_rfd(&filter, multiple, start));
        }
        PickerRequest::Folder { start_path, reply } => {
            let start = start_path.as_deref();
            let _ = reply.send(pick_folder_rfd(start));
        }
    }
}

/// Queue a file picker for the main thread (blocks until the pump runs it).
pub fn dispatch_file(
    filter: &[String],
    multiple: bool,
    start_path: Option<&Path>,
) -> Option<Vec<PathBuf>> {
    let tx = DISPATCH_TX.get()?;
    let (reply_tx, reply_rx) = mpsc::channel();
    tx.send(PickerRequest::File {
        filter: filter.to_vec(),
        multiple,
        start_path: start_path.map(Path::to_path_buf),
        reply: reply_tx,
    })
    .ok()?;
    reply_rx.recv().ok().flatten()
}

/// Queue a folder picker for the main thread (blocks until the pump runs it).
pub fn dispatch_folder(start_path: Option<&Path>) -> Option<PathBuf> {
    let tx = DISPATCH_TX.get()?;
    let (reply_tx, reply_rx) = mpsc::channel();
    tx.send(PickerRequest::Folder {
        start_path: start_path.map(Path::to_path_buf),
        reply: reply_tx,
    })
    .ok()?;
    reply_rx.recv().ok().flatten()
}
