//! Bundled example discovery from `{wyvern_share}/examples/**/README.md` frontmatter.

mod frontmatter;

use std::path::{Path, PathBuf};

use serde::Serialize;

pub use frontmatter::parse_readme_frontmatter;

/// One catalog row from a README with YAML frontmatter.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ExampleRecord {
    /// Display name from frontmatter.
    pub name: String,
    /// Short description from frontmatter.
    pub description: String,
    /// Path to the README, relative to `{wyvern_share}` when possible.
    pub readme: String,
}

/// Failure while scanning example README files.
#[derive(Debug)]
pub enum ExamplesDiscoverError {
    /// The examples root directory could not be read.
    Io {
        /// Affected path.
        path: PathBuf,
        /// Error detail.
        message: String,
    },
}

/// One README that violates the bundled example discovery contract.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExampleReadmeViolation {
    /// Path to the README under audit (absolute or share-relative).
    pub readme: PathBuf,
    /// Human-readable violation detail.
    pub message: String,
}

impl std::fmt::Display for ExampleReadmeViolation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.readme.display(), self.message)
    }
}

impl std::fmt::Display for ExamplesDiscoverError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io { path, message } => {
                write!(f, "failed to read {}: {message}", path.display())
            }
        }
    }
}

impl std::error::Error for ExamplesDiscoverError {}

/// Discover bundled examples under `{share_root}/examples/`.
///
/// Each `README.md` with mandatory `name` and `description` frontmatter becomes
/// one record. READMEs may live in an example folder or in the examples base
/// folder when one README documents multiple related examples.
///
/// # Errors
///
/// Returns [`ExamplesDiscoverError`] when the examples directory cannot be read.
pub fn discover_examples(share_root: &Path) -> Result<Vec<ExampleRecord>, ExamplesDiscoverError> {
    let examples_root = share_root.join("examples");
    if !examples_root.is_dir() {
        return Ok(Vec::new());
    }

    let mut records = Vec::new();
    let mut seen = std::collections::BTreeSet::new();

    let base_readme = examples_root.join("README.md");
    if let Some(record) = record_from_readme(&base_readme, share_root) {
        seen.insert(record.readme.clone());
        records.push(record);
    }

    let entries = std::fs::read_dir(&examples_root).map_err(|err| ExamplesDiscoverError::Io {
        path: examples_root.clone(),
        message: err.to_string(),
    })?;
    for entry in entries {
        let entry = match entry {
            Ok(entry) => entry,
            Err(err) => {
                return Err(ExamplesDiscoverError::Io {
                    path: examples_root.clone(),
                    message: err.to_string(),
                });
            }
        };
        let file_type = match entry.file_type() {
            Ok(file_type) => file_type,
            Err(err) => {
                return Err(ExamplesDiscoverError::Io {
                    path: entry.path(),
                    message: err.to_string(),
                });
            }
        };
        if !file_type.is_dir() {
            continue;
        }
        let readme = entry.path().join("README.md");
        if let Some(record) = record_from_readme(&readme, share_root) {
            if seen.insert(record.readme.clone()) {
                records.push(record);
            }
        }
    }

    records.sort_by(|left, right| {
        left.name
            .to_ascii_lowercase()
            .cmp(&right.name.to_ascii_lowercase())
    });
    Ok(records)
}

/// Audit every immediate child directory under `{share_root}/examples/` for a
/// `README.md` whose YAML frontmatter satisfies the discovery contract
/// (`name` + `description`).
///
/// Optional `examples/README.md` (base-folder doc) is validated when present.
///
/// # Errors
///
/// Returns [`ExamplesDiscoverError`] when the examples directory cannot be read.
pub fn validate_example_folder_readmes(
    share_root: &Path,
) -> Result<Vec<ExampleReadmeViolation>, ExamplesDiscoverError> {
    let examples_root = share_root.join("examples");
    if !examples_root.is_dir() {
        return Ok(Vec::new());
    }

    let mut violations = Vec::new();
    let base_readme = examples_root.join("README.md");
    if base_readme.is_file() {
        violations.extend(audit_readme_contract(&base_readme));
    }

    let entries = std::fs::read_dir(&examples_root).map_err(|err| ExamplesDiscoverError::Io {
        path: examples_root.clone(),
        message: err.to_string(),
    })?;
    for entry in entries {
        let entry = match entry {
            Ok(entry) => entry,
            Err(err) => {
                return Err(ExamplesDiscoverError::Io {
                    path: examples_root.clone(),
                    message: err.to_string(),
                });
            }
        };
        let file_type = match entry.file_type() {
            Ok(file_type) => file_type,
            Err(err) => {
                return Err(ExamplesDiscoverError::Io {
                    path: entry.path(),
                    message: err.to_string(),
                });
            }
        };
        if !file_type.is_dir() {
            continue;
        }
        let readme = entry.path().join("README.md");
        if !readme.is_file() {
            violations.push(ExampleReadmeViolation {
                readme: readme.clone(),
                message: "missing README.md (each examples/<dir>/ must ship a README)".into(),
            });
            continue;
        }
        violations.extend(audit_readme_contract(&readme));
    }

    Ok(violations)
}

