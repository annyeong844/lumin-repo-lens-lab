mod candidate;
mod model;
mod scoring;
mod tokens;

use std::collections::{BTreeMap, BTreeSet};

use crate::protocol::{AstFunctionCloneGroup, FileHealth, RUST_FUNCTION_CLONE_NEAR_MAX_CANDIDATES};

use model::{NearFact, NearFunctionCandidateProjection};

use super::common::{function_members, member_identity};

pub(super) fn build_near_function_candidates(
    files: &BTreeMap<String, FileHealth>,
    exact_body_groups: &[AstFunctionCloneGroup],
    structure_groups: &[AstFunctionCloneGroup],
) -> NearFunctionCandidateProjection {
    let grouped = grouped_identity_set(exact_body_groups, structure_groups);
    let mut eligible = function_members(files)
        .into_iter()
        .filter_map(|member| {
            let identity = member_identity(&member);
            if grouped.contains(&identity) {
                return None;
            }
            let significant_call_tokens = tokens::significant_call_tokens(member.fact);
            (!significant_call_tokens.is_empty()).then(|| NearFact {
                name_tokens: tokens::name_tokens(&member.fact.name),
                member,
                identity,
                significant_call_tokens,
            })
        })
        .collect::<Vec<_>>();
    eligible.sort_by(|left, right| left.identity.cmp(&right.identity));

    let mut by_call_token = BTreeMap::<&str, Vec<usize>>::new();
    for (index, fact) in eligible.iter().enumerate() {
        for token in &fact.significant_call_tokens {
            by_call_token.entry(token.as_str()).or_default().push(index);
        }
    }

    let mut pair_keys = BTreeSet::<(usize, usize)>::new();
    let mut candidates = Vec::new();
    for bucket in by_call_token.values() {
        for (left_offset, left_index) in bucket.iter().enumerate() {
            for right_index in bucket.iter().skip(left_offset + 1) {
                let pair_key = (*left_index, *right_index);
                if !pair_keys.insert(pair_key) {
                    continue;
                }
                if let Some(candidate) = candidate::near_candidate_from_pair(
                    &eligible[*left_index],
                    &eligible[*right_index],
                ) {
                    candidates.push(candidate);
                }
            }
        }
    }

    let review_visible_count = candidates
        .iter()
        .filter(|candidate| !candidate.generated_only)
        .count();

    candidates.sort_by(|left, right| {
        left.generated_only
            .cmp(&right.generated_only)
            .then_with(|| right.score.total_cmp(&left.score))
            .then_with(|| left.identities.join("|").cmp(&right.identities.join("|")))
    });
    candidates.truncate(RUST_FUNCTION_CLONE_NEAR_MAX_CANDIDATES);
    NearFunctionCandidateProjection {
        review_visible_count,
        candidates,
    }
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
