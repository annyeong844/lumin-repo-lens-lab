use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum ActionPolicyTier {
    SafeFix,
    ReviewFix,
    Degraded,
    Muted,
    Unavailable,
}

impl ActionPolicyTier {
    pub(crate) const ALL: [Self; 5] = [
        Self::SafeFix,
        Self::ReviewFix,
        Self::Degraded,
        Self::Muted,
        Self::Unavailable,
    ];
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) struct ActionTierSummary {
    safe_fix: usize,
    review_fix: usize,
    degraded: usize,
    muted: usize,
    unavailable: usize,
    #[serde(rename = "total")]
    total: usize,
}

impl ActionTierSummary {
    pub(super) fn new(
        safe_fix: usize,
        review_fix: usize,
        degraded: usize,
        muted: usize,
        unavailable: usize,
    ) -> Self {
        Self {
            safe_fix,
            review_fix,
            degraded,
            muted,
            unavailable,
            total: safe_fix + review_fix + degraded + muted + unavailable,
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize)]
pub(crate) struct EvidenceTierSummary {
    review: usize,
    degraded: usize,
    muted: usize,
    unavailable: usize,
    total: usize,
}

impl EvidenceTierSummary {
    pub(super) fn new(review: usize, degraded: usize, muted: usize, unavailable: usize) -> Self {
        Self {
            review,
            degraded,
            muted,
            unavailable,
            total: review + degraded + muted + unavailable,
        }
    }
}
