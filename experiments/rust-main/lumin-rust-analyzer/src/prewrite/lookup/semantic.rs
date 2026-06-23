use super::taint::locality;
use super::{CandidateRecord, SemanticHint, SuppressedSemanticHint, SuppressionReason};
use crate::prewrite::index::{Candidate, CandidateLane};
use crate::prewrite::intent::NameDeclaration;

mod order;
mod tokens;

use order::{sort_semantic_hints, sort_suppressed_semantic_hints};
use tokens::semantic_token_match;

pub(super) use tokens::query_tokens;

pub(in crate::prewrite) const SEMANTIC_HINT_MAX_RESULTS: usize = 5;
pub(in crate::prewrite) const SEMANTIC_HINT_MIN_SCORE: usize = 2;

pub(super) fn semantic_hint_candidates(
    intent_name: &str,
    declaration: Option<&NameDeclaration>,
    candidates: &[Candidate<'_>],
) -> (
    Vec<String>,
    Vec<SemanticHint>,
    Vec<SuppressedSemanticHint>,
    usize,
) {
    let intent_tokens = query_tokens(intent_name, declaration);
    if intent_tokens.is_empty() {
        return (intent_tokens, Vec::new(), Vec::new(), 0);
    }
    let owner_file = declaration.and_then(NameDeclaration::effective_owner_file);
    let mut hints = Vec::new();
    let mut suppressed = Vec::new();
    for candidate in candidates.iter().copied() {
        if candidate.name == intent_name && candidate.lane != CandidateLane::ImplMethod {
            continue;
        }
        let Some(token_match) = semantic_token_match(candidate, &intent_tokens) else {
            continue;
        };

        let score = token_match.score;
        let locality = locality(candidate.file, owner_file);
        if score < SEMANTIC_HINT_MIN_SCORE {
            let all_matches_are_weak = token_match.all_matches_are_weak();
            suppressed.push(SuppressedSemanticHint {
                candidate: CandidateRecord::from_candidate(candidate),
                reason: if all_matches_are_weak {
                    SuppressionReason::DomainTokenOverlap
                } else {
                    SuppressionReason::SingleNonWeakTokenOnly
                },
                matched_tokens: token_match.matched_tokens,
                matched_name_tokens: Vec::new(),
                matched_support_tokens: Vec::new(),
                score,
                locality,
                candidate_count: 0,
            });
            continue;
        }

        let all_matches_are_weak = token_match.all_matches_are_weak();
        let has_sufficient_non_weak_support = token_match.has_sufficient_non_weak_support();
        if !has_sufficient_non_weak_support {
            suppressed.push(SuppressedSemanticHint {
                candidate: CandidateRecord::from_candidate(candidate),
                reason: if all_matches_are_weak {
                    SuppressionReason::DomainTokenOverlap
                } else {
                    SuppressionReason::InsufficientNonWeakSupport
                },
                matched_tokens: token_match.matched_tokens,
                matched_name_tokens: token_match.matched_name_tokens,
                matched_support_tokens: token_match.matched_support_tokens,
                score,
                locality,
                candidate_count: 0,
            });
            continue;
        }

        hints.push(SemanticHint {
            candidate: CandidateRecord::from_candidate(candidate),
            matched_tokens: token_match.matched_tokens,
            matched_name_tokens: token_match.matched_name_tokens,
            matched_support_tokens: token_match.matched_support_tokens,
            score,
            locality,
        });
    }

    sort_semantic_hints(&mut hints);
    sort_suppressed_semantic_hints(&mut suppressed);
    let suppressed_count = suppressed.len();
    for hint in &mut suppressed {
        hint.candidate_count = suppressed_count;
    }
    hints.truncate(SEMANTIC_HINT_MAX_RESULTS);
    suppressed.truncate(SEMANTIC_HINT_MAX_RESULTS);
    (intent_tokens, hints, suppressed, suppressed_count)
}
