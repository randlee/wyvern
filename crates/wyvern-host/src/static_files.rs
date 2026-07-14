//! Static UI root checks and path-traversal helpers.

use std::path::{Path, PathBuf};

use crate::error::HostError;

/// Resolve `ui_root` and ensure the dialog type template directory exists.
pub(crate) fn require_type_dir(ui_root: &Path, type_name: &str) -> Result<PathBuf, HostError> {
    let root = ui_root
        .canonicalize()
        .map_err(|source| HostError::UiNotFound {
            path: ui_root.to_path_buf(),
            source: Some(source),
        })?;
    let type_dir = root.join(type_name);
    if !type_dir.is_dir() {
        return Err(HostError::UiNotFound {
            path: type_dir,
            source: None,
        });
    }
    let index = type_dir.join("index.html");
    if !index.is_file() {
        return Err(HostError::UiNotFound {
            path: index,
            source: None,
        });
    }
    Ok(root)
}

/// Join `root` with a URL path, rejecting `..` and absolute components.
#[cfg(test)]
pub(crate) fn safe_join(root: &Path, url_path: &str) -> Option<PathBuf> {
    use std::path::Component;

    let trimmed = url_path.trim_start_matches('/');
    if trimmed.is_empty() {
        return Some(root.to_path_buf());
    }
    let mut out = root.to_path_buf();
    for comp in Path::new(trimmed).components() {
        match comp {
            Component::Normal(seg) => out.push(seg),
            Component::CurDir => {}
            Component::ParentDir | Component::RootDir | Component::Prefix(_) => return None,
        }
    }
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn safe_join_rejects_parent() {
        let root = std::env::temp_dir().join("ui");
        assert!(safe_join(&root, "../etc/passwd").is_none());
        assert!(safe_join(&root, "message/../../etc").is_none());
    }

    #[test]
    fn safe_join_allows_nested() {
        let root = std::env::temp_dir().join("ui");
        let joined = safe_join(&root, "message/app.js").expect("ok");
        assert_eq!(joined, root.join("message").join("app.js"));
    }

    #[test]
    fn require_type_dir_preserves_canonicalize_io_error() {
        let missing = std::env::temp_dir().join(format!(
            "wyvern-missing-ui-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));
        let err = require_type_dir(&missing, "message").expect_err("missing root");
        match err {
            HostError::UiNotFound { path, source } => {
                assert_eq!(path, missing);
                assert!(source.is_some(), "expected preserved IO cause");
            }
            other => panic!("expected UiNotFound, got {other:?}"),
        }
    }
}
