use std::fs;
use std::process::Output;

use anyhow::{Context, Result};
use serde_json::Value;
use tempfile::TempDir;

use crate::support::fixtures::package;
use crate::support::root_command;

pub struct InvalidCalibrationAdjudicationRun {
    pub output: Output,
    pub artifact_exists: bool,
    pub artifact: Option<Value>,
}

pub fn run_invalid_calibration_adjudication() -> Result<InvalidCalibrationAdjudicationRun> {
    run_invalid_calibration_adjudication_with_payload(r#"{"entries": ["#)
}

pub fn run_unknown_calibration_tier() -> Result<InvalidCalibrationAdjudicationRun> {
    run_invalid_calibration_adjudication_with_payload(
        r#"{"candidateCounts":{"available":true,"safeFix":1,"reviewVisibleCleanup":1},"entries":[{"tier":"SAFEISH","verdict":"true_dead","file":"src/lib.rs"}]}"#,
    )
}

pub fn run_unknown_calibration_verdict() -> Result<InvalidCalibrationAdjudicationRun> {
    run_invalid_calibration_adjudication_with_payload(
        r#"{"candidateCounts":{"available":true,"safeFix":1,"reviewVisibleCleanup":1},"entries":[{"tier":"SAFE_FIX","verdict":"mostly_true","file":"src/lib.rs"}]}"#,
    )
}

pub fn run_malformed_calibration_entry_fields() -> Result<InvalidCalibrationAdjudicationRun> {
    run_invalid_calibration_adjudication_with_payload(
        r#"{"candidateCounts":{"available":true,"safeFix":1,"reviewVisibleCleanup":1},"entries":[{"corpusName":123,"tier":"SAFE_FIX","verdict":"true_dead","file":42,"diagnosticCode":["unused_mut"],"lineStart":"1"}]}"#,
    )
}

pub fn run_malformed_candidate_counts_fields() -> Result<InvalidCalibrationAdjudicationRun> {
    run_invalid_calibration_adjudication_with_payload(
        r#"{"candidateCounts":{"available":"yes","safeFix":"1","reviewVisibleCleanup":1,"degraded":[],"muted":{},"byCorpus":{"cal":{"reviewVisibleCleanup":"1"},"ignored":7}},"corpus":[{"name":"cal","commit":"abc","worktreeDirty":false,"locBucket":"25k"}],"entries":[{"corpusName":"cal","tier":"SAFE_FIX","verdict":"true_dead","file":"src/lib.rs"}]}"#,
    )
}

pub fn run_present_but_invalid_candidate_counts() -> Result<InvalidCalibrationAdjudicationRun> {
    run_invalid_calibration_adjudication_with_payload(
        r#"{"candidateCounts":{"safeFix":"1","reviewVisibleCleanup":"1","degraded":[],"muted":{}},"entries":[{"tier":"SAFE_FIX","verdict":"true_dead","file":"src/lib.rs"}]}"#,
    )
}

pub fn run_malformed_calibration_container_fields() -> Result<InvalidCalibrationAdjudicationRun> {
    run_invalid_calibration_adjudication_with_payload(
        r#"{"candidateCounts":"unavailable","corpus":"not-an-array","schemaRoundTrip":{"attempted":"yes","knownSchemaDriftBugs":"known"},"entries":{"tier":"SAFE_FIX","verdict":"true_dead"}}"#,
    )
}

pub fn run_malformed_corpus_fields() -> Result<InvalidCalibrationAdjudicationRun> {
    run_invalid_calibration_adjudication_with_payload(
        r#"{"candidateCounts":{"available":true,"safeFix":1,"reviewVisibleCleanup":1},"corpus":[{"name":7,"commit":"","snapshotId":"","contentHash":"","worktreeDirty":"yes","locBucket":false},null],"schemaRoundTrip":{"attempted":true,"knownSchemaDriftBugs":[]},"entries":[{"corpusName":"(unknown)","tier":"SAFE_FIX","verdict":"true_dead","file":"src/lib.rs"}]}"#,
    )
}

pub fn run_unnamed_corpus_adjudication() -> Result<InvalidCalibrationAdjudicationRun> {
    run_invalid_calibration_adjudication_with_payload(
        r#"{"candidateCounts":{"available":true,"safeFix":1,"reviewVisibleCleanup":1},"corpus":[{"commit":"abc","worktreeDirty":false,"locBucket":"25k"}],"schemaRoundTrip":{"attempted":true,"knownSchemaDriftBugs":[]},"entries":[{"tier":"SAFE_FIX","verdict":"true_dead","file":"src/lib.rs"}]}"#,
    )
}

pub fn run_unknown_corpus_name_adjudication() -> Result<InvalidCalibrationAdjudicationRun> {
    run_invalid_calibration_adjudication_with_payload(
        r#"{"candidateCounts":{"available":true,"safeFix":1,"reviewVisibleCleanup":1,"byCorpus":{"(unknown)":{"reviewVisibleCleanup":1}}},"corpus":[{"name":"(unknown)","commit":"abc","worktreeDirty":false,"locBucket":"25k"}],"schemaRoundTrip":{"attempted":true,"knownSchemaDriftBugs":[]},"entries":[{"tier":"SAFE_FIX","verdict":"true_dead","file":"src/lib.rs"}]}"#,
    )
}

pub fn run_empty_corpus_identity_adjudication() -> Result<InvalidCalibrationAdjudicationRun> {
    run_invalid_calibration_adjudication_with_payload(
        r#"{"candidateCounts":{"available":true,"safeFix":1,"reviewVisibleCleanup":1},"corpus":[{"name":"cal","commit":"","snapshotId":"","contentHash":"","worktreeDirty":true,"locBucket":"25k"}],"schemaRoundTrip":{"attempted":true,"knownSchemaDriftBugs":[]},"unresolvedHighFindings":2,"entries":[{"corpusName":"cal","tier":"SAFE_FIX","verdict":"true_dead","file":"src/lib.rs"}]}"#,
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
    let artifact = if output_path.exists() {
        Some(
            serde_json::from_slice(
                &fs::read(&output_path)
                    .with_context(|| format!("read artifact {}", output_path.display()))?,
            )
            .with_context(|| format!("parse artifact {}", output_path.display()))?,
        )
    } else {
        None
    };

    Ok(InvalidCalibrationAdjudicationRun {
        output,
        artifact_exists: output_path.exists(),
        artifact,
    })
}
