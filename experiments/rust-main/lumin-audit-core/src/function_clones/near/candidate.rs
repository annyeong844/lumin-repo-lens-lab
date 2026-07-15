use std::collections::BTreeMap;

use serde_json::{json, Value};

use super::model::{NearCandidateEvidence, NearFact, SharedCallTokenEvidence};
use super::scoring::{
    number_string, range_similarity, round_score, saturated_call_token_idf_score,
    shared_token_idf_sum, token_idf, token_overlap,
};
use super::{
    BODY_LOC_SIMILARITY_WEIGHT, CALL_TOKEN_IDF_SCORE_WEIGHT, MAX_PARAM_COUNT_DELTA,
    MIN_BODY_LOC_SIMILARITY, MIN_CALL_TOKEN_IDF_SCORE, MIN_NAME_TOKEN_JACCARD_FALLBACK,
    MIN_NEAR_SCORE, MIN_SINGLE_TOKEN_IDF, MIN_STATEMENT_COUNT_SIMILARITY,
    NAME_TOKEN_JACCARD_WEIGHT, STATEMENT_COUNT_SIMILARITY_WEIGHT,
};
use crate::function_clones::projection::{line_json, sorted_unique};

pub(super) enum NearCandidateEvaluation {
    RejectedBeforeScoring,
    Scored(Option<NearCandidateEvidence>),
}

pub(super) fn near_candidate_evidence_from_pair(
    left: &NearFact<'_>,
    right: &NearFact<'_>,
    token_idfs: &BTreeMap<&str, f64>,
    generation_token: &str,
) -> NearCandidateEvaluation {
    if left.fact.async_value != right.fact.async_value {
        return NearCandidateEvaluation::RejectedBeforeScoring;
    }
    if (left.fact.param_count - right.fact.param_count).abs() > MAX_PARAM_COUNT_DELTA {
        return NearCandidateEvaluation::RejectedBeforeScoring;
    }

    let call_token_overlap = token_overlap(
        &left.significant_call_tokens,
        &right.significant_call_tokens,
    );
    if call_token_overlap.shared_tokens.is_empty() {
        return NearCandidateEvaluation::RejectedBeforeScoring;
    }
    if call_token_overlap.shared_tokens.len() == 1
        && token_idf(&call_token_overlap.shared_tokens[0], token_idfs) < MIN_SINGLE_TOKEN_IDF
    {
        return NearCandidateEvaluation::RejectedBeforeScoring;
    }

    let shared_call_token_idf_sum =
        shared_token_idf_sum(&call_token_overlap.shared_tokens, token_idfs);
    let call_token_idf_score = saturated_call_token_idf_score(shared_call_token_idf_sum);
    let name_token_overlap = token_overlap(&left.name_tokens, &right.name_tokens);
    let body_loc_similarity = range_similarity(left.fact.body_loc, right.fact.body_loc);
    let statement_count_similarity =
        range_similarity(left.fact.statement_count, right.fact.statement_count);
    if body_loc_similarity < MIN_BODY_LOC_SIMILARITY
        || statement_count_similarity < MIN_STATEMENT_COUNT_SIMILARITY
    {
        return NearCandidateEvaluation::RejectedBeforeScoring;
    }
    if call_token_idf_score < MIN_CALL_TOKEN_IDF_SCORE
        && name_token_overlap.jaccard < MIN_NAME_TOKEN_JACCARD_FALLBACK
    {
        return NearCandidateEvaluation::RejectedBeforeScoring;
    }

    let score = round_score(
        (call_token_idf_score * CALL_TOKEN_IDF_SCORE_WEIGHT)
            + (name_token_overlap.jaccard * NAME_TOKEN_JACCARD_WEIGHT)
            + (body_loc_similarity * BODY_LOC_SIMILARITY_WEIGHT)
            + (statement_count_similarity * STATEMENT_COUNT_SIMILARITY_WEIGHT),
    );
    if score < MIN_NEAR_SCORE {
        return NearCandidateEvaluation::Scored(None);
    }

    let shared_significant_call_tokens = call_token_overlap
        .shared_tokens
        .iter()
        .map(|token| {
            let idf = token_idf(token, token_idfs);
            SharedCallTokenEvidence {
                token: token.clone(),
                idf: round_score(idf),
                retained: idf >= MIN_SINGLE_TOKEN_IDF,
            }
        })
        .collect();

    NearCandidateEvaluation::Scored(Some(NearCandidateEvidence {
        generation_token: generation_token.to_string(),
        score,
        generated_only: left.fact.generated_file && right.fact.generated_file,
        shared_call_tokens: call_token_overlap.shared_tokens,
        shared_significant_call_tokens,
        shared_name_tokens: name_token_overlap.shared_tokens,
        call_token_jaccard: call_token_overlap.jaccard,
        shared_call_token_idf_sum,
        call_token_idf_score,
        name_token_jaccard: name_token_overlap.jaccard,
        body_loc_similarity,
        statement_count_similarity,
    }))
}

