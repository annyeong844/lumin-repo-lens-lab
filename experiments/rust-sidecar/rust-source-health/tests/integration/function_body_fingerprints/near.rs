use anyhow::{Context, Result};
use serde_json::Value;

use crate::artifact::analyze_file;

use super::helpers::identity_list_contains;

#[test]
fn function_body_clone_groups_include_ts_style_near_candidates() -> Result<()> {
    let mut source = String::new();
    for index in 0..64 {
        source.push_str(&format!("pub fn filler_{index}() -> usize {{ {index} }}\n"));
    }
    source.push_str(
        r#"
pub fn load_user_profile(input: &str) -> usize {
    let parsed = input.trim();
    let normalized = sanitize(parsed);
    let loaded = fetch_user(normalized);
    loaded.len()
}

pub fn load_user_settings(raw: &str) -> usize {
    let cleaned = raw.trim();
    let ready = sanitize(cleaned);
    let fetched = fetch_user(ready);
    if fetched.is_empty() {
        return 0;
    }
    fetched.len()
}
"#,
    );
    let artifact = analyze_file("src/lib.rs", &source);

    assert_eq!(artifact["summary"]["functionBodyFingerprints"], 66);
    assert_eq!(artifact["summary"]["functionCloneExactBodyGroups"], 0);
    assert_eq!(artifact["summary"]["functionCloneStructureGroups"], 0);
    assert_eq!(artifact["summary"]["functionCloneNearCandidates"], 1);

    let groups = &artifact["functionCloneGroups"];
    assert_eq!(
        groups["policy"]["nearCandidatePolicy"]["policyId"],
        "function-clone-near-policy"
    );
    assert_eq!(
        groups["policy"]["nearCandidatePolicy"]["policyVersion"],
        "function-clone-near-policy-v1"
    );
    assert_eq!(
        groups["policy"]["nearCandidatePolicy"]["calibrationVersion"],
        "rust-function-clone-near-calibration.v7"
    );
    assert_eq!(
        groups["policy"]["nearCandidatePolicy"]["retrievalContractVersion"],
        "function-clone-near-retrieval.v1"
    );
    assert_eq!(
        groups["policy"]["nearCandidatePolicy"]["candidateGenerationMode"],
        "bounded-retrieval"
    );
    assert_eq!(
        groups["policy"]["nearCandidatePolicy"]["candidateCountScope"],
        "scored-candidates-from-retained-retrieval-evidence"
    );
    assert_eq!(
        groups["candidateGenerationPolicy"]["mode"],
        "bounded-retrieval"
    );
    assert_eq!(
        groups["candidateGenerationPolicy"]["retrievalContractVersion"],
        "function-clone-near-retrieval.v1"
    );
    assert_eq!(
        groups["candidateGenerationPolicy"]["candidateCountScope"],
        "scored-candidates-from-retained-retrieval-evidence"
    );
    assert_eq!(
        groups["candidateGenerationPolicy"]["pairDedupe"],
        "ordered-shared-retained-token"
    );
    assert_eq!(
        groups["policy"]["nearCandidatePolicy"]["minSignificantCallTokenLen"],
        4
    );
    assert_eq!(
        groups["policy"]["nearCandidatePolicy"]["minSingleTokenIdf"],
        3.0
    );
    assert_eq!(
        groups["policy"]["nearCandidatePolicy"]["callIdfSaturation"],
        6.0
    );
    assert_eq!(
        groups["policy"]["nearCandidatePolicy"]["minCallTokenIdfScore"],
        0.5
    );
    assert_eq!(groups["supports"]["nearFunctionCandidates"], true);
    assert_eq!(groups["supports"]["functionSignatureGroups"], true);
    assert_eq!(groups["supports"]["generatedFileEvidence"], true);
    assert_eq!(groups["supports"]["semanticEquivalence"], false);
    assert_eq!(
        groups["supports"]["normalizedVersion"],
        "rust-function-body.normalized.v3"
    );
    assert_eq!(
        groups["supports"]["functionSignatureNormalizedVersion"],
        "rust-function-signature.normalized.v1"
    );
    assert!(
        groups["policy"]["nearCandidatePolicy"]["suppressedGenericCallTokens"]
            .as_array()
            .is_some_and(|tokens| tokens.iter().any(|token| token == "to_string")
                && tokens.iter().any(|token| token == "unwrap")
                && tokens.iter().any(|token| token == "collect")
                && tokens.iter().any(|token| token == "Some")
                && tokens.iter().any(|token| token == "None")
                && tokens.iter().any(|token| token == "Ok")
                && tokens.iter().any(|token| token == "Err")
                && tokens.iter().any(|token| token == "format"))
    );
    assert!(
        groups["policy"]["nearCandidatePolicy"]["requiredMatchingQualifiers"]
            .as_array()
            .is_some_and(
                |qualifiers| qualifiers.iter().any(|qualifier| qualifier == "async")
                    && qualifiers.iter().any(|qualifier| qualifier == "unsafe")
                    && qualifiers.iter().any(|qualifier| qualifier == "const")
            )
    );
    assert_eq!(
        groups["policy"]["nearCandidatePolicy"]["minNearScore"],
        0.62
    );

    let candidate = &groups["nearFunctionCandidates"][0];
    assert_eq!(candidate["kind"], "near-function-candidate");
    assert_eq!(candidate["risk"], "review-only");
    assert_eq!(candidate["generatedOnly"], false);
    assert!(identity_list_contains(
        candidate,
        "src/lib.rs::load_user_profile"
    ));
    assert!(identity_list_contains(
        candidate,
        "src/lib.rs::load_user_settings"
    ));
    assert!(candidate["score"]
        .as_f64()
        .is_some_and(|score| score >= 0.62));
    assert_eq!(candidate["callTokenJaccard"], 1.0);
    assert!(candidate["sharedCallTokenIdfSum"]
        .as_f64()
        .is_some_and(|score| score >= 6.0));
    assert_eq!(candidate["callTokenIdfScore"], 1.0);
    assert_eq!(candidate["nameTokenJaccard"], 0.5);
    assert!(candidate["sharedCallTokens"]
        .as_array()
        .is_some_and(|tokens| tokens.iter().any(|token| token == "sanitize")
            && tokens.iter().any(|token| token == "fetch_user")));
    assert!(candidate["sharedNameTokens"]
        .as_array()
        .is_some_and(|tokens| tokens.iter().any(|token| token == "load")
            && tokens.iter().any(|token| token == "user")));
    assert!(candidate["reason"]
        .as_str()
        .is_some_and(|reason| reason.contains("not proof of semantic equivalence")));

    Ok(())
}

