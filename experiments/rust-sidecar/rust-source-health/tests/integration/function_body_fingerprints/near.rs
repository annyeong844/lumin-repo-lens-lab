use anyhow::Result;

use crate::artifact::analyze_file;

use super::helpers::identity_list_contains;

#[test]
fn function_body_clone_groups_include_ts_style_near_candidates() -> Result<()> {
    let artifact = analyze_file(
        "src/lib.rs",
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

    assert_eq!(artifact["summary"]["functionBodyFingerprints"], 2);
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
        "rust-function-clone-near-calibration.v2"
    );
    assert_eq!(
        groups["policy"]["nearCandidatePolicy"]["minSignificantCallTokenLen"],
        4
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
fn function_body_near_candidate_count_reports_uncapped_review_visible_total() {
    let mut source = String::new();
    for index in 0..11 {
        source.push_str(&format!(
            "pub fn load_user_{index}(input: &str) -> usize {{ sanitize(fetch_user(input.trim())).len() + {index} }}\n"
        ));
    }

    let artifact = analyze_file("src/lib.rs", &source);

    assert_eq!(artifact["summary"]["functionBodyFingerprints"], 11);
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
