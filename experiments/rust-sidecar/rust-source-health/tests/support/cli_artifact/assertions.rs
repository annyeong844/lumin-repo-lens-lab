use std::fs;
use std::path::Path;

use anyhow::Result;
use serde_json::{json, Value};

pub fn assert_cli_artifact(output_path: &Path) -> Result<()> {
    let artifact: Value = serde_json::from_slice(&fs::read(output_path)?)?;
    assert_eq!(artifact["artifactProfile"], "compact");
    assert!(artifact["meta"]["generated"].is_string());
    assert_eq!(
        artifact["meta"]["sidecar"]["sourceCommit"],
        "test-source-commit"
    );
    assert_eq!(
        artifact["meta"]["input"]["pathPolicy"]["exclude"],
        json!(["**/target/**", "**/vendor/**"])
    );
    assert_eq!(artifact["summary"]["files"], 1);
    assert_eq!(artifact["summary"]["skippedFiles"], 1);
    assert_eq!(artifact["summary"]["shapeHashes"], 0);
    assert_eq!(artifact["summary"]["signalsByKind"]["unwrap-call"], 1);
    assert!(artifact["files"]["src/lib.rs"].is_object());
    assert!(artifact["files"]["src/lib.rs"]["ast"].is_null());
    assert_eq!(
        artifact["files"]["src/lib.rs"]["astSummary"]["definitions"],
        4
    );
    assert_eq!(
        artifact["files"]["src/lib.rs"]["astSummary"]["shapeHashes"],
        0
    );
    assert_eq!(
        artifact["files"]["src/lib.rs"]["astSummary"]["implBlocks"],
        1
    );
    assert_eq!(
        artifact["files"]["src/lib.rs"]["astSummary"]["implMethods"],
        2
    );
    assert_eq!(
        artifact["files"]["src/lib.rs"]["astSummary"]["methodCallSites"],
        1
    );
    assert_eq!(
        artifact["files"]["src/lib.rs"]["signals"][0]["kind"],
        "unwrap-call"
    );
    assert!(artifact["files"]["target/generated.rs"].is_null());
    assert!(artifact["files"]["vendor/vendored.rs"].is_null());
    assert_eq!(artifact["skippedFiles"][0]["path"], "src/bad.rs");
    assert_eq!(artifact["skippedFiles"][0]["reason"], "invalid-utf8");
    Ok(())
}

pub fn assert_full_cli_artifact(output_path: &Path) -> Result<()> {
    let artifact: Value = serde_json::from_slice(&fs::read(output_path)?)?;
    assert!(artifact["artifactProfile"].is_null());
    assert!(artifact["files"]["src/lib.rs"]["ast"]["definitions"].is_array());
    assert!(artifact["files"]["src/lib.rs"]["ast"]["impls"].is_array());
    assert_eq!(
        artifact["files"]["src/lib.rs"]["ast"]["definitions"][0]["kind"],
        "function"
    );
    assert_eq!(
        artifact["files"]["src/lib.rs"]["ast"]["impls"][0]["target"],
        "Runner"
    );
    assert_eq!(
        artifact["files"]["src/lib.rs"]["ast"]["impls"][0]["methods"][0]["name"],
        "run"
    );
    assert!(artifact["files"]["src/lib.rs"]["astSummary"].is_null());
    Ok(())
}
