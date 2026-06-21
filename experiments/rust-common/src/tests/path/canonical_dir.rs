use crate::canonical_existing_dir;
use std::io::{self, ErrorKind};

#[test]
fn canonical_existing_dir_accepts_current_directory() -> io::Result<()> {
    let cwd = std::env::current_dir()?;
    let canonical = canonical_existing_dir(&cwd)?;

    assert!(canonical.is_absolute());
    assert!(canonical.is_dir());
    Ok(())
}

#[test]
fn canonical_existing_dir_rejects_files() -> io::Result<()> {
    let executable = std::env::current_exe()?;
    let error = match canonical_existing_dir(&executable) {
        Ok(path) => panic!("file should be rejected: {}", path.display()),
        Err(error) => error,
    };

    assert_eq!(error.kind(), ErrorKind::InvalidInput);
    Ok(())
}