#[test]
fn function_body_near_candidates_skip_debug_formatter_boilerplate_pairs() {
    let mut source = String::new();
    for index in 0..64 {
        source.push_str(&format!("pub fn filler_{index}() -> usize {{ {index} }}\n"));
    }
    source.push_str(
        r#"
pub struct Alpha {
    id: usize,
}

impl std::fmt::Debug for Alpha {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("Alpha")
            .field("id", &self.id)
            .finish_non_exhaustive()
    }
}

pub struct Beta {
    id: usize,
}

impl std::fmt::Debug for Beta {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("Beta")
            .field("id", &self.id)
            .finish_non_exhaustive()
    }
}
"#,
    );

    let artifact = analyze_file("src/lib.rs", &source);
    let groups = &artifact["functionCloneGroups"];

    assert_eq!(artifact["summary"]["functionCloneNearCandidates"], 0);
    assert_eq!(
        groups["candidateGenerationSummary"]["debugFormatterBoilerplateSkippedPairCount"],
        1
    );
    assert!(near_candidates(&artifact).iter().all(|candidate| {
        !candidate["identities"]
            .as_array()
            .is_some_and(|identities| {
                identities.iter().all(|identity| {
                    identity
                        .as_str()
                        .is_some_and(|identity| identity.contains("Debug#fmt"))
                })
            })
    }));
}

