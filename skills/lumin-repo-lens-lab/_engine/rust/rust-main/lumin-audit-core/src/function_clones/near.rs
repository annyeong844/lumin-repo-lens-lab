mod candidate;
mod model;
mod scoring;
mod tokens;

use serde_json::{json, Value};
use std::collections::{BTreeMap, HashSet};

use super::facts::FunctionFact;
use super::projection::sort_near_candidates;
use model::{
    CandidateGenerationDiagnostics, CompatibilityKey, NearFact, NearFunctionCandidateProjection,
};

const FUNCTION_CLONE_NEAR_POLICY_ID: &str = "function-clone-near-policy";
const FUNCTION_CLONE_NEAR_POLICY_VERSION: &str = "function-clone-near-policy-v1";
const FUNCTION_CLONE_NEAR_POLICY_HASH: &str =
    "sha256:6f524aeaefad2aa07badd8db3b841d2fec22d1228368bc5192387a5ea0116c54";
const FUNCTION_CLONE_NEAR_THRESHOLD_HASH: &str =
    "sha256:bea5f5cd6ce57db1800039b86f54d0ebc8b168b63aafeb3a9fbdc468a241ba29";

const RETRIEVAL_CONTRACT_VERSION: &str = "function-clone-near-retrieval.v1";
const CANDIDATE_GENERATION_MODE: &str = "bounded-retrieval";
const CANDIDATE_COUNT_SCOPE: &str = "scored-candidates-from-retained-retrieval-evidence";
const NEAR_CANDIDATE_COUNT_SCOPE: &str = "bounded-retrieval-retained-evidence";
const PAIR_DEDUPE: &str = "ordered-shared-retained-token";
const PROJECTION_MODE: &str = "streaming-top-n";
const IDF_FORMULA: &str = "ln((functionCount + 1) / (documentFrequency + 1))";
const IDF_SCOPE: &str = "repository-local-function-call-token-document-frequency";
const SCORE_FORMULA_VERSION: &str = "function-clone-near-score-idf-sum-v1";
const RETAINED_PAIR_ESTIMATE_KIND: &str =
    "raw-bucket-pairs-may-double-count-pairs-shared-by-multiple-retained-tokens";
const SKIPPED_PAIR_ESTIMATE_KIND: &str =
    "raw-bucket-pairs-may-double-count-pairs-shared-by-multiple-skipped-tokens";
const COMPATIBILITY_PAIR_ESTIMATE_KIND: &str =
    "raw-partition-estimate-does-not-enumerate-rejected-pairs";

pub(super) const MIN_BODY_LOC_FOR_GROUPING: usize = 3;
pub(super) const MIN_STATEMENTS_FOR_GROUPING: usize = 2;
pub(super) const MIN_GROUP_SIZE: usize = 2;
pub(super) const MAX_PARAM_COUNT_DELTA: i64 = 1;
pub(super) const MIN_BODY_LOC_SIMILARITY: f64 = 0.34;
pub(super) const MIN_STATEMENT_COUNT_SIMILARITY: f64 = 0.34;
pub(super) const MIN_SINGLE_TOKEN_IDF: f64 = 3.0;
pub(super) const CALL_IDF_SATURATION: f64 = 6.0;
pub(super) const MIN_CALL_TOKEN_IDF_SCORE: f64 = 0.5;
pub(super) const MIN_NAME_TOKEN_JACCARD_FALLBACK: f64 = 0.34;
pub(super) const MIN_NEAR_SCORE: f64 = 0.62;
pub(super) const MAX_NEAR_CANDIDATES: usize = 50;
const SKIPPED_BUCKET_SAMPLE_LIMIT: usize = MAX_NEAR_CANDIDATES;
pub(super) const CALL_TOKEN_IDF_SCORE_WEIGHT: f64 = 0.45;
pub(super) const NAME_TOKEN_JACCARD_WEIGHT: f64 = 0.25;
pub(super) const BODY_LOC_SIMILARITY_WEIGHT: f64 = 0.15;
pub(super) const STATEMENT_COUNT_SIMILARITY_WEIGHT: f64 = 0.15;

