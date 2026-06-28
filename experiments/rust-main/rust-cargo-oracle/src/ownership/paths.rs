use std::path::{Path, PathBuf};

use crate::path_util::{has_windows_drive_prefix, is_inside_path, normalize_path_for_compare};

pub(super) fn fallback_user_roots(root: &Path) -> Vec<PathBuf> {
    vec![root.join("src")]
}

pub(super) fn longest_matching_root_len(path: &Path, roots: &[PathBuf]) -> Option<usize> {
    roots
        .iter()
        .filter(|root| is_inside_path(path, root))
        .map(|root| normalize_path_for_compare(root).len())
        .max()
}

pub(super) fn resolve_span_path(root: &Path, file_name: &str) -> Option<PathBuf> {
    if file_name.is_empty() || file_name == "<anon>" {
        return None;
    }
    let normalized = file_name.replace('\\', std::path::MAIN_SEPARATOR_STR);
    let path = PathBuf::from(&normalized);
    if path.is_absolute() || has_windows_drive_prefix(&normalized) {
        Some(path)
    } else {
        Some(root.join(path))
    }
}
