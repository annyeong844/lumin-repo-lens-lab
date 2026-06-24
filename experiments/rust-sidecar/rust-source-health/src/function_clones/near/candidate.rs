use std::collections::BTreeSet;

use crate::protocol::{
    AstFunctionCloneLine, AstNearFunctionCandidate, AstNearFunctionCandidateKind,
    FunctionCloneRisk, RUST_FUNCTION_CLONE_NEAR_BODY_LOC_WEIGHT,
    RUST_FUNCTION_CLONE_NEAR_CALL_TOKEN_WEIGHT, RUST_FUNCTION_CLONE_NEAR_MAX_PARAM_COUNT_DELTA,
    RUST_FUNCTION_CLONE_NEAR_MIN_BODY_LOC_SIMILARITY,
    RUST_FUNCTION_CLONE_NEAR_MIN_CALL_TOKEN_JACCARD,
    RUST_FUNCTION_CLONE_NEAR_MIN_NAME_TOKEN_JACCARD_FALLBACK, RUST_FUNCTION_CLONE_NEAR_MIN_SCORE,
    RUST_FUNCTION_CLONE_NEAR_MIN_STATEMENT_COUNT_SIMILARITY,
    RUST_FUNCTION_CLONE_NEAR_NAME_TOKEN_WEIGHT, RUST_FUNCTION_CLONE_NEAR_STATEMENT_COUNT_WEIGHT,
};

use super::model::NearFact;
use super::scoring::{format_score, jaccard, range_similarity, round_score, sorted_intersection};

pub(super) fn near_candidate_from_pair(
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
