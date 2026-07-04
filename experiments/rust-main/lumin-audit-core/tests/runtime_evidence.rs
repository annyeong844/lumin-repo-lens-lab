use anyhow::Result;
use serde_json::json;
use std::fs;
use std::process::Command;

#[test]
fn cli_runtime_evidence_artifact_writes_result_file() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let input_path = temp.path().join("input.json");
    let output_path = temp.path().join("runtime-evidence.json");
    fs::write(
        &input_path,
        serde_json::to_vec(&json!({
            "schemaVersion": "lumin-runtime-evidence-producer-request.v1",
            "root": "C:/repo",
            "generated": "2026-07-03T00:00:00.000Z",
            "symbolsSource": "symbols.json",
            "coverageSource": "coverage-final.json",
            "coverageMtime": "2026-07-03T00:00:00.000Z",
            "symbols": {
                "deadProdList": [
                    { "file": "src/cold.ts", "line": 1, "symbol": "cold", "kind": "FunctionDeclaration" }
                ]
            },
            "coverage": {
                "C:/repo/src/cold.ts": {
                    "path": "C:/repo/src/cold.ts",
                    "statementMap": {
                        "0": { "start": { "line": 1 }, "end": { "line": 2 } }
                    },
                    "s": { "0": 0 },
                    "fnMap": {
                        "0": { "loc": { "start": { "line": 1 }, "end": { "line": 2 } } }
                    },
                    "f": { "0": 0 }
                }
            }
        }))?,
    )?;

    let output = Command::new(audit_core_bin())
        .arg("runtime-evidence-artifact")
        .arg("--input")
        .arg(&input_path)
        .arg("--result-output")
        .arg(&output_path)
        .output()?;
    assert!(output.status.success());
    assert!(output.stdout.is_empty());

    let artifact = serde_json::from_slice::<serde_json::Value>(&fs::read(&output_path)?)?;
    assert_eq!(artifact["meta"]["tool"], "merge-runtime-evidence.mjs");
    assert_eq!(artifact["summary"]["grounded_dead"], 1);
    assert_eq!(artifact["merged"][0]["runtimeStatus"], "dead-confirmed");
    Ok(())
}

#[test]
fn cli_runtime_evidence_artifact_rejects_bad_schema() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let input_path = temp.path().join("bad-input.json");
    fs::write(
        &input_path,
        br#"{"schemaVersion":"wrong","root":"C:/repo","symbols":{},"coverage":{}}"#,
    )?;
    let output = Command::new(audit_core_bin())
        .arg("runtime-evidence-artifact")
        .arg("--input")
        .arg(&input_path)
        .output()?;
    assert!(!output.status.success());
    Ok(())
}

fn audit_core_bin() -> &'static str {
    env!("CARGO_BIN_EXE_lumin-audit-core")
}
