use anyhow::{Context, Result};
use serde_json::Value;

use crate::artifact::{analyze_file, file, file_health, request, run_sidecar, stdout_json};

#[test]
fn function_body_fingerprints_preserve_exact_and_structure_hashes() -> Result<()> {
    let source = r####"
pub fn read_a(input: &str) -> usize {
    let parsed = input.len();
    parsed + 1
}

pub fn read_b(input: &str) -> usize {
    let parsed = input.len();
    parsed + 1
}

pub fn read_c(input: &str) -> usize {
    let parsed = input.len();
    parsed + 2
}

pub fn literal_space_a() -> &'static str {
    "a  b"
}

pub fn literal_space_b() -> &'static str {
    "a b"
}

pub fn literal_raw_space_a() -> &'static str {
    r#"a
  b"#
}

pub fn literal_raw_space_b() -> &'static str {
    r#"a b"#
}

pub fn byte_a() -> u8 {
    b'a'
}

pub fn byte_b() -> u8 {
    b'b'
}

pub fn thousand_decimal() -> usize {
    1_000
}

pub fn thousand_plain() -> usize {
    1000
}

pub fn hex_byte() -> usize {
    0xff
}

pub fn decimal_byte() -> usize {
    255
}

pub struct Worker;

impl Worker {
    pub async fn refresh(&self) {
        self.load().await;
    }

    pub unsafe fn reset(&self) {
        cleanup();
    }
}
"####;

    let artifact = analyze_file("src/lib.rs", source);
    assert_eq!(artifact["summary"]["functionBodyFingerprints"], 15);

    let facts = file_health(&artifact, "src/lib.rs")["ast"]["functionBodyFingerprints"]
        .as_array()
        .context("function body fingerprints")?;
    let read_a = fact_named(facts, "read_a")?;
    let read_b = fact_named(facts, "read_b")?;
    let read_c = fact_named(facts, "read_c")?;

    assert_eq!(read_a["kind"], "function-body-fingerprint");
    assert_eq!(read_a["callableKind"], "function");
    assert_eq!(read_a["visibility"], "public");
    assert_eq!(read_a["paramCount"], 1);
    assert_eq!(read_a["statementCount"], 2);
    assert_eq!(read_a["bodyLoc"], 4);
    assert_eq!(read_a["async"], false);
    assert_eq!(read_a["unsafe"], false);
    assert!(read_a["callTokens"]
        .as_array()
        .is_some_and(|tokens| tokens.iter().any(|token| token == "len")));

    assert_eq!(read_a["exactBodyHash"], read_b["exactBodyHash"]);
    assert_ne!(read_a["exactBodyHash"], read_c["exactBodyHash"]);
    assert_eq!(
        read_a["normalizedStructureHash"],
        read_c["normalizedStructureHash"]
    );
    assert_ne!(read_a["normalizedExactHash"], read_c["normalizedExactHash"]);

    let literal_space_a = fact_named(facts, "literal_space_a")?;
    let literal_space_b = fact_named(facts, "literal_space_b")?;
    assert_ne!(
        literal_space_a["exactBodyHash"],
        literal_space_b["exactBodyHash"]
    );
    assert_ne!(
        literal_space_a["normalizedExactHash"],
        literal_space_b["normalizedExactHash"]
    );
    assert_eq!(
        literal_space_a["normalizedStructureHash"],
        literal_space_b["normalizedStructureHash"]
    );
    assert_eq!(literal_space_a["statementCount"], 1);

    let literal_raw_space_a = fact_named(facts, "literal_raw_space_a")?;
    let literal_raw_space_b = fact_named(facts, "literal_raw_space_b")?;
    assert_ne!(
        literal_raw_space_a["exactBodyHash"],
        literal_raw_space_b["exactBodyHash"]
    );
    assert_ne!(
        literal_raw_space_a["normalizedExactHash"],
        literal_raw_space_b["normalizedExactHash"]
    );
    assert_eq!(
        literal_raw_space_a["normalizedStructureHash"],
        literal_raw_space_b["normalizedStructureHash"]
    );
    assert_eq!(literal_raw_space_a["statementCount"], 1);

    let byte_a = fact_named(facts, "byte_a")?;
    let byte_b = fact_named(facts, "byte_b")?;
    assert_ne!(byte_a["normalizedExactHash"], byte_b["normalizedExactHash"]);
    assert_eq!(
        byte_a["normalizedStructureHash"],
        byte_b["normalizedStructureHash"]
    );

    let thousand_decimal = fact_named(facts, "thousand_decimal")?;
    let thousand_plain = fact_named(facts, "thousand_plain")?;
    assert_ne!(
        thousand_decimal["exactBodyHash"],
        thousand_plain["exactBodyHash"]
    );
    assert_eq!(
        thousand_decimal["normalizedExactHash"],
        thousand_plain["normalizedExactHash"]
    );

    let hex_byte = fact_named(facts, "hex_byte")?;
    let decimal_byte = fact_named(facts, "decimal_byte")?;
    assert_eq!(
        hex_byte["normalizedExactHash"],
        decimal_byte["normalizedExactHash"]
    );

    let refresh = fact_named(facts, "refresh")?;
    assert_eq!(refresh["callableKind"], "impl-method");
    assert_eq!(refresh["owner"]["target"], "Worker");
    assert_eq!(refresh["async"], true);

    let reset = fact_named(facts, "reset")?;
    assert_eq!(reset["callableKind"], "impl-method");
    assert_eq!(reset["owner"]["target"], "Worker");
    assert_eq!(reset["unsafe"], true);

    Ok(())
}

fn fact_named<'a>(facts: &'a [Value], name: &str) -> Result<&'a Value> {
    facts
        .iter()
        .find(|fact| fact["name"] == name)
        .with_context(|| format!("missing function body fingerprint for {name}"))
}

