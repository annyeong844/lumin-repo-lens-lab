use serde::Deserialize;

use crate::policy::ActionPolicyTier;

use super::{
    input, CalibrationCandidateCounts, CalibrationCorpusEntry, CalibrationSchemaRoundTrip,
    CalibrationVerdict,
};

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CalibrationAdjudication {
    #[serde(default, deserialize_with = "input::deserialize_adjudication_entries")]
    pub(in crate::calibration) entries: Vec<CalibrationAdjudicationEntry>,
    #[serde(default, deserialize_with = "input::deserialize_corpus_entries")]
    pub(in crate::calibration) corpus: Vec<CalibrationCorpusEntry>,
    #[serde(default, deserialize_with = "input::deserialize_candidate_counts")]
    pub(in crate::calibration) candidate_counts: CalibrationCandidateCounts,
    #[serde(default, deserialize_with = "input::deserialize_schema_round_trip")]
    pub(in crate::calibration) schema_round_trip: Option<CalibrationSchemaRoundTrip>,
    #[serde(default, deserialize_with = "input::deserialize_optional_usize")]
    pub(in crate::calibration) unresolved_high_findings: Option<usize>,
    #[serde(default, deserialize_with = "input::deserialize_optional_usize")]
    pub(in crate::calibration) min_adjudicated_per_corpus: Option<usize>,
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
