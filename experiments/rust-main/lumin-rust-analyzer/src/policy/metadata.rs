mod action_tiers;
mod artifact_contract;
mod lane;
mod oracle_bridge;
mod semantic;
mod syntax;

use serde::Serialize;

use action_tiers::{action_tier_policy, ActionTierPolicy};
use artifact_contract::{artifact_contract_policy, ArtifactContractPolicy};
use oracle_bridge::{oracle_bridge_policy, OracleBridgePolicyMetadata};
use semantic::{semantic_policy, SemanticPolicy};
use syntax::{syntax_policy, SyntaxPolicy};

use super::POLICY_VERSION;

pub(crate) fn policy_metadata() -> PolicyMetadata {
    PolicyMetadata {
        owner: PolicyOwner::LuminRustAnalyzer,
        version: POLICY_VERSION,
        js_ts_precedent: [
            "_lib/finding-provenance.mjs",
            "_lib/ranking.mjs",
            "_lib/pre-write-cue-tiers.mjs",
        ],
        syntax: syntax_policy(),
        semantic: semantic_policy(),
        action_tiers: action_tier_policy(),
        oracle_bridge: oracle_bridge_policy(),
        artifact_contract: artifact_contract_policy(),
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
