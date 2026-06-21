use anyhow::{bail, Context, Result};
use lumin_rust_common::sha256_text;
use std::ffi::OsStr;
use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::protocol::CargoTargetDirMode;

const ISOLATED_TARGET_DIR_PREFIX: &str = "lumin-rust-cargo-oracle-target";
const REUSABLE_TARGET_DIR_PREFIX: &str = "lumin-rust-cargo-oracle-reusable-target";
const STALE_ISOLATED_TARGET_DIR_MAX_AGE: Duration = Duration::from_secs(24 * 60 * 60);
const STALE_REUSABLE_TARGET_DIR_MAX_AGE: Duration = Duration::from_secs(7 * 24 * 60 * 60);
static TARGET_DIR_COUNTER: AtomicU64 = AtomicU64::new(0);

pub(crate) struct CargoTargetDir {
    path: PathBuf,
    remove_on_drop: bool,
}

impl CargoTargetDir {
    pub(crate) fn create(
        mode: CargoTargetDirMode,
        root: &Path,
        cargo_bin: &str,
        rustc_bin: &str,
    ) -> Result<Self> {
        match mode {
            CargoTargetDirMode::IsolatedTemp => Self::create_isolated(),
            CargoTargetDirMode::ReusableTemp => Self::create_reusable(root, cargo_bin, rustc_bin),
        }
    }

    fn create_isolated() -> Result<Self> {
        let temp_dir = std::env::temp_dir();
        cleanup_stale_owned_target_dirs(&temp_dir, SystemTime::now());
        let process_id = std::process::id();
        let started_nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_nanos())
            .unwrap_or(0);

        for _ in 0..64 {
            let sequence = TARGET_DIR_COUNTER.fetch_add(1, Ordering::Relaxed);
            let path = temp_dir.join(format!(
                "{ISOLATED_TARGET_DIR_PREFIX}-{process_id}-{started_nanos}-{sequence}"
            ));
            match fs::create_dir(&path) {
                Ok(()) => {
                    return Ok(Self {
                        path,
                        remove_on_drop: true,
                    })
                }
                Err(error) if error.kind() == ErrorKind::AlreadyExists => continue,
                Err(error) => {
                    return Err(error)
                        .with_context(|| format!("failed to create {}", path.display()));
                }
            }
        }

        bail!(
            "failed to create unique cargo target directory in {}",
            temp_dir.display()
        )
    }

    fn create_reusable(root: &Path, cargo_bin: &str, rustc_bin: &str) -> Result<Self> {
        let temp_dir = std::env::temp_dir();
        cleanup_stale_owned_target_dirs(&temp_dir, SystemTime::now());
        let path = temp_dir.join(reusable_target_dir_name(root, cargo_bin, rustc_bin));
        fs::create_dir_all(&path)
            .with_context(|| format!("failed to create {}", path.display()))?;
        Ok(Self {
            path,
            remove_on_drop: false,
        })
    }

    pub(crate) fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for CargoTargetDir {
    fn drop(&mut self) {
        if self.remove_on_drop && is_owned_temp_target_dir(&self.path) {
            let _ = fs::remove_dir_all(&self.path);
        }
    }
}

fn cleanup_stale_owned_target_dirs(temp_dir: &Path, now: SystemTime) {
    cleanup_stale_target_dirs(
        temp_dir,
        ISOLATED_TARGET_DIR_PREFIX,
        now,
        STALE_ISOLATED_TARGET_DIR_MAX_AGE,
    );
    cleanup_stale_target_dirs(
        temp_dir,
        REUSABLE_TARGET_DIR_PREFIX,
        now,
        STALE_REUSABLE_TARGET_DIR_MAX_AGE,
    );
}

fn is_owned_temp_target_dir(path: &Path) -> bool {
    path.parent()
        .is_some_and(|parent| parent == std::env::temp_dir())
        && path
            .file_name()
            .and_then(OsStr::to_str)
            .is_some_and(|name| name.starts_with(ISOLATED_TARGET_DIR_PREFIX))
}

