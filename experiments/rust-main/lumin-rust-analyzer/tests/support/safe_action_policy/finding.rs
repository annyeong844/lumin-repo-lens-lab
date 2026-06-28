use anyhow::Result;
use serde_json::Value;

pub(super) fn assert_safe_action_finding(artifact: &Value) -> Result<()> {
    assert_safe_action_finding_with_diagnostic_code(artifact, "unused_mut")
}

pub(super) fn assert_safe_action_finding_with_diagnostic_code(
    artifact: &Value,
    diagnostic_code: &str,
) -> Result<()> {
    assert_safe_action_finding_with_edit(artifact, diagnostic_code, "")
}

pub(super) fn assert_safe_action_finding_with_edit(
    artifact: &Value,
    diagnostic_code: &str,
    replacement: &str,
) -> Result<()> {
    assert_safe_action_finding_with_edits(artifact, diagnostic_code, &[replacement])
}

pub(super) fn assert_safe_action_finding_with_edits(
    artifact: &Value,
    diagnostic_code: &str,
    replacements: &[&str],
) -> Result<()> {
    let finding = &artifact["semanticFindings"][0];

    assert_eq!(
        finding["safeAction"]["kind"],
        "apply-rustc-machine-applicable-suggestion"
    );
    assert_eq!(finding["safeAction"]["proofComplete"], true);
    assert_eq!(finding["safeAction"]["editCount"], replacements.len());
    assert!(finding["safeAction"].get("actionBlockers").is_none());
    assert!(finding["safeAction"]
        .get("strongerActionBlockers")
        .is_none());
    assert_eq!(
        finding["safeAction"]["proof"]["diagnosticCode"],
        diagnostic_code
    );
    assert!(finding["actionBlockers"]
        .as_array()
        .ok_or_else(|| anyhow::anyhow!("actionBlockers"))?
        .is_empty());
    let edits = finding["safeAction"]["edits"]
        .as_array()
        .ok_or_else(|| anyhow::anyhow!("safe action edits"))?;
    assert_eq!(edits.len(), replacements.len());
    for (edit, replacement) in edits.iter().zip(replacements.iter()) {
        assert_eq!(edit["replacement"], *replacement);
    }

    assert!(finding["span"]["fileName"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("semantic finding span fileName"))?
        .ends_with("lib.rs"));
    assert!(finding["span"].get("file_name").is_none());
    assert!(finding["span"].get("lineStart").is_some());
    assert!(finding["span"].get("line_start").is_none());
    assert!(finding["span"].get("expansion").is_none());
    assert_eq!(finding["span"]["primarySpanClass"], "user-code");
    assert!(
        finding["primarySpanCount"]
            .as_u64()
            .ok_or_else(|| anyhow::anyhow!("semantic finding primary span count"))?
            > 0
    );
    assert!(finding.get("primarySpans").is_none());

    assert_eq!(finding["actionTier"], "SAFE_FIX");
    assert_eq!(finding["parseStatus"], "ok");
    assert_eq!(finding["oracleConfidence"], "high");

    assert!(finding["supportedBy"]
        .as_array()
        .ok_or_else(|| anyhow::anyhow!("finding support"))?
        .iter()
        .any(|entry| entry["kind"] == "rustc-machine-applicable-safe-action"));
    assert!(finding["taintedBy"]
        .as_array()
        .ok_or_else(|| anyhow::anyhow!("finding taint"))?
        .is_empty());

    Ok(())
}
