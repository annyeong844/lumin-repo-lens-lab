use std::collections::BTreeMap;

use serde::de::IgnoredAny;
use serde::{Deserialize, Deserializer};

use super::super::{CalibrationCandidateCounts, CalibrationCorpusCandidateCounts};

#[derive(Deserialize)]
#[serde(untagged)]
enum CalibrationCandidateCountsInput {
    Counts(CalibrationCandidateCounts),
    Other(IgnoredAny),
}

#[derive(Deserialize)]
#[serde(untagged)]
enum CandidateCountsByCorpusInput {
    Map(BTreeMap<String, CalibrationCorpusCandidateCountsInput>),
    Other(IgnoredAny),
}

#[derive(Deserialize)]
#[serde(untagged)]
enum CalibrationCorpusCandidateCountsInput {
    Entry(CalibrationCorpusCandidateCounts),
    Other(IgnoredAny),
}

pub(in crate::calibration) fn deserialize_candidate_counts<'de, D>(
    deserializer: D,
) -> Result<CalibrationCandidateCounts, D::Error>
where
    D: Deserializer<'de>,
{
    Ok(
        match Option::<CalibrationCandidateCountsInput>::deserialize(deserializer)? {
            Some(CalibrationCandidateCountsInput::Counts(counts))
                if counts.has_readiness_evidence() =>
            {
                counts
            }
            Some(CalibrationCandidateCountsInput::Counts(_))
            | Some(CalibrationCandidateCountsInput::Other(_))
            | None => CalibrationCandidateCounts::unavailable(),
        },
    )
}

pub(in crate::calibration) fn deserialize_candidate_counts_by_corpus<'de, D>(
    deserializer: D,
) -> Result<BTreeMap<String, CalibrationCorpusCandidateCounts>, D::Error>
where
    D: Deserializer<'de>,
{
    Ok(
        match Option::<CandidateCountsByCorpusInput>::deserialize(deserializer)? {
            Some(CandidateCountsByCorpusInput::Map(entries)) => entries
                .into_iter()
                .filter_map(|(name, entry)| match entry {
                    CalibrationCorpusCandidateCountsInput::Entry(entry) => Some((name, entry)),
                    CalibrationCorpusCandidateCountsInput::Other(_) => None,
                })
                .collect(),
            Some(CandidateCountsByCorpusInput::Other(_)) | None => BTreeMap::new(),
        },
    )
}
