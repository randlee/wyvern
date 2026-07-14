//! Wyvern HTTP dialog host — bind, serve packaged UI, await `POST /api/result`.
//!
//! Greenfield crate (sprint c.10). No wry/winit. One-shot `run()` serves a single
//! dialog session over loopback HTTP for `none` / `system` / `named` viewers.
//! Embedded one-shot uses [`begin`] + CLI `embedded_viewer_spawn` (c.15).

#![cfg_attr(
    not(test),
    deny(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::panic,
        clippy::unreachable,
        clippy::todo,
        clippy::unimplemented
    )
)]

mod browser_catalog;
mod browser_launch;
mod browser_registry;
mod error;
mod handle;
mod markdown;
mod options;
mod picker;
mod question;
mod routes;
mod server;
mod session;
mod static_files;

pub use browser_registry::{
    list_entries as list_browser_entries, load_or_refresh as load_or_refresh_browser_registry,
    refresh as refresh_browser_registry, registry_path as browser_registry_path,
    BrowserRegistryEntry, BrowserRegistryFile,
};
pub use error::{DialogTypeName, HostError};
pub use handle::{begin, DialogHandle};
pub use options::{
    BrowserId, HostOptions, ViewerLaunchOptions, ViewerMode, DEFAULT_SESSION_TIMEOUT,
};
pub use picker::{MockPickerConfig, MockPickerSlotEvent, MockPickerSlotLog};

use wyvern_schema::{Command, CommandResult};

use crate::handle::{dialog_type_name, run_owned_async};

/// One-shot convenience for viewer modes the host owns (`none` / `system` / `named`).
///
/// Binds HTTP, optionally publishes the dialog URL (stderr / file), opens a system
/// or named browser when requested, serves static UI + API, and returns when the
/// page POSTs `/api/result`.
///
/// **Must not** be used for [`ViewerMode::Embedded`] — the CLI owns embedded spawn
/// via [`begin`] + `embedded_viewer_spawn`.
///
/// # Errors
///
/// Returns [`HostError`] on unsupported embedded mode, bind/UI failures, viewer
/// miss, or internal server faults.
pub fn run(command: Command, options: HostOptions) -> Result<CommandResult, HostError> {
    match options.viewer {
        ViewerMode::None | ViewerMode::System | ViewerMode::Named(_) => {}
        ViewerMode::Embedded => {
            return Err(HostError::ViewerUnsupported {
                mode: ViewerMode::Embedded,
            });
        }
    }

    let type_name = dialog_type_name(&command);

    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .map_err(|e| HostError::Internal {
            message: format!("failed to create tokio runtime: {e}"),
        })?;

    rt.block_on(run_owned_async(command, options, type_name))
}
