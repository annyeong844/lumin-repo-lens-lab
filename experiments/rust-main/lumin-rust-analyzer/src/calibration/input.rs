use std::collections::BTreeMap;

use serde::de::IgnoredAny;
use serde::{Deserialize, Deserializer};

use crate::policy::ActionPolicyTier;

use super::{
    CalibrationAdjudication, CalibrationAdjudicationEntry, CalibrationCandidateCounts,
    CalibrationCorpusCandidateCounts, CalibrationCorpusEntry, CalibrationSchemaDriftBug,
    CalibrationSchemaRoundTrip, CalibrationVerdict,
};

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
#[serde(untagged)]
enum CalibrationCandidateCountsInput {
    Counts(CalibrationCandidateCounts),
    Other(IgnoredAny),
}

#[derive(Deserialize)]
#[serde(untagged)]
enum CalibrationSchemaRoundTripInput {
    SchemaRoundTrip(CalibrationSchemaRoundTrip),
    Other(IgnoredAny),
}

#[derive(Deserialize)]
#[serde(untagged)]
enum SchemaDriftBugsInput {
    Array(Vec<IgnoredAny>),
    String(String),
    Other(IgnoredAny),
}

#[derive(Deserialize)]
struct CalibrationAdjudicationEntries(
    #[serde(deserialize_with = "deserialize_adjudication_entries")]
    Vec<CalibrationAdjudicationEntry>,
);

#[derive(Deserialize)]
#[serde(untagged)]
enum OptionalString {
    String(String),
    Other(IgnoredAny),
}

#[derive(Deserialize)]
#[serde(untagged)]
enum OptionalBool {
    Bool(bool),
    Other(IgnoredAny),
}

#[derive(Deserialize)]
#[serde(untagged)]
enum OptionalJsTruthy {
    Bool(bool),
    Number(f64),
    String(String),
    Array(Vec<IgnoredAny>),
    Object(BTreeMap<String, IgnoredAny>),
    Other(IgnoredAny),
}

#[derive(Deserialize)]
#[serde(untagged)]
enum OptionalUsize {
    Integer(usize),
    Other(IgnoredAny),
}

#[derive(Deserialize)]
#[serde(untagged)]
enum OptionalI64 {
    Integer(i64),
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

pub(super) fn parse_adjudication(bytes: &[u8]) -> serde_json::Result<CalibrationAdjudication> {
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

pub(super) fn deserialize_adjudication_entries<'de, D>(
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

pub(super) fn deserialize_corpus_entries<'de, D>(
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

pub(super) fn deserialize_candidate_counts<'de, D>(
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

pub(super) fn deserialize_schema_round_trip<'de, D>(
    deserializer: D,
) -> Result<Option<CalibrationSchemaRoundTrip>, D::Error>
where
    D: Deserializer<'de>,
{
    Ok(
        match Option::<CalibrationSchemaRoundTripInput>::deserialize(deserializer)? {
            Some(CalibrationSchemaRoundTripInput::SchemaRoundTrip(schema_round_trip)) => {
                Some(schema_round_trip)
            }
            Some(CalibrationSchemaRoundTripInput::Other(_)) | None => None,
        },
    )
}

pub(super) fn deserialize_action_policy_tier<'de, D>(
    deserializer: D,
) -> Result<Option<ActionPolicyTier>, D::Error>
where
    D: Deserializer<'de>,
{
    let Some(raw) = deserialize_optional_string(deserializer)? else {
        return Ok(None);
    };
    Ok(match raw.as_str() {
        "SAFE_FIX" => Some(ActionPolicyTier::SafeFix),
        "REVIEW_FIX" => Some(ActionPolicyTier::ReviewFix),
        "DEGRADED" => Some(ActionPolicyTier::Degraded),
        "MUTED" => Some(ActionPolicyTier::Muted),
        "UNAVAILABLE" => Some(ActionPolicyTier::Unavailable),
        _ => None,
    })
}

