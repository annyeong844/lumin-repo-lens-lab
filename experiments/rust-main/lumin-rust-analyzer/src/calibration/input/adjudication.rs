use serde::de::IgnoredAny;
use serde::{Deserialize, Deserializer};

use super::super::{CalibrationAdjudication, CalibrationAdjudicationEntry, CalibrationCorpusEntry};

enum CalibrationAdjudicationInput {
    Entries(Vec<CalibrationAdjudicationEntry>),
    Object(CalibrationAdjudication),
}

#[derive(Deserialize)]
#[serde(untagged)]
enum CalibrationAdjudicationEntryInput {
    Entry(CalibrationAdjudicationEntry),
    Other(IgnoredAny),
}

#[derive(Deserialize)]
#[serde(untagged)]
enum CalibrationAdjudicationEntriesInput {
    Entries(Vec<CalibrationAdjudicationEntryInput>),
    Other(IgnoredAny),
}

#[derive(Deserialize)]
#[serde(untagged)]
enum CalibrationCorpusEntryInput {
    Entry(CalibrationCorpusEntry),
    Other(IgnoredAny),
}

#[derive(Deserialize)]
#[serde(untagged)]
enum CalibrationCorpusEntriesInput {
    Entries(Vec<CalibrationCorpusEntryInput>),
    Other(IgnoredAny),
}

#[derive(Deserialize)]
struct CalibrationAdjudicationEntries(
    #[serde(deserialize_with = "super::deserialize_adjudication_entries")]
    Vec<CalibrationAdjudicationEntry>,
);

pub(in crate::calibration) fn parse_adjudication(
    bytes: &[u8],
) -> serde_json::Result<CalibrationAdjudication> {
    CalibrationAdjudicationInput::from_slice(bytes).map(CalibrationAdjudicationInput::into_model)
}

impl CalibrationAdjudicationInput {
    fn from_slice(bytes: &[u8]) -> serde_json::Result<Self> {
        if bytes
            .iter()
            .copied()
            .find(|byte| !byte.is_ascii_whitespace())
            == Some(b'[')
        {
            return serde_json::from_slice::<CalibrationAdjudicationEntries>(bytes)
                .map(|entries| Self::Entries(entries.0));
        }
        serde_json::from_slice(bytes).map(Self::Object)
    }

    fn into_model(self) -> CalibrationAdjudication {
        match self {
            Self::Entries(entries) => CalibrationAdjudication {
                entries,
                ..CalibrationAdjudication::default()
            },
            Self::Object(adjudication) => adjudication,
        }
    }
}

pub(in crate::calibration) fn deserialize_adjudication_entries<'de, D>(
    deserializer: D,
) -> Result<Vec<CalibrationAdjudicationEntry>, D::Error>
where
    D: Deserializer<'de>,
{
    Ok(
        match Option::<CalibrationAdjudicationEntriesInput>::deserialize(deserializer)? {
            Some(CalibrationAdjudicationEntriesInput::Entries(entries)) => entries
                .into_iter()
                .filter_map(|entry| match entry {
                    CalibrationAdjudicationEntryInput::Entry(entry) => Some(entry),
                    CalibrationAdjudicationEntryInput::Other(_) => None,
                })
                .collect(),
            Some(CalibrationAdjudicationEntriesInput::Other(_)) | None => Vec::new(),
        },
    )
}

pub(in crate::calibration) fn deserialize_corpus_entries<'de, D>(
    deserializer: D,
) -> Result<Vec<CalibrationCorpusEntry>, D::Error>
where
    D: Deserializer<'de>,
{
    Ok(
        match Option::<CalibrationCorpusEntriesInput>::deserialize(deserializer)? {
            Some(CalibrationCorpusEntriesInput::Entries(entries)) => entries
                .into_iter()
                .map(|entry| match entry {
                    CalibrationCorpusEntryInput::Entry(entry) => entry,
                    CalibrationCorpusEntryInput::Other(_) => CalibrationCorpusEntry::default(),
                })
                .collect(),
            Some(CalibrationCorpusEntriesInput::Other(_)) | None => Vec::new(),
        },
    )
}
