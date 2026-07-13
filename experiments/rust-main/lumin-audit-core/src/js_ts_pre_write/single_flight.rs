use anyhow::{Context, Result};
use lumin_rust_common::sha256_text;
use serde_json::{json, Map, Number, Value};
use std::ffi::OsStr;
use std::fs::{self, File, OpenOptions};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

#[cfg(windows)]
const USER_IDENTITY_ENV: &[&str] = &["USERPROFILE", "USERNAME"];
#[cfg(not(windows))]
const USER_IDENTITY_ENV: &[&str] = &["HOME", "USER", "LOGNAME"];

pub(super) struct ScanLease {
    _file: File,
    wait_ms: u64,
    acquired_at: Instant,
}

impl ScanLease {
    pub(super) fn acquire(root: &Path) -> Result<Self> {
        let (file, lock_path) = open_scan_lock(root)?;
        let wait_started = Instant::now();
        file.lock().with_context(|| {
            format!(
                "js-ts-pre-write-evidence: failed to acquire scan lock {}",
                lock_path.display()
            )
        })?;
        Ok(Self {
            _file: file,
            wait_ms: elapsed_ms(wait_started),
            acquired_at: Instant::now(),
        })
    }

    #[cfg(test)]
    fn acquire_after_observing_contention(
        root: &Path,
        contended: &std::sync::mpsc::Sender<()>,
    ) -> Result<Self> {
        let (file, lock_path) = open_scan_lock(root)?;
        let wait_started = Instant::now();
        match file.try_lock() {
            Ok(()) => {
                anyhow::bail!("second scan acquired the root lock while the first lease was alive")
            }
            Err(std::fs::TryLockError::WouldBlock) => {}
            Err(std::fs::TryLockError::Error(error)) => {
                return Err(error).with_context(|| {
                    format!(
                        "js-ts-pre-write-evidence: failed to probe scan lock {}",
                        lock_path.display()
                    )
                });
            }
        }
        contended
            .send(())
            .context("failed to report observed scan lock contention")?;
        file.lock().with_context(|| {
            format!(
                "js-ts-pre-write-evidence: failed to acquire contended scan lock {}",
                lock_path.display()
            )
        })?;
        Ok(Self {
            _file: file,
            wait_ms: elapsed_ms(wait_started),
            acquired_at: Instant::now(),
        })
    }

    #[cfg(test)]
    pub(super) fn is_available(root: &Path) -> Result<bool> {
        let (file, lock_path) = open_scan_lock(root)?;
        match file.try_lock() {
            Ok(()) => Ok(true),
            Err(std::fs::TryLockError::WouldBlock) => Ok(false),
            Err(std::fs::TryLockError::Error(error)) => Err(error).with_context(|| {
                format!(
                    "js-ts-pre-write-evidence: failed to probe scan lock availability {}",
                    lock_path.display()
                )
            }),
        }
    }

    pub(super) fn wait_ms(&self) -> u64 {
        self.wait_ms
    }

    pub(super) fn held_ms(&self) -> u64 {
        elapsed_ms(self.acquired_at)
    }
}

fn open_scan_lock(root: &Path) -> Result<(File, PathBuf)> {
    let canonical = fs::canonicalize(root).with_context(|| {
        format!(
            "js-ts-pre-write-evidence: failed to canonicalize root {}",
            root.display()
        )
    })?;
    let root_key = sha256_text(&format!(
        "js-ts-pre-write-single-flight.v1\0{}",
        canonical.to_string_lossy().replace('\\', "/")
    ));
    let lock_dir = private_lock_root()?.join("js-ts-pre-write-locks");
    fs::create_dir_all(&lock_dir).with_context(|| {
        format!(
            "js-ts-pre-write-evidence: failed to create lock directory {}",
            lock_dir.display()
        )
    })?;
    restrict_directory_to_current_user(&lock_dir)?;
    let lock_path = lock_dir.join(format!("{}.lock", root_key.trim_start_matches("sha256:")));
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(&lock_path)
        .with_context(|| {
            format!(
                "js-ts-pre-write-evidence: failed to open scan lock {}",
                lock_path.display()
            )
        })?;
    Ok((file, lock_path))
}

fn private_lock_root() -> Result<PathBuf> {
    let (identity_kind, identity) = USER_IDENTITY_ENV
        .iter()
        .find_map(|name| {
            std::env::var_os(name)
                .filter(|value| !value.is_empty())
                .map(|value| (*name, value))
        })
        .context("js-ts-pre-write-evidence: current user identity is unavailable")?;
    let root = private_lock_root_for(identity_kind, &identity);
    fs::create_dir_all(&root).with_context(|| {
        format!(
            "js-ts-pre-write-evidence: failed to create private lock root {}",
            root.display()
        )
    })?;
    restrict_directory_to_current_user(&root)?;
    Ok(root)
}

