use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use serde::Deserialize;

use crate::policy::ActionPolicyTier;

mod input;

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CalibrationAdjudication {
    #[serde(default, deserialize_with = "input::deserialize_adjudication_entries")]
    entries: Vec<CalibrationAdjudicationEntry>,
    #[serde(default, deserialize_with = "input::deserialize_corpus_entries")]
    corpus: Vec<CalibrationCorpusEntry>,
    #[serde(default, deserialize_with = "input::deserialize_candidate_counts")]
    candidate_counts: CalibrationCandidateCounts,
    #[serde(default, deserialize_with = "input::deserialize_schema_round_trip")]
    schema_round_trip: Option<CalibrationSchemaRoundTrip>,
    #[serde(default, deserialize_with = "input::deserialize_optional_usize")]
    unresolved_high_findings: Option<usize>,
    #[serde(default, deserialize_with = "input::deserialize_optional_usize")]
    min_adjudicated_per_corpus: Option<usize>,
}

impl CalibrationAdjudication {
    pub(crate) fn entries(&self) -> &[CalibrationAdjudicationEntry] {
        &self.entries
    }

    pub(crate) fn corpus(&self) -> &[CalibrationCorpusEntry] {
        &self.corpus
    }

    pub(crate) fn candidate_counts(&self) -> &CalibrationCandidateCounts {
        &self.candidate_counts
    }

    pub(crate) fn schema_round_trip(&self) -> Option<&CalibrationSchemaRoundTrip> {
        self.schema_round_trip.as_ref()
    }

    pub(crate) fn unresolved_high_findings(&self) -> usize {
        self.unresolved_high_findings.unwrap_or(0)
    }

    pub(crate) fn min_adjudicated_per_corpus(&self) -> Option<usize> {
        self.min_adjudicated_per_corpus
    }

    pub(crate) fn has_readiness_evidence(&self) -> bool {
        !self.corpus.is_empty()
            || self.schema_round_trip.is_some()
            || self.candidate_counts.has_readiness_evidence()
            || self.unresolved_high_findings.is_some()
            || self.min_adjudicated_per_corpus.is_some()
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CalibrationAdjudicationEntry {
    #[serde(default, deserialize_with = "input::deserialize_optional_string")]
    pub(crate) corpus_name: Option<String>,
    #[serde(default, deserialize_with = "input::deserialize_action_policy_tier")]
    pub(crate) tier: Option<ActionPolicyTier>,
    #[serde(default, deserialize_with = "input::deserialize_calibration_verdict")]
    pub(crate) verdict: CalibrationVerdict,
    #[serde(default, deserialize_with = "input::deserialize_optional_string")]
    pub(crate) file: Option<String>,
    #[serde(default, deserialize_with = "input::deserialize_optional_string")]
    pub(crate) diagnostic_code: Option<String>,
    #[serde(default, deserialize_with = "input::deserialize_optional_i64")]
    pub(crate) line_start: Option<i64>,
}

#[derive(Debug, Clone, Default, Eq, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CalibrationCorpusEntry {
    #[serde(default, deserialize_with = "input::deserialize_optional_string")]
    name: Option<String>,
    #[serde(default, deserialize_with = "input::deserialize_optional_string")]
    commit: Option<String>,
    #[serde(default, deserialize_with = "input::deserialize_optional_string")]
    snapshot_id: Option<String>,
    #[serde(default, deserialize_with = "input::deserialize_optional_string")]
    content_hash: Option<String>,
    #[serde(default, deserialize_with = "input::deserialize_optional_bool")]
    worktree_dirty: Option<bool>,
    #[serde(default, deserialize_with = "input::deserialize_optional_string")]
    loc_bucket: Option<String>,
}

impl CalibrationCorpusEntry {
    pub(crate) fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    pub(crate) fn display_name(&self) -> &str {
        self.name
            .as_deref()
            .filter(|name| !name.is_empty())
            .unwrap_or("(unnamed)")
    }

    pub(crate) fn has_immutable_identity(&self) -> bool {
        has_text(&self.commit) || has_text(&self.snapshot_id)
    }

    pub(crate) fn dirty_state_known(&self) -> bool {
        self.worktree_dirty.is_some()
    }

    pub(crate) fn dirty_state_captured(&self) -> bool {
        self.worktree_dirty != Some(true)
            || has_text(&self.snapshot_id)
            || has_text(&self.content_hash)
    }

