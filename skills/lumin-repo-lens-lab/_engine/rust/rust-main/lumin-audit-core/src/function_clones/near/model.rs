use super::super::facts::FunctionFact;
use serde_json::Value;

pub(in crate::function_clones) struct NearFunctionCandidateProjection {
    pub(in crate::function_clones) review_visible_count: usize,
    pub(in crate::function_clones) candidates: Vec<Value>,
    pub(in crate::function_clones) diagnostics: CandidateGenerationDiagnostics,
}

pub(super) struct NearFact<'a> {
    pub(super) fact: &'a FunctionFact,
    pub(super) significant_call_tokens: Vec<&'a str>,
    pub(super) retained_call_tokens: Vec<&'a str>,
    pub(super) name_tokens: Vec<Box<str>>,
}

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd)]
pub(super) struct CompatibilityKey {
    pub(super) async_value: bool,
    pub(super) param_count: i64,
    pub(super) body_loc_band: usize,
    pub(super) statement_count_band: usize,
}

#[derive(Default)]
pub(in crate::function_clones) struct CandidateGenerationDiagnostics {
    pub(super) eligible_function_count: usize,
    pub(super) retained_call_token_bucket_count: usize,
    pub(super) retained_raw_pair_estimate: usize,
    pub(super) generated_unique_pair_count: usize,
    pub(super) scored_pair_count: usize,
    pub(super) compatibility_skipped_raw_pair_estimate_by_reason: CompatibilitySkippedPairEstimates,
    pub(super) skipped_low_discrimination_buckets: Vec<SkippedLowDiscriminationBucket>,
    pub(in crate::function_clones) skipped_low_discrimination_bucket_count: usize,
    pub(in crate::function_clones) skipped_low_discrimination_raw_pair_estimate: usize,
}

#[derive(Default)]
pub(super) struct CompatibilitySkippedPairEstimates {
    pub(super) async_mismatch: usize,
    pub(super) parameter_count_delta: usize,
    pub(super) body_loc_band_mismatch: usize,
    pub(super) statement_count_band_mismatch: usize,
}

pub(super) struct SkippedLowDiscriminationBucket {
    pub(super) token: String,
    pub(super) idf: f64,
    pub(super) posting_count: usize,
    pub(super) raw_pair_estimate: usize,
}

pub(super) struct SharedCallTokenEvidence {
    pub(super) token: String,
    pub(super) idf: f64,
    pub(super) retained: bool,
}

pub(super) struct NearCandidateEvidence {
    pub(super) generation_token: String,
    pub(super) score: f64,
    pub(super) generated_only: bool,
    pub(super) shared_call_tokens: Vec<String>,
    pub(super) shared_significant_call_tokens: Vec<SharedCallTokenEvidence>,
    pub(super) shared_name_tokens: Vec<String>,
    pub(super) call_token_jaccard: f64,
    pub(super) shared_call_token_idf_sum: f64,
    pub(super) call_token_idf_score: f64,
    pub(super) name_token_jaccard: f64,
    pub(super) body_loc_similarity: f64,
    pub(super) statement_count_similarity: f64,
}