fn private_lock_root_for(identity_kind: &str, identity: &OsStr) -> PathBuf {
    let user_key = sha256_text(&format!(
        "js-ts-pre-write-lock-user.v1\0{identity_kind}\0{}",
        identity.to_string_lossy()
    ));
    std::env::temp_dir().join(format!(
        "lumin-audit-core-user-{}",
        user_key.trim_start_matches("sha256:")
    ))
}

#[cfg(unix)]
fn restrict_directory_to_current_user(path: &Path) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;

    fs::set_permissions(path, fs::Permissions::from_mode(0o700)).with_context(|| {
        format!(
            "js-ts-pre-write-evidence: failed to secure lock directory {}",
            path.display()
        )
    })
}

#[cfg(not(unix))]
fn restrict_directory_to_current_user(_path: &Path) -> Result<()> {
    Ok(())
}

pub(super) fn elapsed_ms(started: Instant) -> u64 {
    duration_ms(started.elapsed())
}

fn duration_ms(duration: Duration) -> u64 {
    u64::try_from(duration.as_millis()).unwrap_or(u64::MAX)
}

pub(super) fn attach_runtime_observations(
    evidence: &mut Value,
    lease: &ScanLease,
    discovery_ms: u64,
    projection_ms: u64,
) -> Result<()> {
    let single_flight = json!({
        "status": "acquired",
        "scope": "canonical-root",
        "backend": "os-file-lock",
        "waitMs": lease.wait_ms(),
    });
    let timing = {
        let incremental = evidence
            .pointer_mut("/anyInventory/meta/incremental")
            .and_then(Value::as_object_mut)
            .context(
                "js-ts-pre-write-evidence: projected incremental metadata must be an object",
            )?;
        let timing = incremental
            .entry("timing")
            .or_insert_with(|| Value::Object(Map::new()))
            .as_object_mut()
            .context("js-ts-pre-write-evidence: incremental timing must be an object")?;
        timing.insert("lockWaitMs".to_string(), number(lease.wait_ms()));
        timing.insert("discoveryMs".to_string(), number(discovery_ms));
        timing.insert("projectionMs".to_string(), number(projection_ms));
        timing.insert("scanHeldMs".to_string(), number(lease.held_ms()));
        timing.insert(
            "totalRuntimeMs".to_string(),
            number(lease.wait_ms().saturating_add(lease.held_ms())),
        );
        let timing = timing.clone();
        incremental.insert("singleFlight".to_string(), single_flight.clone());
        timing
    };

    let summary = evidence
        .get_mut("summary")
        .and_then(Value::as_object_mut)
        .context("js-ts-pre-write-evidence: projected summary must be an object")?;
    summary.insert(
        "runtime".to_string(),
        json!({
            "singleFlight": single_flight,
            "timing": timing,
        }),
    );
    Ok(())
}

fn number(value: u64) -> Value {
    Value::Number(Number::from(value))
}

#[cfg(test)]
mod tests {
    use super::{private_lock_root_for, ScanLease};
    use anyhow::{bail, Context, Result};
    use std::ffi::OsStr;
    use std::sync::mpsc;
    use std::thread;
    use std::time::Duration;
    use tempfile::tempdir;

    #[test]
    fn namespaces_lock_roots_by_user_identity() {
        let first = private_lock_root_for("HOME", OsStr::new("/home/first"));
        let second = private_lock_root_for("HOME", OsStr::new("/home/second"));

        assert_ne!(first, second);
        assert!(first.starts_with(std::env::temp_dir()));
        assert_ne!(first, std::env::temp_dir().join("lumin-audit-core"));
        assert!(!first.to_string_lossy().contains("/home/first"));
    }

    #[test]
    fn serializes_scans_for_the_same_canonical_root() -> Result<()> {
        let root = tempdir()?;
        let first = ScanLease::acquire(root.path())?;
        let second_root = root.path().to_path_buf();
        let (contended_tx, contended_rx) = mpsc::channel();
        let (acquired_tx, acquired_rx) = mpsc::channel();
        let handle = thread::spawn(move || -> Result<()> {
            let second =
                ScanLease::acquire_after_observing_contention(&second_root, &contended_tx)?;
            acquired_tx
                .send(second.wait_ms())
                .context("failed to report second lock acquisition")?;
            Ok(())
        });

        contended_rx
            .recv_timeout(Duration::from_secs(2))
            .context("second scan did not observe lock contention")?;

        drop(first);
        acquired_rx
            .recv_timeout(Duration::from_secs(2))
            .context("second scan did not acquire the released root lock")?;
        match handle.join() {
            Ok(result) => result?,
            Err(_) => bail!("second scan thread panicked"),
        }
        Ok(())
    }
}
