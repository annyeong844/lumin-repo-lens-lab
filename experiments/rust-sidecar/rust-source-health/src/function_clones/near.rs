use std::collections::{BTreeMap, BTreeSet};

use crate::protocol::{
    AstFunctionBodyFingerprint, AstFunctionCloneGroup, AstFunctionCloneLine,
    AstNearFunctionCandidate, AstNearFunctionCandidateKind, FileHealth, FunctionCloneRisk,
    RUST_FUNCTION_CLONE_NEAR_BODY_LOC_WEIGHT, RUST_FUNCTION_CLONE_NEAR_CALL_TOKEN_WEIGHT,
    RUST_FUNCTION_CLONE_NEAR_MAX_CANDIDATES, RUST_FUNCTION_CLONE_NEAR_MAX_PARAM_COUNT_DELTA,
    RUST_FUNCTION_CLONE_NEAR_MIN_BODY_LOC_SIMILARITY,
    RUST_FUNCTION_CLONE_NEAR_MIN_CALL_TOKEN_JACCARD,
    RUST_FUNCTION_CLONE_NEAR_MIN_NAME_TOKEN_JACCARD_FALLBACK, RUST_FUNCTION_CLONE_NEAR_MIN_SCORE,
    RUST_FUNCTION_CLONE_NEAR_MIN_SIGNIFICANT_CALL_TOKEN_LEN,
    RUST_FUNCTION_CLONE_NEAR_MIN_STATEMENT_COUNT_SIMILARITY,
    RUST_FUNCTION_CLONE_NEAR_NAME_TOKEN_WEIGHT, RUST_FUNCTION_CLONE_NEAR_STATEMENT_COUNT_WEIGHT,
    RUST_FUNCTION_CLONE_NEAR_SUPPRESSED_GENERIC_CALL_TOKENS,
};

use super::common::{function_members, member_identity, GroupMember};

