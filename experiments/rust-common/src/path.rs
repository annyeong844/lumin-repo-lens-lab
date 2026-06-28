use std::borrow::Cow;
use std::ffi::OsStr;
use std::io;
use std::path::{Path, PathBuf};

pub fn canonical_existing_dir(path: &Path) -> io::Result<PathBuf> {
    let path = path.canonicalize()?;
    if !path.is_dir() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "not a directory",
        ));
    }
    Ok(path)
}

pub fn canonical_existing_dir_usage(path: &Path, label: &str) -> anyhow::Result<PathBuf> {
    canonical_existing_dir(path)
        .map_err(|error| crate::usage_error(format!("invalid {label} {}: {error}", path.display())))
}

pub fn find_repo_root(start: &Path) -> Option<PathBuf> {
    let mut cursor = if start.is_file() {
        start.parent()?.to_path_buf()
    } else {
        start.to_path_buf()
    };
    loop {
        if cursor
            .join("canonical")
            .join("oracle-registry.json")
            .is_file()
        {
            return Some(cursor);
        }
        if !cursor.pop() {
            return None;
        }
    }
}

pub fn find_repo_root_with_fallback(start: &Path, fallback_start: &Path) -> Option<PathBuf> {
    find_repo_root(start).or_else(|| find_repo_root(fallback_start))
}

pub fn posix_path_has_segment(path: &str, segment: &str) -> bool {
    path.split('/').any(|part| part == segment)
}

pub fn posix_path_text(path: &str) -> Cow<'_, str> {
    if path.as_bytes().contains(&b'\\') {
        Cow::Owned(path.replace('\\', "/"))
    } else {
        Cow::Borrowed(path)
    }
}

pub fn path_has_segment(path: &Path, segment: &str) -> bool {
    path.components()
        .any(|component| component.as_os_str() == OsStr::new(segment))
}
