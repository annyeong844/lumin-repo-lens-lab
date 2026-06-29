use serde::Serialize;

use super::{AstDefinitionKind, AstVisibility, Location};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RustUnusedDefinitionAnalysis {
    pub policy: RustUnusedDefinitionPolicy,
    pub summary: RustUnusedDefinitionSummary,
    pub findings: Vec<RustUnusedDefinitionCandidate>,
    pub excluded_candidates: Vec<RustUnusedDefinitionCandidate>,
    pub degraded_scopes: Vec<RustUnusedDefinitionDegradedScope>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RustUnusedDefinitionPolicy {
    pub policy_id: String,
    pub ts_model: String,
    pub rust_fp_gate_namespace: String,
    pub candidate_count_scope: String,
    pub safe_action_scope: String,
}

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RustUnusedDefinitionSummary {
    pub definition_count: usize,
    pub candidate_count: usize,
    pub review_count: usize,
    pub degraded_count: usize,
    pub blocked_public_surface_count: usize,
    pub blocked_trait_impl_count: usize,
    pub blocked_opaque_count: usize,
    pub blocked_derive_surface_count: usize,
    pub blocked_cfg_count: usize,
    pub blocked_ffi_count: usize,
    pub test_only_support_count: usize,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RustUnusedDefinitionCandidate {
    pub kind: RustUnusedDefinitionCandidateKind,
    pub tier: RustUnusedDefinitionTier,
    pub action: RustUnusedDefinitionAction,
    pub definition: RustUnusedDefinitionDefinition,
    pub observed_references: RustUnusedDefinitionObservedReferences,
    pub fp_gates: Vec<String>,
    pub action_blockers: Vec<String>,
    pub safe_action: Option<RustUnusedDefinitionSafeAction>,
    pub evidence: Vec<RustUnusedDefinitionEvidence>,
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum RustUnusedDefinitionCandidateKind {
    RustUnusedDefinition,
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum RustUnusedDefinitionTier {
    RemoveCandidate,
    DemoteToRestricted,
    Review,
    Degraded,
    Muted,
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum RustUnusedDefinitionAction {
    RemoveCandidate,
    DemoteToRestricted,
    Review,
    Degraded,
    Muted,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RustUnusedDefinitionDefinition {
    pub file: String,
    pub name: String,
    pub kind: AstDefinitionKind,
    pub visibility: AstVisibility,
    pub owner: RustUnusedDefinitionOwner,
    pub location: Location,
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum RustUnusedDefinitionOwner {
    TraitImpl,
    Module,
    Unknown,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RustUnusedDefinitionObservedReferences {
    pub production: usize,
    pub test_only: usize,
    pub searched_scopes: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RustUnusedDefinitionSafeAction {
    pub kind: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RustUnusedDefinitionEvidence {
    pub kind: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RustUnusedDefinitionDegradedScope {
    pub kind: String,
    pub file: String,
    pub message: String,
}
