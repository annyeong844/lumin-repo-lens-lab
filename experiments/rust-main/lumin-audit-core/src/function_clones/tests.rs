use super::*;

#[test]
fn builds_function_clone_groups_from_js_facts() -> Result<()> {
    let artifact = build_function_clones_artifact(FunctionClonesRequest {
        schema_version: FUNCTION_CLONES_REQUEST_SCHEMA_VERSION.to_string(),
        generated: "2026-07-05T00:00:00.000Z".to_string(),
        root: "C:/repo".to_string(),
        include_tests: true,
        exclude: vec![],
        scope: "TS/JS including tests, top-level exported and file-local functions".to_string(),
        observed_at: None,
        file_count: 2,
        facts: vec![
            fact("src/a.ts", "alpha", 1, "exact-a", "structure-a", "sig-a"),
            fact("src/b.ts", "beta", 4, "exact-a", "structure-a", "sig-a"),
        ],
        diagnostics: vec![],
        files_with_parse_errors: vec![],
        files_with_read_errors: vec![],
        incremental: None,
    })?;

    assert_eq!(artifact["schemaVersion"], FUNCTION_CLONE_SCHEMA_VERSION);
    assert_eq!(artifact["meta"]["tool"], "build-function-clone-index.mjs");
    assert_eq!(artifact["meta"]["complete"], true);
    assert_eq!(artifact["meta"]["exactBodyGroupCount"], 1);
    assert_eq!(artifact["meta"]["structureGroupCount"], 1);
    assert_eq!(artifact["meta"]["signatureGroupCount"], 1);
    assert_eq!(
        artifact["facts"][0]["observedAt"],
        "2026-07-05T00:00:00.000Z"
    );
    assert_eq!(
        artifact["exactBodyGroups"][0]["identities"][0],
        "src/a.ts::alpha"
    );
    assert_eq!(
        artifact["signatureGroups"][0]["reason"],
        "same normalized exported function type signature; review cue only; not proof of semantic equivalence or a merge recommendation"
    );
    Ok(())
}

#[test]
fn near_candidates_skip_already_grouped_facts_and_score_remaining_pairs() -> Result<()> {
    let artifact = build_function_clones_artifact(FunctionClonesRequest {
        schema_version: FUNCTION_CLONES_REQUEST_SCHEMA_VERSION.to_string(),
        generated: "2026-07-05T00:00:00.000Z".to_string(),
        root: "C:/repo".to_string(),
        include_tests: true,
        exclude: vec![],
        scope: "scope".to_string(),
        observed_at: None,
        file_count: 2,
        facts: vec![
            fact_with_calls(
                "src/a.ts",
                "loadUserAlpha",
                1,
                "exact-a",
                "structure-a",
                &["fetchUser", "parseBody"],
            ),
            fact_with_calls(
                "src/b.ts",
                "loadUserBeta",
                8,
                "exact-b",
                "structure-b",
                &["fetchUser", "parseBody"],
            ),
        ],
        diagnostics: vec![],
        files_with_parse_errors: vec![],
        files_with_read_errors: vec![],
        incremental: None,
    })?;

    assert_eq!(artifact["meta"]["nearFunctionCandidateCount"], 1);
    assert_eq!(
        artifact["nearFunctionCandidates"][0]["sharedCallTokens"][0],
        "fetchUser"
    );
    assert_eq!(artifact["nearFunctionCandidates"][0]["score"], 0.875);
    Ok(())
}

#[test]
fn parse_or_read_errors_make_artifact_incomplete() -> Result<()> {
    let artifact = build_function_clones_artifact(FunctionClonesRequest {
        schema_version: FUNCTION_CLONES_REQUEST_SCHEMA_VERSION.to_string(),
        generated: "2026-07-05T00:00:00.000Z".to_string(),
        root: "C:/repo".to_string(),
        include_tests: false,
        exclude: vec![json!("dist")],
        scope: "scope".to_string(),
        observed_at: Some("2026-07-05T01:00:00.000Z".to_string()),
        file_count: 1,
        facts: vec![],
        diagnostics: vec![json!({
            "kind": "function-clone-diagnostic",
            "code": "parse-error",
            "severity": "error",
            "file": "bad.ts",
            "message": "bad",
        })],
        files_with_parse_errors: vec![json!({"file": "bad.ts", "message": "bad"})],
        files_with_read_errors: vec![],
        incremental: Some(json!({"enabled": true})),
    })?;

    assert_eq!(artifact["meta"]["complete"], false);
    assert_eq!(artifact["meta"]["includeTests"], false);
    assert_eq!(artifact["meta"]["exclude"][0], "dist");
    assert_eq!(artifact["meta"]["incremental"]["enabled"], true);
    assert_eq!(artifact["diagnostics"][0]["file"], "bad.ts");
    Ok(())
}

#[test]
fn rejects_unknown_schema() {
    let error = match build_function_clones_artifact(FunctionClonesRequest {
        schema_version: "future".to_string(),
        generated: "2026-07-05T00:00:00.000Z".to_string(),
        root: "C:/repo".to_string(),
        include_tests: true,
        exclude: vec![],
        scope: "scope".to_string(),
        observed_at: None,
        file_count: 0,
        facts: vec![],
        diagnostics: vec![],
        files_with_parse_errors: vec![],
        files_with_read_errors: vec![],
        incremental: None,
    }) {
        Ok(_) => panic!("schema should reject"),
        Err(error) => error,
    };

    assert!(error.to_string().contains("unsupported schemaVersion"));
}

fn fact(
    file: &str,
    name: &str,
    line: i64,
    exact_hash: &str,
    structure_hash: &str,
    signature_hash: &str,
) -> Value {
    let mut value = fact_with_calls(file, name, line, exact_hash, structure_hash, &["fetchUser"]);
    if let Value::Object(object) = &mut value {
        object.insert("normalizedSignatureHash".to_string(), json!(signature_hash));
        object.insert("signature".to_string(), json!("fn(value)"));
    }
    value
}

fn fact_with_calls(
    file: &str,
    name: &str,
    line: i64,
    exact_hash: &str,
    structure_hash: &str,
    calls: &[&str],
) -> Value {
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
        "exactBodyHash": format!("raw-{exact_hash}"),
        "normalizedExactHash": exact_hash,
        "normalizedStructureHash": structure_hash,
        "callTokens": calls,
        "source": "fresh-ast-pass",
        "scope": "scope",
        "confidence": "high",
    })
}