#[test]
fn function_body_near_candidates_keep_formatter_named_tokens_outside_debug_impls() -> Result<()> {
    let mut source = String::new();
    for index in 0..64 {
        source.push_str(&format!("pub fn filler_{index}() -> usize {{ {index} }}\n"));
    }
    source.push_str(
        r#"
pub fn project_alpha(input: &Domain) -> usize {
    let value = input.field("alpha");
    let rendered = input.finish_non_exhaustive(value);
    rendered.len()
}

pub fn project_beta(input: &Domain) -> usize {
    let value = input.field("beta");
    let rendered = input.finish_non_exhaustive(value);
    if rendered.is_empty() {
        return 0;
    }
    rendered.len()
}
"#,
    );

    let artifact = analyze_file("src/lib.rs", &source);
    let candidate = near_candidates(&artifact)
        .iter()
        .find(|candidate| {
            identity_list_contains(candidate, "src/lib.rs::project_alpha")
                && identity_list_contains(candidate, "src/lib.rs::project_beta")
        })
        .context("formatter-named non-Debug near candidate")?;

    assert_eq!(
        artifact["functionCloneGroups"]["candidateGenerationSummary"]
            ["debugFormatterBoilerplateSkippedPairCount"],
        0
    );
    assert!(candidate["sharedCallTokens"]
        .as_array()
        .is_some_and(|tokens| tokens.iter().any(|token| token == "field")
            && tokens.iter().any(|token| token == "finish_non_exhaustive")));

    Ok(())
}

#[test]
fn function_body_near_candidates_drop_low_idf_single_token_pairs() {
    let artifact = analyze_file(
        "src/lib.rs",
        r#"
pub fn verify_alpha(input: &str) -> usize {
    assert!(input.len() > 0);
    input.len()
}

pub fn verify_beta(input: &str) -> usize {
    assert!(input.len() < 10);
    if input.is_empty() {
        return 0;
    }
    input.len()
}
"#,
    );

    assert_eq!(artifact["summary"]["functionBodyFingerprints"], 2);
    assert_eq!(artifact["summary"]["functionCloneExactBodyGroups"], 0);
    assert_eq!(artifact["summary"]["functionCloneStructureGroups"], 0);
    assert_eq!(artifact["summary"]["functionCloneNearCandidates"], 0);
    assert_eq!(
        artifact["functionCloneGroups"]["nearFunctionCandidates"]
            .as_array()
            .map(Vec::len),
        Some(0)
    );
    assert_eq!(
        artifact["functionCloneGroups"]["skippedLowDiscriminationBucketCount"],
        1
    );
    assert!(
        artifact["functionCloneGroups"]["skippedLowDiscriminationRawPairEstimate"]
            .as_u64()
            .is_some_and(|estimate| estimate > 0)
    );
    assert_eq!(
        artifact["functionCloneGroups"]["skippedLowDiscriminationPairEstimateKind"],
        "raw-bucket-pairs-may-double-count-pairs-shared-by-multiple-skipped-tokens"
    );
    let skipped = &artifact["functionCloneGroups"]["skippedLowDiscriminationBuckets"][0];
    assert_eq!(skipped["token"], "assert");
    assert_eq!(skipped["reason"], "below-min-single-token-idf");
    assert!(skipped["rawPairEstimate"]
        .as_u64()
        .is_some_and(|estimate| estimate > 0));
}

#[test]
fn function_body_near_candidates_keep_high_idf_single_token_pairs() -> Result<()> {
    let mut source = String::new();
    for index in 0..64 {
        source.push_str(&format!("pub fn filler_{index}() -> usize {{ {index} }}\n"));
    }
    source.push_str(
        r#"
pub fn parse_user_alpha(input: usize) -> usize {
    let value = unwrap_switch(input);
    if value > 10 {
        return value + 1;
    }
    value + 2
}

pub fn parse_user_beta(input: usize) -> usize {
    let value = unwrap_switch(input);
    if value < 20 {
        return value + 3;
    }
    value + 4
}
"#,
    );

    let artifact = analyze_file("src/lib.rs", &source);
    let candidates = near_candidates(&artifact);
    let candidate = candidates
        .iter()
        .find(|candidate| {
            identity_list_contains(candidate, "src/lib.rs::parse_user_alpha")
                && identity_list_contains(candidate, "src/lib.rs::parse_user_beta")
        })
        .context("high-idf single-token near candidate")?;

    assert_eq!(artifact["summary"]["functionBodyFingerprints"], 66);
    assert!(artifact["summary"]["functionCloneNearCandidates"]
        .as_u64()
        .is_some_and(|count| count >= 1));
    assert_eq!(
        candidate["sharedCallTokens"].as_array().map(Vec::len),
        Some(1)
    );
    assert!(candidate["sharedCallTokens"]
        .as_array()
        .is_some_and(|tokens| tokens.iter().any(|token| token == "unwrap_switch")));
    assert!(candidate["sharedCallTokenIdfSum"]
        .as_f64()
        .is_some_and(|score| score >= 3.0));
    assert!(candidate["callTokenIdfScore"]
        .as_f64()
        .is_some_and(|score| (0.5..1.0).contains(&score)));

    Ok(())
}