pub(super) struct NearFunctionCandidateProjection {
    pub(super) review_visible_count: usize,
    pub(super) candidates: Vec<AstNearFunctionCandidate>,
}

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
            let significant_call_tokens = significant_call_tokens(member.fact);
            (!significant_call_tokens.is_empty()).then(|| NearFact {
                member,
                identity,
                significant_call_tokens,
                name_tokens: name_tokens(&member.fact.name),
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
                if let Some(candidate) =
                    near_candidate_from_pair(&eligible[*left_index], &eligible[*right_index])
                {
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

fn near_candidate_from_pair(
    left: &NearFact<'_>,
    right: &NearFact<'_>,
) -> Option<AstNearFunctionCandidate> {
    if left.member.fact.is_async != right.member.fact.is_async
        || left.member.fact.is_unsafe != right.member.fact.is_unsafe
        || left.member.fact.is_const != right.member.fact.is_const
    {
        return None;
    }
    if left
        .member
        .fact
        .param_count
        .abs_diff(right.member.fact.param_count)
        > RUST_FUNCTION_CLONE_NEAR_MAX_PARAM_COUNT_DELTA
    {
        return None;
    }

    let shared_call_tokens = sorted_intersection(
        &left.significant_call_tokens,
        &right.significant_call_tokens,
    );
    if shared_call_tokens.is_empty() {
        return None;
    }

    let call_token_jaccard = jaccard(
        &left.significant_call_tokens,
        &right.significant_call_tokens,
    );
    let shared_name_tokens = sorted_intersection(&left.name_tokens, &right.name_tokens);
    let name_token_jaccard = jaccard(&left.name_tokens, &right.name_tokens);
    let body_loc_similarity =
        range_similarity(left.member.fact.body_loc, right.member.fact.body_loc);
    let statement_count_similarity = range_similarity(
        left.member.fact.statement_count,
        right.member.fact.statement_count,
    );
    if body_loc_similarity < RUST_FUNCTION_CLONE_NEAR_MIN_BODY_LOC_SIMILARITY
        || statement_count_similarity < RUST_FUNCTION_CLONE_NEAR_MIN_STATEMENT_COUNT_SIMILARITY
    {
        return None;
    }
    if call_token_jaccard < RUST_FUNCTION_CLONE_NEAR_MIN_CALL_TOKEN_JACCARD
        && name_token_jaccard < RUST_FUNCTION_CLONE_NEAR_MIN_NAME_TOKEN_JACCARD_FALLBACK
    {
        return None;
    }

    let score = round_score(
        (call_token_jaccard * RUST_FUNCTION_CLONE_NEAR_CALL_TOKEN_WEIGHT)
            + (name_token_jaccard * RUST_FUNCTION_CLONE_NEAR_NAME_TOKEN_WEIGHT)
            + (body_loc_similarity * RUST_FUNCTION_CLONE_NEAR_BODY_LOC_WEIGHT)
            + (statement_count_similarity * RUST_FUNCTION_CLONE_NEAR_STATEMENT_COUNT_WEIGHT),
    );
    if score < RUST_FUNCTION_CLONE_NEAR_MIN_SCORE {
        return None;
    }

    let mut pair = [left, right];
    pair.sort_by(|a, b| a.identity.cmp(&b.identity));
    let lines = pair
        .iter()
        .map(|fact| AstFunctionCloneLine {
            identity: fact.identity.clone(),
            file: fact.member.file.to_string(),
            line: fact.member.fact.location.line,
        })
        .collect::<Vec<_>>();
    let body_locs = pair
        .iter()
        .map(|fact| fact.member.fact.body_loc)
        .collect::<Vec<_>>();
    let statement_counts = pair
        .iter()
        .map(|fact| fact.member.fact.statement_count)
        .collect::<Vec<_>>();
    let mut reasons = vec![
        format!(
            "shared significant call tokens: {}",
            shared_call_tokens.join(", ")
        ),
        format!(
            "body size similarity: {}",
            format_score(body_loc_similarity)
        ),
        format!(
            "statement-count similarity: {}",
            format_score(statement_count_similarity)
        ),
    ];
    if !shared_name_tokens.is_empty() {
        reasons.push(format!(
            "shared exported-name tokens: {}",
            shared_name_tokens.join(", ")
        ));
    }

    Some(AstNearFunctionCandidate {
        kind: AstNearFunctionCandidateKind::NearFunctionCandidate,
        identities: pair.iter().map(|fact| fact.identity.clone()).collect(),
        owner_files: pair
            .iter()
            .map(|fact| fact.member.file.to_string())
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect(),
        names: pair
            .iter()
            .map(|fact| fact.member.fact.name.clone())
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect(),
        lines,
        score,
        risk: FunctionCloneRisk::ReviewOnly,
        generated_only: pair.iter().all(|fact| fact.member.generated),
        shared_call_tokens,
        shared_name_tokens,
        call_token_jaccard: round_score(call_token_jaccard),
        name_token_jaccard: round_score(name_token_jaccard),
        body_loc_range: [
            body_locs.iter().copied().min().unwrap_or(0),
            body_locs.iter().copied().max().unwrap_or(0),
        ],
        statement_count_range: [
            statement_counts.iter().copied().min().unwrap_or(0),
            statement_counts.iter().copied().max().unwrap_or(0),
        ],
        reasons,
        reason: "near function cue only; source review required; not proof of semantic equivalence or an automatic merge",
    })
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

fn significant_call_tokens(fact: &AstFunctionBodyFingerprint) -> Vec<String> {
    fact.call_tokens
        .iter()
        .filter(|token| {
            token.len() >= RUST_FUNCTION_CLONE_NEAR_MIN_SIGNIFICANT_CALL_TOKEN_LEN
                && !RUST_FUNCTION_CLONE_NEAR_SUPPRESSED_GENERIC_CALL_TOKENS
                    .contains(&token.as_str())
        })
        .cloned()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn name_tokens(name: &str) -> Vec<String> {
    let mut expanded = String::new();
    let mut previous_lower_or_digit = false;
    for ch in name.chars() {
        if ch.is_ascii_uppercase() && previous_lower_or_digit {
            expanded.push(' ');
        }
        expanded.push(ch);
        previous_lower_or_digit = ch.is_ascii_lowercase() || ch.is_ascii_digit();
    }
    expanded
        .split(|ch: char| !ch.is_ascii_alphanumeric())
        .map(str::trim)
        .filter(|token| token.len() >= 2)
        .map(str::to_ascii_lowercase)
        .collect()
}

fn sorted_intersection(left: &[String], right: &[String]) -> Vec<String> {
    let right = right.iter().map(String::as_str).collect::<BTreeSet<_>>();
    left.iter()
        .filter(|entry| right.contains(entry.as_str()))
        .cloned()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn jaccard(left: &[String], right: &[String]) -> f64 {
    let left = left.iter().map(String::as_str).collect::<BTreeSet<_>>();
    let right = right.iter().map(String::as_str).collect::<BTreeSet<_>>();
    let union = left.union(&right).count();
    if union == 0 {
        return 0.0;
    }
    left.intersection(&right).count() as f64 / union as f64
}

fn range_similarity(left: usize, right: usize) -> f64 {
    let max = left.max(right);
    if max == 0 {
        return 0.0;
    }
    1.0 - (left.abs_diff(right) as f64 / max as f64)
}

fn round_score(value: f64) -> f64 {
    (value * 1000.0).round() / 1000.0
}

fn format_score(value: f64) -> String {
    let rounded = round_score(value);
    if rounded.fract() == 0.0 {
        format!("{rounded:.0}")
    } else {
        rounded.to_string()
    }
}

struct NearFact<'a> {
    member: GroupMember<'a>,
    identity: String,
    significant_call_tokens: Vec<String>,
    name_tokens: Vec<String>,
}
