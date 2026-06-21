use anyhow::{Context, Result};
use serde_json::Value;

pub fn assert_metadata_only_without_cargo_check(artifact: &Value) -> Result<()> {
    assert_eq!(artifact["meta"]["input"]["semanticMode"], "metadata-only");
    assert_eq!(
        artifact["artifactRefs"]["syntax"]["artifact"],
        "rust-source-health"
    );
    assert_eq!(artifact["artifactRefs"]["syntax"]["rawEmbedded"], false);
    assert_eq!(
        artifact["artifactRefs"]["syntax"]["rawArtifact"]["status"],
        "available-via-compatibility-cli"
    );
    assert_eq!(
        artifact["artifactRefs"]["syntax"]["rawArtifact"]["cli"],
        "lumin-rust-source-health"
    );
    assert_eq!(
        artifact["artifactRefs"]["syntax"]["rawArtifact"]["artifactProfile"],
        "full"
    );
    assert_eq!(
        artifact["artifactRefs"]["semantic"]["artifact"],
        "rust-cargo-oracle"
    );
    assert_eq!(artifact["artifactRefs"]["semantic"]["rawEmbedded"], false);
    assert_eq!(
        artifact["artifactRefs"]["semantic"]["rawArtifact"]["status"],
        "available-via-compatibility-cli"
    );
    assert_eq!(
        artifact["artifactRefs"]["semantic"]["rawArtifact"]["cli"],
        "lumin-rust-cargo-oracle"
    );
    assert_eq!(
        artifact["artifactRefs"]["semantic"]["rawArtifact"]["cargoCheckMode"],
        "cargo-check"
    );
    assert_eq!(
        artifact["artifactRefs"]["semantic"]["defaultMode"],
        "metadata-only"
    );
    assert_eq!(
        artifact["artifactRefs"]["semantic"]["cargoCheckMode"],
        "--semantic-mode cargo-check"
    );
    assert!(artifact["meta"]["phaseTimings"]["syntaxMs"].is_number());
    assert!(artifact["meta"]["phaseTimings"]["semanticMs"].is_number());
    assert!(artifact["meta"]["phaseTimings"]["analyzerMs"].is_number());
    assert_eq!(
        artifact["phases"]["syntax"]["artifact"],
        "rust-source-health"
    );
    assert_eq!(artifact["phases"]["syntax"]["embedded"], "brief");
    assert_eq!(
        artifact["phases"]["semantic"]["artifact"],
        "rust-cargo-oracle"
    );
    assert_eq!(artifact["phases"]["semantic"]["embedded"], "brief");
    assert_eq!(artifact["phases"]["semantic"]["mode"], "metadata-only");
    assert_eq!(artifact["coverage"][0]["streamParseStatus"], "not-run");
    assert_eq!(artifact["coverage"][0]["status"], "unavailable");
    assert!(artifact["coverage"][0]["reason"].is_string());
    assert!(artifact["coverage"][0]["reason"]
        .as_str()
        .unwrap_or_default()
        .contains("metadata-only"));
    assert_eq!(
        artifact["summary"]["semanticClean"]["status"],
        "unavailable"
    );
    assert!(artifact["summary"]["semanticClean"]["reason"].is_string());
    assert!(artifact["summary"]["semanticClean"]["reason"]
        .as_str()
        .unwrap_or_default()
        .contains("metadata-only"));
    assert_eq!(
        artifact["oracleBridge"]["semantic"]["semanticClean"]["status"],
        "unavailable"
    );
    assert!(artifact["oracleBridge"]["semantic"]["semanticClean"]["reason"].is_string());
    assert!(artifact["semanticFindings"]
        .as_array()
        .context("semantic findings")?
        .is_empty());
    assert_eq!(
        artifact["summary"]["oracleBridgeStatus"],
        "oracle-unavailable"
    );
    Ok(())
}
