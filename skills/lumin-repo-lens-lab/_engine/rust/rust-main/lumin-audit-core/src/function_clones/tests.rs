use super::*;
use anyhow::Context;

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
    let mut facts = vec![
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
    ];
    facts.extend(noise_facts(2, 58));
    let artifact = artifact_for_facts(facts)?;

    assert_eq!(artifact["meta"]["nearFunctionCandidateCount"], 1);
    assert_eq!(artifact["meta"]["nearFunctionCandidateProjectionLimit"], 50);
    assert_eq!(
        artifact["meta"]["supports"]["nearFunctionBoundedRetrieval"],
        true
    );
    assert_eq!(
        artifact["candidateGenerationPolicy"]["mode"],
        "bounded-retrieval"
    );
    assert_eq!(
        artifact["candidateGenerationSummary"]["generatedUniquePairCount"],
        1
    );
    assert_eq!(artifact["candidateGenerationSummary"]["scoredPairCount"], 1);
    assert_eq!(
        artifact["nearFunctionCandidates"][0]["sharedCallTokens"][0],
        "fetchUser"
    );
    assert_eq!(
        artifact["nearFunctionCandidates"][0]["generationToken"],
        "fetchUser"
    );
    assert_eq!(
        artifact["nearFunctionCandidates"][0]["callTokenIdfScore"],
        1.0
    );
    assert_eq!(artifact["nearFunctionCandidates"][0]["score"], 0.875);
    assert_eq!(
        artifact["meta"]["thresholdPolicies"][0]["policyHash"],
        "sha256:6f524aeaefad2aa07badd8db3b841d2fec22d1228368bc5192387a5ea0116c54"
    );
    assert_eq!(
        artifact["meta"]["thresholdPolicies"][0]["thresholdHash"],
        "sha256:bea5f5cd6ce57db1800039b86f54d0ebc8b168b63aafeb3a9fbdc468a241ba29"
    );
    assert_eq!(
        artifact["meta"]["thresholdPolicies"][0]["scoreFormulaVersion"],
        "function-clone-near-score-idf-sum-v1"
    );
    Ok(())
}

#[test]
fn low_discrimination_call_buckets_are_skipped_with_visible_work_estimates() -> Result<()> {
    let facts = (0..8)
        .map(|index| unique_fact(index, &["commonLookup"]))
        .collect::<Vec<_>>();
    let artifact = artifact_for_facts(facts)?;

    assert_eq!(artifact["meta"]["nearFunctionCandidateCount"], 0);
    assert_eq!(
        artifact["candidateGenerationSummary"]["generatedUniquePairCount"],
        0
    );
    assert_eq!(artifact["skippedLowDiscriminationBucketCount"], 1);
    assert_eq!(artifact["skippedLowDiscriminationRawPairEstimate"], 28);
    assert_eq!(
        artifact["skippedLowDiscriminationBuckets"][0]["token"],
        "commonLookup"
    );
    assert_eq!(
        artifact["skippedLowDiscriminationBuckets"][0]["postingCount"],
        8
    );
    Ok(())
}

#[test]
fn retained_tokens_generate_pairs_while_low_idf_overlap_remains_scoring_evidence() -> Result<()> {
    let mut facts = vec![
        fact_with_calls(
            "src/a.ts",
            "loadRareAlpha",
            1,
            "exact-a",
            "structure-a",
            &["commonLookup", "rareLookup"],
        ),
        fact_with_calls(
            "src/b.ts",
            "loadRareBeta",
            8,
            "exact-b",
            "structure-b",
            &["commonLookup", "rareLookup"],
        ),
    ];
    for index in 2..60 {
        let token = format!("noiseToken{index}");
        facts.push(unique_fact(index, &["commonLookup", token.as_str()]));
    }
    let artifact = artifact_for_facts(facts)?;
    let candidate = &artifact["nearFunctionCandidates"][0];
    let shared = candidate["sharedSignificantCallTokens"]
        .as_array()
        .context("shared token evidence must be an array")?;

    assert_eq!(candidate["generationToken"], "rareLookup");
    assert!(shared
        .iter()
        .any(|token| { token["token"] == "commonLookup" && token["retained"] == false }));
    assert!(shared
        .iter()
        .any(|token| { token["token"] == "rareLookup" && token["retained"] == true }));
    Ok(())
}

