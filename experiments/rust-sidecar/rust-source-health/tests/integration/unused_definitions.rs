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

pub(crate) fn crate_visible_builder() -> DecompressionMatcherBuilder {
    DecompressionMatcherBuilder
}
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
    for name in ["exported_builder", "crate_visible_builder"] {
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
            "crate-local-name-qualified-path-and-token-refs"
        );
    }

    Ok(())
}

#[test]
fn public_inherent_impl_methods_are_excluded_by_rust_public_surface_gate() -> Result<()> {
    let artifact = analyze_file(
        "src/lib.rs",
        r#"
pub struct Runner;

impl Runner {
    pub fn run(&self) {}

    pub fn new() -> Self {
        Runner
    }
}
"#,
    );

    let analysis = &artifact["unusedDefinitionAnalysis"];
    assert_eq!(analysis["summary"]["candidateCount"], 0);
    assert_eq!(analysis["summary"]["blockedPublicSurfaceCount"], 2);
    assert!(analysis["findings"].as_array().is_some_and(Vec::is_empty));

    for name in ["run", "new"] {
        let candidate = find_excluded(analysis, name)?;
        assert_eq!(candidate["definition"]["owner"], "inherent-impl");
        assert_eq!(candidate["fpGates"][0], "RUST-FP-A");
        assert_eq!(
            candidate["actionBlockers"][0],
            "rust-fp-a-external-public-surface"
        );
        assert_eq!(candidate["safeAction"], serde_json::Value::Null);
    }
    assert!(analysis["excludedCandidates"]
        .as_array()
        .context("excludedCandidates")?
        .iter()
        .all(|candidate| candidate["definition"]["name"] != "Runner"));

    Ok(())
}

#[test]
fn private_zero_ref_definitions_become_raw_remove_candidates_without_safe_actions() -> Result<()> {
    let artifact = analyze_file(
        "src/lib.rs",
        r#"
fn truly_dead_private_helper() {}

fn live_private_helper() {}

fn caller() {
    live_private_helper();
}
"#,
    );

    let analysis = &artifact["unusedDefinitionAnalysis"];
    assert_eq!(analysis["summary"]["candidateCount"], 2);
    assert_eq!(analysis["summary"]["blockedPublicSurfaceCount"], 0);
    assert_eq!(analysis["summary"]["testOnlySupportCount"], 0);

    let findings = analysis["findings"].as_array().context("findings")?;
    assert_eq!(findings.len(), 2);
    let candidate = findings
        .iter()
        .find(|candidate| candidate["definition"]["name"] == "truly_dead_private_helper")
        .context("truly_dead_private_helper finding")?;
    assert!(findings
        .iter()
        .all(|candidate| candidate["definition"]["name"] != "live_private_helper"));
    assert_eq!(candidate["definition"]["visibility"], "private");
    assert_eq!(candidate["tier"], "remove-candidate");
    assert_eq!(candidate["action"], "remove-candidate");
    assert_eq!(candidate["safeAction"], serde_json::Value::Null);
    assert!(candidate["fpGates"].as_array().is_some_and(Vec::is_empty));
    assert!(candidate["actionBlockers"]
        .as_array()
        .is_some_and(Vec::is_empty));
    assert_eq!(candidate["observedReferences"]["production"], 0);
    assert_eq!(candidate["observedReferences"]["testOnly"], 0);
    assert_eq!(
        candidate["observedReferences"]["searchedScopes"][0],
        "crate-local-name-qualified-path-and-token-refs"
    );

    Ok(())
}

#[test]
fn private_references_inside_macro_inputs_block_remove_candidates() -> Result<()> {
    let artifact = analyze_file(
        "src/lib.rs",
        r#"
fn macro_live_helper() -> &'static str {
    "live"
}

fn caller() {
    let _tags = vec![("kind", macro_live_helper().to_string())];
}

pub fn entry() {
    caller();
}
"#,
    );

    let analysis = &artifact["unusedDefinitionAnalysis"];
    assert_eq!(analysis["summary"]["candidateCount"], 0);
    assert!(analysis["findings"].as_array().is_some_and(Vec::is_empty));
    assert!(analysis["excludedCandidates"]
        .as_array()
        .context("excludedCandidates")?
        .iter()
        .all(|candidate| candidate["definition"]["name"] != "macro_live_helper"));

    Ok(())
}

