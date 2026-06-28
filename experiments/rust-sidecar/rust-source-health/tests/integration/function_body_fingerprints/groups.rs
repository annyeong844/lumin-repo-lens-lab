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
    assert_eq!(artifact["summary"]["functionCloneSignatureGroups"], 0);
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
        "rust-function-body.normalized.v3"
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

    assert_eq!(groups["signatureGroupCount"], 0);
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
    assert_eq!(signature["risk"], "muted");
    assert_eq!(signature["generatedOnly"], false);
    assert_eq!(signature["reviewVisible"], false);
    assert_eq!(signature["signatureDomainIdfSum"], 0.0);
    assert_eq!(signature["signature"], "fn() -> usize");
    assert!(identity_list_contains(signature, "src/a.rs::exact_a"));
    assert!(identity_list_contains(signature, "src/b.rs::exact_b"));
    assert!(signature["reason"]
        .as_str()
        .is_some_and(|reason| reason.contains("raw evidence only")));

    Ok(())
}

#[test]
fn exact_body_groups_merge_local_identifier_renames_like_ts_normalized_exact() -> Result<()> {
    let artifact = stdout_json(run_sidecar(request(vec![file(
        "src/local_names.rs",
        r#"
pub fn left(input: usize) -> usize {
    let left_value = input + 1;
    left_value * 2
}

pub fn right(input: usize) -> usize {
    let right_value = input + 1;
    right_value * 2
}
"#,
    )])));

    assert_eq!(artifact["summary"]["functionBodyFingerprints"], 2);
    assert_eq!(artifact["summary"]["functionCloneExactBodyGroups"], 1);

    let groups = artifact["functionCloneGroups"]["exactBodyGroups"]
        .as_array()
        .context("exact clone groups")?;
    let exact = group_with_identity(groups, "src/local_names.rs::left")
        .context("left exact clone group")?;
    assert_eq!(exact["kind"], "exact-function-body-group");
    assert_eq!(exact["size"], 2);
    assert_eq!(exact["exactHashCount"], 2);
    assert!(identity_list_contains(exact, "src/local_names.rs::left"));
    assert!(identity_list_contains(exact, "src/local_names.rs::right"));
    assert_eq!(
        exact["reason"],
        "same normalized function body; verify domain ownership before merging"
    );

    Ok(())
}

#[test]
fn exact_body_groups_do_not_merge_identifier_anonymized_bodies() -> Result<()> {
    let artifact = stdout_json(run_sidecar(request(vec![file(
        "src/category.rs",
        r#"
pub enum Category {
    Output,
    Search,
}

pub fn is_output(category: Category) -> bool {
    let expected = Category::Output;
    category == expected
}

pub fn is_search(category: Category) -> bool {
    let expected = Category::Search;
    category == expected
}
"#,
    )])));

    assert_eq!(artifact["summary"]["functionBodyFingerprints"], 2);
    assert_eq!(artifact["summary"]["functionCloneExactBodyGroups"], 0);
    assert_eq!(artifact["summary"]["functionCloneStructureGroups"], 0);
    assert_eq!(artifact["functionCloneGroups"]["exactBodyGroupCount"], 0);
    assert_eq!(artifact["functionCloneGroups"]["structureGroupCount"], 0);
    assert_eq!(
        artifact["functionCloneGroups"]["exactBodyGroups"]
            .as_array()
            .map(Vec::len),
        Some(0)
    );
    assert_eq!(
        artifact["functionCloneGroups"]["structureGroups"]
            .as_array()
            .map(Vec::len),
        Some(0)
    );

    Ok(())
}

#[test]
fn signature_groups_skip_implicit_unit_return_signatures() -> Result<()> {
    let artifact = stdout_json(run_sidecar(request(vec![file(
        "src/unit.rs",
        r#"
pub fn first() {
    let value = 1;
    let _ = value;
}

pub fn second() {
    let value = 2;
    let _ = value;
}
"#,
    )])));

    assert_eq!(artifact["summary"]["functionSignatures"], 2);
    assert_eq!(artifact["summary"]["functionCloneSignatureGroups"], 0);
    assert_eq!(artifact["functionCloneGroups"]["signatureGroupCount"], 0);
    assert_eq!(
        artifact["functionCloneGroups"]["signatureGroups"]
            .as_array()
            .map(Vec::len),
        Some(0)
    );

    Ok(())
}

