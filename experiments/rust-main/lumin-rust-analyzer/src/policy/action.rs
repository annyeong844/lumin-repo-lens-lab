mod build;
mod model;
mod projection;
mod tiers;

pub(crate) use build::action_policy;
pub(crate) use model::ActionPolicy;
pub(crate) use projection::ActionPolicyProjection;
use serde::Serialize;
pub(crate) use tiers::{ActionPolicyTier, ActionTierSummary, EvidenceTierSummary};

const ACTION_POLICY_SCHEMA_VERSION: &str = "rust-action-tier.v1";
const JS_TS_PRECEDENT: &str = "_lib/ranking.mjs";
const SAFE_FIX_GATE_STATUS: SafeFixGateStatus = SafeFixGateStatus::Strict;
const SAFE_FIX_GATE_REASON: SafeFixGateReason =
    SafeFixGateReason::ProofCompleteSafeActionWithoutBlockers;
const SAFE_FIX_CURRENTLY_SUPPORTED: [SafeFixSupportedProof; 1] =
    [SafeFixSupportedProof::RustcMachineApplicableRuleBackedWarning];
const SAFE_FIX_NOT_SAFE_FOR: [SafeFixUnsupportedSurface; 3] = [
    SafeFixUnsupportedSurface::SyntaxOnlySignals,
    SafeFixUnsupportedSurface::AstOpaqueSurfaces,
    SafeFixUnsupportedSurface::SemanticFindingsWithoutSafeAction,
];
const REVIEW_FIX_GATE_STATUS: ReviewFixGateStatus = ReviewFixGateStatus::Explicit;
const REVIEW_FIX_GATE_REASON: ReviewFixGateReason =
    ReviewFixGateReason::SelectedActionBlockersOrReviewFindings;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(super) enum SafeFixGateStatus {
    Strict,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize)]
pub(super) enum SafeFixGateReason {
    #[serde(rename = "SAFE_FIX requires proofComplete safeAction with empty actionBlockers")]
    ProofCompleteSafeActionWithoutBlockers,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(super) enum SafeFixSupportedProof {
    RustcMachineApplicableRuleBackedWarning,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize)]
pub(super) enum SafeFixUnsupportedSurface {
    #[serde(rename = "syntax-only signals")]
    SyntaxOnlySignals,
    #[serde(rename = "AST opaque surfaces")]
    AstOpaqueSurfaces,
    #[serde(rename = "semantic findings without safeAction")]
    SemanticFindingsWithoutSafeAction,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(super) enum ReviewFixGateStatus {
    Explicit,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize)]
pub(super) enum ReviewFixGateReason {
    #[serde(
        rename = "REVIEW_FIX is selected-action blockers plus verified/rule-backed semantic findings without safe edit proof"
    )]
    SelectedActionBlockersOrReviewFindings,
}
