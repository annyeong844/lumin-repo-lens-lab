use anyhow::{bail, Context, Result};
use lumin_rust_common::atomic_write_json_pretty;
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};

pub(super) struct AdvisoryWriteResult {
    pub(super) latest_path: PathBuf,
    pub(super) specific_path: PathBuf,
}

pub(super) fn read_required_json(path: &Path, label: &str) -> Result<Value> {
    let text = fs::read_to_string(path)
        .with_context(|| format!("{label}: failed to read {}", path.display()))?;
    serde_json::from_str(&text)
        .with_context(|| format!("{label}: invalid JSON in {}", path.display()))
}

pub(super) fn required_invocation_id<'a>(artifact: &'a Value, label: &str) -> Result<&'a str> {
    artifact
        .as_object()
        .with_context(|| format!("{label} must be an object"))?
        .get("invocationId")
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .with_context(|| format!("{label}.invocationId must be a non-empty string"))
}

pub(super) fn validate_matching_json_artifacts(
    latest: &Path,
    specific: &Path,
    label: &str,
) -> Result<()> {
    let latest_json = read_json_artifact(latest, label)?;
    let specific_json = read_json_artifact(specific, label)?;
    if latest_json != specific_json {
        bail!(
            "{label} contract failed: latest and invocation-specific artifacts differ ({} != {})",
            latest.display(),
            specific.display()
        );
    }
    Ok(())
}

pub(super) fn write_advisory(output: &Path, advisory: &Value) -> Result<AdvisoryWriteResult> {
    let invocation_id = advisory
        .get("invocationId")
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .context("writeAdvisory: advisory.invocationId is required")?;
    let latest_path = advisory_latest_path(output);
    let specific_path = advisory_specific_path(output, invocation_id);
    atomic_write_json_pretty(&specific_path, advisory)
        .with_context(|| format!("writeAdvisory: failed to write {}", specific_path.display()))?;
    atomic_write_json_pretty(&latest_path, advisory)
        .with_context(|| format!("writeAdvisory: failed to write {}", latest_path.display()))?;
    Ok(AdvisoryWriteResult {
        latest_path,
        specific_path,
    })
}

pub(super) fn advisory_latest_path(output: &Path) -> PathBuf {
    output.join("pre-write-advisory.latest.json")
}

pub(super) fn advisory_specific_path(output: &Path, invocation_id: &str) -> PathBuf {
    output.join(format!("pre-write-advisory.{invocation_id}.json"))
}

pub(super) fn remove_file_if_present(path: &Path) -> std::io::Result<()> {
    match fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error),
    }
}

pub(super) fn path_string(path: &Path) -> String {
    path.to_string_lossy().to_string()
}

fn read_json_artifact(path: &Path, label: &str) -> Result<Value> {
    let bytes = fs::read(path)
        .with_context(|| format!("{label} contract failed: failed to read {}", path.display()))?;
    serde_json::from_slice(&bytes).with_context(|| {
        format!(
            "{label} contract failed: invalid JSON in {}",
            path.display()
        )
    })
}
