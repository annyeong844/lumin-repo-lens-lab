use std::path::Path;

use anyhow::{bail, Result};
use lumin_rust_common::posix_path_text;
use lumin_rust_source_health::protocol::{
    HealthResponse, ParserMeta, PolicyMeta, RuntimeMeta, SidecarMeta,
};
use serde::Serialize;

use super::cues::{
    self, CueCard, CueMatchedField, CueProjection, CueTier, EvidenceLane, SuppressedCue,
};
use super::index::CandidateIndex;
use super::intent::{IntentWarning, LoadedIntent, NormalizedIntent};
use super::lookup::{
    self, DependencyLookup, FileLookup, InlinePatternLookup, NameLookup, ShapeLookup,
    UnavailableEvidence,
};
use super::tokens::{TOKENIZER_VERSION, TOKEN_POLICY_VERSION, WEAK_COMMON_TOKENS};

const SCHEMA_VERSION: &str = "rust-pre-write.v1";
const LOOKUP_POLICY_JS_TS_PRECEDENT: &[&str] = &[
    "_lib/pre-write-intent.mjs",
    "_lib/pre-write-cue-tiers.mjs",
    "_lib/pre-write-lookup-name.mjs",
    "_lib/pre-write-lookup-file.mjs",
    "_lib/pre-write-lookup-shape.mjs",
    "_lib/pre-write-lookup-dep.mjs",
    "_lib/pre-write-lookup-inline-patterns.mjs",
];

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct PreWriteArtifact {
    schema_version: &'static str,
    policy_version: &'static str,
    meta: PreWriteMeta,
    intent: NormalizedIntent,
    intent_warnings: Vec<IntentWarning>,
    coverage: IntentLaneCoverage,
    lookups: Vec<NameLookup>,
    shape_lookups: Vec<ShapeLookup>,
    file_lookups: Vec<FileLookup>,
    dependency_lookups: Vec<DependencyLookup>,
    inline_pattern_lookups: Vec<InlinePatternLookup>,
    cue_cards: Vec<CueCard>,
    suppressed_cues: Vec<SuppressedCue>,
    unavailable_evidence: Vec<UnavailableEvidence>,
}

impl PreWriteArtifact {
    pub(crate) fn validate_contract(&self) -> Result<()> {
        for card in &self.cue_cards {
            for cue in &card.cues {
                if cue.cue_tier == CueTier::Safe {
                    match cue.evidence_lane {
                        EvidenceLane::ExactSymbol
                            if cue
                                .evidence
                                .iter()
                                .all(|evidence| matches!(
                                    evidence.matched_field,
                                    CueMatchedField::DefIndex
                                        | CueMatchedField::RustSourceHealthUseTrees
                                )) => {}
                        EvidenceLane::ExactFile
                            if cue.evidence.iter().all(|evidence| {
                                evidence.matched_field == CueMatchedField::RustSourceHealthFiles
                            }) => {}
                        EvidenceLane::ShapeHash
                            if cue.evidence.iter().all(|evidence| {
                                evidence.matched_field
                                    == CueMatchedField::RustSourceHealthShapeHash
                                    && evidence.hash.is_some()
                            }) => {}
                        _ => bail!(
                            "blocked-artifact-contract: SAFE cue {} is not exact source-health evidence",
                            card.candidate.identity
                        ),
                    }
                }
                if cue.evidence.iter().any(|evidence| {
                    evidence.matched_field == CueMatchedField::ImplMethodIndex
                        && cue.cue_tier == CueTier::Safe
                }) {
                    bail!(
                        "blocked-artifact-contract: impl method {} entered SAFE",
                        card.candidate.identity
                    );
                }
            }
        }

        for cue in &self.suppressed_cues {
            if cue.reason == cues::MutedReason::PolicyExcluded
                && (cue.original_cue_tier.is_none() || cue.path_classifications.is_empty())
            {
                bail!(
                    "blocked-artifact-contract: policy-muted cue {} lost original tier or path evidence",
                    cue.candidate.identity
                );
            }
        }

        for lookup in &self.lookups {
            if !self.intent.names.contains(&lookup.intent_name) {
                bail!(
                    "blocked-artifact-contract: lookup name {} is absent from normalized intent",
                    lookup.intent_name
                );
            }
        }
        if self.shape_lookups.len() != self.intent.shapes.len() {
            bail!("blocked-artifact-contract: shape lookup count drifted from normalized intent");
        }
        for (lookup, intent_shape) in self.shape_lookups.iter().zip(&self.intent.shapes) {
            if &lookup.shape != intent_shape {
                bail!("blocked-artifact-contract: shape lookup drifted from normalized intent");
            }
            if !lookup.is_unavailable() && !lookup.is_shape_match() {
                bail!("blocked-artifact-contract: shape lookup emitted an invalid result");
            }
        }
        let unavailable_lookup_count = self
            .shape_lookups
            .iter()
            .filter(|lookup| lookup.is_unavailable())
            .count()
            + self
                .inline_pattern_lookups
                .iter()
                .filter(|lookup| lookup.is_unavailable())
                .count();
        if self.unavailable_evidence.len() != unavailable_lookup_count {
            bail!(
                "blocked-artifact-contract: unavailable evidence drifted from unavailable lookups"
            );
        }
        for lookup in &self.file_lookups {
            if !intent_has_file(&self.intent, &lookup.intent_file) {
                bail!(
                    "blocked-artifact-contract: lookup file {} is absent from normalized intent",
                    lookup.intent_file
                );
            }
        }
        if self.dependency_lookups.len() != self.intent.dependencies.len() {
            bail!(
                "blocked-artifact-contract: dependency lookup count drifted from normalized intent"
            );
        }
        for (lookup, dependency) in self
            .dependency_lookups
            .iter()
            .zip(&self.intent.dependencies)
        {
            if &lookup.dep_name != dependency {
                bail!(
                    "blocked-artifact-contract: dependency lookup {} drifted from normalized intent",
                    lookup.dep_name
                );
            }
        }
        let expected_inline_lookups = usize::from(self.intent.has_refactor_sources());
        if self.inline_pattern_lookups.len() != expected_inline_lookups {
            bail!(
                "blocked-artifact-contract: inline-pattern lookup count drifted from refactorSources"
            );
        }
        for lookup in &self.inline_pattern_lookups {
            if !lookup.is_unavailable() {
                bail!(
                    "blocked-artifact-contract: unsupported Rust inline-pattern lane emitted a match"
                );
            }
        }

        self.coverage.validate(&self.intent)?;
        Ok(())
    }

