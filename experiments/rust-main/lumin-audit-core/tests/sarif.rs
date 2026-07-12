use anyhow::Result;
use serde_json::json;
use std::fs;
use std::process::Command;

#[test]
fn cli_sarif_artifact_writes_result_file() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let input_path = temp.path().join("input.json");
    let output_path = temp.path().join("lumin-repo-lens-lab.sarif");
    fs::write(
        &input_path,
        serde_json::to_vec(&json!({
            "schemaVersion": "lumin-sarif-producer-request.v1",
            "root": "C:/repo",
            "generated": "2026-07-04T00:00:00.000Z",
            "fixPlan": {
                "safeFixes": [{
                    "finding": {
                        "file": "src/safe.ts",
                        "line": 10,
                        "symbol": "SafeSym",
                        "kind": "FunctionDeclaration",
                        "bucket": "C"
                    },
                    "evidence": {
                        "runtime": {
                            "status": "dead-confirmed",
                            "grounding": "grounded",
                            "confidence": "high",
                            "hitsInSymbol": 0
                        }
                    },
                    "reason": "runtime-dead-confirmed"
                }],
                "muted": [{
                    "finding": { "file": "eslint.config.mjs", "line": 1, "symbol": "default", "kind": "default" },
                    "reason": "policy-excluded"
                }]
            }
        }))?,
    )?;

    let output = Command::new(audit_core_bin())
        .arg("sarif-artifact")
        .arg("--input")
        .arg(&input_path)
        .arg("--result-output")
        .arg(&output_path)
        .output()?;
    assert!(output.status.success());
    assert!(output.stdout.is_empty());

    let artifact = serde_json::from_slice::<serde_json::Value>(&fs::read(&output_path)?)?;
    assert_eq!(artifact["version"], "2.1.0");
    assert_eq!(artifact["runs"][0]["results"][0]["ruleId"], "GA001");
    assert_eq!(artifact["runs"][0]["results"][0]["level"], "warning");
    assert_eq!(
        artifact["runs"][0]["properties"]["artifactsUsed"],
        json!(["fix-plan.json"])
    );
    Ok(())
}

#[test]
fn cli_sarif_artifact_rejects_bad_schema() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let input_path = temp.path().join("bad-input.json");
    fs::write(
        &input_path,
        br#"{"schemaVersion":"wrong","root":"C:/repo"}"#,
    )?;
    let output = Command::new(audit_core_bin())
        .arg("sarif-artifact")
        .arg("--input")
        .arg(&input_path)
        .output()?;
    assert!(!output.status.success());
    Ok(())
}

fn audit_core_bin() -> &'static str {
    env!("CARGO_BIN_EXE_lumin-audit-core")
}
