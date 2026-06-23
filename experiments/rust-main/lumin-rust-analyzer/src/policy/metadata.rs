use serde::Serialize;

use lumin_rust_cargo_oracle::protocol::{ConfidenceTier, CoverageStatus};
use lumin_rust_source_health::protocol::SignalVisibility;

use super::{
    action::ActionPolicyTier, AST_SAMPLE_LIMIT, DEFINITION_SAMPLE_LIMIT, ORACLE_SCOPE_SAMPLE_LIMIT,
    PARSE_ERROR_SAMPLE_LIMIT, POLICY_VERSION, SIGNAL_SAMPLE_LIMIT, SKIPPED_FILE_SAMPLE_LIMIT,
    SYNTAX_CONFIDENCE_TIER, USE_TREE_SAMPLE_LIMIT,
};
use super::{FILE_AST_SAMPLE_LIMIT, FILE_SIGNAL_SAMPLE_LIMIT, SEMANTIC_FINDING_SPAN_SAMPLE_LIMIT};

pub(crate) fn policy_metadata() -> PolicyMetadata {
    PolicyMetadata {
        owner: PolicyOwner::LuminRustAnalyzer,
        version: POLICY_VERSION,
        js_ts_precedent: [
            "_lib/finding-provenance.mjs",
            "_lib/ranking.mjs",
            "_lib/pre-write-cue-tiers.mjs",
        ],
        syntax: SyntaxPolicy {
            claim: SyntaxPolicyClaim::SyntaxOnly,
            confidence_tier: SYNTAX_CONFIDENCE_TIER,
            raw_evidence_preserved: true,
            raw_evidence_embedded_in_product: false,
            visibility: SyntaxVisibilityPolicy {
                review: SignalVisibility::Review,
                muted: SignalVisibility::Muted,
            },
            muted_still_auditable: true,
            product_projection: SyntaxProductProjectionPolicy {
                signals: SyntaxSignalProjectionPolicy::ReviewAndMutedOnly,
                ast: SyntaxAstProjectionPolicy::SummaryAndCappedReviewExamples,
                parse: SyntaxParseProjectionPolicy::StatusAndCappedErrorExamples,
                raw_lane_owner: RawLaneOwner::RustSourceHealth,
                sample_limits: SyntaxProductSampleLimits {
                    signals: SIGNAL_SAMPLE_LIMIT,
                    file_signals: FILE_SIGNAL_SAMPLE_LIMIT,
                    parse_errors: PARSE_ERROR_SAMPLE_LIMIT,
                    skipped_files: SKIPPED_FILE_SAMPLE_LIMIT,
                    definitions: DEFINITION_SAMPLE_LIMIT,
                    use_trees: USE_TREE_SAMPLE_LIMIT,
                    default_ast: AST_SAMPLE_LIMIT,
                    file_ast: FILE_AST_SAMPLE_LIMIT,
                },
            },
        },
        semantic: SemanticPolicy {
            confidence_tiers: [
                ConfidenceTier::Verified,
                ConfidenceTier::RuleBacked,
                ConfidenceTier::Candidate,
            ],
            coverage_unavailable_status: CoverageStatus::Unavailable,
            raw_evidence_preserved: true,
            raw_evidence_embedded_in_product: false,
            product_projection: SemanticProductProjectionPolicy {
                coverage: SemanticCoverageProjectionPolicy::SummaryAndCappedScopeExamples,
                raw_lane_owner: RawLaneOwner::RustCargoOracle,
                sample_limits: SemanticProductSampleLimits {
                    oracle_scope: ORACLE_SCOPE_SAMPLE_LIMIT,
                    finding_spans: SEMANTIC_FINDING_SPAN_SAMPLE_LIMIT,
                },
            },
        },
        action_tiers: ActionTierPolicy {
            js_ts_precedent: "_lib/ranking.mjs",
            tiers: ActionPolicyTier::ALL,
            safe_fix_gate: SafeFixMetadataGate::RequiresProofCarryingEditAction,
            syntax_only_default: ActionPolicyTier::ReviewFix,
            muted_still_auditable: true,
        },
        oracle_bridge: OracleBridgePolicyMetadata {
            js_ts_precedent: OracleBridgePrecedent {
                parser: "_lib/parse-oxc.mjs",
                oracle: "_lib/tsconfig-paths.mjs",
                provenance: "_lib/finding-provenance.mjs",
            },
            rust_parser_lane: RustParserLane::RaApSyntaxViaRustSourceHealth,
            rust_oracle_lane: RustOracleLane::CargoRustcViaRustCargoOracle,
            file_provenance: true,
        },
        artifact_contract: ArtifactContractPolicy {
            js_ts_precedent: "_lib/rust-topology-prefer.mjs",
            failure_reason: ArtifactContractFailureReason::BlockedArtifactContract,
            status: ArtifactContractStatus::Strict,
            hard_stop: ArtifactContractHardStop::TypedArtifactConstructionBeforeWrite,
        },
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct PolicyMetadata {
    owner: PolicyOwner,
    version: &'static str,
    js_ts_precedent: [&'static str; 3],
    syntax: SyntaxPolicy,
    semantic: SemanticPolicy,
    action_tiers: ActionTierPolicy,
    oracle_bridge: OracleBridgePolicyMetadata,
    artifact_contract: ArtifactContractPolicy,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
enum PolicyOwner {
    LuminRustAnalyzer,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct SyntaxPolicy {
    claim: SyntaxPolicyClaim,
    confidence_tier: ConfidenceTier,
    raw_evidence_preserved: bool,
    raw_evidence_embedded_in_product: bool,
    visibility: SyntaxVisibilityPolicy,
    muted_still_auditable: bool,
    product_projection: SyntaxProductProjectionPolicy,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "kebab-case")]
enum SyntaxPolicyClaim {
    SyntaxOnly,
}

#[derive(Debug, Serialize)]
struct SyntaxVisibilityPolicy {
    review: SignalVisibility,
    muted: SignalVisibility,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct SyntaxProductProjectionPolicy {
    signals: SyntaxSignalProjectionPolicy,
    ast: SyntaxAstProjectionPolicy,
    parse: SyntaxParseProjectionPolicy,
    raw_lane_owner: RawLaneOwner,
    sample_limits: SyntaxProductSampleLimits,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
enum RawLaneOwner {
    RustSourceHealth,
    RustCargoOracle,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "kebab-case")]
enum SyntaxSignalProjectionPolicy {
    ReviewAndMutedOnly,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "kebab-case")]
enum SyntaxAstProjectionPolicy {
    SummaryAndCappedReviewExamples,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "kebab-case")]
enum SyntaxParseProjectionPolicy {
    StatusAndCappedErrorExamples,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct SyntaxProductSampleLimits {
    signals: usize,
    file_signals: usize,
    parse_errors: usize,
    skipped_files: usize,
    definitions: usize,
    use_trees: usize,
    default_ast: usize,
    file_ast: usize,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct SemanticPolicy {
    confidence_tiers: [ConfidenceTier; 3],
    coverage_unavailable_status: CoverageStatus,
    raw_evidence_preserved: bool,
    raw_evidence_embedded_in_product: bool,
    product_projection: SemanticProductProjectionPolicy,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct SemanticProductProjectionPolicy {
    coverage: SemanticCoverageProjectionPolicy,
    raw_lane_owner: RawLaneOwner,
    sample_limits: SemanticProductSampleLimits,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "kebab-case")]
enum SemanticCoverageProjectionPolicy {
    SummaryAndCappedScopeExamples,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct SemanticProductSampleLimits {
    oracle_scope: usize,
    finding_spans: usize,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ActionTierPolicy {
    js_ts_precedent: &'static str,
    tiers: [ActionPolicyTier; 5],
    safe_fix_gate: SafeFixMetadataGate,
    syntax_only_default: ActionPolicyTier,
    muted_still_auditable: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "kebab-case")]
enum SafeFixMetadataGate {
    RequiresProofCarryingEditAction,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct OracleBridgePolicyMetadata {
    js_ts_precedent: OracleBridgePrecedent,
    rust_parser_lane: RustParserLane,
    rust_oracle_lane: RustOracleLane,
    file_provenance: bool,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize)]
enum RustParserLane {
    #[serde(rename = "ra_ap_syntax via rust-source-health")]
    RaApSyntaxViaRustSourceHealth,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize)]
enum RustOracleLane {
    #[serde(rename = "Cargo/rustc via rust-cargo-oracle")]
    CargoRustcViaRustCargoOracle,
}

#[derive(Debug, Serialize)]
struct OracleBridgePrecedent {
    parser: &'static str,
    oracle: &'static str,
    provenance: &'static str,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ArtifactContractPolicy {
    js_ts_precedent: &'static str,
    failure_reason: ArtifactContractFailureReason,
    status: ArtifactContractStatus,
    hard_stop: ArtifactContractHardStop,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "kebab-case")]
enum ArtifactContractFailureReason {
    BlockedArtifactContract,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "kebab-case")]
enum ArtifactContractStatus {
    Strict,
}

#[derive(Debug, Serialize)]
enum ArtifactContractHardStop {
    #[serde(rename = "typed artifact construction before write")]
    TypedArtifactConstructionBeforeWrite,
}
