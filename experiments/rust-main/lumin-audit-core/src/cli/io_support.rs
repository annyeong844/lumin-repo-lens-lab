use anyhow::{bail, Context, Result};
use lumin_audit_core::artifact_read_metrics::ArtifactReadObservation;
use serde::Serialize;
use serde_json::{json, Value};
use std::fs;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::time::Instant;

pub(super) struct OptionalOutputJsonRead {
    pub value: Option<Value>,
    pub observation: Option<ArtifactReadObservation>,
}

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

pub(super) fn read_optional_output_json_observed(
    output: &Path,
    artifact_name: &str,
    label: &str,
) -> Result<OptionalOutputJsonRead> {
    let path = output.join(artifact_name);
    if !path.exists() {
        return Ok(OptionalOutputJsonRead {
            value: None,
            observation: None,
        });
    }
    let read_started = Instant::now();
    let text = fs::read_to_string(&path)
        .with_context(|| format!("{label}: failed to read {}", path.display()))?;
    let read_ms = elapsed_ms(read_started);
    let bytes = text.len() as u64;
    let parse_started = Instant::now();
    let json = serde_json::from_str::<Value>(&text)
        .with_context(|| format!("{label}: invalid JSON in {}", path.display()))?;
    let json_parse_ms = elapsed_ms(parse_started);
    Ok(OptionalOutputJsonRead {
        value: Some(json),
        observation: Some(artifact_read_observation(
            &path,
            bytes,
            read_ms,
            json_parse_ms,
            true,
        )),
    })
}

pub(super) fn read_optional_output_json_tolerant(
    output: &Path,
    artifact_name: &str,
) -> Option<Value> {
    read_optional_output_json_tolerant_observed(output, artifact_name).value
}

pub(super) fn read_optional_output_json_tolerant_observed(
    output: &Path,
    artifact_name: &str,
) -> OptionalOutputJsonRead {
    let path = output.join(artifact_name);
    if !path.exists() {
        return OptionalOutputJsonRead {
            value: None,
            observation: None,
        };
    }
    let read_started = Instant::now();
    let text = match fs::read_to_string(&path) {
        Ok(text) => text,
        Err(error) => {
            return OptionalOutputJsonRead {
                value: Some(malformed_optional_artifact(
                    artifact_name,
                    "read-error",
                    error.to_string(),
                )),
                observation: Some(artifact_read_observation(&path, 0, 0, 0, false)),
            };
        }
    };
    let read_ms = elapsed_ms(read_started);
    let bytes = text.len() as u64;
    let parse_started = Instant::now();
    match serde_json::from_str::<Value>(&text) {
        Ok(json) => OptionalOutputJsonRead {
            value: Some(json),
            observation: Some(artifact_read_observation(
                &path,
                bytes,
                read_ms,
                elapsed_ms(parse_started),
                true,
            )),
        },
        Err(error) => OptionalOutputJsonRead {
            value: Some(malformed_optional_artifact(
                artifact_name,
                "malformed-json",
                error.to_string(),
            )),
            observation: Some(artifact_read_observation(
                &path,
                bytes,
                read_ms,
                elapsed_ms(parse_started),
                false,
            )),
        },
    }
}

fn malformed_optional_artifact(artifact_name: &str, kind: &str, message: String) -> Value {
    json!({
        "schemaVersion": Value::Null,
        "artifact": artifact_name,
        "status": "unavailable",
        "available": false,
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

fn artifact_read_observation(
    path: &Path,
    bytes: u64,
    read_ms: u64,
    json_parse_ms: u64,
    ok: bool,
) -> ArtifactReadObservation {
    ArtifactReadObservation {
        file_path: Some(path.to_string_lossy().to_string()),
        bytes: Value::from(bytes),
        read_ms: Value::from(read_ms),
        json_parse_ms: Value::from(json_parse_ms),
        ok: Some(ok),
    }
}

fn elapsed_ms(started: Instant) -> u64 {
    started.elapsed().as_millis().try_into().unwrap_or(u64::MAX)
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
