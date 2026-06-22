use anyhow::Result;
use serde_json::Value;
use std::path::Path;

pub(super) fn assert_metadata_and_policy(artifact: &Value) -> Result<()> {
    assert_eq!(artifact["schemaVersion"], "rust-analyzer-health.v1");
    assert_eq!(artifact["meta"]["producer"], "lumin-rust-analyzer");
    assert_eq!(
        artifact["meta"]["input"]["cargoTargetDirMode"],
        "isolated-temp"
    );
    assert_eq!(
        artifact["meta"]["input"]["cargoTargetDirPolicy"]["repoTargetDirUsed"],
        false
    );
    assert_eq!(
        artifact["meta"]["input"]["cargoTargetDirPolicy"]["ownedTempTargetDir"],
        true
    );
    assert_eq!(
        artifact["meta"]["input"]["cargoTargetDirPolicy"]["incrementalDisabled"],
        true
    );
    assert_eq!(
        artifact["meta"]["input"]["cargoTargetDirPolicy"]["debugSymbolsDisabled"],
        true
    );
    assert_eq!(
        artifact["meta"]["input"]["cargoTargetDirPolicy"]["staleCleanupOwnedTempTargetDirs"],
        true
    );
    assert_eq!(
        artifact["meta"]["input"]["cargoTargetDirPolicy"]["staleIsolatedTargetDirMaxAgeSeconds"],
        86_400
    );
    assert_eq!(
        artifact["meta"]["input"]["cargoTargetDirPolicy"]["staleReusableTargetDirMaxAgeSeconds"],
        604_800
    );
    let cargo_target_dir = artifact["meta"]["input"]["cargoTargetDir"]
        .as_str()
        .unwrap_or_default();
    assert!(cargo_target_dir.contains("lumin-rust-cargo-oracle-target-"));
    assert!(!Path::new(cargo_target_dir).exists());
    assert_eq!(artifact["policy"]["owner"], "lumin-rust-analyzer");
    assert_eq!(
        artifact["policy"]["jsTsPrecedent"][0],
        "_lib/finding-provenance.mjs"
    );
    assert_eq!(artifact["policy"]["syntax"]["visibility"]["muted"], "muted");
    assert_eq!(artifact["policy"]["syntax"]["claim"], "syntax-only");
    assert_eq!(artifact["policy"]["syntax"]["confidenceTier"], "candidate");
    assert_eq!(artifact["policy"]["syntax"]["rawEvidencePreserved"], true);
    assert_eq!(
        artifact["policy"]["syntax"]["productProjection"]["rawLaneOwner"],
        "rust-source-health"
    );
    assert_eq!(
        artifact["policy"]["syntax"]["productProjection"]["parse"],
        "status-and-capped-error-examples"
    );
    assert_eq!(
        artifact["policy"]["syntax"]["productProjection"]["sampleLimits"]["signals"],
        3
    );
    assert_eq!(
        artifact["policy"]["syntax"]["productProjection"]["sampleLimits"]["fileSignals"],
        1
    );
    assert_eq!(
        artifact["policy"]["syntax"]["productProjection"]["sampleLimits"]["parseErrors"],
        3
    );
    assert_eq!(
        artifact["policy"]["syntax"]["productProjection"]["sampleLimits"]["skippedFiles"],
        3
    );
    assert_eq!(
        artifact["policy"]["syntax"]["productProjection"]["sampleLimits"]["defaultAst"],
        3
    );
    assert_eq!(
        artifact["policy"]["syntax"]["productProjection"]["sampleLimits"]["fileAst"],
        1
    );
    assert_eq!(
        artifact["policy"]["actionTiers"]["safeFixGate"],
        "requires-proof-carrying-edit-action"
    );
    assert_eq!(
        artifact["policy"]["oracleBridge"]["rustParserLane"],
        "ra_ap_syntax via rust-source-health"
    );
    assert_eq!(
        artifact["policy"]["oracleBridge"]["rustOracleLane"],
        "Cargo/rustc via rust-cargo-oracle"
    );
    assert_eq!(
        artifact["policy"]["artifactContract"]["jsTsPrecedent"],
        "_lib/rust-topology-prefer.mjs"
    );
    assert_eq!(
        artifact["policy"]["artifactContract"]["failureReason"],
        "blocked-artifact-contract"
    );
    assert_eq!(
        artifact["policy"]["semantic"]["coverageUnavailableStatus"],
        "unavailable"
    );
    assert_eq!(artifact["policy"]["semantic"]["rawEvidencePreserved"], true);
    assert_eq!(
        artifact["policy"]["semantic"]["rawEvidenceEmbeddedInProduct"],
        false
    );
    assert_eq!(
        artifact["policy"]["semantic"]["productProjection"]["coverage"],
        "summary-and-capped-scope-examples"
    );
    assert_eq!(
        artifact["policy"]["semantic"]["productProjection"]["rawLaneOwner"],
        "rust-cargo-oracle"
    );
    assert_eq!(
        artifact["policy"]["semantic"]["productProjection"]["sampleLimits"]["oracleScope"],
        3
    );
    Ok(())
}
