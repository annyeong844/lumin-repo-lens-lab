use serde::Serialize;

use crate::policy::action::ActionPolicyTier;

pub(super) fn action_tier_policy() -> ActionTierPolicy {
    ActionTierPolicy {
        js_ts_precedent: "_lib/ranking.mjs",
        tiers: ActionPolicyTier::ALL,
        safe_fix_gate: SafeFixMetadataGate::RequiresProofCarryingEditAction,
        syntax_only_default: ActionPolicyTier::ReviewFix,
        muted_still_auditable: true,
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct ActionTierPolicy {
    js_ts_precedent: &'static str,
    tiers: [ActionPolicyTier; 5],
    safe_fix_gate: SafeFixMetadataGate,
    syntax_only_default: ActionPolicyTier,
    muted_still_auditable: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "kebab-case")]
enum SafeFixMetadataGate {
    RequiresProofCarryingEditAction,
}
