use super::taint::locality;
use super::{CandidateRecord, SemanticHint, SuppressedSemanticHint, SuppressionReason};
use crate::prewrite::index::{Candidate, CandidateLane};
use crate::prewrite::intent::NameDeclaration;
use crate::prewrite::tokens::{is_weak_common_token, unique_tokens};

const SEMANTIC_HINT_MAX_RESULTS: usize = 5;
const SEMANTIC_HINT_MIN_SCORE: usize = 2;

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
        let candidate_name_tokens = unique_tokens(&[candidate.name]);
        let file_stem = candidate
            .file
            .rsplit('/')
            .next()
            .unwrap_or(candidate.file)
            .rsplit_once('.')
            .map(|(stem, _)| stem)
            .unwrap_or(candidate.file);
        let owner_dir = candidate
            .file
            .rsplit_once('/')
            .map(|(directory, _)| directory)
            .unwrap_or("");
        let candidate_support_tokens =
            unique_tokens(&[file_stem, owner_dir, candidate.owner_name().unwrap_or("")]);
        let mut candidate_tokens = candidate_name_tokens.clone();
        extend_unique(&mut candidate_tokens, &candidate_support_tokens);
        let matched_tokens = candidate_tokens
            .iter()
            .filter(|token| intent_tokens.contains(token))
            .cloned()
            .collect::<Vec<_>>();
        if matched_tokens.is_empty() {
            continue;
        }

        let score = matched_tokens.len();
        let locality = locality(candidate.file, owner_file);
        if score < SEMANTIC_HINT_MIN_SCORE {
            suppressed.push(SuppressedSemanticHint {
                candidate: CandidateRecord::from_candidate(candidate),
                reason: if matched_tokens
                    .iter()
                    .all(|token| is_weak_common_token(token))
                {
                    SuppressionReason::DomainTokenOverlap
                } else {
                    SuppressionReason::SingleNonWeakTokenOnly
                },
                matched_tokens,
                matched_name_tokens: Vec::new(),
                matched_support_tokens: Vec::new(),
                score,
                locality,
                candidate_count: 0,
            });
            continue;
        }

        let matched_name_tokens = candidate_name_tokens
            .iter()
            .filter(|token| intent_tokens.contains(token))
            .cloned()
            .collect::<Vec<_>>();
        let strong_name_matches = matched_name_tokens
            .iter()
            .filter(|token| !is_weak_common_token(token))
            .cloned()
            .collect::<Vec<_>>();
        let strong_support_matches = candidate_support_tokens
            .iter()
            .filter(|token| {
                intent_tokens.contains(token)
                    && !is_weak_common_token(token)
                    && !strong_name_matches.contains(token)
            })
            .cloned()
            .collect::<Vec<_>>();
        let has_sufficient_non_weak_support = strong_name_matches.len() >= 2
            || (strong_name_matches.len() == 1 && !strong_support_matches.is_empty());
        if !has_sufficient_non_weak_support {
            suppressed.push(SuppressedSemanticHint {
                candidate: CandidateRecord::from_candidate(candidate),
                reason: if matched_tokens
                    .iter()
                    .all(|token| is_weak_common_token(token))
                {
                    SuppressionReason::DomainTokenOverlap
                } else {
                    SuppressionReason::InsufficientNonWeakSupport
                },
                matched_tokens,
                matched_name_tokens,
                matched_support_tokens: strong_support_matches,
                score,
                locality,
                candidate_count: 0,
            });
            continue;
        }

        hints.push(SemanticHint {
            candidate: CandidateRecord::from_candidate(candidate),
            matched_tokens,
            matched_name_tokens,
            matched_support_tokens: strong_support_matches,
            score,
            locality,
        });
    }

    hints.sort_by(|left, right| {
        right
            .locality
            .rank()
            .cmp(&left.locality.rank())
            .then(right.score.cmp(&left.score))
            .then(left.candidate.name.cmp(&right.candidate.name))
            .then(left.candidate.owner_file.cmp(&right.candidate.owner_file))
    });
    suppressed.sort_by(|left, right| {
        right
            .locality
            .rank()
            .cmp(&left.locality.rank())
            .then(right.score.cmp(&left.score))
            .then(left.candidate.name.cmp(&right.candidate.name))
            .then(left.candidate.owner_file.cmp(&right.candidate.owner_file))
    });
    let suppressed_count = suppressed.len();
    for hint in &mut suppressed {
        hint.candidate_count = suppressed_count;
    }
    hints.truncate(SEMANTIC_HINT_MAX_RESULTS);
    suppressed.truncate(SEMANTIC_HINT_MAX_RESULTS);
    (intent_tokens, hints, suppressed, suppressed_count)
}

pub(super) fn query_tokens(
    intent_name: &str,
    declaration: Option<&NameDeclaration>,
) -> Vec<String> {
    unique_tokens(&[
        intent_name,
        declaration
            .and_then(|value| value.kind.as_deref())
            .unwrap_or(""),
        declaration
            .and_then(|value| value.why.as_deref())
            .unwrap_or(""),
    ])
}

fn extend_unique(target: &mut Vec<String>, values: &[String]) {
    for value in values {
        if !target.contains(value) {
            target.push(value.clone());
        }
    }
}
