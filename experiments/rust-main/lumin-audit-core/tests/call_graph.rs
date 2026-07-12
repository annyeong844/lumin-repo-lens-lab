use anyhow::Result;
use serde_json::{json, Value};
use std::fs;
use std::process::Command;

#[test]
fn cli_call_graph_artifact_writes_result_file() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let input = temp.path().join("request.json");
    let result = temp.path().join("result.json");
    fs::write(
        &input,
        serde_json::to_vec(&json!({
            "schemaVersion": "lumin-call-graph-producer-request.v1",
            "generated": "2026-07-05T00:00:00.000Z",
            "root": "C:/repo",
            "fileCount": 2,
            "parseErrors": 0,
            "parseErrorDetails": [],
            "totalCallExpressions": 2,
            "totalDirectCalls": 1,
            "resolvedDirectCalls": 2,
            "typeOnlyResolved": 0,
            "callEdges": [
                {
                    "from": "C:/repo/src/b.ts",
                    "to": "C:/repo/src/a.ts",
                    "callee": "alpha",
                    "count": 2
                }
            ],
            "exportAliasMap": {
                "src/a.ts::alpha": "src/a.ts#FunctionDeclaration:7-37"
            },
            "boundedOutMemberCallsByFile": {
                "src/a.ts": 0,
                "src/b.ts": 0
            },
            "memberCallsByFile": {
                "src/a.ts": 0,
                "src/b.ts": 1
            },
            "semiDeadList": [],
            "semiDeadReactFiltered": 0,
            "prototypeCalls": []
        }))?,
    )?;

    let output = Command::new(audit_core_bin())
        .arg("call-graph-artifact")
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
    assert_eq!(artifact["meta"]["tool"], "build-call-graph.mjs");
    assert_eq!(artifact["summary"]["callEdges"], 1);
    assert_eq!(artifact["topCallees"][0]["file"], "src/a.ts");
    assert_eq!(artifact["callFanInByIdentity"]["src/a.ts::alpha"], 2);
    Ok(())
}

fn audit_core_bin() -> &'static str {
    env!("CARGO_BIN_EXE_lumin-audit-core")
}
