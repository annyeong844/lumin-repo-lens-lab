use lumin_rust_cargo_oracle::protocol::ConfidenceTier;
use serde::{Serialize, Serializer};

mod cleanup;
mod projection;
mod status;

pub(crate) use cleanup::{normalize_candidate_file, CleanupCandidate};
pub(crate) use projection::{
    ProductCacheReuseSummaryProjection, ProductCoverageUnavailableReason,
    ProductPrimarySpanProjection, ProductSemanticCleanSummaryProjection,
};
pub(crate) use status::{
    ActionTier, CalibrationStatus, CoverageRunStatus, DegradedReason, FileParseStatus,
    OracleBridgeStatus, OracleConfidence,
};

pub(crate) const POLICY_VERSION: &str = "rust-unified-analyzer.v1";
pub(crate) const SYNTAX_CONFIDENCE_TIER: ConfidenceTier = ConfidenceTier::Candidate;
pub(crate) const ACTION_SAMPLE_LIMIT: usize = 5;
pub(crate) const SIGNAL_SAMPLE_LIMIT: usize = 3;
pub(crate) const FILE_SIGNAL_SAMPLE_LIMIT: usize = 1;
pub(crate) const PARSE_ERROR_SAMPLE_LIMIT: usize = 3;
pub(crate) const SKIPPED_FILE_SAMPLE_LIMIT: usize = 3;
pub(crate) const AST_SAMPLE_LIMIT: usize = 3;
pub(crate) const FILE_AST_SAMPLE_LIMIT: usize = 1;
pub(crate) const DEFINITION_SAMPLE_LIMIT: usize = 2;
pub(crate) const USE_TREE_SAMPLE_LIMIT: usize = 2;
pub(crate) const ORACLE_SCOPE_SAMPLE_LIMIT: usize = 3;
pub(crate) const SEMANTIC_FINDING_SPAN_SAMPLE_LIMIT: usize = 3;

#[derive(Debug, Copy, Clone, Default)]
pub(crate) struct RawLaneOmitted;

impl Serialize for RawLaneOmitted {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_bool(false)
    }
}
