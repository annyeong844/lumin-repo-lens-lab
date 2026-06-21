use std::collections::BTreeSet;

use anyhow::Result;
use serde_json::Value;

pub(super) fn assert_source_semantic_projection(
    artifact: &Value,
    merged_file: &Value,
) -> Result<()> {
    let semantic_finding_index = merged_file["semantic"]["findings"][0]["index"]
        .as_u64()
        .ok_or_else(|| anyhow::anyhow!("file semantic finding index"))?
        as usize;
    let semantic_finding = artifact["semanticFindings"]
        .as_array()
        .ok_or_else(|| anyhow::anyhow!("top-level semantic findings"))?
        .get(semantic_finding_index)
        .ok_or_else(|| anyhow::anyhow!("file semantic finding ref target"))?;
    let confidence = semantic_finding["confidence"]
        .as_object()
        .ok_or_else(|| anyhow::anyhow!("semantic finding confidence"))?;
    let confidence_keys = confidence
        .keys()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    assert_eq!(
        confidence_keys,
        BTreeSet::from(["authorityIds", "claimKind", "ruleIds", "tier"])
    );
    assert_eq!(
        confidence["claimKind"],
        "verified.rust.rustc-error-diagnostic"
    );
    assert_eq!(confidence["tier"], "verified");
    assert!(!confidence["authorityIds"]
        .as_array()
        .ok_or_else(|| anyhow::anyhow!("semantic finding confidence authorityIds"))?
        .is_empty());
    assert!(confidence["ruleIds"]
        .as_array()
        .ok_or_else(|| anyhow::anyhow!("semantic finding confidence ruleIds"))?
        .is_empty());
    assert!(semantic_finding.get("source").is_none());
    assert!(semantic_finding.get("confidenceTier").is_none());
    assert!(semantic_finding.get("claimKind").is_none());
    assert!(semantic_finding.get("analysisInputSetHash").is_none());
    assert!(semantic_finding.get("rule").is_none());
    assert!(semantic_finding.get("primarySpans").is_none());
    assert!(
        semantic_finding["primarySpanCount"]
            .as_u64()
            .ok_or_else(|| anyhow::anyhow!("semantic finding primary span count"))?
            > 0
    );
    assert_eq!(semantic_finding["actionTier"], "REVIEW_FIX");
    assert_eq!(semantic_finding["parseStatus"], "ok");
    assert_eq!(semantic_finding["oracleConfidence"], "medium");

    assert!(semantic_finding["supportedBy"]
        .as_array()
        .ok_or_else(|| anyhow::anyhow!("finding support"))?
        .iter()
        .any(|entry| entry["kind"] == "cargo-rustc-diagnostic"));
    assert!(semantic_finding["taintedBy"]
        .as_array()
        .ok_or_else(|| anyhow::anyhow!("finding taint"))?
        .iter()
        .any(|entry| entry["kind"] == "cargo-absence-clean-unavailable"));
    assert!(!semantic_finding["taintedBy"]
        .as_array()
        .ok_or_else(|| anyhow::anyhow!("finding taint"))?
        .iter()
        .any(|entry| entry["kind"] == "rust-ast-review-opaque-surface-near-finding"));

    let diagnostic_index = merged_file["semantic"]["diagnostics"][0]["index"]
        .as_u64()
        .ok_or_else(|| anyhow::anyhow!("file semantic diagnostic index"))?
        as usize;
    let file_diagnostics = merged_file["semantic"]["diagnostics"]
        .as_array()
        .ok_or_else(|| anyhow::anyhow!("file semantic diagnostics"))?;
    assert_eq!(file_diagnostics.len(), 1);

    let diagnostics = artifact["semanticDiagnostics"]
        .as_array()
        .ok_or_else(|| anyhow::anyhow!("top-level semantic diagnostics"))?;
    assert_eq!(diagnostics.len(), 2);
    let diagnostic = artifact["semanticDiagnostics"]
        .get(diagnostic_index)
        .ok_or_else(|| anyhow::anyhow!("file semantic diagnostic ref target"))?;
    assert_eq!(diagnostic["classification"]["disposition"], "finding");
    assert!(diagnostic.get("rawCode").is_none());
    assert!(diagnostic.get("renderedFirstLine").is_none());
    assert_eq!(diagnostic["normalized"]["codeKind"], "rustc-error-code");
    assert!(diagnostic["primarySpan"]["fileName"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("semantic diagnostic representative primary span"))?
        .ends_with("src/lib.rs"));
    assert!(diagnostic["primarySpan"].get("expansion").is_none());
    assert!(
        diagnostic["primarySpanCount"]
            .as_u64()
            .ok_or_else(|| anyhow::anyhow!("semantic diagnostic primary span count"))?
            > 0
    );
    assert!(diagnostic.get("primarySpans").is_none());

    let codeless_diagnostic_index = diagnostics
        .iter()
        .enumerate()
        .filter_map(|(index, diagnostic)| (diagnostic["primarySpanCount"] == 0).then_some(index))
        .collect::<Vec<_>>();
    assert_eq!(codeless_diagnostic_index.len(), 1);
    assert_ne!(diagnostic_index, codeless_diagnostic_index[0]);
    let codeless_diagnostic = &diagnostics[codeless_diagnostic_index[0]];
    assert!(codeless_diagnostic.get("primarySpan").is_none());

    Ok(())
}
