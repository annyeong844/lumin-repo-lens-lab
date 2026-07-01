use std::fs;
use std::path::{Component, Path, PathBuf};

use anyhow::{bail, Context, Result};
use lumin_rust_common::{posix_path_has_segment, sha256_bytes};

use crate::analyzer::SourceFileEntry;
use crate::path_policy::is_test_like_rust_path;
use crate::protocol::{SkippedFile, SkippedFileReason};
use crate::usage_error;

#[derive(Debug, Clone)]
pub struct RustFileScanScope {
    include_tests: bool,
    exclude_rules: Vec<ExcludeRule>,
    exclude_patterns: Vec<String>,
}

#[derive(Debug, Clone)]
enum ExcludeRule {
    Directory { pattern: String, needle: String },
    File { pattern: String },
}

impl RustFileScanScope {
    pub fn new(include_tests: bool, exclude: &[String]) -> Self {
        let exclude_patterns = exclude
            .iter()
            .filter_map(|pattern| normalize_exclude_pattern(pattern))
            .collect::<Vec<_>>();
        let exclude_rules = exclude_patterns
            .iter()
            .map(|pattern| {
                let last_segment = pattern.rsplit('/').next().unwrap_or(pattern);
                if last_segment.contains('.') {
                    ExcludeRule::File {
                        pattern: pattern.clone(),
                    }
                } else {
                    ExcludeRule::Directory {
                        pattern: pattern.clone(),
                        needle: format!("/{pattern}/"),
                    }
                }
            })
            .collect();
        Self {
            include_tests,
            exclude_rules,
            exclude_patterns,
        }
    }

    pub fn include_tests(&self) -> bool {
        self.include_tests
    }

    pub fn exclude_patterns(&self) -> &[String] {
        &self.exclude_patterns
    }

    pub fn excludes_path(&self, path: &str, directory: bool) -> bool {
        self.exclusion_pattern_for_path(path, directory).is_some()
    }

    pub fn exclusion_pattern_for_path(&self, path: &str, directory: bool) -> Option<&str> {
        let normalized = if directory {
            format!("/{path}/")
        } else {
            format!("/{path}")
        };
        self.exclude_rules.iter().find_map(|rule| match rule {
            ExcludeRule::Directory { pattern, needle } => {
                normalized.contains(needle).then_some(pattern.as_str())
            }
            ExcludeRule::File { pattern } => (!directory
                && normalized.ends_with(&format!("/{pattern}")))
            .then_some(pattern.as_str()),
        })
    }
}

pub(super) fn absolute_existing_dir(path: &Path) -> Result<PathBuf> {
    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()?.join(path)
    };
    let metadata = fs::metadata(&absolute).map_err(|_| {
        usage_error(format!(
            "rust source health root not found: {}",
            absolute.display()
        ))
    })?;
    if !metadata.is_dir() {
        return Err(usage_error(format!(
            "rust source health root is not a directory: {}",
            absolute.display()
        )));
    }
    Ok(absolute)
}

pub(super) fn collect_rust_file_entries(
    root: &Path,
    scan_scope: &RustFileScanScope,
) -> Result<(Vec<SourceFileEntry>, Vec<SkippedFile>)> {
    let mut files = Vec::new();
    let mut skipped = Vec::new();
    collect_rust_file_entries_inner(root, root, scan_scope, &mut files, &mut skipped)?;
    files.sort_by(|left, right| left.path.cmp(&right.path));
    skipped.sort_by(|left, right| left.path.cmp(&right.path));
    Ok((files, skipped))
}

