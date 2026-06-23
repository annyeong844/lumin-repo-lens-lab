use super::taint::locality;
use super::{CandidateRecord, NearNameHint, SuppressedNearNameHint, SuppressionReason};
use crate::prewrite::index::{Candidate, CandidateLane};
use crate::prewrite::tokens::{common_tokens, has_only_weak_common_tokens};

mod order;
mod scoring;

use order::{lane_rank, suppressed_near_order};
use scoring::{levenshtein_capped, shared_prefix};

pub(in crate::prewrite) const NEAR_NAME_MAX_LENGTH_DELTA: usize = 2;
pub(in crate::prewrite) const NEAR_NAME_SHARED_PREFIX_MIN: usize = 4;
pub(in crate::prewrite) const NEAR_NAME_MAX_DISTANCE: usize = 2;
pub(in crate::prewrite) const NEAR_NAME_MAX_RESULTS: usize = 5;

pub(super) fn near_name_candidates(
    intent_name: &str,
    owner_file: Option<&str>,
    candidates: &[Candidate<'_>],
) -> (Vec<NearNameHint>, Vec<SuppressedNearNameHint>, usize) {
    let mut hints = Vec::new();
    let mut suppressed = Vec::new();
    for candidate in candidates.iter().copied() {
        if candidate.name == intent_name && candidate.lane != CandidateLane::ImplMethod {
            continue;
        }
        let matched_tokens = common_tokens(intent_name, candidate.name);
        let has_common_token_signal = !matched_tokens.is_empty();
        let locality = locality(candidate.file, owner_file);
        if has_only_weak_common_tokens(intent_name, candidate.name) {
            suppressed.push(SuppressedNearNameHint {
                candidate: CandidateRecord::from_candidate(candidate),
                reason: SuppressionReason::DomainTokenOverlap,
                matched_tokens,
                distance: None,
                length_delta: None,
                locality,
                candidate_count: 0,
            });
            continue;
        }

        let prefix = shared_prefix(candidate.name, intent_name);
        if prefix >= NEAR_NAME_SHARED_PREFIX_MIN
            && candidate
                .name
                .chars()
                .count()
                .abs_diff(intent_name.chars().count())
                <= intent_name.chars().count()
        {
            hints.push(NearNameHint {
                candidate: CandidateRecord::from_candidate(candidate),
                distance: levenshtein_capped(
                    candidate.name,
                    intent_name,
                    NEAR_NAME_MAX_DISTANCE * 4,
                ),
                matched_tokens,
                locality,
            });
            continue;
        }

        let length_delta = candidate
            .name
            .chars()
            .count()
            .abs_diff(intent_name.chars().count());
        if length_delta > NEAR_NAME_MAX_LENGTH_DELTA {
            if has_common_token_signal || prefix >= NEAR_NAME_SHARED_PREFIX_MIN {
                suppressed.push(SuppressedNearNameHint {
                    candidate: CandidateRecord::from_candidate(candidate),
                    reason: SuppressionReason::NearLengthDeltaExceeded,
                    matched_tokens,
                    distance: None,
                    length_delta: Some(length_delta),
                    locality,
                    candidate_count: 0,
                });
            }
            continue;
        }

        let distance = levenshtein_capped(candidate.name, intent_name, NEAR_NAME_MAX_DISTANCE);
        if distance <= NEAR_NAME_MAX_DISTANCE {
            hints.push(NearNameHint {
                candidate: CandidateRecord::from_candidate(candidate),
                distance,
                matched_tokens,
                locality,
            });
        } else if has_common_token_signal || prefix >= NEAR_NAME_SHARED_PREFIX_MIN {
            suppressed.push(SuppressedNearNameHint {
                candidate: CandidateRecord::from_candidate(candidate),
                reason: if prefix < NEAR_NAME_SHARED_PREFIX_MIN && !has_common_token_signal {
                    SuppressionReason::NearPrefixMismatch
                } else {
                    SuppressionReason::NearDistanceExceeded
                },
                matched_tokens,
                distance: Some(distance),
                length_delta: None,
                locality,
                candidate_count: 0,
            });
        }
    }

    hints.sort_by(|left, right| {
        left.distance
            .cmp(&right.distance)
            .then(
                lane_rank(left.candidate.matched_field)
                    .cmp(&lane_rank(right.candidate.matched_field)),
            )
            .then(left.candidate.name.cmp(&right.candidate.name))
            .then(left.candidate.owner_file.cmp(&right.candidate.owner_file))
    });
    suppressed.sort_by(suppressed_near_order);
    let suppressed_count = suppressed.len();
    for hint in &mut suppressed {
        hint.candidate_count = suppressed_count;
    }
    hints.truncate(NEAR_NAME_MAX_RESULTS);
    suppressed.truncate(NEAR_NAME_MAX_RESULTS);
    (hints, suppressed, suppressed_count)
}
