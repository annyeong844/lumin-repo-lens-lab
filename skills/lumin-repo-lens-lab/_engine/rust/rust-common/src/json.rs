use std::fs;
use std::io::{self, BufWriter, ErrorKind, Write};
use std::path::Path;

pub fn atomic_write_json_pretty<T: serde::Serialize + ?Sized>(
    path: &Path,
    value: &T,
) -> io::Result<()> {
    let (temp, mut writer) = temp_json_writer(path)?;
    serde_json::to_writer_pretty(&mut writer, value)
        .map_err(|error| io::Error::new(ErrorKind::InvalidData, error))?;
    finish_atomic_json_write(path, &temp, writer)
}

pub fn atomic_write_json<T: serde::Serialize + ?Sized>(path: &Path, value: &T) -> io::Result<()> {
    let (temp, mut writer) = temp_json_writer(path)?;
    serde_json::to_writer(&mut writer, value)
        .map_err(|error| io::Error::new(ErrorKind::InvalidData, error))?;
    finish_atomic_json_write(path, &temp, writer)
}

fn temp_json_writer(path: &Path) -> io::Result<(std::path::PathBuf, BufWriter<fs::File>)> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let temp = path.with_extension("tmp");
    let file = fs::File::create(&temp)?;
    Ok((temp, BufWriter::new(file)))
}

fn finish_atomic_json_write(
    path: &Path,
    temp: &Path,
    mut writer: BufWriter<fs::File>,
) -> io::Result<()> {
    writer.write_all(b"\n")?;
    writer.flush()?;
    #[cfg(windows)]
    if path.exists() {
        fs::remove_file(path)?;
    }
    fs::rename(temp, path)?;
    Ok(())
}