fn cleanup_stale_target_dirs(temp_dir: &Path, prefix: &str, now: SystemTime, max_age: Duration) {
    let Ok(entries) = fs::read_dir(temp_dir) else {
        return;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if !is_owned_target_dir_name(&path, prefix) || !path.is_dir() {
            continue;
        }
        if !is_stale_target_dir(&path, now, max_age) {
            continue;
        }
        let _ = fs::remove_dir_all(path);
    }
}

fn is_owned_target_dir_name(path: &Path, prefix: &str) -> bool {
    path.file_name()
        .and_then(OsStr::to_str)
        .is_some_and(|name| name.starts_with(&format!("{prefix}-")))
}

fn is_stale_target_dir(path: &Path, now: SystemTime, max_age: Duration) -> bool {
    fs::metadata(path)
        .and_then(|metadata| metadata.modified())
        .ok()
        .and_then(|modified| now.duration_since(modified).ok())
        .is_some_and(|age| age >= max_age)
}

fn reusable_target_dir_name(root: &Path, cargo_bin: &str, rustc_bin: &str) -> String {
    let key = format!(
        "root={}\ncargoBin={cargo_bin}\nrustcBin={rustc_bin}\n",
        root.display()
    );
    let hash = sha256_text(&key);
    let suffix = hash.strip_prefix("sha256:").unwrap_or(&hash);
    format!("{REUSABLE_TARGET_DIR_PREFIX}-{}", &suffix[..16])
}

#[cfg(test)]
mod tests {
    use super::{
        cleanup_stale_owned_target_dirs, reusable_target_dir_name, ISOLATED_TARGET_DIR_PREFIX,
        REUSABLE_TARGET_DIR_PREFIX, STALE_REUSABLE_TARGET_DIR_MAX_AGE,
    };
    use anyhow::Result;
    use std::fs;
    use std::path::Path;
    use std::time::{Duration, SystemTime};
    use tempfile::TempDir;

    #[test]
    fn stale_target_cleanup_only_removes_owned_temp_target_dirs() -> Result<()> {
        let temp = TempDir::new()?;
        let old_owned = temp
            .path()
            .join(format!("{ISOLATED_TARGET_DIR_PREFIX}-old"));
        let old_reusable = temp
            .path()
            .join(format!("{REUSABLE_TARGET_DIR_PREFIX}-old"));
        let other = temp.path().join("other-tool-target-old");
        fs::create_dir(&old_owned)?;
        fs::create_dir(&old_reusable)?;
        fs::create_dir(&other)?;

        let future = SystemTime::now() + STALE_REUSABLE_TARGET_DIR_MAX_AGE + Duration::from_secs(1);
        cleanup_stale_owned_target_dirs(temp.path(), future);

        assert!(!old_owned.exists());
        assert!(!old_reusable.exists());
        assert!(other.exists());
        Ok(())
    }

    #[test]
    fn stale_target_cleanup_keeps_owned_recent_temp_target_dirs() -> Result<()> {
        let temp = TempDir::new()?;
        let recent_owned = temp
            .path()
            .join(format!("{ISOLATED_TARGET_DIR_PREFIX}-recent"));
        let recent_reusable = temp
            .path()
            .join(format!("{REUSABLE_TARGET_DIR_PREFIX}-recent"));
        fs::create_dir(&recent_owned)?;
        fs::create_dir(&recent_reusable)?;

        cleanup_stale_owned_target_dirs(temp.path(), SystemTime::now());

        assert!(recent_owned.exists());
        assert!(recent_reusable.exists());
        Ok(())
    }

    #[test]
    fn reusable_target_dir_name_is_stable_and_owned() {
        let name = reusable_target_dir_name(Path::new("repo"), "cargo", "rustc");

        assert!(name.starts_with(&format!("{REUSABLE_TARGET_DIR_PREFIX}-")));
        assert_eq!(
            name,
            reusable_target_dir_name(Path::new("repo"), "cargo", "rustc")
        );
        assert_ne!(
            name,
            reusable_target_dir_name(Path::new("other-repo"), "cargo", "rustc")
        );
    }
}
