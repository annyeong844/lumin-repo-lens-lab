use std::fs;
use std::process::Output;

use anyhow::{Context, Result};
use tempfile::TempDir;

use crate::support::fixtures::package;
use crate::support::root_command;

pub struct InvalidCalibrationAdjudicationRun {
    pub output: Output,
    pub artifact_exists: bool,
}

pub fn run_invalid_calibration_adjudication() -> Result<InvalidCalibrationAdjudicationRun> {
    run_invalid_calibration_adjudication_with_payload(r#"{"entries": ["#)
}

pub fn run_unknown_calibration_tier() -> Result<InvalidCalibrationAdjudicationRun> {
    run_invalid_calibration_adjudication_with_payload(
        r#"{"entries":[{"tier":"SAFEISH","verdict":"true_dead","file":"src/lib.rs"}]}"#,
    )
}

pub fn run_unknown_calibration_verdict() -> Result<InvalidCalibrationAdjudicationRun> {
    run_invalid_calibration_adjudication_with_payload(
        r#"{"entries":[{"tier":"SAFE_FIX","verdict":"mostly_true","file":"src/lib.rs"}]}"#,
    )
}

fn run_invalid_calibration_adjudication_with_payload(
    payload: &str,
) -> Result<InvalidCalibrationAdjudicationRun> {
    let temp = TempDir::new()?;
    let root = temp.path().join("repo");
    package::write_single_package_crate(&root, "app", "pub fn demo() {}\n")?;

    let adjudication_path = temp.path().join("invalid-adjudication.json");
    fs::write(&adjudication_path, payload)?;
    let output_path = temp.path().join("rust-analyzer-health.json");
    let mut command = root_command::unified_analyzer_command_for(&root, &output_path)?;
    let output = command
        .arg("--calibration-adjudication")
        .arg(adjudication_path)
        .output()
        .context("run unified rust analyzer with invalid calibration adjudication")?;

    Ok(InvalidCalibrationAdjudicationRun {
        output,
        artifact_exists: output_path.exists(),
    })
}