pub(super) fn build_near_function_candidates(
    facts: &[FunctionFact],
    exact_body_groups: &[Value],
    structure_groups: &[Value],
) -> NearFunctionCandidateProjection {
    let grouped = grouped_identity_set([exact_body_groups, structure_groups]);
    let mut all_facts = facts
        .iter()
        .map(|fact| NearFact {
            fact,
            significant_call_tokens: tokens::significant_call_tokens(fact),
            retained_call_tokens: Vec::new(),
            name_tokens: tokens::name_tokens(&fact.exported_name),
        })
        .collect::<Vec<_>>();
    let token_idfs = scoring::call_token_idfs(&all_facts);
    for fact in &mut all_facts {
        fact.retained_call_tokens = fact
            .significant_call_tokens
            .iter()
            .filter(|token| scoring::token_idf(token, &token_idfs) >= MIN_SINGLE_TOKEN_IDF)
            .copied()
            .collect();
    }

    let mut eligible = all_facts
        .drain(..)
        .filter(|fact| !grouped.contains(&fact.fact.identity))
        .filter(|fact| !fact.significant_call_tokens.is_empty())
        .filter(|fact| !fact.fact.generator)
        .collect::<Vec<_>>();
    eligible.sort_by(|left, right| left.fact.identity.cmp(&right.fact.identity));
    let eligible_function_count = eligible.len();

    let mut call_token_counts = BTreeMap::<&str, usize>::new();
    for fact in &eligible {
        for token in &fact.significant_call_tokens {
            *call_token_counts.entry(*token).or_default() += 1;
        }
    }

    let mut skipped_low_discrimination_buckets = call_token_counts
        .iter()
        .filter_map(|(token, posting_count)| {
            let idf = scoring::token_idf(token, &token_idfs);
            let raw_pair_estimate = raw_pair_estimate(*posting_count);
            if idf >= MIN_SINGLE_TOKEN_IDF || raw_pair_estimate == 0 {
                return None;
            }
            Some(model::SkippedLowDiscriminationBucket {
                token: (*token).to_string(),
                idf: scoring::round_score(idf),
                posting_count: *posting_count,
                raw_pair_estimate,
            })
        })
        .collect::<Vec<_>>();
    let skipped_low_discrimination_bucket_count = skipped_low_discrimination_buckets.len();
    let skipped_low_discrimination_raw_pair_estimate = skipped_low_discrimination_buckets
        .iter()
        .fold(0usize, |total, bucket| {
            total.saturating_add(bucket.raw_pair_estimate)
        });
    skipped_low_discrimination_buckets.sort_by(|left, right| {
        right
            .raw_pair_estimate
            .cmp(&left.raw_pair_estimate)
            .then_with(|| left.token.cmp(&right.token))
    });
    skipped_low_discrimination_buckets.truncate(SKIPPED_BUCKET_SAMPLE_LIMIT);

    let mut retained = BTreeMap::<&str, BTreeMap<CompatibilityKey, Vec<usize>>>::new();
    for (index, fact) in eligible.iter().enumerate() {
        let key = compatibility_key(fact);
        for token in &fact.retained_call_tokens {
            retained
                .entry(*token)
                .or_default()
                .entry(key.clone())
                .or_default()
                .push(index);
        }
    }
    retained.retain(|_, postings| postings.values().map(Vec::len).sum::<usize>() >= 2);

    let mut diagnostics = CandidateGenerationDiagnostics {
        eligible_function_count,
        retained_call_token_bucket_count: retained.len(),
        skipped_low_discrimination_buckets,
        skipped_low_discrimination_bucket_count,
        skipped_low_discrimination_raw_pair_estimate,
        ..CandidateGenerationDiagnostics::default()
    };
    let mut review_visible_count = 0usize;
    let mut candidates = Vec::<Value>::new();
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
            for (left_offset, (left_key, left_bucket)) in posting_entries.iter().enumerate() {
                for (right_key, right_bucket) in posting_entries.iter().skip(left_offset) {
                    if !compatibility_keys_match(left_key, right_key) {
                        continue;
                    }
                    if left_key == right_key {
                        for (left_index_offset, left_index) in left_bucket.iter().enumerate() {
                            for right_index in left_bucket.iter().skip(left_index_offset + 1) {
                                generation.generate(*left_index, *right_index, call_token);
                            }
                        }
                    } else {
                        for left_index in left_bucket.iter() {
                            for right_index in right_bucket.iter() {
                                generation.generate(*left_index, *right_index, call_token);
                            }
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
    token_idfs: &'state BTreeMap<&'facts str, f64>,
    diagnostics: &'state mut CandidateGenerationDiagnostics,
    review_visible_count: &'state mut usize,
    candidates: &'state mut Vec<Value>,
}

impl<'facts, 'state> CandidateGenerationState<'facts, 'state> {
    fn generate(&mut self, left_index: usize, right_index: usize, call_token: &str) {
        let left = &self.eligible[left_index];
        let right = &self.eligible[right_index];
        if has_earlier_shared_call_token(
            &left.retained_call_tokens,
            &right.retained_call_tokens,
            call_token,
        ) {
            return;
        }

        self.diagnostics.generated_unique_pair_count = self
            .diagnostics
            .generated_unique_pair_count
            .saturating_add(1);
        let evidence = match candidate::near_candidate_evidence_from_pair(
            left,
            right,
            self.token_idfs,
            call_token,
        ) {
            candidate::NearCandidateEvaluation::RejectedBeforeScoring => return,
            candidate::NearCandidateEvaluation::Scored(evidence) => {
                self.diagnostics.scored_pair_count =
                    self.diagnostics.scored_pair_count.saturating_add(1);
                let Some(evidence) = evidence else {
                    return;
                };
                evidence
            }
        };
        if !evidence.generated_only {
            *self.review_visible_count = (*self.review_visible_count).saturating_add(1);
        }
        let candidate = candidate::build_near_candidate_from_evidence(left, right, evidence);
        self.candidates.push(candidate);
        sort_near_candidates(self.candidates);
        self.candidates.truncate(MAX_NEAR_CANDIDATES);
    }
}

fn compatibility_key(fact: &NearFact<'_>) -> CompatibilityKey {
    CompatibilityKey {
        async_value: fact.fact.async_value,
        param_count: fact.fact.param_count,
        body_loc_band: range_band(fact.fact.body_loc, MIN_BODY_LOC_SIMILARITY),
        statement_count_band: range_band(fact.fact.statement_count, MIN_STATEMENT_COUNT_SIMILARITY),
    }
}

fn range_band(value: i64, min_similarity: f64) -> usize {
    if value <= 0 {
        return 0;
    }
    let base = 1.0 / min_similarity;
    ((value as f64).ln() / base.ln()).floor() as usize + 1
}

fn compatibility_keys_match(left: &CompatibilityKey, right: &CompatibilityKey) -> bool {
    left.async_value == right.async_value
        && (left.param_count - right.param_count).abs() <= MAX_PARAM_COUNT_DELTA
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
    let total_count = entries
        .iter()
        .fold(0usize, |total, (_, count)| total.saturating_add(*count));
    let total = raw_pair_estimate(total_count);
    let async_compatible = estimate_pairs_matching(&entries, |left, right| {
        left.async_value == right.async_value
    });
    let parameter_compatible = estimate_pairs_matching(&entries, |left, right| {
        left.async_value == right.async_value
            && (left.param_count - right.param_count).abs() <= MAX_PARAM_COUNT_DELTA
    });
    let body_compatible = estimate_pairs_matching(&entries, |left, right| {
        left.async_value == right.async_value
            && (left.param_count - right.param_count).abs() <= MAX_PARAM_COUNT_DELTA
            && bands_compatible(left.body_loc_band, right.body_loc_band)
    });
    let statement_compatible = estimate_pairs_matching(&entries, compatibility_keys_match);

    diagnostics.retained_raw_pair_estimate =
        diagnostics.retained_raw_pair_estimate.saturating_add(total);
    let estimates = &mut diagnostics.compatibility_skipped_raw_pair_estimate_by_reason;
    estimates.async_mismatch = estimates
        .async_mismatch
        .saturating_add(total.saturating_sub(async_compatible));
    estimates.parameter_count_delta = estimates
        .parameter_count_delta
        .saturating_add(async_compatible.saturating_sub(parameter_compatible));
    estimates.body_loc_band_mismatch = estimates
        .body_loc_band_mismatch
        .saturating_add(parameter_compatible.saturating_sub(body_compatible));
    estimates.statement_count_band_mismatch = estimates
        .statement_count_band_mismatch
        .saturating_add(body_compatible.saturating_sub(statement_compatible));
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
            let count = if left_key == right_key {
                raw_pair_estimate(*left_count)
            } else {
                left_count.saturating_mul(*right_count)
            };
            total = total.saturating_add(count);
        }
    }
    total
}

fn raw_pair_estimate(count: usize) -> usize {
    count.saturating_mul(count.saturating_sub(1)) / 2
}

fn has_earlier_shared_call_token(left: &[&str], right: &[&str], current_token: &str) -> bool {
    left.iter()
        .take_while(|token| **token < current_token)
        .any(|token| right.binary_search(token).is_ok())
}

fn grouped_identity_set<'a>(
    groups_lists: impl IntoIterator<Item = &'a [Value]>,
) -> HashSet<String> {
    let mut out = HashSet::new();
    for groups in groups_lists {
        for group in groups {
            if let Some(identities) = group.get("identities").and_then(Value::as_array) {
                for identity in identities {
                    if let Some(identity) = identity.as_str() {
                        out.insert(identity.to_string());
                    }
                }
            }
        }
    }
    out
}

pub(super) fn candidate_generation_policy() -> Value {
    json!({
        "mode": CANDIDATE_GENERATION_MODE,
        "retrievalContractVersion": RETRIEVAL_CONTRACT_VERSION,
        "idfFormula": IDF_FORMULA,
        "idfScope": IDF_SCOPE,
        "bucketMinIdf": MIN_SINGLE_TOKEN_IDF,
        "callIdfSaturation": CALL_IDF_SATURATION,
        "scoreFormulaVersion": SCORE_FORMULA_VERSION,
        "candidateCountScope": CANDIDATE_COUNT_SCOPE,
        "pairDedupe": PAIR_DEDUPE,
        "projection": PROJECTION_MODE,
        "skippedLowDiscriminationBucketSampleLimit": SKIPPED_BUCKET_SAMPLE_LIMIT,
    })
}

pub(super) fn candidate_generation_summary(diagnostics: &CandidateGenerationDiagnostics) -> Value {
    json!({
        "eligibleFunctionCount": diagnostics.eligible_function_count,
        "retainedCallTokenBucketCount": diagnostics.retained_call_token_bucket_count,
        "retainedRawPairEstimate": diagnostics.retained_raw_pair_estimate,
        "retainedRawPairEstimateKind": RETAINED_PAIR_ESTIMATE_KIND,
        "generatedUniquePairCount": diagnostics.generated_unique_pair_count,
        "scoredPairCount": diagnostics.scored_pair_count,
        "compatibilitySkippedRawPairEstimateByReason": {
            "asyncMismatch": diagnostics.compatibility_skipped_raw_pair_estimate_by_reason.async_mismatch,
            "parameterCountDelta": diagnostics.compatibility_skipped_raw_pair_estimate_by_reason.parameter_count_delta,
            "bodyLocBandMismatch": diagnostics.compatibility_skipped_raw_pair_estimate_by_reason.body_loc_band_mismatch,
            "statementCountBandMismatch": diagnostics.compatibility_skipped_raw_pair_estimate_by_reason.statement_count_band_mismatch,
        },
        "compatibilitySkippedPairEstimateKind": COMPATIBILITY_PAIR_ESTIMATE_KIND,
        "nearFunctionCandidateCountScope": NEAR_CANDIDATE_COUNT_SCOPE,
    })
}

pub(super) fn skipped_low_discrimination_buckets(
    diagnostics: &CandidateGenerationDiagnostics,
) -> Vec<Value> {
    diagnostics
        .skipped_low_discrimination_buckets
        .iter()
        .map(|bucket| {
            json!({
                "token": bucket.token,
                "idf": bucket.idf,
                "postingCount": bucket.posting_count,
                "rawPairEstimate": bucket.raw_pair_estimate,
                "reason": "below-min-single-token-idf",
            })
        })
        .collect()
}

pub(super) fn skipped_pair_estimate_kind() -> &'static str {
    SKIPPED_PAIR_ESTIMATE_KIND
}

