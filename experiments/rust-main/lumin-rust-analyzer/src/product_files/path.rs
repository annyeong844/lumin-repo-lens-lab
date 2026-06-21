use std::path::Path;

use lumin_rust_cargo_oracle::protocol::{DiagnosticEvidence, Finding, PrimarySpan};
use lumin_rust_common::posix_path_text;

pub(super) fn typed_finding_relative_path(root: &Path, finding: &Finding) -> Option<String> {
    let file_name = finding
        .span
        .as_ref()
        .or_else(|| PrimarySpan::representative(&finding.primary_spans))
        .and_then(|span| span.file_name.as_deref())?;
    root_relative_path(root, file_name)
}

pub(super) fn diagnostic_relative_path(
    root: &Path,
    diagnostic: &DiagnosticEvidence,
) -> Option<String> {
    let file_name = PrimarySpan::representative(&diagnostic.primary_spans)
        .and_then(|span| span.file_name.as_deref())?;
    root_relative_path(root, file_name)
}

fn root_relative_path(root: &Path, file_name: &str) -> Option<String> {
    let normalized = posix_path_text(file_name);
    let trimmed = normalized.trim_start_matches("./");
    if !is_absolute_like(trimmed) {
        return Some(trimmed.to_string());
    }

    let root = root.display().to_string();
    let root = posix_path_text(&root);
    let root = root.trim_end_matches('/');
    strip_root_prefix(trimmed, root).map(str::to_string)
}

fn is_absolute_like(path: &str) -> bool {
    path.starts_with('/') || path.as_bytes().get(1).is_some_and(|byte| *byte == b':')
}

fn strip_root_prefix<'a>(path: &'a str, root: &str) -> Option<&'a str> {
    let root_len = root.len();
    let path_root = path.get(..root_len)?;
    let separator = path.as_bytes().get(root_len)?;
    if *separator != b'/' {
        return None;
    }

    let same_root = if cfg!(windows) {
        path_root.eq_ignore_ascii_case(root)
    } else {
        path_root == root
    };

    same_root.then(|| &path[root_len + 1..])
}