    pub(crate) fn is_non_trivial(&self) -> bool {
        matches!(self.loc_bucket.as_deref(), Some("25k" | "50k" | "100k"))
    }
}

fn has_text(value: &Option<String>) -> bool {
    value.as_deref().is_some_and(|value| !value.is_empty())
}

#[derive(Debug, Clone, Default, Eq, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CalibrationCandidateCounts {
    #[serde(
        default,
        deserialize_with = "input::deserialize_optional_js_truthy_bool"
    )]
    available: Option<bool>,
    #[serde(default, deserialize_with = "input::deserialize_optional_usize")]
    safe_fix: Option<usize>,
    #[serde(default, deserialize_with = "input::deserialize_optional_usize")]
    review_fix: Option<usize>,
    #[serde(default, deserialize_with = "input::deserialize_optional_usize")]
    review_visible_cleanup: Option<usize>,
    #[serde(default, deserialize_with = "input::deserialize_optional_usize")]
    degraded: Option<usize>,
    #[serde(default, deserialize_with = "input::deserialize_optional_usize")]
    muted: Option<usize>,
    #[serde(
        default,
        deserialize_with = "input::deserialize_candidate_counts_by_corpus"
    )]
    by_corpus: BTreeMap<String, CalibrationCorpusCandidateCounts>,
}

impl CalibrationCandidateCounts {
    pub(crate) fn is_available(&self) -> bool {
        self.available == Some(true)
    }

    fn has_readiness_evidence(&self) -> bool {
        self.available.is_some()
            || self.safe_fix.is_some()
            || self.review_fix.is_some()
            || self.review_visible_cleanup.is_some()
            || self.degraded.is_some()
            || self.muted.is_some()
            || !self.by_corpus.is_empty()
    }

    pub(crate) fn safe_fix(&self) -> Option<usize> {
        self.safe_fix
    }

    pub(crate) fn review_fix(&self) -> Option<usize> {
        self.review_fix
    }

    pub(crate) fn review_visible_cleanup(&self) -> Option<usize> {
        self.review_visible_cleanup
    }

    pub(crate) fn degraded(&self) -> Option<usize> {
        self.degraded
    }

    pub(crate) fn muted(&self) -> Option<usize> {
        self.muted
    }

    pub(crate) fn expected_review_visible_for_optional_corpus(
        &self,
        corpus_name: Option<&str>,
        corpus_total: usize,
    ) -> Option<usize> {
        corpus_name
            .and_then(|name| self.by_corpus.get(name))
            .and_then(CalibrationCorpusCandidateCounts::review_visible_cleanup)
            .or_else(|| {
                (corpus_total == 1)
                    .then_some(self.review_visible_cleanup)
                    .flatten()
            })
    }
}

#[derive(Debug, Clone, Default, Eq, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CalibrationCorpusCandidateCounts {
    #[serde(default, deserialize_with = "input::deserialize_optional_usize")]
    review_visible_cleanup: Option<usize>,
}

impl CalibrationCorpusCandidateCounts {
    fn review_visible_cleanup(&self) -> Option<usize> {
        self.review_visible_cleanup
    }
}

#[derive(Debug, Clone, Default, Eq, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CalibrationSchemaRoundTrip {
    #[serde(
        default,
        deserialize_with = "input::deserialize_js_truthy_bool_or_false"
    )]
    attempted: bool,
    #[serde(default, deserialize_with = "input::deserialize_schema_drift_bugs")]
    known_schema_drift_bugs: Vec<CalibrationSchemaDriftBug>,
}

impl CalibrationSchemaRoundTrip {
    pub(crate) fn attempted(&self) -> bool {
        self.attempted
    }

    pub(crate) fn has_known_schema_drift_bugs(&self) -> bool {
        !self.known_schema_drift_bugs.is_empty()
    }
}

#[derive(Debug, Clone, Default, Eq, PartialEq, Deserialize)]
struct CalibrationSchemaDriftBug {}

#[derive(Debug, Copy, Clone, Default, Eq, PartialEq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum CalibrationVerdict {
    TrueDead,
    FalsePositive,
    #[default]
    Inconclusive,
    NotApplicable,
}

pub(crate) fn load_adjudication(path: Option<&Path>) -> Result<Option<CalibrationAdjudication>> {
    let Some(path) = path else {
        return Ok(None);
    };
    let bytes = fs::read(path)
        .with_context(|| format!("failed to read calibration adjudication {}", path.display()))?;
    input::parse_adjudication(&bytes)
        .with_context(|| {
            format!(
                "failed to parse calibration adjudication {}",
                path.display()
            )
        })
        .map(Some)
}
