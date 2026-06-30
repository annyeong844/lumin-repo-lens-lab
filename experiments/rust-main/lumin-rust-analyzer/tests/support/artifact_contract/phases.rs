use anyhow::{Context, Result};
use serde_json::Value;

pub(super) fn assert_phase_projection(artifact: &Value) -> Result<()> {
    assert_eq!(artifact["phases"]["syntax"]["rawEmbedded"], false);
    assert!(artifact["phases"]["syntax"]["files"].is_null());
    assert!(artifact["phases"]["syntax"].get("skippedFiles").is_none());
    assert_eq!(
        artifact["phases"]["syntax"]["skippedFileCount"],
        artifact["phases"]["syntax"]["skippedFileExamples"]
            .as_array()
            .context("syntax skipped file examples")?
            .len()
    );
    assert_eq!(artifact["phases"]["syntax"]["summary"]["pathRefs"], 1);
    assert_eq!(artifact["phases"]["syntax"]["summary"]["shapeHashes"], 1);
    assert_eq!(
        artifact["phases"]["syntax"]["summary"]["functionSignatures"],
        10
    );
    assert_eq!(
        artifact["phases"]["syntax"]["summary"]["functionBodyFingerprints"],
        10
    );
    assert_eq!(
        artifact["phases"]["syntax"]["summary"]["functionCloneExactBodyGroups"],
        1
    );
    assert_eq!(
        artifact["phases"]["syntax"]["summary"]["functionCloneStructureGroups"],
        1
    );
    assert_eq!(
        artifact["phases"]["syntax"]["summary"]["functionCloneSignatureGroups"],
        0
    );
    assert_eq!(
        artifact["phases"]["syntax"]["summary"]["functionCloneNearCandidates"],
        0
    );
    assert_eq!(
        artifact["phases"]["syntax"]["summary"]["functionCloneNearCandidateProjectionLimit"],
        50
    );
    assert_eq!(
        artifact["phases"]["syntax"]["summary"]["functionCloneCandidateGenerationPolicy"]["mode"],
        "bounded-retrieval"
    );
    assert_eq!(
        artifact["phases"]["syntax"]["summary"]["functionCloneCandidateGenerationPolicy"]
            ["retrievalContractVersion"],
        "function-clone-near-retrieval.v1"
    );
    assert_eq!(
        artifact["phases"]["syntax"]["summary"]["functionCloneCandidateGenerationSummary"]
            ["nearFunctionCandidateCountScope"],
        "scored-candidates-from-retained-retrieval-evidence"
    );
    assert_eq!(
        artifact["phases"]["syntax"]["summary"]
            ["functionCloneSkippedLowDiscriminationPairEstimateKind"],
        "raw-bucket-pairs-may-double-count-pairs-shared-by-multiple-skipped-tokens"
    );
    assert!(artifact["phases"]["syntax"]["summary"]
        ["functionCloneSkippedLowDiscriminationBuckets"]
        .is_array());
    assert_eq!(artifact["phases"]["syntax"]["summary"]["inlinePatterns"], 3);
    assert!(artifact["phases"]["syntax"]["summary"]
        .get("signalsByKind")
        .is_none());
    assert!(artifact["phases"]["syntax"]["summary"]
        .get("mutedSignalsByReason")
        .is_none());
    let syntax_meta = &artifact["phases"]["syntax"]["meta"];
    assert_eq!(syntax_meta["producer"], "rust-source-health");
    assert_eq!(syntax_meta["parser"]["kind"], "ra_ap_syntax");
    assert_eq!(syntax_meta["parser"]["editionPolicy"], "fixed");
    assert_eq!(syntax_meta["runtime"]["workerStackBytes"], 4 * 1024 * 1024);
    assert_eq!(syntax_meta["sidecar"]["sourceCommit"], "test-source-commit");
    assert!(syntax_meta["sidecar"]["binarySha256"]
        .as_str()
        .context("syntax sidecar binary hash")?
        .starts_with("sha256:"));
    assert!(syntax_meta.get("generated").is_none());
    assert!(syntax_meta.get("input").is_none());
    assert_eq!(artifact["phases"]["semantic"]["rawEmbedded"], false);
    assert_eq!(artifact["phases"]["semantic"]["findingCount"], 1);
    assert!(artifact["phases"]["semantic"]["summary"]
        .get("semanticClean")
        .is_none());
    assert!(artifact["phases"]["semantic"]["summary"]
        .get("cacheReuse")
        .is_none());
    assert!(artifact["phases"]["semantic"]["coverage"].is_null());
    assert!(artifact["phases"]["semantic"]["oraclePlan"].is_null());
    assert_eq!(
        artifact["phases"]["semantic"]["meta"]["producer"],
        "rust-cargo-oracle"
    );
    assert_eq!(
        artifact["phases"]["semantic"]["meta"]["analysisInputSetComplete"],
        false
    );
    assert_eq!(
        artifact["phases"]["semantic"]["meta"]["missingInfluenceKindCount"],
        artifact["phases"]["semantic"]["meta"]["missingInfluenceKinds"]
            .as_array()
            .context("semantic phase missing influence kinds")?
            .len()
    );
    assert!(artifact["phases"]["semantic"]["meta"]
        .get("registryContentHash")
        .is_none());
    assert!(artifact["phases"]["semantic"]["meta"]
        .get("analysisInputSetHash")
        .is_none());
    assert!(artifact["phases"]["semantic"]["meta"]
        .get("input")
        .is_none());
    assert!(artifact["phases"]["semantic"]["meta"]
        .get("cacheReuse")
        .is_none());
    assert!(artifact["phases"]["semantic"]["meta"]["toolchain"]
        .get("rustcVersionVerbose")
        .is_none());
    assert!(artifact["phases"]["semantic"]["meta"]["toolchain"]
        .get("rustcBin")
        .is_none());
    assert!(artifact["coverage"].is_array());
    assert!(artifact["oraclePlan"].is_object());
    let semantic_findings = artifact["semanticFindings"]
        .as_array()
        .context("semantic findings")?;
    assert_eq!(
        semantic_findings[0]["confidence"]["claimKind"],
        "verified.rust.rustc-error-diagnostic"
    );
    assert_eq!(semantic_findings[0]["actionTier"], "REVIEW_FIX");
    assert_eq!(semantic_findings[0]["parseStatus"], "ok");
    assert_eq!(semantic_findings[0]["oracleConfidence"], "medium");
    Ok(())
}