#[test]
fn private_references_inside_format_captures_block_remove_candidates() -> Result<()> {
    let artifact = analyze_file(
        "src/lib.rs",
        r#"
const FORMAT_MESSAGE: &str = "ready";

fn render() -> String {
    format!("{FORMAT_MESSAGE}")
}

pub fn entry() -> String {
    render()
}
"#,
    );

    let analysis = &artifact["unusedDefinitionAnalysis"];
    assert_eq!(analysis["summary"]["candidateCount"], 0);
    assert!(analysis["findings"].as_array().is_some_and(Vec::is_empty));
    assert!(analysis["excludedCandidates"]
        .as_array()
        .context("excludedCandidates")?
        .iter()
        .all(|candidate| candidate["definition"]["name"] != "FORMAT_MESSAGE"));

    Ok(())
}

#[test]
fn private_references_inside_attribute_string_paths_block_remove_candidates() -> Result<()> {
    let artifact = analyze_file(
        "src/lib.rs",
        r#"
fn default_limit() -> usize {
    25
}

struct Args {
    #[serde(default = "default_limit")]
    limit: usize,
}

pub fn entry() {
    let _ = core::mem::size_of::<Args>();
}
"#,
    );

    let analysis = &artifact["unusedDefinitionAnalysis"];
    assert_eq!(analysis["summary"]["candidateCount"], 0);
    assert!(analysis["findings"].as_array().is_some_and(Vec::is_empty));
    assert!(analysis["excludedCandidates"]
        .as_array()
        .context("excludedCandidates")?
        .iter()
        .all(|candidate| candidate["definition"]["name"] != "default_limit"));

    Ok(())
}

#[test]
fn private_references_inside_schema_with_attributes_block_remove_candidates() -> Result<()> {
    let artifact = analyze_file(
        "src/lib.rs",
        r#"
fn hook_event_name_schema() -> usize {
    1
}

struct HookWire {
    #[schemars(schema_with = "hook_event_name_schema")]
    name: String,
}

pub fn entry() {
    let _ = core::mem::size_of::<HookWire>();
}
"#,
    );

    let analysis = &artifact["unusedDefinitionAnalysis"];
    assert_eq!(analysis["summary"]["candidateCount"], 0);
    assert!(analysis["findings"].as_array().is_some_and(Vec::is_empty));
    assert!(analysis["excludedCandidates"]
        .as_array()
        .context("excludedCandidates")?
        .iter()
        .all(|candidate| candidate["definition"]["name"] != "hook_event_name_schema"));

    Ok(())
}

#[test]
fn private_references_inside_format_width_captures_block_remove_candidates() -> Result<()> {
    let artifact = analyze_file(
        "src/lib.rs",
        r#"
const NAME_WIDTH: usize = 12;

fn render(name: &str) -> String {
    format!("{:<NAME_WIDTH$}", name)
}

pub fn entry() -> String {
    render("network")
}
"#,
    );

    let analysis = &artifact["unusedDefinitionAnalysis"];
    assert_eq!(analysis["summary"]["candidateCount"], 0);
    assert!(analysis["findings"].as_array().is_some_and(Vec::is_empty));
    assert!(analysis["excludedCandidates"]
        .as_array()
        .context("excludedCandidates")?
        .iter()
        .all(|candidate| candidate["definition"]["name"] != "NAME_WIDTH"));

    Ok(())
}

#[test]
fn private_const_pattern_references_block_remove_candidates() -> Result<()> {
    let artifact = analyze_file(
        "src/lib.rs",
        r#"
const ANSI_ALPHA_INDEX: u8 = 0x00;
const ANSI_ALPHA_DEFAULT: u8 = 0x01;
const OPAQUE_ALPHA: u8 = 0xFF;

fn convert(alpha: u8) -> Option<u8> {
    match alpha {
        ANSI_ALPHA_INDEX => Some(0),
        ANSI_ALPHA_DEFAULT => None,
        OPAQUE_ALPHA => Some(255),
        _ => Some(alpha),
    }
}

pub fn entry() -> Option<u8> {
    convert(0)
}
"#,
    );

    let analysis = &artifact["unusedDefinitionAnalysis"];
    assert_eq!(analysis["summary"]["candidateCount"], 0);
    assert!(analysis["findings"].as_array().is_some_and(Vec::is_empty));
    for name in ["ANSI_ALPHA_INDEX", "ANSI_ALPHA_DEFAULT", "OPAQUE_ALPHA"] {
        assert!(analysis["excludedCandidates"]
            .as_array()
            .context("excludedCandidates")?
            .iter()
            .all(|candidate| candidate["definition"]["name"] != name));
    }

    Ok(())
}