#[test]
fn function_body_near_candidates_keep_pairs_that_share_low_and_high_idf_tokens() -> Result<()> {
    let mut source = String::new();
    for index in 0..64 {
        source.push_str(&format!("pub fn filler_{index}() -> usize {{ {index} }}\n"));
    }
    for index in 0..64 {
        source.push_str(&format!(
            "pub fn assert_filler_{index}(input: usize) -> usize {{ assert!(input != {index}); input + {index} }}\n"
        ));
    }
    source.push_str(
        r#"
pub fn parse_user_alpha(input: usize) -> usize {
    assert!(input > 0);
    let value = unwrap_switch(input);
    if value > 10 {
        return value + 1;
    }
    value + 2
}

pub fn parse_user_beta(input: usize) -> usize {
    assert!(input < 100);
    let value = unwrap_switch(input);
    if value < 20 {
        return value + 3;
    }
    value + 4
}
"#,
    );

    let artifact = analyze_file("src/lib.rs", &source);
    let candidates = near_candidates(&artifact);
    let candidate = candidates
        .iter()
        .find(|candidate| {
            identity_list_contains(candidate, "src/lib.rs::parse_user_alpha")
                && identity_list_contains(candidate, "src/lib.rs::parse_user_beta")
        })
        .context("high-idf retained token should still generate pair")?;

    assert_eq!(
        artifact["functionCloneGroups"]["skippedLowDiscriminationBucketCount"],
        1
    );
    assert_eq!(
        artifact["functionCloneGroups"]["skippedLowDiscriminationBuckets"][0]["token"],
        "assert"
    );
    assert_eq!(
        candidate["sharedCallTokens"].as_array().map(Vec::len),
        Some(2)
    );
    assert!(candidate["sharedCallTokens"]
        .as_array()
        .is_some_and(|tokens| tokens.iter().any(|token| token == "unwrap_switch")
            && tokens.iter().any(|token| token == "assert")));

    Ok(())
}

#[test]
fn function_body_near_candidates_rank_multi_rare_tokens_above_single_token_pairs() -> Result<()> {
    let mut source = String::new();
    for index in 0..64 {
        source.push_str(&format!("pub fn filler_{index}() -> usize {{ {index} }}\n"));
    }
    source.push_str(
        r#"
pub fn parse_user_alpha(input: usize) -> usize {
    let value = unwrap_switch(input);
    if value > 10 {
        return value + 1;
    }
    value + 2
}

pub fn parse_user_beta(input: usize) -> usize {
    let value = unwrap_switch(input);
    if value < 20 {
        return value + 3;
    }
    value + 4
}

pub fn update_user_alpha(input: usize) -> usize {
    let value = unwrap_value(convert_usize(input));
    if value > 10 {
        return value + 1;
    }
    value + 2
}

pub fn update_user_beta(input: usize) -> usize {
    let value = unwrap_value(convert_usize(input));
    if value < 20 {
        return value + 3;
    }
    value + 4
}
"#,
    );

    let artifact = analyze_file("src/lib.rs", &source);
    let candidates = near_candidates(&artifact);
    let single_token_candidate = candidates
        .iter()
        .find(|candidate| {
            identity_list_contains(candidate, "src/lib.rs::parse_user_alpha")
                && identity_list_contains(candidate, "src/lib.rs::parse_user_beta")
        })
        .context("single-token near candidate")?;
    let multi_token_candidate = candidates
        .iter()
        .find(|candidate| {
            identity_list_contains(candidate, "src/lib.rs::update_user_alpha")
                && identity_list_contains(candidate, "src/lib.rs::update_user_beta")
        })
        .context("multi-token near candidate")?;

    assert_eq!(
        single_token_candidate["sharedCallTokens"]
            .as_array()
            .map(Vec::len),
        Some(1)
    );
    assert_eq!(
        multi_token_candidate["sharedCallTokens"]
            .as_array()
            .map(Vec::len),
        Some(2)
    );
    assert!(multi_token_candidate["sharedCallTokenIdfSum"]
        .as_f64()
        .zip(single_token_candidate["sharedCallTokenIdfSum"].as_f64())
        .is_some_and(|(multi, single)| multi > single));
    assert!(multi_token_candidate["score"]
        .as_f64()
        .zip(single_token_candidate["score"].as_f64())
        .is_some_and(|(multi, single)| multi > single));

    Ok(())
}