    pub(crate) fn to_pretty_string(&self) -> serde_json::Result<String> {
        serde_json::to_string_pretty(self)
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct PreWriteMeta {
    producer: PreWriteProducer,
    source_health: SourceHealthProvenance,
    token_policy: TokenPolicyMeta,
    lookup_policy: LookupPolicyMeta,
}

impl PreWriteMeta {
    fn from_syntax(syntax: &HealthResponse) -> Self {
        Self {
            producer: PreWriteProducer::LuminRustAnalyzer,
            source_health: SourceHealthProvenance {
                schema_version: syntax.schema_version,
                parser: syntax.meta.parser.clone(),
                policy: syntax.meta.policy.clone(),
                runtime: syntax.meta.runtime.clone(),
                sidecar: syntax.meta.sidecar.clone(),
            },
            token_policy: TokenPolicyMeta {
                tokenizer_version: TOKENIZER_VERSION,
                token_policy_version: TOKEN_POLICY_VERSION,
                weak_common_tokens: &WEAK_COMMON_TOKENS,
            },
            lookup_policy: LookupPolicyMeta::from_constants(),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize)]
enum PreWriteProducer {
    #[serde(rename = "lumin-rust-analyzer")]
    LuminRustAnalyzer,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct SourceHealthProvenance {
    schema_version: u32,
    parser: ParserMeta,
    policy: PolicyMeta,
    runtime: RuntimeMeta,
    #[serde(skip_serializing_if = "Option::is_none")]
    sidecar: Option<SidecarMeta>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct TokenPolicyMeta {
    tokenizer_version: &'static str,
    token_policy_version: &'static str,
    weak_common_tokens: &'static [&'static str],
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct LookupPolicyMeta {
    js_ts_precedent: &'static [&'static str],
    near_name: NearNameLookupPolicyMeta,
    semantic_hint: SemanticHintLookupPolicyMeta,
    service_operation_sibling: OperationSiblingPolicyMeta,
    local_operation_sibling: OperationSiblingPolicyMeta,
    file_domain_cluster: FileDomainClusterPolicyMeta,
    dependency_hub: DependencyHubPolicyMeta,
}

impl LookupPolicyMeta {
    fn from_constants() -> Self {
        Self {
            js_ts_precedent: LOOKUP_POLICY_JS_TS_PRECEDENT,
            near_name: NearNameLookupPolicyMeta {
                max_length_delta: lookup::NEAR_NAME_MAX_LENGTH_DELTA,
                shared_prefix_min: lookup::NEAR_NAME_SHARED_PREFIX_MIN,
                max_distance: lookup::NEAR_NAME_MAX_DISTANCE,
                max_results: lookup::NEAR_NAME_MAX_RESULTS,
            },
            semantic_hint: SemanticHintLookupPolicyMeta {
                min_score: lookup::SEMANTIC_HINT_MIN_SCORE,
                max_results: lookup::SEMANTIC_HINT_MAX_RESULTS,
            },
            service_operation_sibling: OperationSiblingPolicyMeta {
                policy_id: lookup::SERVICE_OPERATION_POLICY_ID,
                policy_version: lookup::SERVICE_OPERATION_POLICY_VERSION,
                max_results: lookup::SERVICE_OPERATION_POLICY_MAX_RESULTS,
            },
            local_operation_sibling: OperationSiblingPolicyMeta {
                policy_id: lookup::LOCAL_OPERATION_POLICY_ID,
                policy_version: lookup::LOCAL_OPERATION_POLICY_VERSION,
                max_results: lookup::LOCAL_OPERATION_POLICY_MAX_RESULTS,
            },
            file_domain_cluster: FileDomainClusterPolicyMeta {
                min_matches: lookup::DOMAIN_CLUSTER_MIN_MATCHES,
                max_examples: lookup::DOMAIN_CLUSTER_MAX_EXAMPLES,
                min_prefix_len: lookup::DOMAIN_CLUSTER_MIN_PREFIX_LEN,
            },
            dependency_hub: DependencyHubPolicyMeta {
                example_limit: lookup::DEPENDENCY_EXAMPLE_LIMIT,
                watch_for_threshold: lookup::DEPENDENCY_WATCH_FOR_THRESHOLD,
            },
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct NearNameLookupPolicyMeta {
    max_length_delta: usize,
    shared_prefix_min: usize,
    max_distance: usize,
    max_results: usize,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct SemanticHintLookupPolicyMeta {
    min_score: usize,
    max_results: usize,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct OperationSiblingPolicyMeta {
    policy_id: &'static str,
    policy_version: &'static str,
    max_results: usize,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct FileDomainClusterPolicyMeta {
    min_matches: usize,
    max_examples: usize,
    min_prefix_len: usize,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct DependencyHubPolicyMeta {
    example_limit: usize,
    watch_for_threshold: usize,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
enum LaneStatus {
    Ran,
    Unsupported,
    NotRequested,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct IntentLaneCoverage {
    names: LaneStatus,
    shapes: LaneStatus,
    files: LaneStatus,
    dependencies: LaneStatus,
    inline_patterns: LaneStatus,
    planned_type_escapes: LaneStatus,
}

impl IntentLaneCoverage {
    fn from_intent(intent: &NormalizedIntent) -> Self {
        Self {
            names: LaneStatus::Ran,
            shapes: ran_if_requested(!intent.shapes.is_empty()),
            files: ran_if_requested(!intent.files.is_empty()),
            dependencies: ran_if_requested(!intent.dependencies.is_empty()),
            inline_patterns: unsupported_if_requested(intent.has_refactor_sources()),
            planned_type_escapes: LaneStatus::Ran,
        }
    }

    fn validate(&self, intent: &NormalizedIntent) -> Result<()> {
        if self.names != LaneStatus::Ran
            || self.shapes != ran_if_requested(!intent.shapes.is_empty())
            || self.files != ran_if_requested(!intent.files.is_empty())
            || self.dependencies != ran_if_requested(!intent.dependencies.is_empty())
            || self.inline_patterns != unsupported_if_requested(intent.has_refactor_sources())
            || self.planned_type_escapes != LaneStatus::Ran
        {
            bail!("blocked-artifact-contract: intent lane coverage drifted from normalized input");
        }
        Ok(())
    }
}

fn ran_if_requested(requested: bool) -> LaneStatus {
    if requested {
        LaneStatus::Ran
    } else {
        LaneStatus::NotRequested
    }
}

fn unsupported_if_requested(requested: bool) -> LaneStatus {
    if requested {
        LaneStatus::Unsupported
    } else {
        LaneStatus::NotRequested
    }
}

pub(super) fn build(
    loaded: LoadedIntent,
    syntax: &HealthResponse,
    root: &Path,
) -> Result<PreWriteArtifact> {
    let index = CandidateIndex::from_health(syntax);
    let lookups = lookup::lookup_names(&loaded.intent, &index, syntax);
    let shape_lookups = lookup::lookup_shapes(&loaded.intent, syntax);
    let inline_pattern_lookups = lookup::lookup_inline_patterns(&loaded.intent);
    let mut unavailable_evidence = lookup::unavailable_evidence_from_shape_lookups(&shape_lookups);
    unavailable_evidence.extend(lookup::unavailable_evidence_from_inline_pattern_lookups(
        &inline_pattern_lookups,
    ));
    let file_lookups = lookup::lookup_files(&loaded.intent, syntax, root);
    let dependency_lookups = lookup::lookup_dependencies(&loaded.intent, syntax, root)?;
    let CueProjection {
        cue_cards,
        suppressed_cues,
    } = cues::project(&lookups, &shape_lookups, &file_lookups, &dependency_lookups);
    let coverage = IntentLaneCoverage::from_intent(&loaded.intent);
    let artifact = PreWriteArtifact {
        schema_version: SCHEMA_VERSION,
        policy_version: TOKEN_POLICY_VERSION,
        meta: PreWriteMeta::from_syntax(syntax),
        intent: loaded.intent,
        intent_warnings: loaded.warnings,
        coverage,
        lookups,
        shape_lookups,
        file_lookups,
        dependency_lookups,
        inline_pattern_lookups,
        cue_cards,
        suppressed_cues,
        unavailable_evidence,
    };
    artifact.validate_contract()?;
    Ok(artifact)
}

fn intent_has_file(intent: &NormalizedIntent, lookup_file: &str) -> bool {
    intent
        .files
        .iter()
        .any(|file| posix_path_text(file) == lookup_file)
}
