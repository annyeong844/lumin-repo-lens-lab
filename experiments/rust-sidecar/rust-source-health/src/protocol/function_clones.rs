use serde::Serialize;

use super::AstVisibility;

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AstFunctionCloneGroups {
    pub policy: AstFunctionCloneGroupsPolicy,
    pub exact_body_groups: Vec<AstFunctionCloneGroup>,
    pub structure_groups: Vec<AstFunctionCloneGroup>,
    pub near_function_candidates: Vec<AstNearFunctionCandidate>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AstFunctionCloneGroupsPolicy {
    pub policy_id: &'static str,
    pub policy_version: &'static str,
    pub normalized_version: &'static str,
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
pub struct AstNearFunctionCandidatePolicy {
    pub policy_id: &'static str,
    pub policy_version: &'static str,
    pub policy_class: &'static str,
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
pub enum AstNearFunctionCandidateKind {
    NearFunctionCandidate,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum FunctionCloneRisk {
    ReviewOnly,
}