#[test]
fn function_body_near_candidates_count_multi_token_pairs_once() {
    let mut source = String::new();
    for index in 0..64 {
        source.push_str(&format!("pub fn filler_{index}() -> usize {{ {index} }}\n"));
    }
    source.push_str(
        r#"
pub fn update_user_alpha(input: usize) -> usize {
    let value = unwrap_value(convert_usize(input));
    if value > 10 {
        return value + 1;
    }
    value + 2
}

pub fn update_user_beta(input: usize) -> usize {
    let value = unwrap_value(convert_usize(input));
    if value < 20 {
        return value + 3;
    }
    value + 4
}
"#,
    );

    let artifact = analyze_file("src/lib.rs", &source);

    assert_eq!(artifact["summary"]["functionBodyFingerprints"], 66);
    assert_eq!(artifact["summary"]["functionCloneNearCandidates"], 1);
    assert_eq!(
        artifact["functionCloneGroups"]["nearFunctionCandidateCount"],
        1
    );
    assert_eq!(
        artifact["functionCloneGroups"]["nearFunctionCandidates"]
            .as_array()
            .map(Vec::len),
        Some(1)
    );
}

#[test]
fn function_body_near_candidates_score_from_all_retained_shared_tokens() -> Result<()> {
    let mut source = String::new();
    for index in 0..64 {
        source.push_str(&format!("pub fn filler_{index}() -> usize {{ {index} }}\n"));
    }
    source.push_str(
        r#"
pub fn compose_alpha(input: usize) -> usize {
    let value = rare_second(rare_first(input));
    if value > 10 {
        return value + 1;
    }
    value + 2
}

pub fn compose_beta(input: usize) -> usize {
    let value = rare_first(rare_second(input));
    if value < 20 {
        return value + 3;
    }
    value + 4
}
"#,
    );

    let artifact = analyze_file("src/lib.rs", &source);
    let candidates = near_candidates(&artifact);
    let candidate = candidates
        .iter()
        .find(|candidate| {
            identity_list_contains(candidate, "src/lib.rs::compose_alpha")
                && identity_list_contains(candidate, "src/lib.rs::compose_beta")
        })
        .context("multi-token candidate")?;

    assert_eq!(
        artifact["functionCloneGroups"]["nearFunctionCandidateCount"],
        1
    );
    assert!(candidate["sharedCallTokens"]
        .as_array()
        .is_some_and(|tokens| tokens.iter().any(|token| token == "rare_first")
            && tokens.iter().any(|token| token == "rare_second")));
    assert!(candidate["sharedCallTokenIdfSum"]
        .as_f64()
        .is_some_and(|sum| sum >= 6.0));
    assert_eq!(candidate["callTokenIdfScore"], 1.0);

    Ok(())
}

