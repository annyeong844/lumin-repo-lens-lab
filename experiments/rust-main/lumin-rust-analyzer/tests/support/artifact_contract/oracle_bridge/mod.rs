use anyhow::{Context, Result};
use serde_json::Value;

pub(super) fn assert_oracle_bridge_projection(artifact: &Value) -> Result<()> {
    assert_eq!(artifact["oracleBridge"]["status"], "oracle-partial");
    assert_eq!(
        artifact["oracleBridge"]["syntax"]["reviewOpaqueSurfaces"],
        2
    );
    assert_eq!(artifact["oracleBridge"]["syntax"]["mutedOpaqueSurfaces"], 1);
    assert_eq!(
        artifact["oracleBridge"]["coverage"]["cargoEventStream"]["status"],
        "ran"
    );
    assert_eq!(
        artifact["oracleBridge"]["coverage"]["absenceClean"]["status"],
        "unavailable"
    );
    assert!(artifact["oracleBridge"]["coverage"]["absenceClean"]["reason"].is_string());
    assert_eq!(
        artifact["oracleBridge"]["semantic"]["semanticClean"]["status"],
        "unavailable"
    );
    assert!(artifact["oracleBridge"]["semantic"]["semanticClean"]["reason"].is_string());
    assert_eq!(
        artifact["oracleBridge"]["policy"]["calibrationStatus"],
        "pending"
    );
    assert_eq!(
        artifact["oracleBridge"]["policy"]["calibration"]["status"],
        "pending"
    );
    assert_eq!(
        artifact["oracleBridge"]["policy"]["calibration"]["reason"],
        "rust-safe-fix-calibration-corpus-not-measured"
    );
    let candidate_counts = &artifact["oracleBridge"]["policy"]["calibration"]["candidateCounts"];
    assert_eq!(candidate_counts["muted"], 0);
    assert_eq!(candidate_counts["syntaxMutedEvidence"], 3);
    assert_eq!(
        artifact["oracleBridge"]["policy"]["calibration"]["requiredEvidence"][0],
        "non-empty-safe-fix-population"
    );
    assert_eq!(
        artifact["oracleBridge"]["policy"]["calibration"]["requiredEvidence"][1],
        "known-safe-fix-fp-denominator"
    );
    assert_eq!(
        artifact["oracleBridge"]["policy"]["calibration"]["requiredEvidence"][2],
        "readiness-gate-from-real-corpus"
    );
    let readiness_policy = &artifact["oracleBridge"]["policy"]["calibration"]["readinessPolicy"];
    assert_eq!(
        readiness_policy["source"],
        "_lib/p6-measurement.mjs::computeReadiness"
    );
    assert_eq!(readiness_policy["safeFixFpRedThreshold"], 0.05);
    assert_eq!(readiness_policy["reviewVisibleFpRedThreshold"], 0.25);
    assert_eq!(readiness_policy["reviewVisibleFpGreenThreshold"], 0.1);
    assert_eq!(readiness_policy["minNonTrivialCorpus"], 2);
    assert_eq!(readiness_policy["defaultMinAdjudicatedPerCorpus"], 50);
    assert_eq!(
        artifact["oracleBridge"]["policy"]["calibration"]["jsTsPrecedent"]["measurementArtifact"],
        "p6-measurement.json"
    );
    assert_eq!(
        artifact["oracleBridge"]["policy"]["calibration"]["jsTsPrecedent"]["measurementOwner"],
        "_lib/p6-measurement.mjs"
    );
    assert_eq!(
        artifact["oracleBridge"]["policy"]["calibration"]["jsTsPrecedent"]["readinessGateOwner"],
        "_lib/p6-measurement.mjs::computeReadiness"
    );
    assert_eq!(
        artifact["oracleBridge"]["policy"]["calibration"]["jsTsPrecedent"]
            ["calibrationCorpusRegistry"],
        "_lib/calibration-corpora.mjs"
    );
    assert_eq!(
        artifact["oracleBridge"]["policy"]["calibration"]["jsTsPrecedent"]
            ["thresholdPolicyMetadata"],
        "_lib/threshold-policies.mjs"
    );
    let policy = artifact["oracleBridge"]["policy"]
        .as_object()
        .context("oracle bridge policy")?;
    assert_eq!(policy["doesNotPromoteSafeFix"], true);
    assert_eq!(policy["policyExclusionsRemainAuditable"], true);
    assert!(
        !policy.contains_key("doesNotMutePublicSurface"),
        "public-surface claims must be backed by a Rust public-surface owner"
    );
    Ok(())
}

pub(super) fn assert_top_level_coverage(artifact: &Value) -> Result<()> {
    let coverage = artifact["coverage"].as_array().context("coverage array")?;
    for entry in coverage {
        assert!(entry.get("command").is_none());
        assert!(entry.get("commandArgs").is_none());
        assert!(
            entry["commandArgCount"]
                .as_u64()
                .context("coverage command arg count")?
                > 0
        );
        assert!(entry.get("elapsedMs").is_none());
        assert!(entry.get("analysisInputSetHash").is_none());
        assert!(entry.get("registryContentHash").is_none());
        assert!(entry.get("diagnosticPolicyVersion").is_none());
        assert!(entry["scope"].get("cfgSet").is_none());
        assert!(entry["scope"].get("packageNames").is_none());
        assert!(entry["scope"].get("targets").is_none());
        assert!(entry["scope"].get("featureSet").is_none());
        assert!(entry["scope"].get("targetTriples").is_none());
        assert!(
            entry["scope"]["targetCount"]
                .as_u64()
                .context("coverage scope target count")?
                > 0
        );
    }
    assert!(
        coverage
            .iter()
            .any(|entry| entry["id"] == "cov.cargo-check.absence-clean"
                && entry["status"] == "unavailable"),
        "coverage: {}",
        serde_json::to_string_pretty(&artifact["coverage"])?
    );
    Ok(())
}
