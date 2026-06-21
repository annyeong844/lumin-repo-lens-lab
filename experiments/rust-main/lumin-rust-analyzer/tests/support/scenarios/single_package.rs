use std::ffi::OsStr;
use std::fs;

use anyhow::Result;
use serde_json::{json, Value};
use tempfile::TempDir;

use crate::support::fixtures::package;
use crate::support::scenarios::run;

pub fn analyze_cargo_check_single_package(lib_rs: &str) -> Result<Value> {
    analyze_single_package(lib_rs, Some("cargo-check"))
}

pub fn analyze_cargo_check_single_package_with_adjudication(
    lib_rs: &str,
    adjudication_json: &str,
) -> Result<Value> {
    let temp = TempDir::new()?;
    let root = temp.path().join("repo");
    let adjudication_path = temp.path().join("safe-fix-adjudication.json");
    package::write_single_package_crate(&root, "app", lib_rs)?;
    fs::write(&adjudication_path, adjudication_json)?;
    let extra_args = [
        OsStr::new("--calibration-adjudication"),
        adjudication_path.as_os_str(),
    ];
    run::run_analyzer_with_args(&root, Some("cargo-check"), &extra_args)
}

pub fn analyze_cargo_check_single_package_with_complete_calibration_evidence(
    lib_rs: &str,
) -> Result<Value> {
    let evidence = complete_calibration_evidence(Some(json!({
        "attempted": true,
        "knownSchemaDriftBugs": [],
    })));
    analyze_cargo_check_single_package_with_adjudication(lib_rs, &evidence)
}

pub fn analyze_cargo_check_single_package_with_missing_schema_round_trip_evidence(
    lib_rs: &str,
) -> Result<Value> {
    let evidence = complete_calibration_evidence(None);
    analyze_cargo_check_single_package_with_adjudication(lib_rs, &evidence)
}

pub fn analyze_metadata_only_single_package(lib_rs: &str) -> Result<Value> {
    analyze_single_package(lib_rs, None)
}

pub fn analyze_metadata_only_single_package_with_invalid_utf8_file(lib_rs: &str) -> Result<Value> {
    let temp = TempDir::new()?;
    let root = temp.path().join("repo");
    package::write_single_package_crate(&root, "app", lib_rs)?;
    fs::write(root.join("src").join("bad.rs"), [0xff, 0xfe, b'\n'])?;
    run::run_analyzer(&root, None)
}

pub fn analyze_targeted_single_package(lib_rs: &str) -> Result<Value> {
    analyze_single_package(lib_rs, Some("targeted-cargo-check"))
}

pub fn analyze_targeted_single_package_with_integration(
    lib_rs: &str,
    integration_rs: &str,
) -> Result<Value> {
    let temp = TempDir::new()?;
    let root = temp.path().join("repo");
    package::write_single_package_crate(&root, "app", lib_rs)?;
    fs::create_dir_all(root.join("tests"))?;
    fs::write(root.join("tests").join("integration.rs"), integration_rs)?;
    run::run_analyzer(&root, Some("targeted-cargo-check"))
}

fn analyze_single_package(lib_rs: &str, semantic_mode: Option<&str>) -> Result<Value> {
    let temp = TempDir::new()?;
    let root = temp.path().join("repo");
    package::write_single_package_crate(&root, "app", lib_rs)?;
    run::run_analyzer(&root, semantic_mode)
}

fn complete_calibration_evidence(schema_round_trip: Option<Value>) -> String {
    let mut evidence = json!({
        "corpus": [
            {
                "name": "rust-corpus-a",
                "commit": "abc123",
                "worktreeDirty": false,
                "locBucket": "25k",
            },
            {
                "name": "rust-corpus-b",
                "snapshotId": "snapshot-b",
                "worktreeDirty": false,
                "locBucket": "50k",
            },
        ],
        "candidateCounts": {
            "available": true,
            "reviewVisibleCleanup": 2,
            "safeFix": 2,
            "reviewFix": 0,
            "degraded": 0,
            "muted": 0,
            "byCorpus": {
                "rust-corpus-a": {
                    "reviewVisibleCleanup": 1,
                    "safeFix": 1,
                    "reviewFix": 0,
                },
                "rust-corpus-b": {
                    "reviewVisibleCleanup": 1,
                    "safeFix": 1,
                    "reviewFix": 0,
                },
            },
        },
        "entries": [
            {
                "corpusName": "rust-corpus-a",
                "tier": "SAFE_FIX",
                "verdict": "true_dead",
                "file": "src/lib.rs",
                "diagnosticCode": "unused_mut",
                "lineStart": 1,
                "symbol": "demo",
            },
            {
                "corpusName": "rust-corpus-b",
                "tier": "SAFE_FIX",
                "verdict": "true_dead",
                "file": "src/lib.rs",
                "diagnosticCode": "unused_mut",
                "lineStart": 1,
                "symbol": "demo",
            },
        ],
        "minAdjudicatedPerCorpus": 1,
    });
    if let Some(schema_round_trip) = schema_round_trip {
        evidence["schemaRoundTrip"] = schema_round_trip;
    }
    evidence.to_string()
}
