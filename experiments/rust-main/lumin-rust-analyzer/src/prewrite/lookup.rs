use std::cmp::Ordering;
use std::collections::BTreeMap;

use lumin_rust_source_health::protocol::{
    AstDefinitionKind, AstOpaqueSurfaceKind, AstOpaqueSurfaceVisibility, AstVisibility,
    HealthResponse, PathClassification,
};
use serde::Serialize;

use super::index::{Candidate, CandidateIndex, CandidateLane, MatchedField};
use super::intent::{NameDeclaration, NormalizedIntent};
use super::tokens::{
    common_tokens, has_only_weak_common_tokens, is_weak_common_token, unique_tokens,
};

const NEAR_NAME_MAX_LENGTH_DELTA: usize = 2;
const NEAR_NAME_SHARED_PREFIX_MIN: usize = 4;
const NEAR_NAME_MAX_DISTANCE: usize = 2;
const NEAR_NAME_MAX_RESULTS: usize = 5;
const SEMANTIC_HINT_MAX_RESULTS: usize = 5;
const SEMANTIC_HINT_MIN_SCORE: usize = 2;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize)]
pub(super) enum LookupResult {
    #[serde(rename = "NOT_OBSERVED")]
    NotObserved,
    #[serde(rename = "EXISTS")]
    Exists,
    #[serde(rename = "EXISTS_MULTIPLE")]
    ExistsMultiple,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct CandidateRecord {
    pub(super) identity: String,
    pub(super) owner_file: String,
    pub(super) name: String,
    pub(super) matched_field: MatchedField,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) definition_kind: Option<AstDefinitionKind>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) impl_target: Option<String>,
    #[serde(rename = "trait", skip_serializing_if = "Option::is_none")]
    pub(super) trait_path: Option<String>,
    pub(super) visibility: AstVisibility,
    pub(super) line: usize,
    pub(super) column: usize,
    pub(super) policy_excluded: bool,
    pub(super) path_classifications: Vec<PathClassification>,
}

impl CandidateRecord {
    fn from_candidate(candidate: Candidate<'_>) -> Self {
        Self {
            identity: candidate.identity(),
            owner_file: candidate.file.to_string(),
            name: candidate.name.to_string(),
            matched_field: candidate.lane.matched_field(),
            definition_kind: candidate.definition_kind,
            impl_target: candidate.owner.map(|owner| owner.target.to_string()),
            trait_path: candidate
                .owner
                .and_then(|owner| owner.trait_path)
                .map(str::to_string),
            visibility: candidate.visibility,
            line: candidate.location.line,
            column: candidate.location.column,
            policy_excluded: candidate.path.suppressed,
            path_classifications: candidate.classifications(),
        }
    }
}

#[derive(Debug, Clone, Copy, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct Locality {
    pub(super) same_dir: bool,
    pub(super) same_file: bool,
}

