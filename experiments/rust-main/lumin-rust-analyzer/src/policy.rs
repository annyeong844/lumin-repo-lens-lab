mod action;
mod bridge;
mod evidence;
mod file;
mod finding;
mod metadata;
mod semantic;
mod semantic_examples;
mod span_overlap;
mod syntax;
mod types;

pub(crate) use action::ActionPolicyProjection;
pub(crate) use action::{action_policy, ActionPolicy};
pub(crate) use action::{ActionPolicyTier, ActionTierSummary, EvidenceTierSummary};
pub(crate) use bridge::OracleBridgeProjection;
pub(crate) use bridge::{oracle_bridge, OracleBridge};
pub(crate) use evidence::CoverageEvidence;
pub(crate) use file::{
    product_file_oracle_bridge, ProductFileOracleBridgeProjection, ProductFileSemanticSummary,
};
pub(crate) use finding::{product_semantic_finding, ProductSemanticFindingProjection};
pub(crate) use metadata::{policy_metadata, PolicyMetadata};
pub(crate) use syntax::{
    product_syntax_file, syntax_review_opaque_surface_examples, syntax_review_signal_examples,
    ProductSyntaxFileProjection, ProductSyntaxFileSummary, SyntaxReviewOpaqueSurfaceExample,
    SyntaxReviewSignalExample,
};
pub(crate) use types::{
    normalize_candidate_file, ActionTier, CalibrationStatus, CleanupCandidate, CoverageRunStatus,
    DegradedReason, FileParseStatus, OracleBridgeStatus, OracleConfidence,
    ProductCacheReuseSummaryProjection, ProductCoverageUnavailableReason,
    ProductPrimarySpanProjection, ProductSemanticCleanSummaryProjection, RawLaneOmitted,
    ACTION_SAMPLE_LIMIT, AST_SAMPLE_LIMIT, DEFINITION_SAMPLE_LIMIT, FILE_AST_SAMPLE_LIMIT,
    FILE_SIGNAL_SAMPLE_LIMIT, ORACLE_SCOPE_SAMPLE_LIMIT, PARSE_ERROR_SAMPLE_LIMIT, POLICY_VERSION,
    SIGNAL_SAMPLE_LIMIT, SKIPPED_FILE_SAMPLE_LIMIT, SYNTAX_CONFIDENCE_TIER, USE_TREE_SAMPLE_LIMIT,
};