#[test]
fn function_body_near_candidates_expose_shared_idf_sum_and_saturation() -> Result<()> {
    let mut source = String::new();
    for index in 0..64 {
        source.push_str(&format!(
            "pub fn filler_{index}() -> usize {{ common_noise() + {index} }}\n"
        ));
    }
    source.push_str(
        r#"
pub fn update_alpha(input: usize) -> usize {
    let value = unwrap_value(convert_usize(input));
    value + common_noise()
}

pub fn update_beta(input: usize) -> usize {
    let value = unwrap_value(convert_usize(input));
    value + 1
}
"#,
    );

    let artifact = analyze_file("src/lib.rs", &source);
    let candidates = near_candidates(&artifact);
    let candidate = candidates
        .iter()
        .find(|candidate| {
            identity_list_contains(candidate, "src/lib.rs::update_alpha")
                && identity_list_contains(candidate, "src/lib.rs::update_beta")
        })
        .context("shared-IDF multi-token near candidate")?;

    assert_eq!(artifact["summary"]["functionBodyFingerprints"], 66);
    assert!(candidate["sharedCallTokens"]
        .as_array()
        .is_some_and(|tokens| tokens.iter().any(|token| token == "convert_usize")
            && tokens.iter().any(|token| token == "unwrap_value")));
    assert!(candidate["sharedCallTokenIdfSum"]
        .as_f64()
        .is_some_and(|score| score >= 6.0));
    assert_eq!(candidate["callTokenIdfScore"], 1.0);
    assert!(candidate["reasons"]
        .as_array()
        .is_some_and(|reasons| reasons.iter().any(|reason| reason
            .as_str()
            .is_some_and(|reason| reason.contains("shared call-token IDF sum")))));

    Ok(())
}

#[test]
fn function_body_near_candidates_ignore_rust_option_constructor_tokens() {
    let artifact = analyze_file(
        "src/lib.rs",
        r#"
pub fn maybe_alpha(flag: bool, value: u8) -> Option<u8> {
    if flag {
        Some(value + 1)
    } else {
        Some(value + 2)
    }
}

pub fn maybe_beta(flag: bool, value: u8) -> Option<u8> {
    match flag {
        true => Some(value + 3),
        false => Some(value + 4),
    }
}
"#,
    );

    assert_eq!(artifact["summary"]["functionBodyFingerprints"], 2);
    assert_eq!(artifact["summary"]["functionCloneExactBodyGroups"], 0);
    assert_eq!(artifact["summary"]["functionCloneStructureGroups"], 0);
    assert_eq!(artifact["summary"]["functionCloneNearCandidates"], 0);
    assert_eq!(
        artifact["functionCloneGroups"]["nearFunctionCandidates"]
            .as_array()
            .map(Vec::len),
        Some(0)
    );
}

fn near_candidates(artifact: &Value) -> &[Value] {
    artifact["functionCloneGroups"]["nearFunctionCandidates"]
        .as_array()
        .map(Vec::as_slice)
        .unwrap_or(&[])
}

#[test]
fn function_body_near_candidate_count_reports_uncapped_review_visible_total() {
    let mut source = String::new();
    for index in 0..512 {
        source.push_str(&format!("pub fn filler_{index}() -> usize {{ {index} }}\n"));
    }
    for index in 0..11 {
        source.push_str(&format!(
            "pub fn load_user_{index}(input: &str) -> usize {{ sanitize(fetch_user(input.trim())).len() + {index} }}\n"
        ));
    }

    let artifact = analyze_file("src/lib.rs", &source);

    assert_eq!(artifact["summary"]["functionBodyFingerprints"], 523);
    assert_eq!(artifact["summary"]["functionCloneExactBodyGroups"], 0);
    assert_eq!(artifact["summary"]["functionCloneStructureGroups"], 0);
    assert_eq!(artifact["summary"]["functionCloneNearCandidates"], 55);
    assert_eq!(
        artifact["functionCloneGroups"]["nearFunctionCandidateCount"],
        55
    );
    assert_eq!(
        artifact["functionCloneGroups"]["nearFunctionCandidateProjectionLimit"],
        50
    );
    assert_eq!(
        artifact["functionCloneGroups"]["nearFunctionCandidates"]
            .as_array()
            .map(Vec::len),
        Some(50)
    );
}

#[test]
fn function_body_near_candidates_ignore_rust_generic_call_tokens() {
    let artifact = analyze_file(
        "src/lib.rs",
        r#"
pub fn generic_alpha(value: Option<String>) -> usize {
    let copied = value.clone();
    let text = copied.unwrap();
    text.to_string().len()
}

pub fn generic_beta(value: Option<String>) -> usize {
    let copied = value.clone();
    if copied.is_none() {
        return 0;
    }
    let text = copied.unwrap();
    text.to_string().len()
}
"#,
    );

    assert_eq!(artifact["summary"]["functionBodyFingerprints"], 2);
    assert_eq!(artifact["summary"]["functionCloneExactBodyGroups"], 0);
    assert_eq!(artifact["summary"]["functionCloneStructureGroups"], 0);
    assert_eq!(artifact["summary"]["functionCloneNearCandidates"], 0);
    assert_eq!(
        artifact["functionCloneGroups"]["nearFunctionCandidates"]
            .as_array()
            .map(Vec::len),
        Some(0)
    );
}

