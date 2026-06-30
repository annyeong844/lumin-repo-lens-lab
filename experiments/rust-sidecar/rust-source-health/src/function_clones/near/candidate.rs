use std::collections::{BTreeMap, BTreeSet};

use crate::protocol::{
    AstFunctionCloneLine, AstNearFunctionCandidate, AstNearFunctionCandidateKind,
    FunctionCloneRisk, RUST_FUNCTION_CLONE_NEAR_BODY_LOC_WEIGHT,
    RUST_FUNCTION_CLONE_NEAR_CALL_TOKEN_WEIGHT, RUST_FUNCTION_CLONE_NEAR_MAX_PARAM_COUNT_DELTA,
    RUST_FUNCTION_CLONE_NEAR_MIN_BODY_LOC_SIMILARITY,
    RUST_FUNCTION_CLONE_NEAR_MIN_CALL_TOKEN_IDF_SCORE,
    RUST_FUNCTION_CLONE_NEAR_MIN_NAME_TOKEN_JACCARD_FALLBACK, RUST_FUNCTION_CLONE_NEAR_MIN_SCORE,
    RUST_FUNCTION_CLONE_NEAR_MIN_SINGLE_TOKEN_IDF,
    RUST_FUNCTION_CLONE_NEAR_MIN_STATEMENT_COUNT_SIMILARITY,
    RUST_FUNCTION_CLONE_NEAR_NAME_TOKEN_WEIGHT, RUST_FUNCTION_CLONE_NEAR_STATEMENT_COUNT_WEIGHT,
};

use super::model::NearFact;
use super::scoring::{
    format_score, range_similarity, round_score, saturated_call_token_idf_score,
    shared_token_idf_sum, token_idf, token_overlap,
};

pub(super) struct NearCandidateEvidence {
    pub(super) score: f64,
    pub(super) generated_only: bool,
    shared_call_tokens: Vec<String>,
    shared_name_tokens: Vec<String>,
    call_token_jaccard: f64,
    shared_call_token_idf_sum: f64,
    call_token_idf_score: f64,
    name_token_jaccard: f64,
    body_loc_similarity: f64,
    statement_count_similarity: f64,
}

pub(super) fn near_candidate_evidence_from_pair(
    left: &NearFact<'_>,
    right: &NearFact<'_>,
    token_idfs: &BTreeMap<String, f64>,
) -> Option<NearCandidateEvidence> {
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

    let call_token_overlap = token_overlap(
        &left.significant_call_tokens,
        &right.significant_call_tokens,
    );
    if call_token_overlap.shared_tokens.is_empty() {
        return None;
    }
    if call_token_overlap.shared_tokens.len() == 1
        && token_idf(&call_token_overlap.shared_tokens[0], token_idfs)
            < RUST_FUNCTION_CLONE_NEAR_MIN_SINGLE_TOKEN_IDF
    {
        return None;
    }

    let shared_call_token_idf_sum =
        shared_token_idf_sum(&call_token_overlap.shared_tokens, token_idfs);
    let call_token_idf_score = saturated_call_token_idf_score(shared_call_token_idf_sum);
    let name_token_overlap = token_overlap(&left.name_tokens, &right.name_tokens);
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
    if call_token_idf_score < RUST_FUNCTION_CLONE_NEAR_MIN_CALL_TOKEN_IDF_SCORE
        && name_token_overlap.jaccard < RUST_FUNCTION_CLONE_NEAR_MIN_NAME_TOKEN_JACCARD_FALLBACK
    {
        return None;
    }

    let score = round_score(
        (call_token_idf_score * RUST_FUNCTION_CLONE_NEAR_CALL_TOKEN_WEIGHT)
            + (name_token_overlap.jaccard * RUST_FUNCTION_CLONE_NEAR_NAME_TOKEN_WEIGHT)
            + (body_loc_similarity * RUST_FUNCTION_CLONE_NEAR_BODY_LOC_WEIGHT)
            + (statement_count_similarity * RUST_FUNCTION_CLONE_NEAR_STATEMENT_COUNT_WEIGHT),
    );
    if score < RUST_FUNCTION_CLONE_NEAR_MIN_SCORE {
        return None;
    }

    Some(NearCandidateEvidence {
        score,
        generated_only: left.member.generated && right.member.generated,
        shared_call_tokens: call_token_overlap.shared_tokens,
        shared_name_tokens: name_token_overlap.shared_tokens,
        call_token_jaccard: call_token_overlap.jaccard,
        shared_call_token_idf_sum,
        call_token_idf_score,
        name_token_jaccard: name_token_overlap.jaccard,
        body_loc_similarity,
        statement_count_similarity,
    })
}

pub(super) fn near_pair_order_against_projected(
    left: &NearFact<'_>,
    right: &NearFact<'_>,
    evidence: &NearCandidateEvidence,
    projected: &AstNearFunctionCandidate,
) -> std::cmp::Ordering {
    evidence
        .generated_only
        .cmp(&projected.generated_only)
        .then_with(|| projected.score.total_cmp(&evidence.score))
        .then_with(|| pair_identity_key(left, right).cmp(&projected.identities.join("|")))
}

pub(super) fn build_near_candidate_from_evidence(
    left: &NearFact<'_>,
    right: &NearFact<'_>,
    evidence: NearCandidateEvidence,
) -> AstNearFunctionCandidate {
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
            evidence.shared_call_tokens.join(", ")
        ),
        format!(
            "shared call-token IDF sum: {}",
            format_score(evidence.shared_call_token_idf_sum)
        ),
        format!(
            "call-token IDF score: {}",
            format_score(evidence.call_token_idf_score)
        ),
        format!(
            "body size similarity: {}",
            format_score(evidence.body_loc_similarity)
        ),
        format!(
            "statement-count similarity: {}",
            format_score(evidence.statement_count_similarity)
        ),
    ];
    if !evidence.shared_name_tokens.is_empty() {
        reasons.push(format!(
            "shared exported-name tokens: {}",
            evidence.shared_name_tokens.join(", ")
        ));
    }

    AstNearFunctionCandidate {
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
        score: evidence.score,
        risk: FunctionCloneRisk::ReviewOnly,
        generated_only: evidence.generated_only,
        shared_call_tokens: evidence.shared_call_tokens,
        shared_name_tokens: evidence.shared_name_tokens,
        call_token_jaccard: round_score(evidence.call_token_jaccard),
        shared_call_token_idf_sum: round_score(evidence.shared_call_token_idf_sum),
        call_token_idf_score: round_score(evidence.call_token_idf_score),
        name_token_jaccard: round_score(evidence.name_token_jaccard),
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
    }
}

fn pair_identity_key(left: &NearFact<'_>, right: &NearFact<'_>) -> String {
    if left.identity <= right.identity {
        format!("{}|{}", left.identity, right.identity)
    } else {
        format!("{}|{}", right.identity, left.identity)
    }
}
