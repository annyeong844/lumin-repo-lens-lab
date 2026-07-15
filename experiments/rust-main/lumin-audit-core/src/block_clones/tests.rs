use super::groups::{signature_interval_starts, BlockCloneGroup, Instance};
use super::noise::apply_noise_policy;
use super::policy::normalize_thresholds;
use super::suffix_array::build_lcp_array;
use super::suffix_array::build_suffix_array;
use super::suffix_array::reference::build_suffix_array_doubling;
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

fn clone_group(id: &str, token_count: usize, files: &[&str]) -> BlockCloneGroup {
    BlockCloneGroup {
        id: id.to_string(),
        claim: "repeated-normalized-token-region".to_string(),
        confidence: "review-only".to_string(),
        token_count,
        line_count: 1,
        occurrence_count: files.len(),
        normalization_mode: "alpha-identifier".to_string(),
        reasons: vec![],
        instances: files
            .iter()
            .enumerate()
            .map(|(index, file)| Instance {
                file: (*file).to_string(),
                start_line: 1,
                end_line: 1,
                start_token: index * 10,
                end_token: index * 10 + 1,
                container: None,
            })
            .collect(),
        review_only: true,
        eligible_for_safe_fix: false,
        visibility: None,
        mute_reason: None,
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
fn bounds_containment_storage_by_actual_candidate_count() -> Result<()> {
    let artifact = build_block_clones_artifact(request(
        vec![
            file("src/a.ts", &["A", "B", "C", "D"]),
            file("src/b.ts", &["A", "B", "C", "D"]),
        ],
        json!({
            "minTokens": 3,
            "minLines": 1,
            "minOccurrences": 2,
            "maxInstancesPerGroup": 20,
            "maxCandidateGroups": u64::MAX,
            "maxReviewGroups": 100,
            "maxMutedGroups": 100,
            "maxTokensPerFile": 200000,
        }),
    ))?;

    assert_eq!(artifact["summary"]["reviewGroupCount"], 1);
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
fn classifies_noise_without_deleting_candidate_groups() {
    let thresholds = normalize_thresholds(Some(&json!({
        "maxCandidateGroups": 10,
        "maxReviewGroups": 10,
        "maxMutedGroups": 10,
        "maxGroups": 5,
    })));
    let result = apply_noise_policy(
        vec![
            clone_group(
                "mirror",
                100,
                &[
                    "tests/hook-event-store.test.mjs",
                    "tests/test-hook-event-store.mjs",
                ],
            ),
            clone_group("scaffold", 90, &["tests/test-a.mjs", "tests/test-b.mjs"]),
            clone_group("same-file", 80, &["_lib/a.mjs", "_lib/a.mjs"]),
            clone_group(
                "directory-collision",
                70,
                &["tests/auth/test-index.mjs", "tests/payments/index.test.mjs"],
            ),
            clone_group("engine", 60, &["_lib/a.mjs", "_lib/b.mjs"]),
        ],
        &thresholds,
    );

    assert_eq!(result.groups.len(), 5);
    assert_eq!(result.review_group_count, 1);
    assert_eq!(result.muted_group_count, 4);
    assert_eq!(
        result.muted_by_reason.get("node-vitest-mirror-pair"),
        Some(&1)
    );
    assert_eq!(result.muted_by_reason.get("same-file-repeat"), Some(&1));
    assert_eq!(result.muted_by_reason.get("test-scaffold-repeat"), Some(&2));
    assert!(!result.candidate_cap_saturated);
    assert!(!result.review_cap_saturated);
    assert!(!result.muted_cap_saturated);

    let by_id = result
        .groups
        .iter()
        .map(|group| (group.id.as_str(), group))
        .collect::<std::collections::BTreeMap<_, _>>();
    assert_eq!(
        by_id["mirror"].mute_reason.as_deref(),
        Some("node-vitest-mirror-pair")
    );
    assert_eq!(
        by_id["directory-collision"].mute_reason.as_deref(),
        Some("test-scaffold-repeat")
    );
    assert_eq!(by_id["engine"].visibility.as_deref(), Some("review"));
}

#[test]
fn preserves_review_groups_under_muted_cap_pressure_and_legacy_total_cap() {
    let groups = vec![
        clone_group(
            "same-file-large",
            300,
            &["src/noisy-fixture.ts", "src/noisy-fixture.ts"],
        ),
        clone_group(
            "test-scaffold-large",
            250,
            &["tests/test-a.mjs", "tests/test-b.mjs"],
        ),
        clone_group("review-small", 50, &["src/a.ts", "src/b.ts"]),
    ];
    let thresholds = normalize_thresholds(Some(&json!({
        "maxCandidateGroups": 10,
        "maxReviewGroups": 1,
        "maxMutedGroups": 1,
    })));
    let result = apply_noise_policy(groups.clone(), &thresholds);

    assert_eq!(result.groups.len(), 2);
    assert_eq!(result.groups[0].id, "review-small");
    assert_eq!(result.groups[0].visibility.as_deref(), Some("review"));
    assert_eq!(result.groups[1].visibility.as_deref(), Some("muted"));
    assert_eq!(result.review_group_count, 1);
    assert_eq!(result.muted_group_count, 1);
    assert!(!result.candidate_cap_saturated);
    assert!(!result.review_cap_saturated);
    assert!(result.muted_cap_saturated);

    let legacy_thresholds = normalize_thresholds(Some(&json!({
        "maxCandidateGroups": 10,
        "maxReviewGroups": 100,
        "maxMutedGroups": 100,
        "maxGroups": 2,
    })));
    let legacy = apply_noise_policy(groups, &legacy_thresholds);
    assert_eq!(legacy.groups.len(), 2);
    assert_eq!(legacy.groups[0].id, "review-small");
    assert_eq!(legacy.review_group_count, 1);
    assert_eq!(legacy.muted_group_count, 1);
}

#[test]
fn preserves_legacy_max_groups_in_artifact_thresholds() -> Result<()> {
    let artifact = build_block_clones_artifact(request(vec![], json!({ "maxGroups": 2 })))?;

    assert_eq!(artifact["thresholds"]["maxGroups"], 2);
    assert_eq!(
        artifact["noisePolicy"]["policyId"],
        "block-clone-noise-policy-v1"
    );
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

#[test]
fn suffix_array_matches_lexicographic_suffix_order() {
    let fixtures = [
        vec![],
        vec![1],
        vec![1, 1, 1, 1],
        vec![1, 2, 1, 2, -1, 1, 2, 1, 2, -2],
        vec![3, 1, 4, 1, 5, 9, 2, 6, 5, 3, 5],
        vec![i64::MIN, i64::MAX, 0, i64::MIN],
    ];
    for values in fixtures {
        assert_eq!(build_suffix_array(&values), reference_suffix_array(&values));
    }

    let mut seed = 0x5eed_u64;
    for len in 0..128usize {
        let values = (0..len)
            .map(|index| {
                seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
                if index > 0 && index % 17 == 0 {
                    -(index as i64)
                } else {
                    ((seed >> 32) % 9 + 1) as i64
                }
            })
            .collect::<Vec<_>>();
        assert_eq!(
            build_suffix_array(&values),
            reference_suffix_array(&values),
            "suffix order mismatch for len={len}"
        );
    }
}

#[test]
fn sais_matches_lexicographic_order_exhaustively_for_small_inputs() {
    for len in 0..=8u32 {
        for encoded in 0..3usize.pow(len) {
            let mut remaining = encoded;
            let values = (0..len)
                .map(|_| {
                    let value = (remaining % 3) as i64 - 1;
                    remaining /= 3;
                    value
                })
                .collect::<Vec<_>>();
            assert_eq!(
                build_suffix_array(&values),
                reference_suffix_array(&values),
                "suffix order mismatch for exhaustive input {values:?}"
            );
        }
    }
}

#[test]
fn lcp_interval_keys_match_exact_signature_identity() {
    let mut seed = 0xc10e_u64;
    for len in 2..96usize {
        let values = (0..len)
            .map(|index| {
                seed = seed
                    .wrapping_mul(2862933555777941757)
                    .wrapping_add(3037000493);
                if index > 0 && index % 19 == 0 {
                    -(index as i64)
                } else {
                    ((seed >> 33) % 7 + 1) as i64
                }
            })
            .collect::<Vec<_>>();
        let suffix_array = build_suffix_array(&values);
        let lcp = build_lcp_array(&values, &suffix_array);
        let interval_starts = signature_interval_starts(&lcp);
        for left in 1..lcp.len() {
            if lcp[left] == 0 {
                continue;
            }
            let left_signature =
                &values[suffix_array[left - 1]..suffix_array[left - 1] + lcp[left]];
            for right in 1..lcp.len() {
                if lcp[left] != lcp[right] {
                    continue;
                }
                let right_signature =
                    &values[suffix_array[right - 1]..suffix_array[right - 1] + lcp[right]];
                assert_eq!(
                    interval_starts[left] == interval_starts[right],
                    left_signature == right_signature,
                    "signature identity mismatch for len={len}, left={left}, right={right}"
                );
            }
        }
    }
}

#[test]
fn sais_matches_doubling_on_large_repetitive_input() {
    let mut seed = 0xdead_beef_cafe_f00d_u64;
    let block = (0..64)
        .map(|_| {
            seed ^= seed << 13;
            seed ^= seed >> 7;
            seed ^= seed << 17;
            ((seed >> 32) % 8 + 1) as i64
        })
        .collect::<Vec<_>>();
    let mut values = Vec::<i64>::new();
    for repetition in 0..64 {
        values.extend_from_slice(&block);
        values.push(-(repetition + 1));
    }
    assert_eq!(
        build_suffix_array(&values),
        build_suffix_array_doubling(&values)
    );
}

fn reference_suffix_array(values: &[i64]) -> Vec<usize> {
    let mut suffixes = (0..values.len()).collect::<Vec<_>>();
    suffixes.sort_by(|left, right| {
        values[*left..]
            .cmp(&values[*right..])
            .then_with(|| left.cmp(right))
    });
    suffixes
}
