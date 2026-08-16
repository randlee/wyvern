//! `{wyvern_share}` path resolution: workspace materialization and embedded extract.

use std::path::{Path, PathBuf};

use super::{ScriptAssets, ShareAssets};

/// Resolve `{wyvern_share}` to a unified directory (dev workspace or embed).
///
/// Layout: `extensions.json`, `scripts/ext/*`, `ext/csv/**`.
#[must_use]
pub fn resolve_wyvern_share() -> PathBuf {
    resolve_wyvern_share_with(
        std::env::var("WYVERN_SHARE").ok().as_deref(),
        std::env::current_dir().ok().as_deref(),
        std::env::current_exe()
            .ok()
            .as_deref()
            .and_then(Path::parent),
        Some(Path::new(env!("CARGO_MANIFEST_DIR"))),
        true,
    )
}

/// Resolve `{wyvern_share}` from injectable inputs (no process-global mutation).
#[must_use]
pub fn resolve_wyvern_share_with(
    share_var: Option<&str>,
    cwd: Option<&Path>,
    exe_dir: Option<&Path>,
    manifest_dir: Option<&Path>,
    use_embedded: bool,
) -> PathBuf {
    if let Some(path) = share_var {
        tracing::debug!(path, "resolve_wyvern_share: using WYVERN_SHARE override");
        return PathBuf::from(path);
    }
    for start in [cwd, exe_dir, manifest_dir].into_iter().flatten() {
        if let Some(workspace) = find_workspace_root(start) {
            if let Some(unified) = materialize_workspace_share(&workspace) {
                tracing::debug!(
                    path = %unified.display(),
                    "resolve_wyvern_share: materialized workspace share"
                );
                return unified;
            }
            tracing::debug!(
                workspace = %workspace.display(),
                "resolve_wyvern_share: workspace found but materialize failed; trying next start"
            );
        }
    }
    if use_embedded {
        if let Some(extracted) = extract_embedded_share() {
            tracing::debug!(
                path = %extracted.display(),
                "resolve_wyvern_share: using embedded extract"
            );
            return extracted;
        }
        tracing::debug!("resolve_wyvern_share: embedded extract failed; using relative fallback");
    }
    tracing::debug!("resolve_wyvern_share: falling back to share/wyvern");
    PathBuf::from("share/wyvern")
}

/// Walk `start` and parents looking for `share/wyvern/extensions.json`.
#[must_use]
pub fn find_workspace_root(start: &Path) -> Option<PathBuf> {
    let mut current = Some(start);
    while let Some(dir) = current {
        if dir.join("share/wyvern/extensions.json").is_file() {
            return Some(dir.to_path_buf());
        }
        current = dir.parent();
    }
    None
}

fn materialize_workspace_share(workspace: &Path) -> Option<PathBuf> {
    let dest = workspace.join(format!("target/wyvern-share-{}", unique_fs_suffix()));
    let marker = dest.join("extensions.json");
    if file_nonempty(&marker) {
        return Some(dest);
    }
    copy_dir_contents(&workspace.join("share/wyvern"), &dest)?;
    let scripts_src = workspace.join("scripts/ext");
    if scripts_src.is_dir() {
        copy_dir_contents(&scripts_src, &dest.join("scripts/ext"))?;
    }
    file_nonempty(&dest.join("extensions.json")).then_some(dest)
}

fn file_nonempty(path: &Path) -> bool {
    path.is_file() && path.metadata().is_ok_and(|m| m.len() > 0)
}

fn copy_dir_contents(src: &Path, dest: &Path) -> Option<()> {
    if !src.is_dir() {
        return Some(());
    }
    if let Err(e) = std::fs::create_dir_all(dest) {
        tracing::warn!(
            "copy_dir_contents: failed to copy {} to {}: {e}",
            src.display(),
            dest.display()
        );
        return None;
    }
    let entries = match std::fs::read_dir(src) {
        Ok(entries) => entries,
        Err(e) => {
            tracing::warn!(
                "copy_dir_contents: failed to copy {} to {}: {e}",
                src.display(),
                dest.display()
            );
            return None;
        }
    };
    for entry in entries {
        let entry = match entry {
            Ok(entry) => entry,
            Err(e) => {
                tracing::warn!(
                    "copy_dir_contents: failed to copy {} to {}: {e}",
                    src.display(),
                    dest.display()
                );
                return None;
            }
        };
        let from = entry.path();
        let to = dest.join(entry.file_name());
        if from.is_dir() {
            copy_dir_contents(&from, &to)?;
        } else {
            if file_nonempty(&to) {
                continue;
            }
            copy_file_replace(&from, &to)?;
        }
    }
    Some(())
}

fn unique_fs_suffix() -> String {
    let tid = format!("{:?}", std::thread::current().id());
    let tid: String = tid.chars().filter(|c| c.is_ascii_alphanumeric()).collect();
    format!("{}-{tid}", std::process::id())
}

fn copy_file_replace(from: &Path, to: &Path) -> Option<()> {
    let tmp = to.with_file_name(format!(
        ".{}.part-{}",
        to.file_name()?.to_string_lossy(),
        unique_fs_suffix()
    ));
    std::fs::copy(from, &tmp).ok()?;
    match std::fs::rename(&tmp, to) {
        Ok(()) => Some(()),
        Err(_) => {
            let _ = std::fs::remove_file(&tmp);
            file_nonempty(to).then_some(())
        }
    }
}

fn write_embedded_atomically(out: &Path, data: &[u8]) -> Option<()> {
    if let Some(parent) = out.parent() {
        if let Err(e) = std::fs::create_dir_all(parent) {
            tracing::warn!(
                "extract_embedded_share: failed to write {}: proceeding with fallback ({e})",
                out.display()
            );
            return None;
        }
    }
    let tmp_out = out.with_file_name(format!(
        ".{}.part-{}",
        out.file_name()?.to_string_lossy(),
        unique_fs_suffix()
    ));
    if let Err(e) = std::fs::write(&tmp_out, data) {
        tracing::warn!(
            "extract_embedded_share: failed to write {}: proceeding with fallback ({e})",
            out.display()
        );
        return None;
    }
    if let Err(e) = std::fs::rename(&tmp_out, out) {
        let _ = std::fs::remove_file(&tmp_out);
        tracing::warn!(
            "extract_embedded_share: failed to write {}: proceeding with fallback ({e})",
            out.display()
        );
        return None;
    }
    Some(())
}

fn extract_embedded_share() -> Option<PathBuf> {
    let dest = dirs::cache_dir()?
        .join("wyvern")
        .join(env!("CARGO_PKG_VERSION"))
        .join("share");
    if let Err(e) = std::fs::create_dir_all(&dest) {
        tracing::warn!(
            "extract_embedded_share: failed to write {}: proceeding with fallback ({e})",
            dest.display()
        );
        return None;
    }
    for path in ShareAssets::iter() {
        let out = dest.join(path.as_ref());
        let file = ShareAssets::get(path.as_ref())?;
        write_embedded_atomically(&out, file.data.as_ref())?;
    }
    let scripts_dest = dest.join("scripts/ext");
    if let Err(e) = std::fs::create_dir_all(&scripts_dest) {
        tracing::warn!(
            "extract_embedded_share: failed to write {}: proceeding with fallback ({e})",
            scripts_dest.display()
        );
        return None;
    }
    for path in ScriptAssets::iter() {
        let out = scripts_dest.join(path.as_ref());
        let file = ScriptAssets::get(path.as_ref())?;
        write_embedded_atomically(&out, file.data.as_ref())?;
    }
    dest.join("extensions.json").is_file().then_some(dest)
}
