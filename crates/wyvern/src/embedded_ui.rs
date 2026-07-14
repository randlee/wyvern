//! Embedded UI assets and cache-directory extraction for `cargo install` users.
//!
//! When wyvern is installed via `cargo install`, no `share/wyvern/ui` directory
//! exists next to the binary.  This module embeds every file from `ui/` into the
//! binary at compile time (compressed via `rust-embed`) and, on first call to
//! [`extract_to_cache`], writes them to
//! `<user-cache>/wyvern/<version>/ui/` so the HTTP host can serve them from disk.
//!
//! The version tag in the path means a new release automatically invalidates the
//! previous cache without a manual clean-up step.
//!
//! # Errors
//!
//! Returns `None` (silently) when the platform cache directory is unavailable or
//! a write fails; the caller falls back to the documented `./ui` path and the
//! host emits a clear "UI not found" error rather than a cryptic extraction error.

use std::path::PathBuf;

#[derive(rust_embed::RustEmbed)]
#[folder = "ui"]
struct Assets;

/// Extract embedded UI assets to the user cache directory and return the root.
///
/// Returns the path of the extracted `ui/` root on success, or `None` when the
/// platform cache directory is unavailable or any write fails.
pub fn extract_to_cache() -> Option<PathBuf> {
    let cache_root = dirs::cache_dir()?
        .join("wyvern")
        .join(env!("CARGO_PKG_VERSION"))
        .join("ui");

    if is_valid_ui_root(&cache_root) {
        return Some(cache_root);
    }

    for path in Assets::iter() {
        let dest = cache_root.join(path.as_ref());
        if let Some(parent) = dest.parent() {
            if std::fs::create_dir_all(parent).is_err() {
                return None;
            }
        }
        let embedded = Assets::get(path.as_ref())?;
        if std::fs::write(&dest, embedded.data.as_ref()).is_err() {
            return None;
        }
    }

    if is_valid_ui_root(&cache_root) {
        Some(cache_root)
    } else {
        None
    }
}

/// Returns `true` when `root` contains all five required dialog type directories.
fn is_valid_ui_root(root: &std::path::Path) -> bool {
    ["message", "input", "markdown", "question", "chrome"]
        .iter()
        .all(|t| root.join(t).join("index.html").is_file())
}
