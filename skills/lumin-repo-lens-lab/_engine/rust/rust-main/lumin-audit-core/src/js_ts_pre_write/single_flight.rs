use anyhow::{Context, Result};
use lumin_rust_common::sha256_text;
use serde_json::{json, Map, Number, Value};
use std::fs::{self, File, OpenOptions};
use std::path::Path;
use std::time::{Duration, Instant};

pub(super) struct ScanLease {
    _file: File,
    wait_ms: u64,
    acquired_at: Instant,
}

impl ScanLease {
    pub(super) fn acquire(root: &Path) -> Result<Self> {
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
        let lock_dir = std::env::temp_dir()
            .join("lumin-audit-core")
            .join("js-ts-pre-write-locks");
        fs::create_dir_all(&lock_dir).with_context(|| {
            format!(
                "js-ts-pre-write-evidence: failed to create lock directory {}",
                lock_dir.display()
            )
        })?;
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

    pub(super) fn wait_ms(&self) -> u64 {
        self.wait_ms
    }

    pub(super) fn held_ms(&self) -> u64 {
        elapsed_ms(self.acquired_at)
    }
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
    use super::ScanLease;
    use anyhow::{bail, Context, Result};
    use std::sync::mpsc;
    use std::thread;
    use std::time::Duration;
    use tempfile::tempdir;

    #[test]
    fn serializes_scans_for_the_same_canonical_root() -> Result<()> {
        let root = tempdir()?;
        let first = ScanLease::acquire(root.path())?;
        let second_root = root.path().to_path_buf();
        let (attempted_tx, attempted_rx) = mpsc::channel();
        let (acquired_tx, acquired_rx) = mpsc::channel();
        let handle = thread::spawn(move || -> Result<()> {
            attempted_tx
                .send(())
                .context("failed to report second lock attempt")?;
            let second = ScanLease::acquire(&second_root)?;
            acquired_tx
                .send(second.wait_ms())
                .context("failed to report second lock acquisition")?;
            Ok(())
        });

        attempted_rx
            .recv_timeout(Duration::from_secs(2))
            .context("second scan did not attempt lock acquisition")?;
        if acquired_rx.recv_timeout(Duration::from_millis(50)).is_ok() {
            bail!("second scan acquired the root lock while the first lease was alive");
        }

        drop(first);
        let wait_ms = acquired_rx
            .recv_timeout(Duration::from_secs(2))
            .context("second scan did not acquire the released root lock")?;
        if wait_ms == 0 {
            bail!("contended scan did not record lock wait time");
        }
        match handle.join() {
            Ok(result) => result?,
            Err(_) => bail!("second scan thread panicked"),
        }
        Ok(())
    }
}
