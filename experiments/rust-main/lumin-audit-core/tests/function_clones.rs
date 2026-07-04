use anyhow::Result;
use serde_json::{json, Value};
use std::fs;
use std::process::Command;

#[test]
fn cli_function_clones_artifact_writes_result_file() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let input = temp.path().join("request.json");
    let result = temp.path().join("result.json");
    fs::write(
        &input,
        serde_json::to_vec(&json!({
            "schemaVersion": "lumin-function-clones-producer-request.v1",
            "generated": "2026-07-05T00:00:00.000Z",
            "root": "C:/repo",
            "includeTests": true,
            "exclude": [],
            "scope": "TS/JS including tests, top-level exported and file-local functions",
            "fileCount": 2,
            "facts": [
                function_fact("src/a.ts", "alpha", 1),
                function_fact("src/b.ts", "beta", 4)
            ],
            "diagnostics": [],
            "filesWithParseErrors": [],
            "filesWithReadErrors": []
        }))?,
    )?;

    let output = Command::new(audit_core_bin())
        .arg("function-clones-artifact")
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
    assert_eq!(artifact["schemaVersion"], "function-clones.v3");
    assert_eq!(artifact["meta"]["tool"], "build-function-clone-index.mjs");
    assert_eq!(artifact["meta"]["exactBodyGroupCount"], 1);
    assert_eq!(
        artifact["exactBodyGroups"][0]["identities"][0],
        "src/a.ts::alpha"
    );
    Ok(())
}

fn function_fact(file: &str, name: &str, line: i64) -> Value {
    json!({
        "kind": "function-body-fingerprint",
        "identity": format!("{file}::{name}"),
        "exportedName": name,
        "localName": name,
        "visibility": "exported",
        "exported": true,
        "ownerFile": file,
        "line": line,
        "endLine": line + 4,
        "bodyLineStart": line + 1,
        "bodyLineEnd": line + 3,
        "bodyLoc": 3,
        "declarationKind": "FunctionDeclaration",
        "functionKind": "FunctionDeclaration",
        "async": false,
        "generator": false,
        "paramCount": 1,
        "statementCount": 2,
        "exactBodyHash": "raw-a",
        "normalizedExactHash": "exact-a",
        "normalizedStructureHash": "structure-a",
        "normalizedSignatureHash": "sig-a",
        "signature": "fn(value)",
        "callTokens": ["fetchUser"],
        "source": "fresh-ast-pass",
        "scope": "scope",
        "confidence": "high"
    })
}

fn audit_core_bin() -> &'static str {
    env!("CARGO_BIN_EXE_lumin-audit-core")
}
