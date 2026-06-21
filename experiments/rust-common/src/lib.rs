use std::borrow::Cow;
use std::error::Error;
use std::ffi::OsStr;
use std::fmt;
use std::io;
use std::path::{Path, PathBuf};

#[cfg(any(feature = "hash", feature = "json"))]
use std::fs;

#[cfg(feature = "json")]
use std::io::ErrorKind;

#[cfg(feature = "hash")]
use std::io::Read;

#[cfg(feature = "hash")]
use sha2::{Digest, Sha256};

mod cli;

pub use cli::{
    parse_enum, parse_min_usize, parse_nonzero_usize, parse_u64, take_path, take_string, CliAction,
    CliResult,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UsageError {
    message: String,
}

impl UsageError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for UsageError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.message.fmt(formatter)
    }
}

impl Error for UsageError {}

pub fn usage_error(message: impl Into<String>) -> anyhow::Error {
    UsageError::new(message).into()
}

pub fn is_usage_error(error: &anyhow::Error) -> bool {
    error.downcast_ref::<UsageError>().is_some()
        || error
            .chain()
            .any(|cause| cause.downcast_ref::<UsageError>().is_some())
}

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
        .map_err(|error| usage_error(format!("invalid {label} {}: {error}", path.display())))
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

#[cfg(feature = "json")]
pub fn atomic_write_json_pretty<T: serde::Serialize + ?Sized>(
    path: &Path,
    value: &T,
) -> io::Result<()> {
    let bytes = serde_json::to_vec_pretty(value)
        .map_err(|error| io::Error::new(ErrorKind::InvalidData, error))?;
    atomic_write_bytes_with_newline(path, bytes)
}

#[cfg(feature = "json")]
pub fn atomic_write_json<T: serde::Serialize + ?Sized>(path: &Path, value: &T) -> io::Result<()> {
    let bytes =
        serde_json::to_vec(value).map_err(|error| io::Error::new(ErrorKind::InvalidData, error))?;
    atomic_write_bytes_with_newline(path, bytes)
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

#[cfg(feature = "json")]
fn atomic_write_bytes_with_newline(path: &Path, mut bytes: Vec<u8>) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let temp = path.with_extension("tmp");
    bytes.push(b'\n');
    fs::write(&temp, bytes)?;
    fs::rename(&temp, path)?;
    Ok(())
}

#[cfg(feature = "hash")]
pub fn sha256_bytes(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("sha256:{:x}", hasher.finalize())
}

#[cfg(feature = "hash")]
pub fn sha256_text(text: &str) -> String {
    sha256_bytes(text.as_bytes())
}

#[cfg(feature = "hash")]
pub fn sha256_file(path: &Path) -> io::Result<String> {
    let mut file = fs::File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buf = [0; 8192];
    loop {
        let n = file.read(&mut buf)?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    Ok(format!("sha256:{:x}", hasher.finalize()))
}

#[cfg(test)]
mod tests;
