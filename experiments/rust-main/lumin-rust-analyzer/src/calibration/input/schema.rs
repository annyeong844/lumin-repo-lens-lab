use serde::de::IgnoredAny;
use serde::{Deserialize, Deserializer};

use super::super::{CalibrationSchemaDriftBug, CalibrationSchemaRoundTrip};

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

pub(in crate::calibration) fn deserialize_schema_round_trip<'de, D>(
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

pub(in crate::calibration) fn deserialize_schema_drift_bugs<'de, D>(
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
