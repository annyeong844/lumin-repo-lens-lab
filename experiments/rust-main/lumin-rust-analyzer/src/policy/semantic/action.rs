use std::borrow::Cow;
use std::collections::BTreeMap;

use lumin_rust_cargo_oracle::protocol::{
    ActionBlockerReason, ClaimKind, ConfidenceTier as OracleConfidenceTier, Finding,
};

use crate::policy::ActionTier;

pub(in crate::policy) struct FindingActionSummary {
    pub(in crate::policy) safe_actions: usize,
    pub(in crate::policy) action_blockers: usize,
    pub(in crate::policy) action_blockers_by_reason: BTreeMap<ActionBlockerReason, usize>,
    pub(in crate::policy) review_findings: usize,
    pub(in crate::policy) review_by_reason: BTreeMap<ClaimKind, usize>,
}

pub(in crate::policy) struct FindingAction<'a> {
    pub(in crate::policy) tier: ActionTier,
    pub(in crate::policy) has_safe_action: bool,
    pub(in crate::policy) action_blockers: Cow<'a, [ActionBlockerReason]>,
    pub(in crate::policy) is_review: bool,
    pub(in crate::policy) is_candidate: bool,
}

pub(in crate::policy) struct FindingActionRecord<'a> {
    pub(in crate::policy) finding: &'a Finding,
    pub(in crate::policy) action: FindingAction<'a>,
}

pub(in crate::policy) fn finding_action(finding: &Finding) -> FindingAction<'_> {
    let action_blockers = finding_action_blockers(finding);
    let has_safe_action = finding.safe_action.as_ref().is_some_and(|action| {
        action.proof_complete && action.action_blockers.is_empty() && action_blockers.is_empty()
    });
    let is_candidate = finding.confidence_tier == OracleConfidenceTier::Candidate;
    let is_review = !has_safe_action
        && action_blockers.is_empty()
        && matches!(
            finding.confidence_tier,
            OracleConfidenceTier::Verified | OracleConfidenceTier::RuleBacked
        );
    let tier = if has_safe_action {
        ActionTier::SafeFix
    } else if !action_blockers.is_empty() || is_review {
        ActionTier::ReviewFix
    } else if is_candidate {
        ActionTier::Degraded
    } else {
        ActionTier::ReviewFix
    };
    FindingAction {
        tier,
        has_safe_action,
        action_blockers,
        is_review,
        is_candidate,
    }
}

pub(in crate::policy) fn finding_actions(findings: &[Finding]) -> Vec<FindingActionRecord<'_>> {
    findings
        .iter()
        .map(|finding| FindingActionRecord {
            finding,
            action: finding_action(finding),
        })
        .collect()
}

fn finding_action_blockers(finding: &Finding) -> Cow<'_, [ActionBlockerReason]> {
    let safe_action_blockers = finding
        .safe_action
        .as_ref()
        .map(|action| action.action_blockers.as_slice())
        .unwrap_or_default();
    normalize_action_blockers(&finding.action_blockers, safe_action_blockers)
}

fn normalize_action_blockers<'a>(
    finding_blockers: &'a [ActionBlockerReason],
    safe_action_blockers: &'a [ActionBlockerReason],
) -> Cow<'a, [ActionBlockerReason]> {
    match (finding_blockers.is_empty(), safe_action_blockers.is_empty()) {
        (true, true) => Cow::Borrowed(&[]),
        (false, true) if is_sorted_unique(finding_blockers) => Cow::Borrowed(finding_blockers),
        (true, false) if is_sorted_unique(safe_action_blockers) => {
            Cow::Borrowed(safe_action_blockers)
        }
        _ => {
            let mut blockers =
                Vec::with_capacity(finding_blockers.len() + safe_action_blockers.len());
            blockers.extend(finding_blockers.iter().copied());
            blockers.extend(safe_action_blockers.iter().copied());
            blockers.sort_unstable();
            blockers.dedup();
            Cow::Owned(blockers)
        }
    }
}

fn is_sorted_unique(blockers: &[ActionBlockerReason]) -> bool {
    blockers.windows(2).all(|pair| pair[0] < pair[1])
}

pub(in crate::policy) fn finding_action_summary(
    records: &[FindingActionRecord<'_>],
) -> FindingActionSummary {
    let mut summary = FindingActionSummary {
        safe_actions: 0,
        action_blockers: 0,
        action_blockers_by_reason: BTreeMap::new(),
        review_findings: 0,
        review_by_reason: BTreeMap::new(),
    };
    for record in records {
        let action = &record.action;
        if action.has_safe_action {
            summary.safe_actions += 1;
        } else if !action.action_blockers.is_empty() {
            summary.action_blockers += 1;
            for blocker in action.action_blockers.iter().copied() {
                *summary
                    .action_blockers_by_reason
                    .entry(blocker)
                    .or_insert(0usize) += 1;
            }
        } else if action.is_review {
            summary.review_findings += 1;
            *summary
                .review_by_reason
                .entry(record.finding.claim_kind)
                .or_insert(0usize) += 1;
        }
    }
    summary
}
