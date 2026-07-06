use anyhow::Result;
use serde_json::{json, Value};
use std::fs;
use std::process::Command;

#[test]
fn cli_compare_repos_artifact_projects_summaries_and_deltas() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let left = temp.path().join("left");
    let right = temp.path().join("right");
    let output = temp.path().join("compare.json");
    let input = temp.path().join("request.json");
    fs::create_dir_all(&left)?;
    fs::create_dir_all(&right)?;

    fs::write(
        left.join("triage.json"),
        serde_json::to_vec(&json!({
            "summary": { "files": 4, "loc": 100, "buildSystem": "npm" }
        }))?,
    )?;
    fs::write(
        right.join("triage.json"),
        serde_json::to_vec(&json!({
            "files": 7,
            "loc": 140,
            "buildSystem": "pnpm"
        }))?,
    )?;
    fs::write(
        left.join("symbols.json"),
        serde_json::to_vec(&json!({
            "totalDefs": 20,
            "deadInProd": 5,
            "uses": {
                "resolvedInternal": 11,
                "external": 3,
                "unresolvedInternal": 2,
                "unresolvedInternalRatio": 0.25
            }
        }))?,
    )?;
    fs::write(
        right.join("symbols.json"),
        serde_json::to_vec(&json!({
            "totalDefs": 23,
            "deadInProd": 4,
            "uses": {
                "resolvedInternal": 12,
                "external": 4,
                "unresolvedInternal": 1,
                "unresolvedInternalRatio": 0.10
            }
        }))?,
    )?;
    fs::write(
        left.join("topology.json"),
        serde_json::to_vec(&json!({
            "summary": { "sccCount": 2, "typeOnlyEdges": 6 }
        }))?,
    )?;
    fs::write(
        right.join("topology.json"),
        serde_json::to_vec(&json!({
            "summary": { "sccCount": 1, "typeOnlyEdges": 9 }
        }))?,
    )?;
    fs::write(
        left.join("fix-plan.json"),
        serde_json::to_vec(&json!({
            "summary": { "SAFE_FIX": 3, "REVIEW_FIX": 2, "DEGRADED": 1, "MUTED": 4, "total": 10 },
            "meta": { "resolverBlindness": { "gate": "high" } }
        }))?,
    )?;
    fs::write(
        right.join("fix-plan.json"),
        serde_json::to_vec(&json!({
            "summary": { "SAFE_FIX": 5, "REVIEW_FIX": 1, "DEGRADED": 1, "MUTED": 6, "total": 13 },
            "meta": { "resolverBlindness": { "gate": "medium" } }
        }))?,
    )?;
    fs::write(right.join("runtime-evidence.json"), b"not-json")?;

    fs::write(
        &input,
        serde_json::to_vec(&json!({
            "schemaVersion": "lumin-compare-repos-producer-request.v1",
            "generated": "2026-07-06T00:00:00.000Z",
            "left": left,
            "right": right,
            "leftLabel": "before",
            "rightLabel": "after"
        }))?,
    )?;

    let status = Command::new(audit_core_bin())
        .arg("compare-repos-artifact")
        .arg("--input")
        .arg(&input)
        .arg("--result-output")
        .arg(&output)
        .status()?;
    assert!(status.success());

    let artifact: Value = serde_json::from_slice(&fs::read(&output)?)?;
    assert_eq!(artifact["meta"]["tool"], "compare-repos.mjs");
    assert_eq!(artifact["left"]["label"], "before");
    assert_eq!(artifact["right"]["label"], "after");
    assert_eq!(artifact["left"]["summaries"]["triage"]["files"], 4);
    assert_eq!(artifact["right"]["summaries"]["triage"]["files"], 7);
    assert_eq!(artifact["deltas"]["files"], 3);
    assert_eq!(artifact["deltas"]["loc"], 40);
    assert_eq!(artifact["deltas"]["totalDefs"], 3);
    assert_eq!(artifact["deltas"]["deadInProd"], -1);
    assert_eq!(artifact["deltas"]["runtimeSccs"], -1);
    assert_eq!(artifact["deltas"]["typeOnlyEdges"], 3);
    assert_eq!(artifact["deltas"]["safeFixes"], 2);
    assert_eq!(artifact["deltas"]["reviewFixes"], -1);
    assert_eq!(artifact["deltas"]["degraded"], 0);
    assert_eq!(artifact["deltas"]["muted"], 2);
    assert_eq!(artifact["deltas"]["unresolvedInternalRatio"], -0.15);
    assert!(artifact["right"]["artifactsFound"]
        .as_array()
        .is_some_and(|items| items.iter().any(|item| item == "runtime-evidence.json")));
    assert!(artifact["missingArtifacts"]["left"]
        .as_array()
        .is_some_and(|items| items.iter().any(|item| item == "runtime-evidence.json")));
    Ok(())
}

fn audit_core_bin() -> &'static str {
    env!("CARGO_BIN_EXE_lumin-audit-core")
}
