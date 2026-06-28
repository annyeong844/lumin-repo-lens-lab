use crate::protocol::{
    AstNearFunctionCandidate, AstSkippedLowDiscriminationBucket,
    RUST_FUNCTION_CLONE_NEAR_CANDIDATE_COUNT_SCOPE,
    RUST_FUNCTION_CLONE_NEAR_COMPATIBILITY_SKIPPED_PAIR_ESTIMATE_KIND,
};

use crate::function_clones::common::GroupMember;

pub(in crate::function_clones) struct NearFunctionCandidateProjection {
    pub(in crate::function_clones) review_visible_count: usize,
    pub(in crate::function_clones) candidates: Vec<AstNearFunctionCandidate>,
    pub(in crate::function_clones) diagnostics: CandidateGenerationDiagnostics,
}

pub(super) struct NearFact<'a> {
    pub(super) member: GroupMember<'a>,
    pub(super) identity: String,
    pub(super) significant_call_tokens: Vec<String>,
    pub(super) name_tokens: Vec<String>,
}

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd)]
pub(super) struct CompatibilityKey {
    pub(super) qualifier_signature: QualifierSignature,
    pub(super) param_count: usize,
    pub(super) body_loc_band: usize,
    pub(super) statement_count_band: usize,
}

#[derive(Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
pub(super) struct QualifierSignature {
    pub(super) is_async: bool,
    pub(super) is_unsafe: bool,
    pub(super) is_const: bool,
}

#[derive(Default)]
pub(in crate::function_clones) struct CandidateGenerationDiagnostics {
    pub(in crate::function_clones) eligible_function_count: usize,
    pub(in crate::function_clones) retained_call_token_bucket_count: usize,
    pub(in crate::function_clones) retained_raw_pair_estimate: usize,
    pub(in crate::function_clones) generated_unique_pair_count: usize,
    pub(in crate::function_clones) scored_pair_count: usize,
    pub(in crate::function_clones) compatibility_skipped_raw_pair_estimate_by_reason:
        CompatibilitySkippedPairEstimates,
    pub(in crate::function_clones) skipped_low_discrimination_buckets:
        Vec<AstSkippedLowDiscriminationBucket>,
    pub(in crate::function_clones) skipped_low_discrimination_bucket_count: usize,
    pub(in crate::function_clones) skipped_low_discrimination_raw_pair_estimate: usize,
}

#[derive(Default)]
pub(in crate::function_clones) struct CompatibilitySkippedPairEstimates {
    pub(in crate::function_clones) qualifier_mismatch: usize,
    pub(in crate::function_clones) parameter_count_delta: usize,
    pub(in crate::function_clones) body_loc_band_mismatch: usize,
    pub(in crate::function_clones) statement_count_band_mismatch: usize,
}

impl CandidateGenerationDiagnostics {
    pub(in crate::function_clones) fn compatibility_skipped_pair_estimate_kind(
        &self,
    ) -> &'static str {
        RUST_FUNCTION_CLONE_NEAR_COMPATIBILITY_SKIPPED_PAIR_ESTIMATE_KIND
    }

    pub(in crate::function_clones) fn near_function_candidate_count_scope(&self) -> &'static str {
        RUST_FUNCTION_CLONE_NEAR_CANDIDATE_COUNT_SCOPE
    }
}
