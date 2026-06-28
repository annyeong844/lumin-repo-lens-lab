mod candidate;
mod model;
mod scoring;
mod tokens;

use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};

use crate::protocol::{
    AstFunctionCloneGroup, AstNearFunctionCandidate, FileHealth,
    RUST_FUNCTION_CLONE_NEAR_MAX_CANDIDATES,
};

use model::{NearFact, NearFunctionCandidateProjection};

use super::common::{function_members, member_identity};

pub(super) fn build_near_function_candidates(
    files: &BTreeMap<String, FileHealth>,
    exact_body_groups: &[AstFunctionCloneGroup],
    structure_groups: &[AstFunctionCloneGroup],
) -> NearFunctionCandidateProjection {
    let grouped = grouped_identity_set(exact_body_groups, structure_groups);
    let mut all_facts = function_members(files)
        .into_iter()
        .map(|member| {
            let identity = member_identity(&member);
            let significant_call_tokens = tokens::significant_call_tokens(member.fact);
            NearFact {
                name_tokens: tokens::name_tokens(&member.fact.name),
                member,
                identity,
                significant_call_tokens,
            }
        })
        .collect::<Vec<_>>();
    let token_idfs = scoring::call_token_idfs(&all_facts);
    let mut eligible = all_facts
        .drain(..)
        .filter(|fact| {
            !grouped.contains(&fact.identity) && !fact.significant_call_tokens.is_empty()
        })
        .collect::<Vec<_>>();
    eligible.sort_by(|left, right| left.identity.cmp(&right.identity));

    let mut by_call_token = BTreeMap::<&str, Vec<usize>>::new();
    for (index, fact) in eligible.iter().enumerate() {
        for token in &fact.significant_call_tokens {
            by_call_token.entry(token.as_str()).or_default().push(index);
        }
    }

    let mut review_visible_count = 0;
    let mut candidates = Vec::new();
    for (call_token, bucket) in &by_call_token {
        for (left_offset, left_index) in bucket.iter().enumerate() {
            for right_index in bucket.iter().skip(left_offset + 1) {
                if has_earlier_shared_call_token(
                    &eligible[*left_index].significant_call_tokens,
                    &eligible[*right_index].significant_call_tokens,
                    call_token,
                ) {
                    continue;
                }
                if let Some(candidate) = candidate::near_candidate_from_pair(
                    &eligible[*left_index],
                    &eligible[*right_index],
                    &token_idfs,
                ) {
                    if !candidate.generated_only {
                        review_visible_count += 1;
                    }
                    push_projected_candidate(&mut candidates, candidate);
                }
            }
        }
    }

    NearFunctionCandidateProjection {
        review_visible_count,
        candidates,
    }
}

fn has_earlier_shared_call_token(left: &[String], right: &[String], current_token: &str) -> bool {
    left.iter()
        .take_while(|token| token.as_str() < current_token)
        .any(|token| right.binary_search(token).is_ok())
}

fn push_projected_candidate(
    candidates: &mut Vec<AstNearFunctionCandidate>,
    candidate: AstNearFunctionCandidate,
) {
    if candidates.len() < RUST_FUNCTION_CLONE_NEAR_MAX_CANDIDATES {
        candidates.push(candidate);
        sort_projected_candidates(candidates);
        return;
    }

    if let Some(worst) = candidates.last() {
        if near_candidate_order(&candidate, worst) != Ordering::Less {
            return;
        }
    }

    candidates.pop();
    candidates.push(candidate);
    sort_projected_candidates(candidates);
}

fn sort_projected_candidates(candidates: &mut [AstNearFunctionCandidate]) {
    candidates.sort_by(near_candidate_order);
}

fn near_candidate_order(
    left: &AstNearFunctionCandidate,
    right: &AstNearFunctionCandidate,
) -> Ordering {
    left.generated_only
        .cmp(&right.generated_only)
        .then_with(|| right.score.total_cmp(&left.score))
        .then_with(|| left.identities.join("|").cmp(&right.identities.join("|")))
}

fn grouped_identity_set(
    exact_body_groups: &[AstFunctionCloneGroup],
    structure_groups: &[AstFunctionCloneGroup],
) -> BTreeSet<String> {
    exact_body_groups
        .iter()
        .chain(structure_groups)
        .flat_map(|group| group.identities.iter().cloned())
        .collect()
}
