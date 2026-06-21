use std::path::Path;

pub(crate) fn normalize_path_for_compare(path: &Path) -> String {
    let mut text = path.to_string_lossy().replace('\\', "/");
    if let Some(stripped) = text.strip_prefix("//?/") {
        text = stripped.to_string();
    }
    if let Some(stripped) = text.strip_prefix("//./") {
        text = stripped.to_string();
    }
    if cfg!(windows) {
        text.to_ascii_lowercase()
    } else {
        text
    }
}

pub(crate) fn is_inside_path(path: &Path, root: &Path) -> bool {
    let path = normalize_path_for_compare(path);
    let root = normalize_path_for_compare(root);
    path == root || path.starts_with(&format!("{root}/"))
}

pub(crate) fn has_windows_drive_prefix(value: &str) -> bool {
    value.len() >= 3 && value.as_bytes()[1] == b':' && matches!(value.as_bytes()[2], b'\\' | b'/')
}