#[test]
fn private_test_context_definitions_are_blocked_instead_of_remove_candidates() -> Result<()> {
    let artifact = analyze_file(
        "src/lib.rs",
        r#"
#[cfg(test)]
mod tests {
    fn test_only_helper() {}
}
"#,
    );

    let analysis = &artifact["unusedDefinitionAnalysis"];
    assert_eq!(analysis["summary"]["candidateCount"], 0);
    assert_eq!(analysis["summary"]["testOnlySupportCount"], 1);
    assert!(analysis["findings"].as_array().is_some_and(Vec::is_empty));

    let candidate = find_excluded(analysis, "test_only_helper")?;
    assert_eq!(candidate["definition"]["visibility"], "private");
    assert_eq!(candidate["fpGates"][0], "RUST-FP-G");
    assert_eq!(
        candidate["actionBlockers"][0],
        "rust-fp-g-test-only-reachability"
    );
    assert_eq!(candidate["safeAction"], serde_json::Value::Null);

    Ok(())
}

#[test]
fn private_test_path_definitions_are_blocked_instead_of_remove_candidates() -> Result<()> {
    let artifact = analyze_file(
        "tests/helper.rs",
        r#"
fn helper_only_for_tests() {}
"#,
    );

    let analysis = &artifact["unusedDefinitionAnalysis"];
    assert_eq!(analysis["summary"]["candidateCount"], 0);
    assert_eq!(analysis["summary"]["testOnlySupportCount"], 1);
    assert!(analysis["findings"].as_array().is_some_and(Vec::is_empty));

    let candidate = find_excluded(analysis, "helper_only_for_tests")?;
    assert_eq!(candidate["fpGates"][0], "RUST-FP-G");
    assert_eq!(
        candidate["actionBlockers"][0],
        "rust-fp-g-test-only-reachability"
    );

    Ok(())
}

#[test]
fn generated_path_definitions_are_muted_instead_of_remove_candidates() -> Result<()> {
    let artifact = analyze_file(
        "src/generated.rs",
        r#"
fn generated_helper() {}
"#,
    );

    let analysis = &artifact["unusedDefinitionAnalysis"];
    assert_eq!(analysis["summary"]["candidateCount"], 0);
    assert_eq!(analysis["summary"]["blockedGeneratedCount"], 1);

    let candidate = find_excluded(analysis, "generated_helper")?;
    assert_eq!(candidate["tier"], "muted");
    assert_eq!(candidate["action"], "muted");
    assert_eq!(candidate["fpGates"][0], "RUST-FP-H");
    assert_eq!(candidate["actionBlockers"][0], "rust-fp-h-generated-source");

    Ok(())
}

#[test]
fn rust_entrypoint_main_is_blocked_instead_of_remove_candidate() -> Result<()> {
    let artifact = analyze_file(
        "build.rs",
        r#"
fn main() {}
"#,
    );

    let analysis = &artifact["unusedDefinitionAnalysis"];
    assert_eq!(analysis["summary"]["candidateCount"], 0);
    assert_eq!(analysis["summary"]["blockedEntrypointCount"], 1);

    let candidate = find_excluded(analysis, "main")?;
    assert_eq!(candidate["fpGates"][0], "RUST-FP-I");
    assert_eq!(candidate["actionBlockers"][0], "rust-fp-i-rust-entrypoint");
    assert_eq!(candidate["safeAction"], serde_json::Value::Null);

    Ok(())
}

#[test]
fn observed_qualified_refs_keep_public_definitions_out_of_excluded_dead_surface() -> Result<()> {
    let artifact = analyze_file(
        "src/lib.rs",
        r#"
pub fn live() {}

pub fn caller() {
    crate::live();
}
"#,
    );

    let analysis = &artifact["unusedDefinitionAnalysis"];
    assert_eq!(analysis["summary"]["candidateCount"], 0);
    assert_eq!(analysis["summary"]["blockedPublicSurfaceCount"], 1);
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
    assert_eq!(analysis["summary"]["testOnlySupportCount"], 2);

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

custom_macro!(external_symbol);
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
