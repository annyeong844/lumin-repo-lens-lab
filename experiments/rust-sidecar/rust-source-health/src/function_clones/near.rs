mod candidate;
mod model;
mod scoring;
mod tokens;

use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};

use crate::protocol::{
    AstFunctionCloneGroup, AstNearFunctionCandidate, AstSkippedLowDiscriminationBucket, FileHealth,
    RUST_FUNCTION_CLONE_NEAR_MAX_CANDIDATES, RUST_FUNCTION_CLONE_NEAR_MAX_PARAM_COUNT_DELTA,
    RUST_FUNCTION_CLONE_NEAR_MIN_BODY_LOC_SIMILARITY,
    RUST_FUNCTION_CLONE_NEAR_MIN_SINGLE_TOKEN_IDF,
    RUST_FUNCTION_CLONE_NEAR_MIN_STATEMENT_COUNT_SIMILARITY,
    RUST_FUNCTION_CLONE_NEAR_SKIPPED_BUCKET_SAMPLE_LIMIT,
};

use model::{
    CandidateGenerationDiagnostics, CompatibilityKey, NearFact, NearFunctionCandidateProjection,
    QualifierSignature,
};

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
                is_debug_formatter_boilerplate: tokens::is_debug_formatter_boilerplate(member.fact),
                is_display_formatter: tokens::is_display_formatter(member.fact),
                member,
                identity,
                significant_call_tokens,
                retained_call_tokens: Vec::new(),
            }
        })
        .collect::<Vec<_>>();
    let token_idfs = scoring::call_token_idfs(&all_facts);
    for fact in &mut all_facts {
        fact.retained_call_tokens = fact
            .significant_call_tokens
            .iter()
            .filter(|token| {
                scoring::token_idf(token, &token_idfs)
                    >= RUST_FUNCTION_CLONE_NEAR_MIN_SINGLE_TOKEN_IDF
            })
            .cloned()
            .collect();
    }
    let mut eligible = all_facts
        .drain(..)
        .filter(|fact| {
            !grouped.contains(&fact.identity) && !fact.significant_call_tokens.is_empty()
        })
        .collect::<Vec<_>>();
    eligible.sort_by(|left, right| left.identity.cmp(&right.identity));
    let eligible_function_count = eligible.len();

    let mut all_by_call_token = BTreeMap::<String, Vec<usize>>::new();
    for (index, fact) in eligible.iter().enumerate() {
        for token in &fact.significant_call_tokens {
            all_by_call_token
                .entry(token.clone())
                .or_default()
                .push(index);
        }
    }

    let mut skipped_low_discrimination_buckets = Vec::new();
    for (token, bucket) in &all_by_call_token {
        if scoring::token_idf(token, &token_idfs) >= RUST_FUNCTION_CLONE_NEAR_MIN_SINGLE_TOKEN_IDF {
            continue;
        }
        let raw_pair_estimate = raw_pair_estimate(bucket.len());
        if raw_pair_estimate == 0 {
            continue;
        }
        skipped_low_discrimination_buckets.push(AstSkippedLowDiscriminationBucket {
            token: token.clone(),
            idf: scoring::round_score(scoring::token_idf(token, &token_idfs)),
            function_count: bucket.len(),
            raw_pair_estimate,
            reason: "below-min-single-token-idf",
        });
    }
    let skipped_low_discrimination_bucket_count = skipped_low_discrimination_buckets.len();
    let skipped_low_discrimination_raw_pair_estimate = skipped_low_discrimination_buckets
        .iter()
        .map(|bucket| bucket.raw_pair_estimate)
        .sum();
    skipped_low_discrimination_buckets.sort_by(|left, right| {
        right
            .raw_pair_estimate
            .cmp(&left.raw_pair_estimate)
            .then_with(|| left.token.cmp(&right.token))
    });
    skipped_low_discrimination_buckets
        .truncate(RUST_FUNCTION_CLONE_NEAR_SKIPPED_BUCKET_SAMPLE_LIMIT);

    let mut retained = BTreeMap::<String, BTreeMap<CompatibilityKey, Vec<usize>>>::new();
    for (index, fact) in eligible.iter().enumerate() {
        let key = compatibility_key(fact);
        for token in &fact.retained_call_tokens {
            retained
                .entry(token.clone())
                .or_default()
                .entry(key.clone())
                .or_default()
                .push(index);
        }
    }

    let mut diagnostics = CandidateGenerationDiagnostics {
        eligible_function_count,
        retained_call_token_bucket_count: retained.len(),
        skipped_low_discrimination_buckets,
        skipped_low_discrimination_bucket_count,
        skipped_low_discrimination_raw_pair_estimate,
        ..CandidateGenerationDiagnostics::default()
    };

    let mut review_visible_count = 0;
    let mut candidates = Vec::new();
    {
        let mut generation = CandidateGenerationState {
            eligible: &eligible,
            token_idfs: &token_idfs,
            diagnostics: &mut diagnostics,
            review_visible_count: &mut review_visible_count,
            candidates: &mut candidates,
        };
        for (call_token, postings) in &retained {
            update_retained_pair_estimates(postings, generation.diagnostics);
            let posting_entries = postings.iter().collect::<Vec<_>>();
            for (left_posting_offset, (left_key, left_bucket)) in posting_entries.iter().enumerate()
            {
                for (right_key, right_bucket) in posting_entries.iter().skip(left_posting_offset) {
                    if !compatibility_keys_match(left_key, right_key) {
                        continue;
                    }
                    if left_key == right_key {
                        for (left_offset, left_index) in left_bucket.iter().enumerate() {
                            for right_index in left_bucket.iter().skip(left_offset + 1) {
                                generation.generate_candidate_from_pair(
                                    *left_index,
                                    *right_index,
                                    call_token,
                                );
                            }
                        }
                        continue;
                    }
                    for left_index in left_bucket.iter() {
                        for right_index in right_bucket.iter() {
                            generation.generate_candidate_from_pair(
                                *left_index,
                                *right_index,
                                call_token,
                            );
                        }
                    }
                }
            }
        }
    }

    NearFunctionCandidateProjection {
        review_visible_count,
        candidates,
        diagnostics,
    }
}