impl Locality {
    fn rank(self) -> usize {
        if self.same_file {
            2
        } else if self.same_dir {
            1
        } else {
            0
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct NearNameHint {
    #[serde(flatten)]
    pub(super) candidate: CandidateRecord,
    pub(super) distance: usize,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub(super) matched_tokens: Vec<String>,
    pub(super) locality: Locality,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct SemanticHint {
    #[serde(flatten)]
    pub(super) candidate: CandidateRecord,
    pub(super) matched_tokens: Vec<String>,
    pub(super) matched_name_tokens: Vec<String>,
    pub(super) matched_support_tokens: Vec<String>,
    pub(super) score: usize,
    pub(super) locality: Locality,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(super) enum SuppressionReason {
    DomainTokenOverlap,
    NearLengthDeltaExceeded,
    NearPrefixMismatch,
    NearDistanceExceeded,
    SingleNonWeakTokenOnly,
    InsufficientNonWeakSupport,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct SuppressedNearNameHint {
    #[serde(flatten)]
    pub(super) candidate: CandidateRecord,
    pub(super) reason: SuppressionReason,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub(super) matched_tokens: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) distance: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) length_delta: Option<usize>,
    pub(super) locality: Locality,
    pub(super) candidate_count: usize,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct SuppressedSemanticHint {
    #[serde(flatten)]
    pub(super) candidate: CandidateRecord,
    pub(super) reason: SuppressionReason,
    pub(super) matched_tokens: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub(super) matched_name_tokens: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub(super) matched_support_tokens: Vec<String>,
    pub(super) score: usize,
    pub(super) locality: Locality,
    pub(super) candidate_count: usize,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct TaintSummary {
    parse_error_files: usize,
    review_opaque_surfaces: usize,
    review_opaque_surfaces_by_kind: BTreeMap<AstOpaqueSurfaceKind, usize>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct NameLookup {
    pub(super) intent_name: String,
    pub(super) result: LookupResult,
    pub(super) identities: Vec<CandidateRecord>,
    pub(super) intent_tokens: Vec<String>,
    pub(super) near_names: Vec<NearNameHint>,
    pub(super) semantic_hints: Vec<SemanticHint>,
    pub(super) suppressed_near_names: Vec<SuppressedNearNameHint>,
    pub(super) suppressed_near_name_count: usize,
    pub(super) suppressed_semantic_hints: Vec<SuppressedSemanticHint>,
    pub(super) suppressed_semantic_hint_count: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) tainted_by: Option<TaintSummary>,
    pub(super) citations: Vec<String>,
}

pub(super) fn lookup_names(
    intent: &NormalizedIntent,
    index: &CandidateIndex<'_>,
    syntax: &HealthResponse,
) -> Vec<NameLookup> {
    intent
        .names
        .iter()
        .map(|name| lookup_name(name, intent.declaration_for(name), index, syntax))
        .collect()
}

fn lookup_name(
    intent_name: &str,
    declaration: Option<&NameDeclaration>,
    index: &CandidateIndex<'_>,
    syntax: &HealthResponse,
) -> NameLookup {
    let identities = index
        .candidates
        .iter()
        .copied()
        .filter(|candidate| {
            candidate.lane == CandidateLane::Definition && candidate.name == intent_name
        })
        .map(CandidateRecord::from_candidate)
        .collect::<Vec<_>>();
    let result = match identities.len() {
        0 => LookupResult::NotObserved,
        1 => LookupResult::Exists,
        _ => LookupResult::ExistsMultiple,
    };
    let owner_file = declaration.and_then(NameDeclaration::effective_owner_file);
    let (near_names, suppressed_near_names, suppressed_near_name_count) = if identities.is_empty() {
        near_name_candidates(intent_name, owner_file, &index.candidates)
    } else {
        (Vec::new(), Vec::new(), 0)
    };
    let (intent_tokens, semantic_hints, suppressed_semantic_hints, suppressed_semantic_hint_count) =
        if identities.is_empty() {
            semantic_hint_candidates(intent_name, declaration, &index.candidates)
        } else {
            (
                query_tokens(intent_name, declaration),
                Vec::new(),
                Vec::new(),
                0,
            )
        };

    let mut citations = identities
        .iter()
        .map(|identity| {
            format!(
                "[grounded, rust-source-health.files['{}'].ast.definitions contains '{}' at line {}]",
                identity.owner_file, identity.name, identity.line
            )
        })
        .collect::<Vec<_>>();
    if !near_names.is_empty() {
        citations.push(
            "[degraded, fuzzy-name match; source: Rust AST definition/impl-method scan; search hint only]"
                .to_string(),
        );
    }
    if !semantic_hints.is_empty() {
        citations.push(
            "[degraded, intent-token match; source: Rust AST owner/name tokens; search hint only]"
                .to_string(),
        );
    }
    if identities.is_empty() && near_names.is_empty() && semantic_hints.is_empty() {
        citations.push(format!(
            "[확인 불가, Rust AST scan did not observe '{intent_name}'; this is not an absence claim]"
        ));
    }

    NameLookup {
        intent_name: intent_name.to_string(),
        result,
        identities,
        intent_tokens,
        near_names,
        semantic_hints,
        suppressed_near_names,
        suppressed_near_name_count,
        suppressed_semantic_hints,
        suppressed_semantic_hint_count,
        tainted_by: (result == LookupResult::NotObserved)
            .then(|| taint_summary(syntax))
            .flatten(),
        citations,
    }
}

fn near_name_candidates(
    intent_name: &str,
    owner_file: Option<&str>,
    candidates: &[Candidate<'_>],
) -> (Vec<NearNameHint>, Vec<SuppressedNearNameHint>, usize) {
    let mut hints = Vec::new();
    let mut suppressed = Vec::new();
    for candidate in candidates.iter().copied() {
        if candidate.name == intent_name && candidate.lane != CandidateLane::ImplMethod {
            continue;
        }
        let matched_tokens = common_tokens(intent_name, candidate.name);
        let has_common_token_signal = !matched_tokens.is_empty();
        let locality = locality(candidate.file, owner_file);
        if has_only_weak_common_tokens(intent_name, candidate.name) {
            suppressed.push(SuppressedNearNameHint {
                candidate: CandidateRecord::from_candidate(candidate),
                reason: SuppressionReason::DomainTokenOverlap,
                matched_tokens,
                distance: None,
                length_delta: None,
                locality,
                candidate_count: 0,
            });
            continue;
        }

        let prefix = shared_prefix(candidate.name, intent_name);
        if prefix >= NEAR_NAME_SHARED_PREFIX_MIN
            && candidate
                .name
                .chars()
                .count()
                .abs_diff(intent_name.chars().count())
                <= intent_name.chars().count()
        {
            hints.push(NearNameHint {
                candidate: CandidateRecord::from_candidate(candidate),
                distance: levenshtein_capped(
                    candidate.name,
                    intent_name,
                    NEAR_NAME_MAX_DISTANCE * 4,
                ),
                matched_tokens,
                locality,
            });
            continue;
        }

        let length_delta = candidate
            .name
            .chars()
            .count()
            .abs_diff(intent_name.chars().count());
        if length_delta > NEAR_NAME_MAX_LENGTH_DELTA {
            if has_common_token_signal || prefix >= NEAR_NAME_SHARED_PREFIX_MIN {
                suppressed.push(SuppressedNearNameHint {
                    candidate: CandidateRecord::from_candidate(candidate),
                    reason: SuppressionReason::NearLengthDeltaExceeded,
                    matched_tokens,
                    distance: None,
                    length_delta: Some(length_delta),
                    locality,
                    candidate_count: 0,
                });
            }
            continue;
        }

        let distance = levenshtein_capped(candidate.name, intent_name, NEAR_NAME_MAX_DISTANCE);
        if distance <= NEAR_NAME_MAX_DISTANCE {
            hints.push(NearNameHint {
                candidate: CandidateRecord::from_candidate(candidate),
                distance,
                matched_tokens,
                locality,
            });
        } else if has_common_token_signal || prefix >= NEAR_NAME_SHARED_PREFIX_MIN {
            suppressed.push(SuppressedNearNameHint {
                candidate: CandidateRecord::from_candidate(candidate),
                reason: if prefix < NEAR_NAME_SHARED_PREFIX_MIN && !has_common_token_signal {
                    SuppressionReason::NearPrefixMismatch
                } else {
                    SuppressionReason::NearDistanceExceeded
                },
                matched_tokens,
                distance: Some(distance),
                length_delta: None,
                locality,
                candidate_count: 0,
            });
        }
    }

    hints.sort_by(|left, right| {
        left.distance
            .cmp(&right.distance)
            .then(
                lane_rank(left.candidate.matched_field)
                    .cmp(&lane_rank(right.candidate.matched_field)),
            )
            .then(left.candidate.name.cmp(&right.candidate.name))
            .then(left.candidate.owner_file.cmp(&right.candidate.owner_file))
    });
    suppressed.sort_by(suppressed_near_order);
    let suppressed_count = suppressed.len();
    for hint in &mut suppressed {
        hint.candidate_count = suppressed_count;
    }
    hints.truncate(NEAR_NAME_MAX_RESULTS);
    suppressed.truncate(NEAR_NAME_MAX_RESULTS);
    (hints, suppressed, suppressed_count)
}

fn semantic_hint_candidates(
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

fn query_tokens(intent_name: &str, declaration: Option<&NameDeclaration>) -> Vec<String> {
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

fn locality(candidate_file: &str, intent_owner_file: Option<&str>) -> Locality {
    let Some(intent_owner_file) = intent_owner_file else {
        return Locality::default();
    };
    let intent_owner_file = intent_owner_file.replace('\\', "/");
    Locality {
        same_file: candidate_file == intent_owner_file,
        same_dir: dirname(candidate_file) == dirname(&intent_owner_file),
    }
}

fn dirname(path: &str) -> &str {
    path.rsplit_once('/')
        .map(|(directory, _)| directory)
        .unwrap_or("")
}

fn lane_rank(field: MatchedField) -> usize {
    match field {
        MatchedField::ImplMethodIndex => 0,
        MatchedField::DefIndex => 1,
    }
}

fn suppressed_near_order(
    left: &SuppressedNearNameHint,
    right: &SuppressedNearNameHint,
) -> Ordering {
    right
        .locality
        .rank()
        .cmp(&left.locality.rank())
        .then(
            left.distance
                .unwrap_or(usize::MAX)
                .cmp(&right.distance.unwrap_or(usize::MAX)),
        )
        .then(
            left.length_delta
                .unwrap_or(usize::MAX)
                .cmp(&right.length_delta.unwrap_or(usize::MAX)),
        )
        .then(left.candidate.name.cmp(&right.candidate.name))
        .then(left.candidate.owner_file.cmp(&right.candidate.owner_file))
}

fn shared_prefix(left: &str, right: &str) -> usize {
    left.chars()
        .zip(right.chars())
        .take_while(|(left, right)| left == right)
        .count()
}

fn levenshtein_capped(left: &str, right: &str, cap: usize) -> usize {
    let left = left.chars().collect::<Vec<_>>();
    let right = right.chars().collect::<Vec<_>>();
    if left.len().abs_diff(right.len()) > cap {
        return cap + 1;
    }
    let mut previous = (0..=right.len()).collect::<Vec<_>>();
    let mut current = vec![0; right.len() + 1];
    for (left_index, left_char) in left.iter().enumerate() {
        current[0] = left_index + 1;
        let mut row_minimum = current[0];
        for (right_index, right_char) in right.iter().enumerate() {
            let cost = usize::from(left_char != right_char);
            current[right_index + 1] = (current[right_index] + 1)
                .min(previous[right_index + 1] + 1)
                .min(previous[right_index] + cost);
            row_minimum = row_minimum.min(current[right_index + 1]);
        }
        if row_minimum > cap {
            return cap + 1;
        }
        std::mem::swap(&mut previous, &mut current);
    }
    previous[right.len()].min(cap + 1)
}

fn extend_unique(target: &mut Vec<String>, values: &[String]) {
    for value in values {
        if !target.contains(value) {
            target.push(value.clone());
        }
    }
}

fn taint_summary(syntax: &HealthResponse) -> Option<TaintSummary> {
    let mut by_kind = BTreeMap::new();
    for file in syntax.files.values() {
        for surface in &file.ast.opaque_surfaces {
            if surface.visibility == AstOpaqueSurfaceVisibility::Review {
                *by_kind.entry(surface.kind).or_insert(0) += 1;
            }
        }
    }
    if syntax.summary.parse_error_files == 0 && syntax.summary.review_opaque_surfaces == 0 {
        return None;
    }
    Some(TaintSummary {
        parse_error_files: syntax.summary.parse_error_files,
        review_opaque_surfaces: syntax.summary.review_opaque_surfaces,
        review_opaque_surfaces_by_kind: by_kind,
    })
}
