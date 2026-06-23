use serde::Serialize;

use lumin_rust_cargo_oracle::protocol::ConfidenceTier;
use lumin_rust_source_health::protocol::SignalVisibility;

use super::lane::RawLaneOwner;
use crate::policy::{
    AST_SAMPLE_LIMIT, DEFINITION_SAMPLE_LIMIT, FILE_AST_SAMPLE_LIMIT, FILE_SIGNAL_SAMPLE_LIMIT,
    PARSE_ERROR_SAMPLE_LIMIT, SIGNAL_SAMPLE_LIMIT, SKIPPED_FILE_SAMPLE_LIMIT,
    SYNTAX_CONFIDENCE_TIER, USE_TREE_SAMPLE_LIMIT,
};

pub(super) fn syntax_policy() -> SyntaxPolicy {
    SyntaxPolicy {
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
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct SyntaxPolicy {
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
