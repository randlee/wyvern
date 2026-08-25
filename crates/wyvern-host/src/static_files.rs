//! Static UI root checks and path-traversal helpers.

use std::path::{Path, PathBuf};

use crate::error::{DialogTypeName, HostError};

/// Resolve `ui_root` and ensure the dialog type template directory exists.
pub(crate) fn require_type_dir(
    ui_root: &Path,
    type_name: DialogTypeName,
) -> Result<PathBuf, HostError> {
    let root = ui_root
        .canonicalize()
        .map_err(|source| HostError::UiNotFound {
            path: ui_root.to_path_buf(),
            source: Some(source),
        })?;
    let type_dir = root.join(type_name.as_str());
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

/// Resolve `--ui-root` for a wizard and ensure `page.html` exists under it.
pub(crate) fn require_wizard_page(ui_root: &Path, page_html: &str) -> Result<PathBuf, HostError> {
    let root = ui_root
        .canonicalize()
        .map_err(|source| HostError::UiNotFound {
            path: ui_root.to_path_buf(),
            source: Some(source),
        })?;
    let page_path = safe_join(&root, page_html).ok_or_else(|| HostError::UiNotFound {
        path: root.join(page_html),
        source: None,
    })?;
    if !page_path.is_file() {
        return Err(HostError::UiNotFound {
            path: page_path,
            source: None,
        });
    }
    Ok(root)
}

/// Resolve `--ui-root` for a report and ensure `page` exists under it.
///
/// Must **not** use [`require_type_dir`] / `{ui_root}/report/index.html`
/// packaged-dialog layout (ADR-0025 / REQ-HOST-0140).
pub(crate) fn require_report_page(ui_root: &Path, page: &str) -> Result<PathBuf, HostError> {
    let root = ui_root
        .canonicalize()
        .map_err(|source| HostError::UiNotFound {
            path: ui_root.to_path_buf(),
            source: Some(source),
        })?;
    let page_path = safe_join(&root, page).ok_or_else(|| HostError::UiNotFound {
        path: root.join(page),
        source: None,
    })?;
    if !page_path.is_file() {
        return Err(HostError::UiNotFound {
            path: page_path,
            source: None,
        });
    }
    Ok(root)
}

/// Canonicalize packaged `shared_ui_root` and require `shared/wyvern-api.js`.
pub(crate) fn require_shared_ui_root(shared_ui_root: &Path) -> Result<PathBuf, HostError> {
    let root = shared_ui_root
        .canonicalize()
        .map_err(|source| HostError::UiNotFound {
            path: shared_ui_root.to_path_buf(),
            source: Some(source),
        })?;
    let api_js = root.join("shared").join("wyvern-api.js");
    if !api_js.is_file() {
        return Err(HostError::UiNotFound {
            path: api_js,
            source: None,
        });
    }
    Ok(root)
}

/// Join `root` with a URL path, rejecting `..` and absolute components.
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
        let err = require_type_dir(&missing, DialogTypeName::Message).expect_err("missing root");
        match err {
            HostError::UiNotFound { path, source } => {
                assert_eq!(path, missing);
                assert!(source.is_some(), "expected preserved IO cause");
            }
            other => panic!("expected UiNotFound, got {other:?}"),
        }
    }

    #[test]
    fn report_bind_require_report_page_rejects_packaged_index_pattern() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let root = tmp.path();
        std::fs::create_dir_all(root.join("report")).expect("report dir");
        std::fs::write(root.join("report").join("index.html"), "<html></html>").expect("index");
        let err = require_report_page(root, "pages/view.xhtml").expect_err("missing page");
        match err {
            HostError::UiNotFound { path, .. } => {
                assert!(
                    path.ends_with("pages/view.xhtml"),
                    "must fail on command page, not report/index.html: {}",
                    path.display()
                );
            }
            other => panic!("expected UiNotFound, got {other:?}"),
        }
    }

    #[test]
    fn report_bind_require_report_page_accepts_existing_page() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let root = tmp.path();
        std::fs::create_dir_all(root.join("pages")).expect("pages");
        std::fs::write(root.join("pages").join("view.xhtml"), "<section></section>").expect("page");
        let resolved = require_report_page(root, "pages/view.xhtml").expect("page exists");
        assert_eq!(resolved, root.canonicalize().expect("canon"));
    }
}
