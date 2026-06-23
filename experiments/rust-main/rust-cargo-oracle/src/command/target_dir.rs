use anyhow::{bail, Context, Result};
use lumin_rust_common::sha256_text;
use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::protocol::CargoTargetDirMode;

mod cleanup;

use cleanup::{cleanup_stale_owned_target_dirs, is_owned_isolated_temp_target_dir};

const ISOLATED_TARGET_DIR_PREFIX: &str = "lumin-rust-cargo-oracle-target";
const REUSABLE_TARGET_DIR_PREFIX: &str = "lumin-rust-cargo-oracle-reusable-target";
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
        if self.remove_on_drop && is_owned_isolated_temp_target_dir(&self.path) {
            let _ = fs::remove_dir_all(&self.path);
        }
    }
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
    use super::{reusable_target_dir_name, REUSABLE_TARGET_DIR_PREFIX};
    use std::path::Path;

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
