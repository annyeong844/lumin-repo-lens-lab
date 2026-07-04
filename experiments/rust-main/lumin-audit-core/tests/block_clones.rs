use anyhow::Result;
use serde_json::{json, Value};
use std::fs;
use std::process::Command;

#[test]
fn cli_block_clones_artifact_writes_result_file() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let input = temp.path().join("request.json");
    let result = temp.path().join("result.json");
    let values = ["A", "B", "C", "D"];
    fs::write(
        &input,
        serde_json::to_vec(&json!({
            "schemaVersion": "lumin-block-clones-producer-request.v1",
            "generated": "2026-07-04T00:00:00.000Z",
            "root": "C:/repo",
            "includeTests": true,
            "exclude": [],
            "files": [
                {
                    "relFile": "src/a.ts",
                    "tokens": tokens("src/a.ts", &values),
                    "skipped": null,
                    "diagnostics": [],
                    "tokenLimitExceeded": false
                },
                {
                    "relFile": "src/b.ts",
                    "tokens": tokens("src/b.ts", &values),
                    "skipped": null,
                    "diagnostics": [],
                    "tokenLimitExceeded": false
                }
            ],
            "thresholds": {
                "minTokens": 3,
                "minLines": 1,
                "minOccurrences": 2,
                "maxInstancesPerGroup": 20,
                "maxCandidateGroups": 100,
                "maxReviewGroups": 100,
                "maxMutedGroups": 100,
                "maxTokensPerFile": 200000
            },
            "incremental": {
                "enabled": false,
                "reason": "contract-probe"
            }
        }))?,
    )?;

    let output = Command::new(audit_core_bin())
        .arg("block-clones-artifact")
        .arg("--input")
        .arg(&input)
        .arg("--result-output")
        .arg(&result)
        .output()?;

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stdout.is_empty());
    let artifact: Value = serde_json::from_slice(&fs::read(&result)?)?;
    assert_eq!(artifact["schemaVersion"], "block-clones.v1");
    assert_eq!(artifact["summary"]["fileCount"], 2);
    assert_eq!(artifact["summary"]["reviewGroupCount"], 1);
    assert_eq!(artifact["groups"][0]["visibility"], "review");
    Ok(())
}

fn tokens(file: &str, values: &[&str]) -> Vec<Value> {
    values
        .iter()
        .enumerate()
        .map(|(index, value)| {
            json!({
                "value": value,
                "file": file,
                "start": index,
                "end": index + 1,
                "line": index + 1,
                "endLine": index + 1,
                "container": null
            })
        })
        .collect()
}

fn audit_core_bin() -> &'static str {
    env!("CARGO_BIN_EXE_lumin-audit-core")
}
