use anyhow::Result;
use serde_json::Value;

pub(super) fn assert_safe_action_policy(artifact: &Value) -> Result<()> {
    assert_safe_action_policy_with_diagnostic_code(artifact, "unused_mut")
}

pub(super) fn assert_safe_action_policy_with_diagnostic_code(
    artifact: &Value,
    diagnostic_code: &str,
) -> Result<()> {
    assert_safe_action_policy_with_edit(artifact, diagnostic_code, "")
}

pub(super) fn assert_safe_action_policy_with_edit(
    artifact: &Value,
    diagnostic_code: &str,
    replacement: &str,
) -> Result<()> {
    assert_safe_action_policy_with_edits(artifact, diagnostic_code, &[replacement])
}

pub(super) fn assert_safe_action_policy_with_edits(
    artifact: &Value,
    diagnostic_code: &str,
    replacements: &[&str],
) -> Result<()> {
    assert_eq!(
        artifact["actionPolicy"]["reasons"]["SAFE_FIX"]["semantic-safe-action"],
        1
    );
    assert_eq!(
        artifact["actionPolicy"]["semanticFindingConfidence"]["safeAction"],
        1
    );

    let safe_action = &artifact["actionPolicy"]["semanticSafeActions"]["examples"][0]["safeAction"];
    let span = &artifact["actionPolicy"]["semanticSafeActions"]["examples"][0]["span"];
    assert!(span["fileName"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("safe action policy example span fileName"))?
        .ends_with("lib.rs"));
    assert!(span.get("file_name").is_none());
    assert!(span.get("lineStart").is_some());
    assert!(span.get("line_start").is_none());
    assert!(span.get("expansion").is_none());
    assert_eq!(
        safe_action["kind"],
        "apply-rustc-machine-applicable-suggestion"
    );
    assert_eq!(safe_action["editCount"], replacements.len());
    assert_eq!(safe_action["firstEdit"]["replacement"], replacements[0]);
    assert_eq!(safe_action["proof"]["diagnosticCode"], diagnostic_code);

    assert!(
        artifact["actionPolicy"]["semanticActionBlockers"]["examples"]
            .as_array()
            .ok_or_else(|| anyhow::anyhow!("action blocker examples"))?
            .is_empty()
    );
    assert!(artifact["actionPolicy"]["semanticReview"]["examples"]
        .as_array()
        .ok_or_else(|| anyhow::anyhow!("semantic review examples"))?
        .is_empty());

    Ok(())
}
