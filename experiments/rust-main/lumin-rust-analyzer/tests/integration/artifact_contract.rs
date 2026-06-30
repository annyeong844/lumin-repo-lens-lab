#[path = "../support/artifact_contract/action_policy/mod.rs"]
mod action_policy_contract;
#[path = "../support/artifact_contract/files/mod.rs"]
mod files_contract;
#[path = "../support/artifact_contract/metadata.rs"]
mod metadata_contract;
#[path = "../support/artifact_contract/oracle_bridge/mod.rs"]
mod oracle_bridge_contract;
#[path = "../support/artifact_contract/phases.rs"]
mod phases_contract;
#[path = "../support/artifact_contract/summary/mod.rs"]
mod summary_contract;

use anyhow::{anyhow, Result};
use serde_json::Value;
use std::sync::OnceLock;

use crate::support::scenarios::cargo_check_unified_workspace::analyze_cargo_check_unified_workspace;
use crate::support::scenarios::compact_unified_workspace::{
    analyze_metadata_only_unified_workspace, analyze_metadata_only_unified_workspace_with_args,
};

static UNIFIED_ARTIFACT: OnceLock<Result<Value, String>> = OnceLock::new();

fn unified_artifact() -> Result<&'static Value> {
    UNIFIED_ARTIFACT
        .get_or_init(|| analyze_cargo_check_unified_workspace().map_err(|error| error.to_string()))
        .as_ref()
        .map_err(|error| anyhow!(error.clone()))
}

fn assert_compact_source_health_product_route(artifact: &Value) {
    assert_eq!(artifact["meta"]["input"]["sourceHealthProfile"], "compact");
    assert_eq!(
        artifact["meta"]["input"]["effectiveSourceHealthProfile"],
        "compact"
    );
    assert_eq!(
        artifact["phases"]["syntax"]["meta"]["producer"],
        "rust-source-health"
    );
    assert_eq!(
        artifact["phases"]["syntax"]["meta"]["incremental"]["enabled"],
        true
    );
    assert_eq!(
        artifact["summary"]["syntaxFunctionCloneCandidateGenerationMode"],
        "bounded-retrieval"
    );
    let source_file = &artifact["files"]["src/lib.rs"]["syntax"];
    assert!(source_file.get("signals").is_none());
    assert!(source_file.get("ast").is_none());
}

#[test]
fn unified_cli_emits_action_policy_projection() -> Result<()> {
    action_policy_contract::assert_action_policy_projection(unified_artifact()?)?;
    Ok(())
}

#[test]
fn unified_cli_emits_file_projections() -> Result<()> {
    let artifact = unified_artifact()?;
    files_contract::assert_source_file_projection(artifact)?;
    files_contract::assert_muted_file_projections(artifact)?;
    Ok(())
}

#[test]
fn unified_cli_emits_metadata_and_policy_contract() -> Result<()> {
    metadata_contract::assert_metadata_and_policy(unified_artifact()?)?;
    Ok(())
}

#[test]
fn unified_cli_emits_oracle_bridge_projection() -> Result<()> {
    let artifact = unified_artifact()?;
    oracle_bridge_contract::assert_oracle_bridge_projection(artifact)?;
    oracle_bridge_contract::assert_top_level_coverage(artifact)?;
    Ok(())
}

#[test]
fn unified_cli_emits_syntax_and_semantic_phases_in_one_artifact() -> Result<()> {
    phases_contract::assert_phase_projection(unified_artifact()?)?;
    Ok(())
}

#[test]
fn unified_cli_emits_summary_projection() -> Result<()> {
    summary_contract::assert_summary_projection(unified_artifact()?);
    Ok(())
}

#[test]
fn unified_cli_uses_compact_source_health_by_default_for_metadata_only() -> Result<()> {
    let artifact = analyze_metadata_only_unified_workspace()?;
    assert_compact_source_health_product_route(&artifact);
    Ok(())
}

#[test]
fn unified_cli_preserves_full_source_health_diagnostic_mode() -> Result<()> {
    let artifact = analyze_metadata_only_unified_workspace_with_args(&[
        std::ffi::OsStr::new("--source-health-profile"),
        std::ffi::OsStr::new("full"),
    ])?;
    assert_eq!(artifact["meta"]["input"]["sourceHealthProfile"], "full");
    assert_eq!(
        artifact["meta"]["input"]["effectiveSourceHealthProfile"],
        "full"
    );
    assert!(artifact["phases"]["syntax"]["meta"]
        .get("incremental")
        .is_none());
    let source_file = &artifact["files"]["src/lib.rs"]["syntax"];
    assert!(source_file.get("signals").is_none());
    assert!(source_file.get("ast").is_none());
    Ok(())
}
