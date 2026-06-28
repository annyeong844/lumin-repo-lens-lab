use serde::Serialize;

use crate::protocol::{
    RUST_FUNCTION_BODY_NORMALIZED_VERSION, RUST_FUNCTION_CLONE_EXACT_MIN_BODY_LOC,
    RUST_FUNCTION_CLONE_EXACT_MIN_STATEMENTS, RUST_FUNCTION_CLONE_GROUP_POLICY_ID,
    RUST_FUNCTION_CLONE_GROUP_POLICY_VERSION, RUST_FUNCTION_CLONE_MIN_GROUP_SIZE,
    RUST_FUNCTION_CLONE_NEAR_BODY_LOC_WEIGHT, RUST_FUNCTION_CLONE_NEAR_CALIBRATION_VERSION,
    RUST_FUNCTION_CLONE_NEAR_CALL_IDF_SATURATION, RUST_FUNCTION_CLONE_NEAR_CALL_TOKEN_WEIGHT,
    RUST_FUNCTION_CLONE_NEAR_MAX_CANDIDATES, RUST_FUNCTION_CLONE_NEAR_MAX_PARAM_COUNT_DELTA,
    RUST_FUNCTION_CLONE_NEAR_MIN_BODY_LOC_SIMILARITY,
    RUST_FUNCTION_CLONE_NEAR_MIN_CALL_TOKEN_IDF_SCORE,
    RUST_FUNCTION_CLONE_NEAR_MIN_NAME_TOKEN_JACCARD_FALLBACK, RUST_FUNCTION_CLONE_NEAR_MIN_SCORE,
    RUST_FUNCTION_CLONE_NEAR_MIN_SIGNIFICANT_CALL_TOKEN_LEN,
    RUST_FUNCTION_CLONE_NEAR_MIN_SINGLE_TOKEN_IDF,
    RUST_FUNCTION_CLONE_NEAR_MIN_STATEMENT_COUNT_SIMILARITY,
    RUST_FUNCTION_CLONE_NEAR_NAME_TOKEN_WEIGHT, RUST_FUNCTION_CLONE_NEAR_POLICY_CLASS,
    RUST_FUNCTION_CLONE_NEAR_POLICY_ID, RUST_FUNCTION_CLONE_NEAR_POLICY_VERSION,
    RUST_FUNCTION_CLONE_NEAR_REQUIRED_MATCHING_QUALIFIERS,
    RUST_FUNCTION_CLONE_NEAR_STATEMENT_COUNT_WEIGHT,
    RUST_FUNCTION_CLONE_NEAR_SUPPRESSED_GENERIC_CALL_TOKENS,
    RUST_FUNCTION_CLONE_STRUCTURE_MIN_BODY_LOC, RUST_FUNCTION_CLONE_STRUCTURE_MIN_STATEMENTS,
    RUST_FUNCTION_SIGNATURE_NORMALIZED_VERSION,
};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AstFunctionCloneGroupsPolicy {
    pub policy_id: &'static str,
    pub policy_version: &'static str,
    pub normalized_version: &'static str,
    pub function_signature_normalized_version: &'static str,
    pub min_group_size: usize,
    pub exact_min_body_loc: usize,
    pub exact_min_statements: usize,
    pub structure_min_body_loc: usize,
    pub structure_min_statements: usize,
    pub near_candidate_policy: AstNearFunctionCandidatePolicy,
    pub caveat: &'static str,
}

