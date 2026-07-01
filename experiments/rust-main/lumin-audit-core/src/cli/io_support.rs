use anyhow::{bail, Context, Result};
use serde::Serialize;
use serde_json::{json, Value};
use std::fs;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};

pub(super) fn read_optional_json(path: Option<PathBuf>, label: &str) -> Result<Option<Value>> {
    let Some(path) = path else {
        return Ok(None);
    };
    let text = fs::read_to_string(&path)
        .with_context(|| format!("{label}: failed to read {}", path.display()))?;
    let json = serde_json::from_str::<Value>(&text)
        .with_context(|| format!("{label}: invalid JSON in {}", path.display()))?;
    Ok(Some(json))
}

pub(super) fn read_required_json(path: &Path, label: &str) -> Result<Value> {
    let text = fs::read_to_string(path)
        .with_context(|| format!("{label}: failed to read {}", path.display()))?;
    serde_json::from_str::<Value>(&text)
        .with_context(|| format!("{label}: invalid JSON in {}", path.display()))
}

pub(super) fn read_json_input(input: &str, label: &str) -> Result<Value> {
    if input == "-" {
        let mut text = String::new();
        io::stdin()
            .read_to_string(&mut text)
            .with_context(|| format!("{label}: failed to read stdin"))?;
        return serde_json::from_str::<Value>(&text)
            .with_context(|| format!("{label}: invalid JSON in stdin"));
    }
    read_required_json(Path::new(input), label)
}

pub(super) fn read_optional_json_input(
    input: Option<String>,
    label: &str,
) -> Result<Option<Value>> {
    input
        .as_deref()
        .map(|input| read_json_input(input, label))
        .transpose()
}

pub(super) fn read_optional_output_json(
    output: &Path,
    artifact_name: &str,
    label: &str,
) -> Result<Option<Value>> {
    let path = output.join(artifact_name);
    if !path.exists() {
        return Ok(None);
    }
    read_optional_json(Some(path), label)
}

pub(super) fn read_optional_output_json_tolerant(
    output: &Path,
    artifact_name: &str,
) -> Option<Value> {
    let path = output.join(artifact_name);
    if !path.exists() {
        return None;
    }
    let text = match fs::read_to_string(&path) {
        Ok(text) => text,
        Err(error) => {
            return Some(malformed_optional_artifact(
                artifact_name,
                "read-error",
                error.to_string(),
            ));
        }
    };
    match serde_json::from_str::<Value>(&text) {
        Ok(json) => Some(json),
        Err(error) => Some(malformed_optional_artifact(
            artifact_name,
            "malformed-json",
            error.to_string(),
        )),
    }
}

fn malformed_optional_artifact(artifact_name: &str, kind: &str, message: String) -> Value {
    json!({
        "schemaVersion": Value::Null,
        "artifact": artifact_name,
        "status": "unavailable",
        "reason": {
            "kind": kind,
            "message": message,
        },
        "summary": {
            "status": "unavailable",
            "unavailableReason": kind,
        }
    })
}

pub(super) fn take_path(args: &mut impl Iterator<Item = String>, flag: &str) -> Result<PathBuf> {
    let Some(value) = args.next() else {
        bail!("{flag} requires a value");
    };
    if value.starts_with("--") {
        bail!("{flag} requires a value");
    }
    Ok(PathBuf::from(value))
}

pub(super) fn take_string(args: &mut impl Iterator<Item = String>, flag: &str) -> Result<String> {
    let Some(value) = args.next() else {
        bail!("{flag} requires a value");
    };
    if value.starts_with("--") {
        bail!("{flag} requires a value");
    }
    Ok(value)
}

pub(super) fn write_stdout_json<T: Serialize>(value: &T) -> Result<()> {
    let stdout = io::stdout();
    let mut stdout = stdout.lock();
    serde_json::to_writer(&mut stdout, value).context("failed to write audit-core JSON stdout")?;
    stdout
        .write_all(b"\n")
        .context("failed to write audit-core JSON newline")
}

pub(super) fn write_json_file<T: Serialize>(path: &Path, value: &T) -> Result<()> {
    let mut bytes =
        serde_json::to_vec(value).context("failed to serialize audit-core JSON file")?;
    bytes.push(b'\n');
    fs::write(path, bytes)
        .with_context(|| format!("failed to write audit-core JSON file {}", path.display()))
}
