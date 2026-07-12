use anyhow::Result;
use serde_json::{json, Value};
use std::fs;
use std::process::Command;

#[test]
fn cli_module_reachability_artifact_writes_result_file() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let input = temp.path().join("request.json");
    let result = temp.path().join("result.json");
    fs::write(
        &input,
        serde_json::to_vec(&json!({
            "schemaVersion": "lumin-module-reachability-producer-request.v1",
            "root": "C:/repo",
            "generated": "2026-07-03T00:00:00.000Z",
            "symbols": {
                "defIndex": { "src/index.ts": {}, "src/unused.ts": {} },
                "reExportsByFile": {},
                "resolvedInternalEdges": [
                    { "from": "src/index.ts", "to": "src/reachable.ts" }
                ]
            },
            "entrySurface": {
                "entryFiles": ["src/index.ts"],
                "globalCompleteness": "high",
                "completenessBySubmodule": {}
            }
        }))?,
    )?;

    let output = Command::new(audit_core_bin())
        .arg("module-reachability-artifact")
        .arg("--input")
        .arg(&input)
        .arg("--result-output")
        .arg(&result)
        .output()?;

    assert!(output.status.success());
    assert!(output.stdout.is_empty());
    let artifact: Value = serde_json::from_slice(&fs::read(&result)?)?;
    assert_eq!(artifact["meta"]["schemaVersion"], "module-reachability.v1");
    assert_eq!(artifact["summary"]["reachable"], 2);
    assert_eq!(artifact["summary"]["unreachable"], 1);
    Ok(())
}

fn audit_core_bin() -> &'static str {
    env!("CARGO_BIN_EXE_lumin-audit-core")
}
