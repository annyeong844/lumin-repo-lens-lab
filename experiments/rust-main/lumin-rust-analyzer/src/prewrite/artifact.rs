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
use super::lookup::{self, FileLookup, NameLookup};
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
    file_lookups: Vec<FileLookup>,
    cue_cards: Vec<CueCard>,
    suppressed_cues: Vec<SuppressedCue>,
}

impl PreWriteArtifact {
    pub(crate) fn validate_contract(&self) -> Result<()> {
        for card in &self.cue_cards {
            for cue in &card.cues {
                if cue.cue_tier == CueTier::Safe
                    && (cue.evidence_lane != EvidenceLane::ExactSymbol
                        || cue
                            .evidence
                            .iter()
                            .any(|evidence| evidence.matched_field != CueMatchedField::DefIndex))
                {
                    bail!(
                        "blocked-artifact-contract: SAFE cue {} is not exact definition evidence",
                        card.candidate.identity
                    );
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
        for lookup in &self.file_lookups {
            if !intent_has_file(&self.intent, &lookup.intent_file) {
                bail!(
                    "blocked-artifact-contract: lookup file {} is absent from normalized intent",
                    lookup.intent_file
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
            dependencies: unsupported_if_requested(!intent.dependencies.is_empty()),
            planned_type_escapes: unsupported_if_requested(!intent.planned_type_escapes.is_empty()),
        }
    }

    fn validate(&self, intent: &NormalizedIntent) -> Result<()> {
        if self.names != LaneStatus::Ran
            || self.shapes != unsupported_if_requested(!intent.shapes.is_empty())
            || self.files != ran_if_requested(!intent.files.is_empty())
            || self.dependencies != unsupported_if_requested(!intent.dependencies.is_empty())
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

pub(super) fn build(loaded: LoadedIntent, syntax: &HealthResponse) -> Result<PreWriteArtifact> {
    let index = CandidateIndex::from_health(syntax);
    let lookups = lookup::lookup_names(&loaded.intent, &index, syntax);
    let file_lookups = lookup::lookup_files(&loaded.intent, syntax);
    let CueProjection {
        cue_cards,
        suppressed_cues,
    } = cues::project(&lookups);
    let coverage = IntentLaneCoverage::from_intent(&loaded.intent);
    let artifact = PreWriteArtifact {
        schema_version: SCHEMA_VERSION,
        policy_version: TOKEN_POLICY_VERSION,
        meta: PreWriteMeta::from_syntax(syntax),
        intent: loaded.intent,
        intent_warnings: loaded.warnings,
        coverage,
        lookups,
        file_lookups,
        cue_cards,
        suppressed_cues,
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
