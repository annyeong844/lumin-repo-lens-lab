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
    self, DependencyLookup, FileLookup, NameLookup, ShapeLookup, UnavailableEvidence,
};
use super::tokens::{TOKENIZER_VERSION, TOKEN_POLICY_VERSION, WEAK_COMMON_TOKENS};

const SCHEMA_VERSION: &str = "rust-pre-write.v1";

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
                                .all(|evidence| evidence.matched_field == CueMatchedField::DefIndex) => {}
                        EvidenceLane::ExactFile
                            if cue.evidence.iter().all(|evidence| {
                                evidence.matched_field == CueMatchedField::RustSourceHealthFiles
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
            if !lookup.is_unavailable() {
                bail!("blocked-artifact-contract: unsupported Rust shape lane emitted a match");
            }
        }
        if self.unavailable_evidence.len() != self.shape_lookups.len() {
            bail!("blocked-artifact-contract: unavailable evidence drifted from shape lookups");
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
    planned_type_escapes: LaneStatus,
}

impl IntentLaneCoverage {
    fn from_intent(intent: &NormalizedIntent) -> Self {
        Self {
            names: LaneStatus::Ran,
            shapes: unsupported_if_requested(!intent.shapes.is_empty()),
            files: ran_if_requested(!intent.files.is_empty()),
            dependencies: ran_if_requested(!intent.dependencies.is_empty()),
            planned_type_escapes: unsupported_if_requested(!intent.planned_type_escapes.is_empty()),
        }
    }

    fn validate(&self, intent: &NormalizedIntent) -> Result<()> {
        if self.names != LaneStatus::Ran
            || self.shapes != unsupported_if_requested(!intent.shapes.is_empty())
            || self.files != ran_if_requested(!intent.files.is_empty())
            || self.dependencies != ran_if_requested(!intent.dependencies.is_empty())
            || self.planned_type_escapes
                != unsupported_if_requested(!intent.planned_type_escapes.is_empty())
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
    let shape_lookups = lookup::lookup_shapes(&loaded.intent);
    let unavailable_evidence = lookup::unavailable_evidence_from_shape_lookups(&shape_lookups);
    let file_lookups = lookup::lookup_files(&loaded.intent, syntax, root);
    let dependency_lookups = lookup::lookup_dependencies(&loaded.intent, syntax, root)?;
    let CueProjection {
        cue_cards,
        suppressed_cues,
    } = cues::project(&lookups, &file_lookups, &dependency_lookups);
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