pub(super) fn function_clone_near_policy_summary() -> Value {
    json!({
        "schemaVersion": "threshold-policy.v1",
        "policyId": FUNCTION_CLONE_NEAR_POLICY_ID,
        "policyVersion": FUNCTION_CLONE_NEAR_POLICY_VERSION,
        "policyClass": "review",
        "policyHash": FUNCTION_CLONE_NEAR_POLICY_HASH,
        "thresholdHash": FUNCTION_CLONE_NEAR_THRESHOLD_HASH,
        "thresholds": {
            "minBodyLocForGrouping": MIN_BODY_LOC_FOR_GROUPING,
            "minStatementsForGrouping": MIN_STATEMENTS_FOR_GROUPING,
            "minGroupSize": MIN_GROUP_SIZE,
            "maxParamCountDelta": MAX_PARAM_COUNT_DELTA,
            "minBodyLocSimilarity": MIN_BODY_LOC_SIMILARITY,
            "minStatementCountSimilarity": MIN_STATEMENT_COUNT_SIMILARITY,
            "minSingleTokenIdf": MIN_SINGLE_TOKEN_IDF,
            "callIdfSaturation": CALL_IDF_SATURATION,
            "minCallTokenIdfScore": MIN_CALL_TOKEN_IDF_SCORE,
            "minNameTokenJaccardFallback": MIN_NAME_TOKEN_JACCARD_FALLBACK,
            "minNearScore": MIN_NEAR_SCORE,
            "maxNearCandidates": MAX_NEAR_CANDIDATES,
            "weights": {
                "callTokenIdfScore": CALL_TOKEN_IDF_SCORE_WEIGHT,
                "nameTokenJaccard": NAME_TOKEN_JACCARD_WEIGHT,
                "bodyLocSimilarity": BODY_LOC_SIMILARITY_WEIGHT,
                "statementCountSimilarity": STATEMENT_COUNT_SIMILARITY_WEIGHT,
            },
        },
        "retrievalContractVersion": RETRIEVAL_CONTRACT_VERSION,
        "candidateGenerationMode": CANDIDATE_GENERATION_MODE,
        "candidateCountScope": CANDIDATE_COUNT_SCOPE,
        "idfFormula": IDF_FORMULA,
        "idfScope": IDF_SCOPE,
        "pairDedupe": PAIR_DEDUPE,
        "projection": PROJECTION_MODE,
        "skippedLowDiscriminationBucketSampleLimit": SKIPPED_BUCKET_SAMPLE_LIMIT,
        "scoreFormulaVersion": SCORE_FORMULA_VERSION,
        "scoreCalibration": {
            "callTokenComponent": "shared-idf-sum-saturated",
            "previousCallTokenComponent": "jaccard",
            "callIdfSaturation": CALL_IDF_SATURATION,
            "thresholdCompatibility": "threshold-number-retained-but-call-component-changed",
        },
        "calibration": {
            "corpus": "calibration-2026-05-prewrite-v1",
            "note": "bounded JS/TS near-function retrieval calibration",
        },
        "calibrationCorpus": {
            "schemaVersion": "calibration-corpus.v1",
            "corpusId": "calibration-2026-05-prewrite-v1",
            "purpose": "pre-write cue and threshold calibration",
            "status": "registry-anchor",
            "metrics": [
                "precisionProxy",
                "noiseRate",
                "runtimeMs",
                "suppressedCueRate",
            ],
            "entryCount": 3,
        },
    })
}
