use std::fs;
use std::io::{self, ErrorKind};
use std::path::Path;

pub fn atomic_write_json_pretty<T: serde::Serialize + ?Sized>(
    path: &Path,
    value: &T,
) -> io::Result<()> {
    let bytes = serde_json::to_vec_pretty(value)
        .map_err(|error| io::Error::new(ErrorKind::InvalidData, error))?;
    atomic_write_bytes_with_newline(path, bytes)
}

pub fn atomic_write_json<T: serde::Serialize + ?Sized>(path: &Path, value: &T) -> io::Result<()> {
    let bytes =
        serde_json::to_vec(value).map_err(|error| io::Error::new(ErrorKind::InvalidData, error))?;
    atomic_write_bytes_with_newline(path, bytes)
}

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
