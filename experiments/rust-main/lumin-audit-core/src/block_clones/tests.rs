use super::*;

fn token(value: &str, file: &str, start: usize, line: usize) -> BlockCloneToken {
    BlockCloneToken {
        value: value.to_string(),
        file: file.to_string(),
        start,
        end: start + 1,
        line,
        end_line: line,
        container: None,
    }
}

fn file(rel_file: &str, values: &[&str]) -> TokenizedFile {
    TokenizedFile {
        rel_file: rel_file.to_string(),
        tokens: values
            .iter()
            .enumerate()
            .map(|(index, value)| token(value, rel_file, index, index + 1))
            .collect(),
        skipped: None,
        diagnostics: vec![],
        token_limit_exceeded: false,
    }
}

fn request(files: Vec<TokenizedFile>, thresholds: Value) -> BlockClonesRequest {
    BlockClonesRequest {
        schema_version: BLOCK_CLONES_REQUEST_SCHEMA_VERSION.to_string(),
        generated: "2026-07-04T00:00:00.000Z".to_string(),
        root: "C:/repo".to_string(),
        include_tests: true,
        exclude: vec![],
        files,
        thresholds: Some(thresholds),
        incremental: Some(json!({
            "enabled": false,
            "reason": "disabled-by-flag",
        })),
    }
}

#[test]
fn builds_review_group_from_js_tokenized_files() -> Result<()> {
    let artifact = build_block_clones_artifact(request(
        vec![
            file("src/a.ts", &["A", "B", "C", "D", "E", "F"]),
            file("src/b.ts", &["A", "B", "C", "D", "E", "F"]),
        ],
        json!({
            "minTokens": 3,
            "minLines": 1,
            "minOccurrences": 2,
            "maxInstancesPerGroup": 20,
            "maxCandidateGroups": 100,
            "maxReviewGroups": 100,
            "maxMutedGroups": 100,
            "maxTokensPerFile": 200000,
        }),
    ))?;

    assert_eq!(artifact["schemaVersion"], "block-clones.v1");
    assert_eq!(artifact["policyVersion"], "block-clone-review-policy-v1");
    assert_eq!(artifact["summary"]["reviewGroupCount"], 1);
    assert_eq!(artifact["summary"]["mutedGroupCount"], 0);
    assert_eq!(artifact["groups"][0]["visibility"], "review");
    assert_eq!(
        artifact["groups"][0]["instances"]
            .as_array()
            .map_or(0, Vec::len),
        2
    );
    Ok(())
}

#[test]
fn mutes_same_file_repeats_without_deleting_group() -> Result<()> {
    let artifact = build_block_clones_artifact(request(
        vec![file(
            "src/a.ts",
            &[
                "A", "B", "C", "D", "E", "F", "X", "A", "B", "C", "D", "E", "F",
            ],
        )],
        json!({
            "minTokens": 3,
            "minLines": 1,
            "minOccurrences": 2,
            "maxInstancesPerGroup": 20,
            "maxCandidateGroups": 100,
            "maxReviewGroups": 100,
            "maxMutedGroups": 100,
            "maxTokensPerFile": 200000,
        }),
    ))?;

    assert_eq!(artifact["summary"]["reviewGroupCount"], 0);
    assert_eq!(artifact["summary"]["mutedGroupCount"], 1);
    assert_eq!(artifact["groups"][0]["visibility"], "muted");
    assert_eq!(artifact["groups"][0]["muteReason"], "same-file-repeat");
    Ok(())
}

#[test]
fn rejects_unknown_schema() {
    let mut request = request(vec![], json!({}));
    request.schema_version = "block-clones.future".to_string();
    let error = match build_block_clones_artifact(request) {
        Ok(_) => panic!("schema should reject"),
        Err(error) => error,
    };
    assert!(error.to_string().contains("unsupported schemaVersion"));
}
