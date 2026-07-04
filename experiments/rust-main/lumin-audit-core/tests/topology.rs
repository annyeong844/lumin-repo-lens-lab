use anyhow::{Context, Result};
use serde_json::{json, Map, Value};
use std::fs;
use std::process::Command;

#[test]
fn cli_topology_artifact_writes_result_file() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let root = temp.path().join("repo");
    let input = temp.path().join("request.json");
    let result = temp.path().join("result.json");
    fs::create_dir_all(&root)?;
    let index = root.join("src").join("index.ts");
    let dep = root.join("src").join("dep.ts");
    fs::create_dir_all(index.parent().context("index parent")?)?;
    let index_key = index.to_string_lossy().to_string();
    let dep_key = dep.to_string_lossy().to_string();
    let mut source_entries = Map::new();
    source_entries.insert(
        index_key.clone(),
        json!({
            "loc": 1,
            "edges": [{ "to": dep_key }],
            "externalCount": 0,
            "unresolvedCount": 0,
            "parseError": false
        }),
    );
    source_entries.insert(
        dep_key.clone(),
        json!({
            "loc": 1,
            "edges": [],
            "externalCount": 0,
            "unresolvedCount": 0,
            "parseError": false
        }),
    );

    fs::write(
        &input,
        serde_json::to_vec(&json!({
            "schemaVersion": "lumin-topology-producer-request.v1",
            "generated": "2026-07-04T00:00:00.000Z",
            "root": root,
            "mode": "single-package",
            "includeTypeEdges": false,
            "files": [index_key, dep_key],
            "sourceEntries": source_entries,
            "performance": { "filesCollected": 2 },
            "rustMetadata": {}
        }))?,
    )?;

    let output = Command::new(audit_core_bin())
        .arg("topology-artifact")
        .arg("--input")
        .arg(&input)
        .arg("--result-output")
        .arg(&result)
        .output()?;

    assert!(output.status.success());
    assert!(output.stdout.is_empty());
    let artifact: Value = serde_json::from_slice(&fs::read(&result)?)?;
    assert_eq!(artifact["meta"]["tool"], "m2s1-topology.mjs");
    assert_eq!(artifact["summary"]["files"], 2);
    assert_eq!(artifact["summary"]["internalEdges"], 1);
    assert_eq!(artifact["edges"][0]["typeOnly"], false);
    Ok(())
}

fn audit_core_bin() -> &'static str {
    env!("CARGO_BIN_EXE_lumin-audit-core")
}