struct CandidateGenerationState<'facts, 'state> {
    eligible: &'state [NearFact<'facts>],
    token_idfs: &'state BTreeMap<String, f64>,
    diagnostics: &'state mut CandidateGenerationDiagnostics,
    review_visible_count: &'state mut usize,
    candidates: &'state mut Vec<AstNearFunctionCandidate>,
}

impl<'facts, 'state> CandidateGenerationState<'facts, 'state> {
    fn generate_candidate_from_pair(
        &mut self,
        left_index: usize,
        right_index: usize,
        call_token: &str,
    ) {
        if has_earlier_shared_call_token(
            &self.eligible[left_index].retained_call_tokens,
            &self.eligible[right_index].retained_call_tokens,
            call_token,
        ) {
            return;
        }

        if self.eligible[left_index].is_debug_formatter_boilerplate
            && self.eligible[right_index].is_debug_formatter_boilerplate
        {
            self.diagnostics
                .debug_formatter_boilerplate_skipped_pair_count += 1;
            return;
        }
        if self.eligible[left_index].is_display_formatter
            && self.eligible[right_index].is_display_formatter
            && tokens::shared_tokens_are_only_display_formatter_sinks(
                &self.eligible[left_index].significant_call_tokens,
                &self.eligible[right_index].significant_call_tokens,
            )
        {
            self.diagnostics
                .display_formatter_boilerplate_skipped_pair_count += 1;
            return;
        }

        self.diagnostics.generated_unique_pair_count += 1;
        if let Some(evidence) = candidate::near_candidate_evidence_from_pair(
            &self.eligible[left_index],
            &self.eligible[right_index],
            self.token_idfs,
        ) {
            self.diagnostics.scored_pair_count += 1;
            if !evidence.generated_only {
                *self.review_visible_count += 1;
            }
            if should_project_candidate(
                self.candidates,
                &self.eligible[left_index],
                &self.eligible[right_index],
                &evidence,
            ) {
                let candidate = candidate::build_near_candidate_from_evidence(
                    &self.eligible[left_index],
                    &self.eligible[right_index],
                    evidence,
                );
                push_projected_candidate(self.candidates, candidate);
            }
        }
    }
}

