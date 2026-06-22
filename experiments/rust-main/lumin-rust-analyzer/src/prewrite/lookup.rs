use std::collections::BTreeMap;

use lumin_rust_source_health::protocol::{
    AstDefinitionKind, AstOpaqueSurfaceKind, AstOpaqueSurfaceVisibility, AstVisibility,
    HealthResponse, PathClassification,
};
use serde::Serialize;

use super::index::{Candidate, CandidateIndex, CandidateLane, MatchedField};
use super::intent::{NameDeclaration, NormalizedIntent};

mod near;
mod semantic;

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
        let path_classifications = candidate.classifications();
        let policy_excluded = candidate.path.suppressed
            || path_classifications.iter().any(|classification| {
                matches!(
                    classification,
                    PathClassification::Test | PathClassification::Generated
                )
            });
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
            policy_excluded,
            path_classifications,
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
        near::near_name_candidates(intent_name, owner_file, &index.candidates)
    } else {
        (Vec::new(), Vec::new(), 0)
    };
    let (intent_tokens, semantic_hints, suppressed_semantic_hints, suppressed_semantic_hint_count) =
        if identities.is_empty() {
            semantic::semantic_hint_candidates(intent_name, declaration, &index.candidates)
        } else {
            (
                semantic::query_tokens(intent_name, declaration),
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