fn audit_readme_contract(readme: &Path) -> Vec<ExampleReadmeViolation> {
    let content = match std::fs::read_to_string(readme) {
        Ok(content) => content,
        Err(err) => {
            return vec![ExampleReadmeViolation {
                readme: readme.to_path_buf(),
                message: format!("could not read README: {err}"),
            }];
        }
    };
    match parse_readme_frontmatter(&content) {
        Some(meta) => {
            let mut violations = Vec::new();
            if meta.name.trim().is_empty() {
                violations.push(ExampleReadmeViolation {
                    readme: readme.to_path_buf(),
                    message: "frontmatter name must be non-empty".into(),
                });
            }
            if meta.description.trim().is_empty() {
                violations.push(ExampleReadmeViolation {
                    readme: readme.to_path_buf(),
                    message: "frontmatter description must be non-empty".into(),
                });
            }
            violations
        }
        None => vec![ExampleReadmeViolation {
            readme: readme.to_path_buf(),
            message: "README must begin with YAML frontmatter containing name and description"
                .into(),
        }],
    }
}

fn record_from_readme(readme: &Path, share_root: &Path) -> Option<ExampleRecord> {
    let content = std::fs::read_to_string(readme).ok()?;
    let meta = parse_readme_frontmatter(&content)?;
    Some(ExampleRecord {
        name: meta.name,
        description: meta.description,
        readme: relativize_share_path(readme, share_root),
    })
}

fn relativize_share_path(path: &Path, share_root: &Path) -> String {
    path.strip_prefix(share_root)
        .map(|rel| rel.to_string_lossy().replace('\\', "/"))
        .unwrap_or_else(|_| path.to_string_lossy().replace('\\', "/"))
}

/// Format example records as human-readable text blocks.
#[must_use]
pub fn format_examples_list(records: &[ExampleRecord]) -> String {
    if records.is_empty() {
        return String::new();
    }
    records
        .iter()
        .map(format_example_record)
        .collect::<Vec<_>>()
        .join("\n")
}

fn format_example_record(record: &ExampleRecord) -> String {
    format!(
        "{}\n{}\nREADME: {}",
        record.name, record.description, record.readme
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn discover_examples_finds_readme_frontmatter_in_child_dirs() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let share = tmp.path();
        let example = share.join("examples/path-picker");
        fs::create_dir_all(&example).expect("mkdir");
        fs::write(
            example.join("README.md"),
            "---\nname: Path picker\ndescription: Native pickers.\n---\n",
        )
        .expect("write");
        let records = discover_examples(share).expect("discover");
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].name, "Path picker");
        assert_eq!(records[0].description, "Native pickers.");
        assert_eq!(records[0].readme, "examples/path-picker/README.md");
    }

    #[test]
    fn discover_examples_includes_base_readme() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let share = tmp.path();
        fs::create_dir_all(share.join("examples/group-a")).expect("mkdir");
        fs::write(
            share.join("examples/README.md"),
            "---\nname: Group\ndescription: Shared docs.\n---\n",
        )
        .expect("write");
        let records = discover_examples(share).expect("discover");
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].readme, "examples/README.md");
    }

    #[test]
    fn validate_example_folder_readmes_requires_readme_per_dir() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let share = tmp.path();
        fs::create_dir_all(share.join("examples/ok")).expect("mkdir");
        fs::create_dir_all(share.join("examples/missing")).expect("mkdir");
        fs::write(
            share.join("examples/ok/README.md"),
            "---\nname: OK\ndescription: Valid.\n---\n",
        )
        .expect("write");

        let violations = validate_example_folder_readmes(share).expect("audit");
        assert_eq!(violations.len(), 1);
        assert_eq!(
            relativize_share_path(&violations[0].readme, share),
            "examples/missing/README.md"
        );
    }

    #[test]
    fn validate_example_folder_readmes_rejects_invalid_frontmatter() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let share = tmp.path();
        fs::create_dir_all(share.join("examples/bad")).expect("mkdir");
        fs::write(share.join("examples/bad/README.md"), "# no frontmatter\n").expect("write");

        let violations = validate_example_folder_readmes(share).expect("audit");
        assert_eq!(violations.len(), 1);
        assert!(
            violations[0].message.contains("frontmatter"),
            "{violations:?}"
        );
    }
}
