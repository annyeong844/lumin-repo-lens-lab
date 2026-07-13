use super::memory::{memory_delta, memory_snapshot};
use super::protocol::ExecutorMemoryObservation;
use anyhow::{Context, Result};
use std::fs;
use std::io::ErrorKind;
use std::path::Path;
use std::process::{Command, Output};
use std::time::Instant;

pub(super) struct ChildObservation {
    pub(super) status: String,
    pub(super) ms: u64,
    pub(super) stderr_snippet: Option<String>,
    pub(super) memory: ExecutorMemoryObservation,
}

pub(super) fn run_child(command: &str, args: &[String], verbose: bool) -> Result<ChildObservation> {
    let before = memory_snapshot();
    let started = Instant::now();
    let output = if verbose {
        Command::new(command)
            .args(args)
            .status()
            .map(|status| Output {
                status,
                stdout: Vec::new(),
                stderr: Vec::new(),
            })
    } else {
        Command::new(command).args(args).output()
    }?;
    let ms = started.elapsed().as_millis().try_into().unwrap_or(u64::MAX);
    let after = memory_snapshot();
    let status = if output.status.success() {
        "ok"
    } else {
        "failed"
    }
    .to_string();
    let stderr_text = String::from_utf8_lossy(&output.stderr).to_string();
    let stderr_snippet =
        (!stderr_text.trim().is_empty()).then(|| stderr_text.chars().take(500).collect::<String>());
    Ok(ChildObservation {
        status,
        ms,
        stderr_snippet,
        memory: ExecutorMemoryObservation {
            delta: memory_delta(&before, &after),
            before,
            after,
        },
    })
}

pub(super) fn failed_child_observation_from_spawn_error(
    error: &dyn std::fmt::Display,
) -> ChildObservation {
    let before = memory_snapshot();
    let after = memory_snapshot();
    let stderr = format!("failed to start child process: {error}");
    ChildObservation {
        status: "failed".to_string(),
        ms: 0,
        stderr_snippet: Some(stderr.chars().take(500).collect()),
        memory: ExecutorMemoryObservation {
            delta: memory_delta(&before, &after),
            before,
            after,
        },
    }
}

pub(super) fn command_status(observed: &ChildObservation, required: bool) -> String {
    if observed.status == "ok" {
        "ok"
    } else if required {
        "failed-required"
    } else {
        "failed-optional"
    }
    .to_string()
}

pub(super) fn clear_producer_phase_timing(output: &Path, producer: &str) -> Result<()> {
    let phase_path = output
        .join(".producer-phases")
        .join(format!("{}.json", safe_producer_file_name(producer)));
    remove_file_if_present(&phase_path)
}

pub(super) fn remove_file_if_present(path: &Path) -> Result<()> {
    match fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error)
            .with_context(|| format!("failed to remove stale artifact at {}", path.display())),
    }
}

fn safe_producer_file_name(producer: &str) -> String {
    let base = producer
        .replace('\\', "/")
        .rsplit('/')
        .next()
        .filter(|value| !value.is_empty())
        .unwrap_or("unknown")
        .to_string();
    base.chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || matches!(ch, '.' | '_' | '-') {
                ch
            } else {
                '_'
            }
        })
        .collect()
}