impl Default for AstFunctionCloneGroupsPolicy {
    fn default() -> Self {
        Self {
            policy_id: RUST_FUNCTION_CLONE_GROUP_POLICY_ID,
            policy_version: RUST_FUNCTION_CLONE_GROUP_POLICY_VERSION,
            normalized_version: RUST_FUNCTION_BODY_NORMALIZED_VERSION,
            function_signature_normalized_version: RUST_FUNCTION_SIGNATURE_NORMALIZED_VERSION,
            min_group_size: RUST_FUNCTION_CLONE_MIN_GROUP_SIZE,
            exact_min_body_loc: RUST_FUNCTION_CLONE_EXACT_MIN_BODY_LOC,
            exact_min_statements: RUST_FUNCTION_CLONE_EXACT_MIN_STATEMENTS,
            structure_min_body_loc: RUST_FUNCTION_CLONE_STRUCTURE_MIN_BODY_LOC,
            structure_min_statements: RUST_FUNCTION_CLONE_STRUCTURE_MIN_STATEMENTS,
            near_candidate_policy: AstNearFunctionCandidatePolicy::default(),
            caveat: "Function clone groups and near candidates are deterministic review evidence. They do not prove semantic equivalence, auto-reuse, auto-fix safety, or a merge recommendation.",
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AstNearFunctionCandidatePolicy {
    pub policy_id: &'static str,
    pub policy_version: &'static str,
    pub policy_class: &'static str,
    pub calibration_version: &'static str,
    pub min_significant_call_token_len: usize,
    pub min_single_token_idf: f64,
    pub call_idf_saturation: f64,
    pub suppressed_generic_call_tokens: &'static [&'static str],
    pub required_matching_qualifiers: &'static [&'static str],
    pub max_param_count_delta: usize,
    pub min_body_loc_similarity: f64,
    pub min_statement_count_similarity: f64,
    pub min_call_token_idf_score: f64,
    pub min_name_token_jaccard_fallback: f64,
    pub min_near_score: f64,
    pub max_near_candidates: usize,
    pub weights: AstNearFunctionCandidateWeights,
    pub notes: [&'static str; 2],
}

impl Default for AstNearFunctionCandidatePolicy {
    fn default() -> Self {
        Self {
            policy_id: RUST_FUNCTION_CLONE_NEAR_POLICY_ID,
            policy_version: RUST_FUNCTION_CLONE_NEAR_POLICY_VERSION,
            policy_class: RUST_FUNCTION_CLONE_NEAR_POLICY_CLASS,
            calibration_version: RUST_FUNCTION_CLONE_NEAR_CALIBRATION_VERSION,
            min_significant_call_token_len: RUST_FUNCTION_CLONE_NEAR_MIN_SIGNIFICANT_CALL_TOKEN_LEN,
            min_single_token_idf: RUST_FUNCTION_CLONE_NEAR_MIN_SINGLE_TOKEN_IDF,
            call_idf_saturation: RUST_FUNCTION_CLONE_NEAR_CALL_IDF_SATURATION,
            suppressed_generic_call_tokens: RUST_FUNCTION_CLONE_NEAR_SUPPRESSED_GENERIC_CALL_TOKENS,
            required_matching_qualifiers: RUST_FUNCTION_CLONE_NEAR_REQUIRED_MATCHING_QUALIFIERS,
            max_param_count_delta: RUST_FUNCTION_CLONE_NEAR_MAX_PARAM_COUNT_DELTA,
            min_body_loc_similarity: RUST_FUNCTION_CLONE_NEAR_MIN_BODY_LOC_SIMILARITY,
            min_statement_count_similarity: RUST_FUNCTION_CLONE_NEAR_MIN_STATEMENT_COUNT_SIMILARITY,
            min_call_token_idf_score: RUST_FUNCTION_CLONE_NEAR_MIN_CALL_TOKEN_IDF_SCORE,
            min_name_token_jaccard_fallback:
                RUST_FUNCTION_CLONE_NEAR_MIN_NAME_TOKEN_JACCARD_FALLBACK,
            min_near_score: RUST_FUNCTION_CLONE_NEAR_MIN_SCORE,
            max_near_candidates: RUST_FUNCTION_CLONE_NEAR_MAX_CANDIDATES,
            weights: AstNearFunctionCandidateWeights::default(),
            notes: [
                "Near-function candidates are review-only cues.",
                "Scores do not prove semantic equivalence or automatic merge safety.",
            ],
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AstNearFunctionCandidateWeights {
    pub call_token_idf_score: f64,
    pub name_token_jaccard: f64,
    pub body_loc_similarity: f64,
    pub statement_count_similarity: f64,
}

impl Default for AstNearFunctionCandidateWeights {
    fn default() -> Self {
        Self {
            call_token_idf_score: RUST_FUNCTION_CLONE_NEAR_CALL_TOKEN_WEIGHT,
            name_token_jaccard: RUST_FUNCTION_CLONE_NEAR_NAME_TOKEN_WEIGHT,
            body_loc_similarity: RUST_FUNCTION_CLONE_NEAR_BODY_LOC_WEIGHT,
            statement_count_similarity: RUST_FUNCTION_CLONE_NEAR_STATEMENT_COUNT_WEIGHT,
        }
    }
}
