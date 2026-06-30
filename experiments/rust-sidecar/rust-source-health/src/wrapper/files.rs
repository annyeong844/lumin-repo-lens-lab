use std::fs;
use std::path::{Component, Path, PathBuf};

use anyhow::{bail, Context, Result};
use lumin_rust_common::{posix_path_has_segment, sha256_bytes};

use crate::analyzer::SourceFileEntry;
use crate::protocol::{SkippedFile, SkippedFileReason};
use crate::usage_error;

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
) -> Result<(Vec<SourceFileEntry>, Vec<SkippedFile>)> {
    let mut files = Vec::new();
    let mut skipped = Vec::new();
    collect_rust_file_entries_inner(root, root, &mut files, &mut skipped)?;
    files.sort_by(|left, right| left.path.cmp(&right.path));
    skipped.sort_by(|left, right| left.path.cmp(&right.path));
    Ok((files, skipped))
}

fn collect_rust_file_entries_inner(
    root: &Path,
    dir: &Path,
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
        if is_excluded_by_path_policy(&relative) {
            continue;
        }
        if file_type.is_dir() {
            collect_rust_file_entries_inner(root, &absolute, files, skipped)?;
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
