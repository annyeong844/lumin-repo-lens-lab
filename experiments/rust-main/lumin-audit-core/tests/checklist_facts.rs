use anyhow::Result;
use serde_json::{json, Value};
use std::fs;
use std::process::Command;

#[test]
fn cli_checklist_facts_artifact_writes_result_file() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let input_path = temp.path().join("request.json");
    let output_path = temp.path().join("result.json");
    fs::write(
        &input_path,
        serde_json::to_vec(&json!({
            "schemaVersion": "lumin-checklist-facts-producer-request.v1",
            "generated": "2026-07-04T00:00:00.000Z",
            "root": "C:/repo",
            "filesScanned": 1,
            "inputs": {
                "topology": {
                    "summary": { "internalEdges": 100, "sccCount": 0, "maxSccSize": 0, "lens": "runtime" },
                    "crossSubmoduleEdges": [
                        { "from": "root", "to": "_lib", "count": 60 },
                        { "from": "tests", "to": "_lib", "count": 20 }
                    ],
                    "sccs": []
                },
                "fixPlan": {
                    "summary": { "SAFE_FIX": 11, "REVIEW_FIX": 1, "DEGRADED": 0, "MUTED": 0, "total": 12 }
                },
                "triage": {
                    "boundaries": [{ "rule": "no-restricted-imports" }]
                },
                "barrels": { "mode": "single-package" }
            },
            "astFacts": {
                "functionSize": {
                    "parseErrors": 0,
                    "entries": [
                        { "file": "src/huge.ts", "line": 1, "name": "huge", "loc": 160, "fileRole": "production" }
                    ]
                },
                "silentCatch": {
                    "analysis": "oxc-ast-catch-clause",
                    "parseErrors": 0,
                    "sites": [],
                    "documentedSites": [],
                    "anonymousSites": [],
                    "nonEmptyAnonymousSites": [],
                    "unusedParamSites": []
                }
            }
        }))?,
    )?;

    let status = Command::new(audit_core_bin())
        .arg("checklist-facts-artifact")
        .arg("--input")
        .arg(&input_path)
        .arg("--result-output")
        .arg(&output_path)
        .status()?;
    assert!(status.success());

    let artifact: Value = serde_json::from_slice(&fs::read(&output_path)?)?;
    assert_eq!(artifact["meta"]["tool"], "checklist-facts.mjs");
    assert_eq!(artifact["A2_function_size"]["gate"], "watch");
    assert_eq!(artifact["A5_decoupling_ratio"]["rawGate"], "fix");
    assert_eq!(artifact["A5_decoupling_ratio"]["gate"], "ok");
    assert_eq!(artifact["B3_dead_code"]["gate"], "fix");
    assert_eq!(artifact["E2_silent_catch"]["gate"], "ok");
    assert!(artifact["A2_function_size"]["_citation_hint"]
        .as_str()
        .is_some_and(|hint| hint.starts_with("[grounded,")));
    assert!(artifact["_not_computed"]
        .as_array()
        .is_some_and(|entries| entries.len() >= 20));
    Ok(())
}

fn audit_core_bin() -> &'static str {
    env!("CARGO_BIN_EXE_lumin-audit-core")
}
