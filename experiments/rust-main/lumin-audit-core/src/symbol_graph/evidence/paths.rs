use crate::symbol_graph::normalize_slashes;

pub(in crate::symbol_graph) fn rel_path(root: &str, file: &str) -> String {
    let root = normalize_slashes(root).trim_end_matches('/').to_string();
    let file = normalize_slashes(file);
    let prefix = format!("{root}/");
    if let Some(stripped) = file.strip_prefix(&prefix) {
        stripped.to_string()
    } else {
        file
    }
}

pub(super) fn resolve_prefix_target(file: &str, prefix: &str) -> String {
    let normalized_file = normalize_slashes(file);
    let base = normalized_file
        .rsplit_once('/')
        .map(|(base, _)| base)
        .unwrap_or("");
    normalize_path_segments(&format!("{base}/{prefix}"))
}

pub(in crate::symbol_graph) fn normalize_path_segments(path: &str) -> String {
    let mut prefix = String::new();
    let mut rest = normalize_slashes(path);
    if rest.len() >= 3 && rest.as_bytes()[1] == b':' && rest.as_bytes()[2] == b'/' {
        prefix = rest[..3].to_string();
        rest = rest[3..].to_string();
    } else if rest.starts_with('/') {
        prefix = "/".to_string();
        rest = rest.trim_start_matches('/').to_string();
    }

    let mut parts = Vec::new();
    for part in rest.split('/') {
        match part {
            "" | "." => {}
            ".." => {
                parts.pop();
            }
            _ => parts.push(part),
        }
    }
    format!("{prefix}{}", parts.join("/"))
}

pub(in crate::symbol_graph) fn is_absolute_like_path(path: &str) -> bool {
    path.starts_with('/')
        || (path.len() >= 3 && path.as_bytes()[1] == b':' && path.as_bytes()[2] == b'/')
}
