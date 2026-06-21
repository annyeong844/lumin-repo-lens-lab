use anyhow::Result;
use lumin_rust_common::sha256_text;

use crate::protocol::{HealthRequest, DEFAULT_EXCLUDE, DEFAULT_INCLUDE, SCHEMA_VERSION};
use crate::usage_error;

pub(super) fn validate_request(request: &HealthRequest) -> Result<()> {
    if request.schema_version != SCHEMA_VERSION {
        return Err(usage_error(format!(
            "unsupported schemaVersion {schema_version}",
            schema_version = request.schema_version
        )));
    }
    validate_root(&request.root)?;
    if request.path_policy.include != DEFAULT_INCLUDE {
        return Err(usage_error("unsupported pathPolicy.include"));
    }
    if request.path_policy.exclude != DEFAULT_EXCLUDE {
        return Err(usage_error("unsupported pathPolicy.exclude"));
    }
    let mut seen_paths = std::collections::BTreeSet::<&str>::new();
    for file in &request.files {
        validate_file_path(&file.path)?;
        if !seen_paths.insert(file.path.as_str()) {
            return Err(usage_error(format!(
                "duplicate file path: {path}",
                path = file.path
            )));
        }
        validate_sha256(&file.sha256, &file.path)?;
        validate_text_sha256(&file.text, &file.sha256, &file.path)?;
    }
    Ok(())
}

fn validate_root(root: &str) -> Result<()> {
    if root.trim().is_empty() {
        return Err(usage_error("root is required"));
    }
    if !is_absoluteish_root(root) {
        return Err(usage_error("root must be absolute"));
    }
    Ok(())
}

fn is_absoluteish_root(root: &str) -> bool {
    root.starts_with('/')
        || (root.len() >= 3
            && root.as_bytes().get(1) == Some(&b':')
            && matches!(root.as_bytes().get(2), Some(b'/') | Some(b'\\')))
}

fn validate_file_path(path: &str) -> Result<()> {
    if path.is_empty() {
        return Err(usage_error("file path is required"));
    }
    if path.starts_with('/') || path.starts_with('\\') {
        return Err(usage_error(format!(
            "file path must be root-relative: {path}"
        )));
    }
    if path.contains('\\') {
        return Err(usage_error(format!(
            "file path must use POSIX slash separators: {path}"
        )));
    }
    if path.contains(':') {
        return Err(usage_error(format!(
            "file path must not contain drive prefixes or colons: {path}"
        )));
    }
    if path
        .split('/')
        .any(|part| part.is_empty() || part == "." || part == "..")
    {
        return Err(usage_error(format!(
            "file path must not contain empty, ., or .. segments: {path}"
        )));
    }
    Ok(())
}

fn validate_sha256(value: &str, path: &str) -> Result<()> {
    let hex = value
        .strip_prefix("sha256:")
        .ok_or_else(|| usage_error(format!("invalid sha256 for {path}")))?;
    if hex.len() != 64 || !hex.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(usage_error(format!("invalid sha256 for {path}")));
    }
    Ok(())
}

fn validate_text_sha256(text: &str, expected: &str, path: &str) -> Result<()> {
    let actual = sha256_text(text);
    if actual != expected {
        return Err(usage_error(format!("sha256/text mismatch for {path}")));
    }
    Ok(())
}