#[test]
fn function_body_clone_groups_are_repo_wide_review_evidence() -> Result<()> {
    let artifact = stdout_json(run_sidecar(request(vec![
        file(
            "src/a.rs",
            r#"
pub fn exact_a() -> usize {
    let answer = 42;
    answer
}

pub fn structure_a(input: &str) -> usize {
    let parsed = input.len();
    let adjusted = parsed + 1;
    adjusted
}
"#,
        ),
        file(
            "src/b.rs",
            r#"
pub fn exact_b() -> usize {
    let answer = 42;
    answer
}

pub fn structure_b(value: &str) -> usize {
    let amount = value.len();
    let total = amount + 2;
    total
}
"#,
        ),
    ])));

    assert_eq!(artifact["summary"]["functionBodyFingerprints"], 4);
    assert_eq!(artifact["summary"]["functionCloneExactBodyGroups"], 1);
    assert_eq!(artifact["summary"]["functionCloneStructureGroups"], 2);
    assert_eq!(artifact["summary"]["functionCloneSignatureGroups"], 2);
    assert_eq!(artifact["summary"]["functionCloneNearCandidates"], 0);

    let groups = &artifact["functionCloneGroups"];
    assert_eq!(
        groups["policy"]["policyId"],
        "rust-function-clone-group-policy"
    );
    assert_eq!(
        groups["policy"]["caveat"],
        "Function clone groups and near candidates are deterministic review evidence. They do not prove semantic equivalence, auto-reuse, auto-fix safety, or a merge recommendation."
    );

    let exact = &groups["exactBodyGroups"][0];
    assert_eq!(exact["kind"], "exact-function-body-group");
    assert_eq!(exact["risk"], "review-only");
    assert_eq!(exact["size"], 2);
    assert!(identity_list_contains(exact, "src/a.rs::exact_a"));
    assert!(identity_list_contains(exact, "src/b.rs::exact_b"));
    assert_eq!(
        exact["reason"],
        "same normalized function body; verify domain ownership before merging"
    );

    let structure_groups = groups["structureGroups"]
        .as_array()
        .context("structure clone groups")?;
    let structure = group_with_identity(structure_groups, "src/a.rs::structure_a")
        .context("structure_a clone group")?;
    assert_eq!(structure["kind"], "function-body-structure-group");
    assert_eq!(structure["risk"], "review-only");
    assert_eq!(structure["size"], 2);
    assert!(identity_list_contains(structure, "src/a.rs::structure_a"));
    assert!(identity_list_contains(structure, "src/b.rs::structure_b"));
    assert_eq!(structure["exactHashCount"], 2);
    assert!(structure["sharedCallTokens"]
        .as_array()
        .is_some_and(|tokens| tokens.iter().any(|token| token == "len")));
    assert!(structure["reason"]
        .as_str()
        .is_some_and(|reason| reason.contains("not proof of semantic equivalence")));

    assert_eq!(groups["signatureGroupCount"], 2);
    let signature_groups = groups["signatureGroups"]
        .as_array()
        .context("signature clone groups")?;
    let signature = group_with_identity(signature_groups, "src/a.rs::exact_a")
        .context("exact_a signature group")?;
    assert_eq!(signature["kind"], "function-signature-group");
    assert_eq!(signature["risk"], "review-only");
    assert_eq!(signature["generatedOnly"], false);
    assert_eq!(signature["signature"], "fn() -> usize");
    assert!(identity_list_contains(signature, "src/a.rs::exact_a"));
    assert!(identity_list_contains(signature, "src/b.rs::exact_b"));
    assert!(signature["reason"]
        .as_str()
        .is_some_and(|reason| reason.contains("not proof of semantic equivalence")));

    Ok(())
}

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
        "rust-function-clone-near-calibration.v1"
    );
    assert_eq!(
        groups["policy"]["nearCandidatePolicy"]["minSignificantCallTokenLen"],
        4
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
fn function_body_clone_counts_exclude_generated_only_groups() {
    let artifact = analyze_file(
        "generated/bindings.rs",
        r#"
pub fn generated_alpha() -> usize {
    let generated = 7;
    generated
}

pub fn generated_beta() -> usize {
    let generated = 7;
    generated
}
"#,
    );

    assert_eq!(artifact["summary"]["functionBodyFingerprints"], 2);
    assert_eq!(artifact["summary"]["functionCloneExactBodyGroups"], 0);
    assert_eq!(artifact["summary"]["functionCloneSignatureGroups"], 0);
    assert_eq!(artifact["functionCloneGroups"]["exactBodyGroupCount"], 0);
    assert_eq!(artifact["functionCloneGroups"]["signatureGroupCount"], 0);
    assert_eq!(artifact["functionCloneGroups"]["generatedFileFactCount"], 2);
    assert_eq!(
        artifact["functionCloneGroups"]["exactBodyGroups"]
            .as_array()
            .map(Vec::len),
        Some(1)
    );
    assert_eq!(
        artifact["functionCloneGroups"]["exactBodyGroups"][0]["generatedOnly"],
        true
    );
    assert_eq!(
        artifact["functionCloneGroups"]["signatureGroups"]
            .as_array()
            .map(Vec::len),
        Some(1)
    );
    assert_eq!(
        artifact["functionCloneGroups"]["signatureGroups"][0]["generatedOnly"],
        true
    );
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

fn identity_list_contains(group: &Value, identity: &str) -> bool {
    group["identities"]
        .as_array()
        .is_some_and(|identities| identities.iter().any(|entry| entry == identity))
}

fn group_with_identity<'a>(groups: &'a [Value], identity: &str) -> Option<&'a Value> {
    groups
        .iter()
        .find(|group| identity_list_contains(group, identity))
}
