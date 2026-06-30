use serde::Serialize;

use crate::protocol::{
    AstVisibility, RUST_FUNCTION_CLONE_NEAR_CANDIDATE_COUNT_SCOPE,
    RUST_FUNCTION_CLONE_NEAR_CANDIDATE_GENERATION_MODE,
    RUST_FUNCTION_CLONE_NEAR_COMPATIBILITY_SKIPPED_PAIR_ESTIMATE_KIND,
    RUST_FUNCTION_CLONE_NEAR_MIN_SINGLE_TOKEN_IDF, RUST_FUNCTION_CLONE_NEAR_PAIR_DEDUPE,
    RUST_FUNCTION_CLONE_NEAR_PROJECTION, RUST_FUNCTION_CLONE_NEAR_RETRIEVAL_CONTRACT_VERSION,
    RUST_FUNCTION_CLONE_NEAR_SKIPPED_BUCKET_SAMPLE_LIMIT,
};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AstFunctionCloneGroup {
    pub kind: AstFunctionCloneGroupKind,
    pub hash: String,
    pub size: usize,
    pub risk: FunctionCloneRisk,
    pub generated_only: bool,
    pub exact_hash_count: usize,
    pub identities: Vec<String>,
    pub owner_files: Vec<String>,
    pub names: Vec<String>,
    pub visibilities: Vec<AstVisibility>,
    pub lines: Vec<AstFunctionCloneLine>,
    pub body_loc_range: [usize; 2],
    pub shared_call_tokens: Vec<String>,
    pub reason: &'static str,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AstFunctionSignatureGroup {
    pub kind: AstFunctionSignatureGroupKind,
    pub normalized_version: &'static str,
    pub hash: String,
    pub size: usize,
    pub risk: FunctionCloneRisk,
    pub generated_only: bool,
    pub review_visible: bool,
    pub signature_domain_idf_sum: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signature: Option<String>,
    pub identities: Vec<String>,
    pub owner_files: Vec<String>,
    pub names: Vec<String>,
    pub visibilities: Vec<AstVisibility>,
    pub lines: Vec<AstFunctionCloneLine>,
    pub reason: &'static str,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AstNearFunctionCandidate {
    pub kind: AstNearFunctionCandidateKind,
    pub identities: Vec<String>,
    pub owner_files: Vec<String>,
    pub names: Vec<String>,
    pub lines: Vec<AstFunctionCloneLine>,
    pub score: f64,
    pub risk: FunctionCloneRisk,
    pub generated_only: bool,
    pub shared_call_tokens: Vec<String>,
    pub shared_name_tokens: Vec<String>,
    pub call_token_jaccard: f64,
    pub shared_call_token_idf_sum: f64,
    pub call_token_idf_score: f64,
    pub name_token_jaccard: f64,
    pub body_loc_range: [usize; 2],
    pub statement_count_range: [usize; 2],
    pub reasons: Vec<String>,
    pub reason: &'static str,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AstNearFunctionCandidateGenerationPolicy {
    pub mode: &'static str,
    pub retrieval_contract_version: &'static str,
    pub bucket_min_idf: f64,
    pub candidate_count_scope: &'static str,
    pub pair_dedupe: &'static str,
    pub projection: &'static str,
    pub skipped_low_discrimination_bucket_sample_limit: usize,
}

impl Default for AstNearFunctionCandidateGenerationPolicy {
    fn default() -> Self {
        Self {
            mode: RUST_FUNCTION_CLONE_NEAR_CANDIDATE_GENERATION_MODE,
            retrieval_contract_version: RUST_FUNCTION_CLONE_NEAR_RETRIEVAL_CONTRACT_VERSION,
            bucket_min_idf: RUST_FUNCTION_CLONE_NEAR_MIN_SINGLE_TOKEN_IDF,
            candidate_count_scope: RUST_FUNCTION_CLONE_NEAR_CANDIDATE_COUNT_SCOPE,
            pair_dedupe: RUST_FUNCTION_CLONE_NEAR_PAIR_DEDUPE,
            projection: RUST_FUNCTION_CLONE_NEAR_PROJECTION,
            skipped_low_discrimination_bucket_sample_limit:
                RUST_FUNCTION_CLONE_NEAR_SKIPPED_BUCKET_SAMPLE_LIMIT,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AstNearFunctionCandidateGenerationSummary {
    pub eligible_function_count: usize,
    pub retained_call_token_bucket_count: usize,
    pub retained_raw_pair_estimate: usize,
    pub generated_unique_pair_count: usize,
    pub scored_pair_count: usize,
    pub compatibility_skipped_raw_pair_estimate_by_reason:
        AstNearFunctionCompatibilitySkippedPairEstimates,
    pub debug_formatter_boilerplate_skipped_pair_count: usize,
    pub compatibility_skipped_pair_estimate_kind: &'static str,
    pub near_function_candidate_count_scope: &'static str,
}

impl Default for AstNearFunctionCandidateGenerationSummary {
    fn default() -> Self {
        Self {
            eligible_function_count: 0,
            retained_call_token_bucket_count: 0,
            retained_raw_pair_estimate: 0,
            generated_unique_pair_count: 0,
            scored_pair_count: 0,
            compatibility_skipped_raw_pair_estimate_by_reason:
                AstNearFunctionCompatibilitySkippedPairEstimates::default(),
            debug_formatter_boilerplate_skipped_pair_count: 0,
            compatibility_skipped_pair_estimate_kind:
                RUST_FUNCTION_CLONE_NEAR_COMPATIBILITY_SKIPPED_PAIR_ESTIMATE_KIND,
            near_function_candidate_count_scope: RUST_FUNCTION_CLONE_NEAR_CANDIDATE_COUNT_SCOPE,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AstNearFunctionCompatibilitySkippedPairEstimates {
    pub qualifier_mismatch: usize,
    pub parameter_count_delta: usize,
    pub body_loc_band_mismatch: usize,
    pub statement_count_band_mismatch: usize,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AstSkippedLowDiscriminationBucket {
    pub token: String,
    pub idf: f64,
    pub function_count: usize,
    pub raw_pair_estimate: usize,
    pub reason: &'static str,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AstFunctionCloneLine {
    pub identity: String,
    pub file: String,
    pub line: usize,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum AstFunctionCloneGroupKind {
    ExactFunctionBodyGroup,
    FunctionBodyStructureGroup,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum AstFunctionSignatureGroupKind {
    FunctionSignatureGroup,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum AstNearFunctionCandidateKind {
    NearFunctionCandidate,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum FunctionCloneRisk {
    Muted,
    ReviewOnly,
}