#[test]
fn signature_groups_demote_generic_bool_method_groups() -> Result<()> {
    let artifact = stdout_json(run_sidecar(request(vec![file(
        "src/generic.rs",
        r#"
pub struct Flags;

impl Flags {
    pub fn is_ready(&self) -> bool {
        true
    }

    pub fn is_enabled(&self) -> bool {
        false
    }
}
"#,
    )])));

    assert_eq!(artifact["summary"]["functionSignatures"], 2);
    assert_eq!(artifact["summary"]["functionCloneSignatureGroups"], 0);
    assert_eq!(artifact["functionCloneGroups"]["signatureGroupCount"], 0);

    let signature_groups = artifact["functionCloneGroups"]["signatureGroups"]
        .as_array()
        .context("signature groups")?;
    assert_eq!(signature_groups.len(), 1);
    let group = &signature_groups[0];
    assert_eq!(group["signature"], "fn(&self) -> bool");
    assert_eq!(group["risk"], "muted");
    assert_eq!(group["reviewVisible"], false);
    assert_eq!(group["signatureDomainIdfSum"], 0.0);
    assert!(group["reason"]
        .as_str()
        .is_some_and(|reason| reason.contains("raw evidence only")));

    Ok(())
}

#[test]
fn signature_groups_keep_domain_type_signatures_review_visible() -> Result<()> {
    let mut source = String::from(
        r#"
pub struct CustomType;

pub fn domain_alpha(value: CustomType) -> CustomType {
    value
}

pub fn domain_beta(value: CustomType) -> CustomType {
    value
}
"#,
    );
    for index in 0..32 {
        source.push_str(&format!("pub fn generic_{index}() -> bool {{ true }}\n"));
    }

    let artifact = stdout_json(run_sidecar(request(vec![file("src/domain.rs", &source)])));

    assert_eq!(artifact["summary"]["functionSignatures"], 34);
    assert_eq!(artifact["summary"]["functionCloneSignatureGroups"], 1);
    assert_eq!(artifact["functionCloneGroups"]["signatureGroupCount"], 1);

    let signature_groups = artifact["functionCloneGroups"]["signatureGroups"]
        .as_array()
        .context("signature groups")?;
    let group = group_with_identity(signature_groups, "src/domain.rs::domain_alpha")
        .context("domain signature group")?;
    assert_eq!(group["signature"], "fn(CustomType) -> CustomType");
    assert_eq!(group["risk"], "review-only");
    assert_eq!(group["reviewVisible"], true);
    assert!(group["signatureDomainIdfSum"]
        .as_f64()
        .is_some_and(|idf| idf >= 2.0));
    assert!(group["reason"]
        .as_str()
        .is_some_and(|reason| reason.contains("not proof of semantic equivalence")));

    Ok(())
}

#[test]
fn signature_groups_demote_domain_type_signatures_below_idf_threshold() -> Result<()> {
    let artifact = stdout_json(run_sidecar(request(vec![file(
        "src/edge.rs",
        r#"
pub struct EdgeType;

pub fn edge_alpha(value: EdgeType) -> EdgeType {
    value
}

pub fn edge_beta(value: EdgeType) -> EdgeType {
    value
}

pub fn generic() -> bool {
    true
}
"#,
    )])));

    assert_eq!(artifact["summary"]["functionSignatures"], 3);
    assert_eq!(artifact["summary"]["functionCloneSignatureGroups"], 0);
    assert_eq!(artifact["functionCloneGroups"]["signatureGroupCount"], 0);

    let signature_groups = artifact["functionCloneGroups"]["signatureGroups"]
        .as_array()
        .context("signature groups")?;
    let group = group_with_identity(signature_groups, "src/edge.rs::edge_alpha")
        .context("edge signature group")?;
    assert_eq!(group["signature"], "fn(EdgeType) -> EdgeType");
    assert_eq!(group["risk"], "muted");
    assert_eq!(group["reviewVisible"], false);
    assert!(group["signatureDomainIdfSum"]
        .as_f64()
        .is_some_and(|idf| idf > 0.0 && idf < 2.0));

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
