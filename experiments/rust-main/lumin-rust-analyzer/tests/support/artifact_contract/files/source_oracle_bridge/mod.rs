use anyhow::Result;
use serde_json::Value;

pub(super) fn assert_source_oracle_bridge_projection(
    artifact: &Value,
    merged_file: &Value,
) -> Result<()> {
    assert_eq!(
        artifact["policy"]["oracleBridge"]["jsTsPrecedent"]["parser"],
        "_lib/parse-oxc.mjs"
    );
    assert_eq!(
        artifact["policy"]["oracleBridge"]["jsTsPrecedent"]["oracle"],
        "_lib/tsconfig-paths.mjs"
    );
    assert_eq!(
        artifact["policy"]["oracleBridge"]["jsTsPrecedent"]["provenance"],
        "_lib/finding-provenance.mjs"
    );
    assert!(merged_file["oracleBridge"]["jsTsPrecedent"].is_null());
    assert!(merged_file["oracleBridge"]["coverage"].is_null());
    assert!(merged_file["oracleBridge"]["policy"].is_null());
    assert!(merged_file["oracleBridge"].get("schemaVersion").is_none());
    assert!(merged_file["oracleBridge"].get("status").is_none());
    assert!(merged_file["oracleBridge"].get("parseStatus").is_none());
    assert_eq!(merged_file["oracleBridge"]["oracleConfidence"], "medium");

    assert_eq!(
        merged_file["oracleBridge"]["syntax"]["reviewOpaqueSurfaces"],
        2
    );
    assert_eq!(merged_file["oracleBridge"]["semantic"]["findings"], 1);
    assert_eq!(merged_file["oracleBridge"]["semantic"]["diagnostics"], 1);
    assert_eq!(
        merged_file["oracleBridge"]["semantic"]["actionBlockedFindings"],
        1
    );
    assert!(merged_file["oracleBridge"]["semantic"]
        .get("reviewFindings")
        .is_none());

    assert!(merged_file["oracleBridge"]["supportedBy"]
        .as_array()
        .ok_or_else(|| anyhow::anyhow!("file bridge support"))?
        .iter()
        .any(|entry| entry["kind"] == "cargo-rustc-diagnostics"));
    assert!(merged_file["oracleBridge"]["taintedBy"]
        .as_array()
        .ok_or_else(|| anyhow::anyhow!("file bridge taint"))?
        .iter()
        .any(|entry| entry["kind"] == "rust-ast-review-opaque-surface"));
    assert!(!merged_file["oracleBridge"]["taintedBy"]
        .as_array()
        .ok_or_else(|| anyhow::anyhow!("file bridge taint"))?
        .iter()
        .any(|entry| entry["kind"] == "cargo-absence-clean-unavailable"));

    Ok(())
}
