use anyhow::{Context, Result};

use crate::artifact::analyze_file;

#[test]
fn public_zero_ref_definitions_are_excluded_by_rust_public_surface_gate() -> Result<()> {
    let artifact = analyze_file(
        "src/lib.rs",
        r#"
pub struct DecompressionMatcherBuilder;

pub fn exported_builder() -> DecompressionMatcherBuilder {
    DecompressionMatcherBuilder
}

fn local_helper() {}
"#,
    );

    let analysis = &artifact["unusedDefinitionAnalysis"];
    assert_eq!(
        analysis["policy"]["policyId"],
        "rust-unused-definition-policy-v1"
    );
    assert_eq!(analysis["policy"]["rustFpGateNamespace"], "RUST-FP");
    assert_eq!(analysis["summary"]["candidateCount"], 0);
    assert_eq!(analysis["summary"]["blockedPublicSurfaceCount"], 2);
    assert!(analysis["findings"].as_array().is_some_and(Vec::is_empty));

    let excluded = analysis["excludedCandidates"]
        .as_array()
        .context("excludedCandidates")?;
    assert_eq!(excluded.len(), 2);
    for name in ["DecompressionMatcherBuilder", "exported_builder"] {
        let candidate = excluded
            .iter()
            .find(|candidate| candidate["definition"]["name"] == name)
            .with_context(|| format!("{name} excluded by public surface"))?;
        assert_eq!(candidate["tier"], "review");
        assert_eq!(candidate["action"], "demote-to-restricted");
        assert_eq!(candidate["safeAction"], serde_json::Value::Null);
        assert_eq!(candidate["fpGates"][0], "RUST-FP-A");
        assert_eq!(
            candidate["actionBlockers"][0],
            "rust-fp-a-external-public-surface"
        );
        assert_eq!(candidate["observedReferences"]["production"], 0);
        assert_eq!(
            candidate["observedReferences"]["searchedScopes"][0],
            "crate-local-qualified-path-refs"
        );
    }

    Ok(())
}

#[test]
fn observed_qualified_refs_keep_public_definitions_out_of_excluded_dead_surface() -> Result<()> {
    let artifact = analyze_file(
        "src/lib.rs",
        r#"
pub fn live() {}

fn caller() {
    crate::live();
}
"#,
    );

    let analysis = &artifact["unusedDefinitionAnalysis"];
    assert_eq!(analysis["summary"]["candidateCount"], 0);
    assert_eq!(analysis["summary"]["blockedPublicSurfaceCount"], 0);
    assert!(analysis["excludedCandidates"]
        .as_array()
        .context("excludedCandidates")?
        .iter()
        .all(|candidate| candidate["definition"]["name"] != "live"));

    Ok(())
}

#[test]
fn parse_error_files_degrade_dead_export_absence_claims() -> Result<()> {
    let artifact = analyze_file("src/bad.rs", "pub fn broken( {");
    let analysis = &artifact["unusedDefinitionAnalysis"];

    assert_eq!(analysis["summary"]["candidateCount"], 0);
    assert_eq!(analysis["summary"]["degradedCount"], 1);
    assert!(analysis["findings"].as_array().is_some_and(Vec::is_empty));

    let degraded = analysis["degradedScopes"]
        .as_array()
        .context("degradedScopes")?;
    assert_eq!(degraded.len(), 1);
    assert_eq!(degraded[0]["kind"], "parse-error-file");
    assert_eq!(degraded[0]["file"], "src/bad.rs");
    assert!(degraded[0]["message"]
        .as_str()
        .is_some_and(|message| message.contains("absence claims are not grounded")));

    Ok(())
}

#[test]
fn ffi_linker_exports_are_blocked_by_rust_ffi_gate() -> Result<()> {
    let artifact = analyze_file(
        "src/lib.rs",
        r#"
#[no_mangle]
pub extern "C" fn exported_to_c() {}
"#,
    );

    let analysis = &artifact["unusedDefinitionAnalysis"];
    assert_eq!(analysis["summary"]["candidateCount"], 0);
    assert_eq!(analysis["summary"]["blockedFfiCount"], 1);

    let candidate = find_excluded(analysis, "exported_to_c")?;
    assert_eq!(candidate["fpGates"][0], "RUST-FP-D");
    assert_eq!(
        candidate["actionBlockers"][0],
        "rust-fp-d-ffi-linker-surface"
    );
    assert_eq!(candidate["safeAction"], serde_json::Value::Null);

    Ok(())
}

