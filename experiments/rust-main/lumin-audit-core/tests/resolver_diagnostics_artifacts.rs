use anyhow::Result;
use serde_json::{json, Value};
use std::fs;
use std::process::Command;

#[test]
fn cli_resolver_diagnostics_artifacts_writes_result_file() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let input = temp.path().join("request.json");
    let result = temp.path().join("result.json");
    fs::write(
        &input,
        serde_json::to_vec(&json!({
            "schemaVersion": "lumin-resolver-diagnostics-producer-request.v1",
            "symbols": {
                "uses": {
                    "unresolvedInternal": 1,
                    "unresolvedInternalRatio": 1.0,
                    "external": 0
                },
                "unresolvedInternalSpecifierRecords": [
                    {
                        "specifier": "#app/config",
                        "consumerFile": "packages/app/src/a.ts",
                        "kind": "import",
                        "reason": "hash-import-target-missing",
                        "resolverStage": "hash-imports",
                        "targetCandidates": ["packages/app/src/config"]
                    }
                ]
            }
        }))?,
    )?;

    let output = Command::new(audit_core_bin())
        .arg("resolver-diagnostics-artifacts")
        .arg("--input")
        .arg(&input)
        .arg("--result-output")
        .arg(&result)
        .output()?;

    assert!(output.status.success());
    assert!(output.stdout.is_empty());
    let artifact: Value = serde_json::from_slice(&fs::read(&result)?)?;
    assert_eq!(
        artifact["capabilities"]["schemaVersion"],
        "resolver-capabilities.v1"
    );
    assert_eq!(
        artifact["diagnostics"]["schemaVersion"],
        "resolver-diagnostics.v1"
    );
    assert_eq!(
        artifact["diagnostics"]["summary"]["blockedCandidateHintCount"],
        1
    );
    Ok(())
}

fn audit_core_bin() -> &'static str {
    env!("CARGO_BIN_EXE_lumin-audit-core")
}
