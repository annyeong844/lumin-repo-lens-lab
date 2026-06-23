use std::ffi::OsStr;
use std::fs;
use std::path::Path;
use std::time::{Duration, SystemTime};

use crate::protocol::CargoTargetDirPolicy;

use super::{ISOLATED_TARGET_DIR_PREFIX, REUSABLE_TARGET_DIR_PREFIX};

pub(super) fn cleanup_stale_owned_target_dirs(temp_dir: &Path, now: SystemTime) {
    cleanup_stale_target_dirs(
        temp_dir,
        ISOLATED_TARGET_DIR_PREFIX,
        now,
        stale_isolated_target_dir_max_age(),
    );
    cleanup_stale_target_dirs(
        temp_dir,
        REUSABLE_TARGET_DIR_PREFIX,
        now,
        stale_reusable_target_dir_max_age(),
    );
}

pub(super) fn is_owned_isolated_temp_target_dir(path: &Path) -> bool {
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

fn stale_isolated_target_dir_max_age() -> Duration {
    Duration::from_secs(CargoTargetDirPolicy::STALE_ISOLATED_TARGET_DIR_MAX_AGE_SECONDS)
}

fn stale_reusable_target_dir_max_age() -> Duration {
    Duration::from_secs(CargoTargetDirPolicy::STALE_REUSABLE_TARGET_DIR_MAX_AGE_SECONDS)
}

#[cfg(test)]
mod tests {
    use super::{
        cleanup_stale_owned_target_dirs, stale_reusable_target_dir_max_age,
        ISOLATED_TARGET_DIR_PREFIX, REUSABLE_TARGET_DIR_PREFIX,
    };
    use anyhow::Result;
    use std::fs;
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

        let future =
            SystemTime::now() + stale_reusable_target_dir_max_age() + Duration::from_secs(1);
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
}