#[test]
fn function_body_near_candidates_ignore_format_macro_token() {
    let artifact = analyze_file(
        "src/lib.rs",
        r#"
pub fn render_invoice(input: &str) -> usize { format!("invoice: {input}").len() }

pub fn compose_alert(input: &str) -> usize { format!("alert: {input}").len() }
"#,
    );

    assert_eq!(artifact["summary"]["functionBodyFingerprints"], 2);
    assert_eq!(artifact["summary"]["functionCloneExactBodyGroups"], 0);
    assert_eq!(artifact["summary"]["functionCloneStructureGroups"], 0);
    assert_eq!(artifact["summary"]["functionCloneNearCandidates"], 0);
    assert_eq!(
        artifact["functionCloneGroups"]["nearFunctionCandidates"]
            .as_array()
            .map(Vec::len),
        Some(0)
    );
}

#[test]
fn function_body_near_candidates_respect_rust_qualifiers() {
    let artifact = analyze_file(
        "src/lib.rs",
        r#"
pub fn refresh_cache(input: &str) -> usize {
    let parsed = input.trim();
    let normalized = sanitize(parsed);
    let loaded = fetch_user(normalized);
    loaded.len()
}

pub unsafe fn refresh_cache_unchecked(raw: &str) -> usize {
    let cleaned = raw.trim();
    let ready = sanitize(cleaned);
    let fetched = fetch_user(ready);
    if fetched.is_empty() {
        return 0;
    }
    fetched.len()
}
"#,
    );

    assert_eq!(artifact["summary"]["functionBodyFingerprints"], 2);
    assert_eq!(artifact["summary"]["functionCloneExactBodyGroups"], 0);
    assert_eq!(artifact["summary"]["functionCloneStructureGroups"], 0);
    assert_eq!(artifact["summary"]["functionCloneNearCandidates"], 0);
    assert_eq!(
        artifact["functionCloneGroups"]["nearFunctionCandidates"]
            .as_array()
            .map(Vec::len),
        Some(0)
    );
}

#[test]
fn function_body_near_candidates_partition_retained_buckets_before_pair_generation() {
    let mut source = String::new();
    for index in 0..256 {
        source.push_str(&format!("pub fn filler_{index}() -> usize {{ {index} }}\n"));
    }
    source.push_str(
        r#"
pub fn compatible_alpha(input: usize) -> usize {
    let value = rare_bridge(input);
    if value > 10 {
        return value + 1;
    }
    value + 2
}

pub fn compatible_beta(input: usize) -> usize {
    let value = rare_bridge(input);
    if value < 20 {
        return value + 3;
    }
    value + 4
}

pub unsafe fn unsafe_gamma(input: usize) -> usize {
    let value = rare_bridge(input);
    value + 5
}

pub fn param_delta(input: usize, extra: usize, more: usize) -> usize {
    let value = rare_bridge(input + extra + more);
    value + 6
}
"#,
    );

    let artifact = analyze_file("src/lib.rs", &source);
    let groups = &artifact["functionCloneGroups"];
    let summary = &groups["candidateGenerationSummary"];

    assert!(summary["retainedRawPairEstimate"]
        .as_u64()
        .zip(summary["generatedUniquePairCount"].as_u64())
        .is_some_and(|(raw, generated)| raw > generated));
    assert_eq!(summary["generatedUniquePairCount"], 1);
    assert_eq!(summary["scoredPairCount"], 1);
    assert!(
        summary["compatibilitySkippedRawPairEstimateByReason"]["qualifierMismatch"]
            .as_u64()
            .is_some_and(|count| count > 0)
    );
    assert!(
        summary["compatibilitySkippedRawPairEstimateByReason"]["parameterCountDelta"]
            .as_u64()
            .is_some_and(|count| count > 0)
    );
    assert_eq!(
        summary["compatibilitySkippedPairEstimateKind"],
        "raw-partition-estimate-does-not-enumerate-rejected-pairs"
    );
}
