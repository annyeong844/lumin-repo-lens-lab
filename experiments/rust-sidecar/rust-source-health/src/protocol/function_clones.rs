use serde::Serialize;

use super::AstVisibility;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AstFunctionCloneGroups {
    pub policy: AstFunctionCloneGroupsPolicy,
    pub supports: AstFunctionCloneGroupsSupports,
    pub complete: bool,
    pub files_with_parse_errors: Vec<AstFunctionCloneInputError>,
    pub files_with_read_errors: Vec<AstFunctionCloneInputError>,
    pub exact_body_group_count: usize,
    pub structure_group_count: usize,
    pub signature_group_count: usize,
    pub near_function_candidate_count: usize,
    pub near_function_candidate_projection_limit: usize,
    pub generated_file_fact_count: usize,
    pub exact_body_groups: Vec<AstFunctionCloneGroup>,
    pub structure_groups: Vec<AstFunctionCloneGroup>,
    pub signature_groups: Vec<AstFunctionSignatureGroup>,
    pub near_function_candidates: Vec<AstNearFunctionCandidate>,
}

impl Default for AstFunctionCloneGroups {
    fn default() -> Self {
        Self {
            policy: AstFunctionCloneGroupsPolicy::default(),
            supports: AstFunctionCloneGroupsSupports::default(),
            complete: true,
            files_with_parse_errors: Vec::new(),
            files_with_read_errors: Vec::new(),
            exact_body_group_count: 0,
            structure_group_count: 0,
            signature_group_count: 0,
            near_function_candidate_count: 0,
            near_function_candidate_projection_limit:
                super::RUST_FUNCTION_CLONE_NEAR_MAX_CANDIDATES,
            generated_file_fact_count: 0,
            exact_body_groups: Vec::new(),
            structure_groups: Vec::new(),
            signature_groups: Vec::new(),
            near_function_candidates: Vec::new(),
        }
    }
}

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
            policy_id: super::RUST_FUNCTION_CLONE_GROUP_POLICY_ID,
            policy_version: super::RUST_FUNCTION_CLONE_GROUP_POLICY_VERSION,
            normalized_version: super::RUST_FUNCTION_BODY_NORMALIZED_VERSION,
            function_signature_normalized_version:
                super::RUST_FUNCTION_SIGNATURE_NORMALIZED_VERSION,
            min_group_size: super::RUST_FUNCTION_CLONE_MIN_GROUP_SIZE,
            exact_min_body_loc: super::RUST_FUNCTION_CLONE_EXACT_MIN_BODY_LOC,
            exact_min_statements: super::RUST_FUNCTION_CLONE_EXACT_MIN_STATEMENTS,
            structure_min_body_loc: super::RUST_FUNCTION_CLONE_STRUCTURE_MIN_BODY_LOC,
            structure_min_statements: super::RUST_FUNCTION_CLONE_STRUCTURE_MIN_STATEMENTS,
            near_candidate_policy: AstNearFunctionCandidatePolicy::default(),
            caveat: "Function clone groups and near candidates are deterministic review evidence. They do not prove semantic equivalence, auto-reuse, auto-fix safety, or a merge recommendation.",
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AstFunctionCloneInputError {
    pub file: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AstFunctionCloneGroupsSupports {
    pub top_level_functions: bool,
    pub impl_methods: bool,
    pub function_fact_visibility: bool,
    pub exact_body_hash: bool,
    pub normalized_exact_hash: bool,
    pub normalized_structure_hash: bool,
    pub normalized_version: &'static str,
    pub normalized_function_signature_hash: bool,
    pub function_signature_groups: bool,
    pub function_signature_normalized_version: &'static str,
    pub near_function_candidates: bool,
    pub generated_file_evidence: bool,
    pub semantic_equivalence: bool,
}

impl Default for AstFunctionCloneGroupsSupports {
    fn default() -> Self {
        Self {
            top_level_functions: true,
            impl_methods: true,
            function_fact_visibility: true,
            exact_body_hash: true,
            normalized_exact_hash: true,
            normalized_structure_hash: true,
            normalized_version: super::RUST_FUNCTION_BODY_NORMALIZED_VERSION,
            normalized_function_signature_hash: true,
            function_signature_groups: true,
            function_signature_normalized_version:
                super::RUST_FUNCTION_SIGNATURE_NORMALIZED_VERSION,
            near_function_candidates: true,
            generated_file_evidence: true,
            semantic_equivalence: false,
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
    pub suppressed_generic_call_tokens: &'static [&'static str],
    pub required_matching_qualifiers: &'static [&'static str],
    pub max_param_count_delta: usize,
    pub min_body_loc_similarity: f64,
    pub min_statement_count_similarity: f64,
    pub min_call_token_jaccard: f64,
    pub min_name_token_jaccard_fallback: f64,
    pub min_near_score: f64,
    pub max_near_candidates: usize,
    pub weights: AstNearFunctionCandidateWeights,
    pub notes: [&'static str; 2],
}

impl Default for AstNearFunctionCandidatePolicy {
    fn default() -> Self {
        Self {
            policy_id: super::RUST_FUNCTION_CLONE_NEAR_POLICY_ID,
            policy_version: super::RUST_FUNCTION_CLONE_NEAR_POLICY_VERSION,
            policy_class: super::RUST_FUNCTION_CLONE_NEAR_POLICY_CLASS,
            calibration_version: super::RUST_FUNCTION_CLONE_NEAR_CALIBRATION_VERSION,
            min_significant_call_token_len:
                super::RUST_FUNCTION_CLONE_NEAR_MIN_SIGNIFICANT_CALL_TOKEN_LEN,
            suppressed_generic_call_tokens:
                super::RUST_FUNCTION_CLONE_NEAR_SUPPRESSED_GENERIC_CALL_TOKENS,
            required_matching_qualifiers:
                super::RUST_FUNCTION_CLONE_NEAR_REQUIRED_MATCHING_QUALIFIERS,
            max_param_count_delta: super::RUST_FUNCTION_CLONE_NEAR_MAX_PARAM_COUNT_DELTA,
            min_body_loc_similarity: super::RUST_FUNCTION_CLONE_NEAR_MIN_BODY_LOC_SIMILARITY,
            min_statement_count_similarity:
                super::RUST_FUNCTION_CLONE_NEAR_MIN_STATEMENT_COUNT_SIMILARITY,
            min_call_token_jaccard: super::RUST_FUNCTION_CLONE_NEAR_MIN_CALL_TOKEN_JACCARD,
            min_name_token_jaccard_fallback:
                super::RUST_FUNCTION_CLONE_NEAR_MIN_NAME_TOKEN_JACCARD_FALLBACK,
            min_near_score: super::RUST_FUNCTION_CLONE_NEAR_MIN_SCORE,
            max_near_candidates: super::RUST_FUNCTION_CLONE_NEAR_MAX_CANDIDATES,
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
    pub call_token_jaccard: f64,
    pub name_token_jaccard: f64,
    pub body_loc_similarity: f64,
    pub statement_count_similarity: f64,
}

impl Default for AstNearFunctionCandidateWeights {
    fn default() -> Self {
        Self {
            call_token_jaccard: super::RUST_FUNCTION_CLONE_NEAR_CALL_TOKEN_WEIGHT,
            name_token_jaccard: super::RUST_FUNCTION_CLONE_NEAR_NAME_TOKEN_WEIGHT,
            body_loc_similarity: super::RUST_FUNCTION_CLONE_NEAR_BODY_LOC_WEIGHT,
            statement_count_similarity: super::RUST_FUNCTION_CLONE_NEAR_STATEMENT_COUNT_WEIGHT,
        }
    }
}

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
    pub hash: String,
    pub size: usize,
    pub risk: FunctionCloneRisk,
    pub generated_only: bool,
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
    pub name_token_jaccard: f64,
    pub body_loc_range: [usize; 2],
    pub statement_count_range: [usize; 2],
    pub reasons: Vec<String>,
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
    ReviewOnly,
}
