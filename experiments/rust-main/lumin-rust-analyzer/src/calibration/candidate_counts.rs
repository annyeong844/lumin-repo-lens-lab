use std::collections::BTreeMap;

use serde::Deserialize;

use super::input;

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
    pub(in crate::calibration) fn unavailable() -> Self {
        Self {
            available: Some(false),
            ..Self::default()
        }
    }

    pub(crate) fn is_available(&self) -> bool {
        self.available == Some(true)
    }

    pub(in crate::calibration) fn has_readiness_evidence(&self) -> bool {
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
pub(in crate::calibration) struct CalibrationCorpusCandidateCounts {
    #[serde(default, deserialize_with = "input::deserialize_optional_usize")]
    review_visible_cleanup: Option<usize>,
}

impl CalibrationCorpusCandidateCounts {
    fn review_visible_cleanup(&self) -> Option<usize> {
        self.review_visible_cleanup
    }
}
