use crate::relative_source_resolver::normalize_path_text;

pub(super) fn root_relative(root: &str, path: &str) -> String {
    let normalized = normalize_path_text(path);
    let trimmed_root = root.trim_end_matches('/');
    if let Some(rest) = normalized.strip_prefix(&format!("{trimmed_root}/")) {
        return rest.to_string();
    }
    if normalized == trimmed_root {
        return ".".to_string();
    }
    normalized
}

pub(super) fn basename_text(path: &str) -> Option<String> {
    normalize_path_text(path)
        .rsplit('/')
        .next()
        .map(ToString::to_string)
        .filter(|value| !value.is_empty())
}

pub(super) fn is_inside_or_same(parent: &str, child: &str) -> bool {
    let parent = normalize_path_text(parent);
    let child = normalize_path_text(child);
    let parent = parent.trim_end_matches('/');
    child == parent || child.starts_with(&format!("{parent}/"))
}

pub(super) fn relative_scope(root: &str, path: &str) -> String {
    let root = normalize_path_text(root);
    let path = normalize_path_text(path);
    let root = root.trim_end_matches('/');
    if path == root {
        String::new()
    } else {
        path.strip_prefix(&format!("{root}/"))
            .map(ToString::to_string)
            .unwrap_or(path)
    }
}
