use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use serde::Deserialize;

use crate::policy::ActionPolicyTier;

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CalibrationAdjudication {
    #[serde(default)]
    entries: Vec<CalibrationAdjudicationEntry>,
    #[serde(default)]
    corpus: Vec<CalibrationCorpusEntry>,
    #[serde(default)]
    candidate_counts: CalibrationCandidateCounts,
    schema_round_trip: Option<CalibrationSchemaRoundTrip>,
    unresolved_high_findings: Option<usize>,
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
    pub(crate) corpus_name: Option<String>,
    pub(crate) tier: Option<ActionPolicyTier>,
    #[serde(default)]
    pub(crate) verdict: CalibrationVerdict,
    pub(crate) file: Option<String>,
    pub(crate) diagnostic_code: Option<String>,
    pub(crate) line_start: Option<i64>,
}

#[derive(Debug, Clone, Default, Eq, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CalibrationCorpusEntry {
    name: Option<String>,
    commit: Option<String>,
    snapshot_id: Option<String>,
    content_hash: Option<String>,
    worktree_dirty: Option<bool>,
    loc_bucket: Option<String>,
}

impl CalibrationCorpusEntry {
    pub(crate) fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    pub(crate) fn has_immutable_identity(&self) -> bool {
        self.commit.is_some() || self.snapshot_id.is_some()
    }

    pub(crate) fn dirty_state_known(&self) -> bool {
        self.worktree_dirty.is_some()
    }

    pub(crate) fn dirty_state_captured(&self) -> bool {
        self.worktree_dirty != Some(true)
            || self.snapshot_id.is_some()
            || self.content_hash.is_some()
    }

    pub(crate) fn is_non_trivial(&self) -> bool {
        matches!(self.loc_bucket.as_deref(), Some("25k" | "50k" | "100k"))
    }
}

#[derive(Debug, Clone, Default, Eq, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CalibrationCandidateCounts {
    available: Option<bool>,
    safe_fix: Option<usize>,
    review_fix: Option<usize>,
    review_visible_cleanup: Option<usize>,
    degraded: Option<usize>,
    muted: Option<usize>,
    #[serde(default)]
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

    pub(crate) fn expected_review_visible_for_corpus(
        &self,
        corpus_name: &str,
        corpus_total: usize,
    ) -> Option<usize> {
        self.by_corpus
            .get(corpus_name)
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
    attempted: bool,
    #[serde(default)]
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

enum CalibrationAdjudicationInput {
    Entries(Vec<CalibrationAdjudicationEntry>),
    Object(CalibrationAdjudication),
}

pub(crate) fn load_adjudication(path: Option<&Path>) -> Result<Option<CalibrationAdjudication>> {
    let Some(path) = path else {
        return Ok(None);
    };
    let bytes = fs::read(path)
        .with_context(|| format!("failed to read calibration adjudication {}", path.display()))?;
    let input = CalibrationAdjudicationInput::from_slice(&bytes).with_context(|| {
        format!(
            "failed to parse calibration adjudication {}",
            path.display()
        )
    })?;
    Ok(Some(input.into_adjudication()))
}

impl CalibrationAdjudicationInput {
    fn from_slice(bytes: &[u8]) -> serde_json::Result<Self> {
        if bytes
            .iter()
            .copied()
            .find(|byte| !byte.is_ascii_whitespace())
            == Some(b'[')
        {
            return serde_json::from_slice(bytes).map(Self::Entries);
        }
        serde_json::from_slice(bytes).map(Self::Object)
    }

    fn into_adjudication(self) -> CalibrationAdjudication {
        match self {
            Self::Entries(entries) => CalibrationAdjudication {
                entries,
                ..CalibrationAdjudication::default()
            },
            Self::Object(adjudication) => adjudication,
        }
    }
}
