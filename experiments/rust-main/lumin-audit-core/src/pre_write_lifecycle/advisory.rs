use anyhow::{Context, Result};
use lumin_rust_common::atomic_write_json_pretty;
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};

pub(super) struct AdvisoryWriteResult {
    pub(super) latest_path: PathBuf,
    pub(super) specific_path: PathBuf,
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