#[test]
fn cfg_gated_exports_are_degraded_by_rust_cfg_gate() -> Result<()> {
    let artifact = analyze_file(
        "src/lib.rs",
        r#"
#[cfg(feature = "fast")]
pub fn gated_export() {}
"#,
    );

    let analysis = &artifact["unusedDefinitionAnalysis"];
    assert_eq!(analysis["summary"]["candidateCount"], 0);
    assert_eq!(analysis["summary"]["blockedCfgCount"], 1);
    assert_eq!(analysis["summary"]["degradedCount"], 1);

    let candidate = find_excluded(analysis, "gated_export")?;
    assert_eq!(candidate["tier"], "degraded");
    assert_eq!(candidate["action"], "degraded");
    assert_eq!(candidate["fpGates"][0], "RUST-FP-F");
    assert_eq!(
        candidate["actionBlockers"][0],
        "rust-fp-f-cfg-gated-definition"
    );

    Ok(())
}

#[test]
fn trait_impl_methods_are_blocked_by_rust_trait_contract_gate() -> Result<()> {
    let artifact = analyze_file(
        "src/lib.rs",
        r#"
pub trait Runner {
    fn run(&self);
}

pub struct Worker;

impl Runner for Worker {
    fn run(&self) {}
}
"#,
    );

    let analysis = &artifact["unusedDefinitionAnalysis"];
    assert_eq!(analysis["summary"]["candidateCount"], 0);
    assert_eq!(analysis["summary"]["blockedTraitImplCount"], 1);

    let candidate = find_excluded(analysis, "run")?;
    assert_eq!(candidate["definition"]["owner"], "trait-impl");
    assert_eq!(candidate["fpGates"][0], "RUST-FP-B");
    assert_eq!(
        candidate["actionBlockers"][0],
        "rust-fp-b-trait-impl-contract"
    );

    Ok(())
}

#[test]
fn test_only_references_are_visible_without_grounding_production_use() -> Result<()> {
    let artifact = analyze_file(
        "src/lib.rs",
        r#"
pub fn helper() {}

#[cfg(test)]
mod tests {
    #[test]
    fn exercises_helper() {
        crate::helper();
    }
}
"#,
    );

    let analysis = &artifact["unusedDefinitionAnalysis"];
    assert_eq!(analysis["summary"]["candidateCount"], 0);
    assert_eq!(analysis["summary"]["testOnlySupportCount"], 1);

    let candidate = find_excluded(analysis, "helper")?;
    assert_eq!(candidate["fpGates"][0], "RUST-FP-G");
    assert_eq!(
        candidate["actionBlockers"][0],
        "rust-fp-g-test-only-reachability"
    );
    assert_eq!(candidate["observedReferences"]["production"], 0);
    assert_eq!(candidate["observedReferences"]["testOnly"], 1);

    Ok(())
}

#[test]
fn review_opaque_surfaces_block_dead_export_absence_claims() -> Result<()> {
    let artifact = analyze_file(
        "src/lib.rs",
        r#"
pub fn macro_visible() {}

custom_macro!(macro_visible);
"#,
    );

    let analysis = &artifact["unusedDefinitionAnalysis"];
    assert_eq!(analysis["summary"]["candidateCount"], 0);
    assert_eq!(analysis["summary"]["blockedOpaqueCount"], 1);

    let candidate = find_excluded(analysis, "macro_visible")?;
    assert_eq!(candidate["fpGates"][0], "RUST-FP-C");
    assert_eq!(
        candidate["actionBlockers"][0],
        "rust-fp-c-review-opaque-syntax"
    );
    assert!(candidate["evidence"][0]["message"]
        .as_str()
        .is_some_and(|message| message.contains("custom_macro")));

    Ok(())
}

#[test]
fn derive_surfaces_block_dead_export_absence_claims_before_public_surface_gate() -> Result<()> {
    let artifact = analyze_file(
        "src/lib.rs",
        r#"
#[derive(Debug)]
pub struct WireShape {
    value: String,
}
"#,
    );

    let analysis = &artifact["unusedDefinitionAnalysis"];
    assert_eq!(analysis["summary"]["candidateCount"], 0);
    assert_eq!(analysis["summary"]["blockedDeriveSurfaceCount"], 1);
    assert_eq!(analysis["summary"]["blockedPublicSurfaceCount"], 0);

    let candidate = find_excluded(analysis, "WireShape")?;
    assert_eq!(candidate["fpGates"][0], "RUST-FP-E");
    assert_eq!(
        candidate["actionBlockers"][0],
        "rust-fp-e-derive-trait-requirement"
    );
    assert!(candidate["evidence"][0]["message"]
        .as_str()
        .is_some_and(|message| message.contains("derive(Debug)")));

    Ok(())
}

fn find_excluded<'a>(analysis: &'a serde_json::Value, name: &str) -> Result<&'a serde_json::Value> {
    analysis["excludedCandidates"]
        .as_array()
        .context("excludedCandidates")?
        .iter()
        .find(|candidate| candidate["definition"]["name"] == name)
        .with_context(|| format!("{name} excluded candidate"))
}
