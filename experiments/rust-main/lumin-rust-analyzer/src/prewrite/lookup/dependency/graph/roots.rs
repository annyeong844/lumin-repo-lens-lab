const LOCAL_RUST_PATH_ROOTS: &[&str] = &["Self", "crate", "self", "super", "std", "core", "alloc"];

pub(super) fn rust_path_root(path: &str) -> Option<String> {
    let normalized = path.trim_start_matches("::");
    let root = normalized.split("::").next().unwrap_or(normalized);
    if root.is_empty() || LOCAL_RUST_PATH_ROOTS.contains(&root) {
        None
    } else {
        Some(root.to_string())
    }
}