pub(super) fn build_near_candidate_from_evidence(
    left: &NearFact<'_>,
    right: &NearFact<'_>,
    evidence: NearCandidateEvidence,
) -> Value {
    let mut pair = [left.fact, right.fact];
    pair.sort_by(|a, b| a.identity.cmp(&b.identity));
    let body_locs = pair.iter().map(|fact| fact.body_loc).collect::<Vec<_>>();
    let statement_counts = pair
        .iter()
        .map(|fact| fact.statement_count)
        .collect::<Vec<_>>();
    let mut reasons = vec![
        format!(
            "shared significant call tokens: {}",
            evidence.shared_call_tokens.join(", ")
        ),
        format!(
            "shared call-token IDF sum: {}",
            number_string(evidence.shared_call_token_idf_sum)
        ),
        format!(
            "call-token IDF score: {}",
            number_string(evidence.call_token_idf_score)
        ),
        format!(
            "body size similarity: {}",
            number_string(evidence.body_loc_similarity)
        ),
        format!(
            "statement-count similarity: {}",
            number_string(evidence.statement_count_similarity)
        ),
    ];
    if !evidence.shared_name_tokens.is_empty() {
        reasons.push(format!(
            "shared exported-name tokens: {}",
            evidence.shared_name_tokens.join(", ")
        ));
    }

    let shared_significant_call_tokens = evidence
        .shared_significant_call_tokens
        .iter()
        .map(|token| {
            json!({
                "token": token.token,
                "idf": token.idf,
                "retained": token.retained,
            })
        })
        .collect::<Vec<_>>();
    let mut exported_names = pair
        .iter()
        .map(|fact| fact.exported_name.clone())
        .collect::<Vec<_>>();
    exported_names.sort();

    json!({
        "kind": "near-function-candidate",
        "identities": pair.iter().map(|fact| fact.identity.clone()).collect::<Vec<_>>(),
        "ownerFiles": sorted_unique(pair.iter().map(|fact| fact.owner_file.clone())),
        "exportedNames": exported_names,
        "lines": pair.iter().copied().map(line_json).collect::<Vec<_>>(),
        "score": evidence.score,
        "risk": "review-only",
        "generatedOnly": evidence.generated_only,
        "generationToken": evidence.generation_token,
        "sharedCallTokens": evidence.shared_call_tokens,
        "sharedSignificantCallTokens": shared_significant_call_tokens,
        "sharedNameTokens": evidence.shared_name_tokens,
        "callTokenJaccard": round_score(evidence.call_token_jaccard),
        "sharedCallTokenIdfSum": round_score(evidence.shared_call_token_idf_sum),
        "callTokenIdfScore": round_score(evidence.call_token_idf_score),
        "nameTokenJaccard": round_score(evidence.name_token_jaccard),
        "bodyLocRange": [
            body_locs.iter().copied().min().unwrap_or(0),
            body_locs.iter().copied().max().unwrap_or(0),
        ],
        "statementCountRange": [
            statement_counts.iter().copied().min().unwrap_or(0),
            statement_counts.iter().copied().max().unwrap_or(0),
        ],
        "reasons": reasons,
        "reason": "near function cue only; source review required; not proof of semantic equivalence or an automatic merge",
    })
}
