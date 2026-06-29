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
