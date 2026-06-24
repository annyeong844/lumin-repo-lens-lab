use anyhow::{Context, Result};

use crate::artifact::{analyze_file, file, request, run_sidecar, stdout_json};

use super::helpers::{group_with_identity, identity_list_contains};

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
    assert_eq!(groups["complete"], true);
    assert_eq!(
        groups["filesWithParseErrors"].as_array().map(Vec::len),
        Some(0)
    );
    assert_eq!(
        groups["filesWithReadErrors"].as_array().map(Vec::len),
        Some(0)
    );
    assert_eq!(
        groups["policy"]["policyId"],
        "rust-function-clone-group-policy"
    );
    assert_eq!(
        groups["policy"]["normalizedVersion"],
        "rust-function-body.normalized.v2"
    );
    assert_eq!(
        groups["policy"]["functionSignatureNormalizedVersion"],
        "rust-function-signature.normalized.v1"
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
    assert_eq!(
        signature["normalizedVersion"],
        "rust-function-signature.normalized.v1"
    );
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
fn function_body_clone_groups_keep_good_evidence_when_parse_errors_make_artifact_incomplete(
) -> Result<()> {
    let artifact = stdout_json(run_sidecar(request(vec![
        file(
            "src/a.rs",
            r#"
pub fn exact_a() -> usize {
    let answer = 42;
    answer
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
"#,
        ),
        file("src/bad.rs", "fn main( {"),
    ])));

    assert_eq!(artifact["summary"]["parseErrorFiles"], 1);
    assert!(artifact["summary"]["functionBodyFingerprints"]
        .as_u64()
        .is_some_and(|count| count >= 2));
    assert_eq!(artifact["summary"]["functionCloneExactBodyGroups"], 1);

    let groups = &artifact["functionCloneGroups"];
    assert_eq!(groups["complete"], false);
    assert_eq!(
        groups["filesWithReadErrors"].as_array().map(Vec::len),
        Some(0)
    );
    assert_eq!(groups["filesWithParseErrors"][0]["file"], "src/bad.rs");
    assert!(groups["filesWithParseErrors"][0]["message"].is_string());
    assert!(identity_list_contains(
        &groups["exactBodyGroups"][0],
        "src/a.rs::exact_a"
    ));
    assert!(identity_list_contains(
        &groups["exactBodyGroups"][0],
        "src/b.rs::exact_b"
    ));

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
fn function_body_clone_counts_exclude_ts_js_generated_path_and_header_policy() {
    let source = r#"
pub fn generated_alpha() -> usize {
    let generated = 7;
    generated
}
"#;
    let artifact = stdout_json(run_sidecar(request(vec![
        file("src/bindings.gen.rs", source),
        file("src/header_marker.rs", &format!("// @generated\n{source}")),
    ])));

    assert_eq!(artifact["summary"]["functionBodyFingerprints"], 2);
    assert_eq!(artifact["summary"]["functionCloneExactBodyGroups"], 0);
    assert_eq!(artifact["summary"]["functionCloneSignatureGroups"], 0);
    assert_eq!(artifact["functionCloneGroups"]["exactBodyGroupCount"], 0);
    assert_eq!(artifact["functionCloneGroups"]["signatureGroupCount"], 0);
    assert_eq!(artifact["functionCloneGroups"]["generatedFileFactCount"], 2);
    assert_eq!(
        artifact["functionCloneGroups"]["exactBodyGroups"][0]["generatedOnly"],
        true
    );
    assert_eq!(
        artifact["functionCloneGroups"]["signatureGroups"][0]["generatedOnly"],
        true
    );
}