pub(super) fn deserialize_calibration_verdict<'de, D>(
    deserializer: D,
) -> Result<CalibrationVerdict, D::Error>
where
    D: Deserializer<'de>,
{
    let Some(raw) = deserialize_optional_string(deserializer)? else {
        return Ok(CalibrationVerdict::Inconclusive);
    };
    Ok(match raw.as_str() {
        "true_dead" => CalibrationVerdict::TrueDead,
        "false_positive" => CalibrationVerdict::FalsePositive,
        "not_applicable" => CalibrationVerdict::NotApplicable,
        "inconclusive" => CalibrationVerdict::Inconclusive,
        _ => CalibrationVerdict::Inconclusive,
    })
}

pub(super) fn deserialize_optional_string<'de, D>(
    deserializer: D,
) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    Ok(match Option::<OptionalString>::deserialize(deserializer)? {
        Some(OptionalString::String(value)) => Some(value),
        Some(OptionalString::Other(_)) | None => None,
    })
}

pub(super) fn deserialize_optional_bool<'de, D>(deserializer: D) -> Result<Option<bool>, D::Error>
where
    D: Deserializer<'de>,
{
    Ok(match Option::<OptionalBool>::deserialize(deserializer)? {
        Some(OptionalBool::Bool(value)) => Some(value),
        Some(OptionalBool::Other(_)) | None => None,
    })
}

pub(super) fn deserialize_optional_js_truthy_bool<'de, D>(
    deserializer: D,
) -> Result<Option<bool>, D::Error>
where
    D: Deserializer<'de>,
{
    Ok(
        match Option::<OptionalJsTruthy>::deserialize(deserializer)? {
            Some(OptionalJsTruthy::Bool(value)) => Some(value),
            Some(OptionalJsTruthy::Number(value)) => Some(value != 0.0),
            Some(OptionalJsTruthy::String(value)) => Some(!value.is_empty()),
            Some(OptionalJsTruthy::Array(entries)) => {
                let _ = entries.len();
                Some(true)
            }
            Some(OptionalJsTruthy::Object(entries)) => {
                let _ = entries.len();
                Some(true)
            }
            Some(OptionalJsTruthy::Other(_)) => Some(false),
            None => None,
        },
    )
}

pub(super) fn deserialize_js_truthy_bool_or_false<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: Deserializer<'de>,
{
    Ok(deserialize_optional_js_truthy_bool(deserializer)?.unwrap_or(false))
}

pub(super) fn deserialize_optional_usize<'de, D>(deserializer: D) -> Result<Option<usize>, D::Error>
where
    D: Deserializer<'de>,
{
    Ok(match Option::<OptionalUsize>::deserialize(deserializer)? {
        Some(OptionalUsize::Integer(value)) => Some(value),
        Some(OptionalUsize::Other(_)) | None => None,
    })
}

pub(super) fn deserialize_optional_i64<'de, D>(deserializer: D) -> Result<Option<i64>, D::Error>
where
    D: Deserializer<'de>,
{
    Ok(match Option::<OptionalI64>::deserialize(deserializer)? {
        Some(OptionalI64::Integer(value)) => Some(value),
        Some(OptionalI64::Other(_)) | None => None,
    })
}

pub(super) fn deserialize_candidate_counts_by_corpus<'de, D>(
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

pub(super) fn deserialize_schema_drift_bugs<'de, D>(
    deserializer: D,
) -> Result<Vec<CalibrationSchemaDriftBug>, D::Error>
where
    D: Deserializer<'de>,
{
    Ok(
        match Option::<SchemaDriftBugsInput>::deserialize(deserializer)? {
            Some(SchemaDriftBugsInput::Array(entries)) => entries
                .into_iter()
                .map(|_| CalibrationSchemaDriftBug {})
                .collect(),
            Some(SchemaDriftBugsInput::String(value)) if !value.is_empty() => {
                vec![CalibrationSchemaDriftBug {}]
            }
            Some(SchemaDriftBugsInput::String(_)) | Some(SchemaDriftBugsInput::Other(_)) | None => {
                Vec::new()
            }
        },
    )
}
