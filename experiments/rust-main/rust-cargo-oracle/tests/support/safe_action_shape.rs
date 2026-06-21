use anyhow::{Context, Result};
use serde_json::Value;

pub fn assert_rule_backed_safe_action(artifact: &Value) -> Result<()> {
    assert_rule_backed_safe_action_with_diagnostic_code(artifact, "unused_mut")
}

pub fn assert_rule_backed_safe_action_with_diagnostic_code(
    artifact: &Value,
    diagnostic_code: &str,
) -> Result<()> {
    assert_rule_backed_safe_action_with_edit(artifact, diagnostic_code, "")
}

pub fn assert_rule_backed_safe_action_with_edit(
    artifact: &Value,
    diagnostic_code: &str,
    replacement: &str,
) -> Result<()> {
    assert_rule_backed_safe_action_with_edits(artifact, diagnostic_code, &[replacement])
}

pub fn assert_rule_backed_safe_action_with_edits(
    artifact: &Value,
    diagnostic_code: &str,
    replacements: &[&str],
) -> Result<()> {
    let finding = crate::support::findings::single::single_finding(artifact)?;
    assert_confidence(finding)?;
    assert_safe_action(finding, diagnostic_code, replacements)
}

pub fn assert_rule_backed_safe_action_contract(artifact: &Value) -> Result<()> {
    let finding = crate::support::findings::single::single_finding(artifact)?;
    assert_confidence(finding)?;
    assert_span_shape(finding)?;
    assert_safe_action(finding, "unused_mut", &[""])?;
    assert_clean_semantic_coverage(artifact)
}

fn assert_confidence(finding: &Value) -> Result<()> {
    assert_eq!(finding["confidence"]["tier"], "rule-backed");
    assert_eq!(
        finding["confidence"]["claimKind"],
        "rule-backed.rust.rustc-lint-diagnostic"
    );
    assert!(finding["confidence"]["authorityIds"]
        .as_array()
        .context("authorityIds")?
        .is_empty());
    assert_eq!(
        finding["confidence"]["ruleIds"][0],
        "rust.rustc.lint-diagnostic"
    );
    Ok(())
}

fn assert_span_shape(finding: &Value) -> Result<()> {
    assert!(finding["span"]["fileName"]
        .as_str()
        .context("span fileName")?
        .ends_with("lib.rs"));
    assert!(finding["span"].get("file_name").is_none());
    assert!(finding["span"].get("lineStart").is_some());
    assert!(finding["span"].get("line_start").is_none());
    assert_eq!(finding["span"]["primarySpanClass"], "user-code");
    assert!(finding["primarySpans"][0]["fileName"]
        .as_str()
        .context("primarySpans[0] fileName")?
        .ends_with("lib.rs"));
    assert!(finding["primarySpans"][0].get("file_name").is_none());
    Ok(())
}

fn assert_safe_action(finding: &Value, diagnostic_code: &str, replacements: &[&str]) -> Result<()> {
    let safe_action = &finding["safeAction"];
    assert_eq!(
        safe_action["kind"],
        "apply-rustc-machine-applicable-suggestion"
    );
    assert_eq!(safe_action["proofComplete"], true);
    assert!(safe_action["actionBlockers"]
        .as_array()
        .context("selected action blockers")?
        .is_empty());
    assert!(safe_action["strongerActionBlockers"]
        .as_array()
        .context("stronger action blockers")?
        .is_empty());
    assert!(finding["actionBlockers"]
        .as_array()
        .context("finding action blockers")?
        .is_empty());
    let edits = safe_action["edits"]
        .as_array()
        .context("safeAction edits")?;
    assert_eq!(edits.len(), replacements.len());
    for (edit, replacement) in edits.iter().zip(replacements.iter()) {
        assert!(edit["fileName"]
            .as_str()
            .context("safeAction edit fileName")?
            .ends_with("lib.rs"));
        assert!(edit.get("file_name").is_none());
        assert_eq!(edit["lineStart"], 1);
        assert!(edit.get("line_start").is_none());
        assert_eq!(edit["replacement"], *replacement);
    }
    assert_eq!(safe_action["proof"]["diagnosticCode"], diagnostic_code);
    assert!(safe_action["proof"].get("diagnostic_code").is_none());
    assert_eq!(safe_action["proof"]["applicability"], "MachineApplicable");
    Ok(())
}

fn assert_clean_semantic_coverage(artifact: &Value) -> Result<()> {
    let absence = crate::support::coverage::coverage(artifact, "cov.cargo-check.absence-clean")?;
    assert_eq!(absence["status"], "ran");
    assert_eq!(absence["clean"], true);
    assert_eq!(absence["cleanKind"], "verified-rustc-error-absence");
    assert_eq!(artifact["summary"]["semanticClean"]["status"], "ran");
    assert_eq!(artifact["summary"]["semanticClean"]["clean"], true);
    assert_eq!(
        artifact["summary"]["semanticClean"]["cleanKind"],
        "verified-rustc-error-absence"
    );
    assert_eq!(artifact["summary"]["cacheReuse"]["status"], "not-reusable");
    Ok(())
}