fn compatibility_key(fact: &NearFact<'_>) -> CompatibilityKey {
    CompatibilityKey {
        qualifier_signature: QualifierSignature {
            is_async: fact.member.fact.is_async,
            is_unsafe: fact.member.fact.is_unsafe,
            is_const: fact.member.fact.is_const,
        },
        param_count: fact.member.fact.param_count,
        body_loc_band: range_band(
            fact.member.fact.body_loc,
            RUST_FUNCTION_CLONE_NEAR_MIN_BODY_LOC_SIMILARITY,
        ),
        statement_count_band: range_band(
            fact.member.fact.statement_count,
            RUST_FUNCTION_CLONE_NEAR_MIN_STATEMENT_COUNT_SIMILARITY,
        ),
    }
}

fn range_band(value: usize, min_similarity: f64) -> usize {
    if value == 0 {
        return 0;
    }
    let base = 1.0 / min_similarity;
    ((value as f64).ln() / base.ln()).floor() as usize + 1
}

fn compatibility_keys_match(left: &CompatibilityKey, right: &CompatibilityKey) -> bool {
    left.qualifier_signature == right.qualifier_signature
        && left.param_count.abs_diff(right.param_count)
            <= RUST_FUNCTION_CLONE_NEAR_MAX_PARAM_COUNT_DELTA
        && bands_compatible(left.body_loc_band, right.body_loc_band)
        && bands_compatible(left.statement_count_band, right.statement_count_band)
}

fn bands_compatible(left: usize, right: usize) -> bool {
    left.abs_diff(right) <= 1
}

fn update_retained_pair_estimates(
    postings: &BTreeMap<CompatibilityKey, Vec<usize>>,
    diagnostics: &mut CandidateGenerationDiagnostics,
) {
    let entries = postings
        .iter()
        .map(|(key, bucket)| (key, bucket.len()))
        .collect::<Vec<_>>();
    let total = raw_pair_estimate(entries.iter().map(|(_, count)| *count).sum());
    let qualifier = estimate_pairs_matching(&entries, |left, right| {
        left.qualifier_signature == right.qualifier_signature
    });
    let parameter = estimate_pairs_matching(&entries, |left, right| {
        left.qualifier_signature == right.qualifier_signature
            && left.param_count.abs_diff(right.param_count)
                <= RUST_FUNCTION_CLONE_NEAR_MAX_PARAM_COUNT_DELTA
    });
    let body = estimate_pairs_matching(&entries, |left, right| {
        left.qualifier_signature == right.qualifier_signature
            && left.param_count.abs_diff(right.param_count)
                <= RUST_FUNCTION_CLONE_NEAR_MAX_PARAM_COUNT_DELTA
            && bands_compatible(left.body_loc_band, right.body_loc_band)
    });
    let statement = estimate_pairs_matching(&entries, compatibility_keys_match);

    diagnostics.retained_raw_pair_estimate += total;
    diagnostics
        .compatibility_skipped_raw_pair_estimate_by_reason
        .qualifier_mismatch += total.saturating_sub(qualifier);
    diagnostics
        .compatibility_skipped_raw_pair_estimate_by_reason
        .parameter_count_delta += qualifier.saturating_sub(parameter);
    diagnostics
        .compatibility_skipped_raw_pair_estimate_by_reason
        .body_loc_band_mismatch += parameter.saturating_sub(body);
    diagnostics
        .compatibility_skipped_raw_pair_estimate_by_reason
        .statement_count_band_mismatch += body.saturating_sub(statement);
}

fn estimate_pairs_matching<F>(entries: &[(&CompatibilityKey, usize)], matches: F) -> usize
where
    F: Fn(&CompatibilityKey, &CompatibilityKey) -> bool,
{
    let mut total = 0usize;
    for (left_offset, (left_key, left_count)) in entries.iter().enumerate() {
        for (right_key, right_count) in entries.iter().skip(left_offset) {
            if !matches(left_key, right_key) {
                continue;
            }
            if left_key == right_key {
                total += raw_pair_estimate(*left_count);
            } else {
                total += left_count * right_count;
            }
        }
    }
    total
}

fn raw_pair_estimate(count: usize) -> usize {
    count.saturating_mul(count.saturating_sub(1)) / 2
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

fn should_project_candidate(
    candidates: &[AstNearFunctionCandidate],
    left: &NearFact<'_>,
    right: &NearFact<'_>,
    evidence: &candidate::NearCandidateEvidence,
) -> bool {
    if candidates.len() < RUST_FUNCTION_CLONE_NEAR_MAX_CANDIDATES {
        return true;
    }

    candidates.last().is_some_and(|worst| {
        candidate::near_pair_order_against_projected(left, right, evidence, worst) == Ordering::Less
    })
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
