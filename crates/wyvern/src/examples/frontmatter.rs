//! Minimal YAML frontmatter parser for example README files.

/// Parsed frontmatter fields required on example README files.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReadmeFrontmatter {
    /// Human-readable example name.
    pub name: String,
    /// Short example summary.
    pub description: String,
}

/// Parse mandatory `name` and `description` from a README frontmatter block.
#[must_use]
pub fn parse_readme_frontmatter(content: &str) -> Option<ReadmeFrontmatter> {
    let content = content.strip_prefix('\u{feff}').unwrap_or(content);
    let rest = content.strip_prefix("---")?;
    let rest = rest
        .strip_prefix('\n')
        .or_else(|| rest.strip_prefix("\r\n"))?;
    let end = rest.find("\n---")?;
    let block = &rest[..end];
    let mut name = None;
    let mut description = None;
    for line in block.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let (key, value) = line.split_once(':')?;
        let key = key.trim().to_ascii_lowercase();
        let value = parse_scalar(value.trim());
        if value.is_empty() {
            continue;
        }
        match key.as_str() {
            "name" => name = Some(value),
            "description" => description = Some(value),
            _ => {}
        }
    }
    Some(ReadmeFrontmatter {
        name: name?,
        description: description?,
    })
}

fn parse_scalar(raw: &str) -> String {
    if (raw.starts_with('"') && raw.ends_with('"'))
        || (raw.starts_with('\'') && raw.ends_with('\''))
    {
        raw[1..raw.len().saturating_sub(1)].trim().to_string()
    } else {
        raw.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_required_frontmatter_fields() {
        let meta = parse_readme_frontmatter(
            "---\nname: Path picker\ndescription: Native file and folder pickers.\n---\n# Title\n",
        )
        .expect("frontmatter");
        assert_eq!(meta.name, "Path picker");
        assert_eq!(meta.description, "Native file and folder pickers.");
    }

    #[test]
    fn rejects_missing_description() {
        assert!(parse_readme_frontmatter("---\nname: Only name\n---\n").is_none());
    }

    #[test]
    fn rejects_missing_frontmatter() {
        assert!(parse_readme_frontmatter("# No frontmatter\n").is_none());
    }
}