fn collect_rust_file_entries_inner(
    root: &Path,
    dir: &Path,
    scan_scope: &RustFileScanScope,
    files: &mut Vec<SourceFileEntry>,
    skipped: &mut Vec<SkippedFile>,
) -> Result<()> {
    let mut entries = fs::read_dir(dir)
        .with_context(|| format!("failed to read directory {}", dir.display()))?
        .collect::<std::result::Result<Vec<_>, _>>()?;
    entries.sort_by_key(std::fs::DirEntry::file_name);

    for entry in entries {
        let absolute = entry.path();
        let relative = relative_posix(root, &absolute)?;
        assert_safe_relative_path(&relative)?;
        let file_type = entry.file_type()?;
        if file_type.is_symlink() {
            continue;
        }
        if should_skip_path(&relative, file_type.is_dir(), scan_scope) {
            continue;
        }
        if file_type.is_dir() {
            collect_rust_file_entries_inner(root, &absolute, scan_scope, files, skipped)?;
            continue;
        }
        if !file_type.is_file() || !relative.ends_with(".rs") {
            continue;
        }

        let raw = fs::read(&absolute)
            .with_context(|| format!("failed to read Rust source {}", absolute.display()))?;
        let sha256 = sha256_bytes(&raw);
        if std::str::from_utf8(&raw).is_err() {
            skipped.push(SkippedFile {
                path: relative,
                reason: SkippedFileReason::InvalidUtf8,
            });
            continue;
        }
        files.push(SourceFileEntry {
            path: relative,
            absolute_path: absolute,
            sha256,
        });
    }
    Ok(())
}

fn relative_posix(root: &Path, path: &Path) -> Result<String> {
    let relative = path
        .strip_prefix(root)
        .with_context(|| format!("failed to relativize {}", path.display()))?;
    let mut parts = Vec::new();
    for component in relative.components() {
        match component {
            Component::Normal(value) => parts.push(value.to_string_lossy().to_string()),
            _ => bail!("unsafe rust source health path: {}", relative.display()),
        }
    }
    Ok(parts.join("/"))
}

fn assert_safe_relative_path(path: &str) -> Result<()> {
    if path.is_empty()
        || path.starts_with('/')
        || path.starts_with('\\')
        || path.contains('\\')
        || path.contains(':')
        || path
            .split('/')
            .any(|segment| segment.is_empty() || segment == "." || segment == "..")
    {
        bail!("unsafe rust source health path: {path}");
    }
    Ok(())
}

fn is_excluded_by_path_policy(path: &str) -> bool {
    posix_path_has_segment(path, "target") || posix_path_has_segment(path, "vendor")
}

fn should_skip_path(path: &str, directory: bool, scan_scope: &RustFileScanScope) -> bool {
    is_excluded_by_path_policy(path)
        || scan_scope.excludes_path(path, directory)
        || (!scan_scope.include_tests() && is_test_like_rust_path(path))
}

fn normalize_exclude_pattern(pattern: &str) -> Option<String> {
    let mut normalized = pattern.trim().replace('\\', "/");
    if let Some(stripped) = normalized.strip_prefix("*/") {
        normalized = stripped.to_string();
    }
    if let Some(stripped) = normalized.strip_suffix("/*") {
        normalized = stripped.to_string();
    }
    if let Some(stripped) = normalized.strip_prefix("./") {
        normalized = stripped.to_string();
    }
    normalized = normalized
        .trim_start_matches('/')
        .trim_end_matches('/')
        .to_string();
    (!normalized.is_empty()).then_some(normalized)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scan_scope_excludes_test_like_rust_paths_without_substring_matches() {
        let scope = RustFileScanScope::new(false, &[]);

        assert!(should_skip_path("tests/support.rs", false, &scope));
        assert!(should_skip_path("src/foo_test.rs", false, &scope));
        assert!(should_skip_path("examples/demo.rs", false, &scope));
        assert!(!should_skip_path("src/contest.rs", false, &scope));
    }

    #[test]
    fn scan_scope_applies_directory_and_file_excludes_like_js_scan_scope() {
        let scope = RustFileScanScope::new(
            true,
            &[
                "generated".to_string(),
                "./src/skip_me.rs".to_string(),
                "crates/*".to_string(),
            ],
        );

        assert!(should_skip_path("src/generated/model.rs", false, &scope));
        assert!(should_skip_path("src/skip_me.rs", false, &scope));
        assert!(should_skip_path("crates/foo.rs", false, &scope));
        assert!(!should_skip_path("src/generated_name.rs", false, &scope));
        assert_eq!(
            scope.exclude_patterns(),
            &["generated", "src/skip_me.rs", "crates"]
        );
    }
}