#[test]
fn pairs_shared_by_multiple_retained_tokens_are_generated_once() -> Result<()> {
    let mut facts = vec![
        fact_with_calls(
            "src/a.ts",
            "loadRareAlpha",
            1,
            "exact-a",
            "structure-a",
            &["rareAlpha", "rareBeta"],
        ),
        fact_with_calls(
            "src/b.ts",
            "loadRareBeta",
            8,
            "exact-b",
            "structure-b",
            &["rareAlpha", "rareBeta"],
        ),
    ];
    facts.extend(noise_facts(2, 58));
    let artifact = artifact_for_facts(facts)?;

    assert_eq!(
        artifact["candidateGenerationSummary"]["retainedCallTokenBucketCount"],
        2
    );
    assert_eq!(
        artifact["candidateGenerationSummary"]["retainedRawPairEstimate"],
        2
    );
    assert_eq!(
        artifact["candidateGenerationSummary"]["generatedUniquePairCount"],
        1
    );
    assert_eq!(
        artifact["nearFunctionCandidates"]
            .as_array()
            .context("nearFunctionCandidates must be an array")?
            .len(),
        1
    );
    Ok(())
}

#[test]
fn compatibility_partitions_avoid_scoring_incompatible_pairs() -> Result<()> {
    let mut facts = (0..4)
        .map(|index| unique_fact(index, &["rareLookup"]))
        .collect::<Vec<_>>();
    facts[2]["async"] = json!(true);
    facts[3]["paramCount"] = json!(4);
    facts.extend(noise_facts(4, 97));
    let artifact = artifact_for_facts(facts)?;
    let summary = &artifact["candidateGenerationSummary"];

    assert_eq!(summary["retainedRawPairEstimate"], 6);
    assert_eq!(summary["generatedUniquePairCount"], 1);
    assert_eq!(summary["scoredPairCount"], 1);
    assert_eq!(
        summary["compatibilitySkippedRawPairEstimateByReason"]["asyncMismatch"],
        3
    );
    assert_eq!(
        summary["compatibilitySkippedRawPairEstimateByReason"]["parameterCountDelta"],
        2
    );
    Ok(())
}

#[test]
fn near_candidate_count_is_uncapped_while_projection_stays_bounded() -> Result<()> {
    let mut facts = Vec::new();
    for pair_index in 0..60 {
        let token = format!("rareCloneToken{pair_index}");
        let alpha = format!("clone{pair_index}Alpha");
        let beta = format!("clone{pair_index}Beta");
        facts.push(unique_named_fact(pair_index * 2, &alpha, &[token.as_str()]));
        facts.push(unique_named_fact(
            pair_index * 2 + 1,
            &beta,
            &[token.as_str()],
        ));
    }
    let artifact = artifact_for_facts(facts)?;

    assert_eq!(artifact["meta"]["nearFunctionCandidateCount"], 60);
    assert_eq!(
        artifact["candidateGenerationSummary"]["generatedUniquePairCount"],
        60
    );
    assert_eq!(
        artifact["nearFunctionCandidates"]
            .as_array()
            .context("nearFunctionCandidates must be an array")?
            .len(),
        50
    );
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

fn artifact_for_facts(facts: Vec<Value>) -> Result<Value> {
    build_function_clones_artifact(FunctionClonesRequest {
        schema_version: FUNCTION_CLONES_REQUEST_SCHEMA_VERSION.to_string(),
        generated: "2026-07-05T00:00:00.000Z".to_string(),
        root: "C:/repo".to_string(),
        include_tests: true,
        exclude: vec![],
        scope: "scope".to_string(),
        observed_at: None,
        file_count: facts.len(),
        facts,
        diagnostics: vec![],
        files_with_parse_errors: vec![],
        files_with_read_errors: vec![],
        incremental: None,
    })
}

fn noise_facts(start: usize, count: usize) -> Vec<Value> {
    (start..start + count)
        .map(|index| {
            let token = format!("noiseToken{index}");
            unique_fact(index, &[token.as_str()])
        })
        .collect()
}

fn unique_fact(index: usize, calls: &[&str]) -> Value {
    let name = format!("uniqueFunction{index}");
    unique_named_fact(index, &name, calls)
}

fn unique_named_fact(index: usize, name: &str, calls: &[&str]) -> Value {
    let file = format!("src/fixture-{index}.ts");
    let exact_hash = format!("exact-{index}");
    let structure_hash = format!("structure-{index}");
    fact_with_calls(
        &file,
        name,
        (index * 10 + 1) as i64,
        &exact_hash,
        &structure_hash,
        calls,
    )
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
