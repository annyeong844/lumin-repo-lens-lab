use std::collections::BTreeMap;

use serde::de::IgnoredAny;
use serde::{Deserialize, Deserializer};

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

pub(in crate::calibration) fn deserialize_optional_string<'de, D>(
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

pub(in crate::calibration) fn deserialize_optional_bool<'de, D>(
    deserializer: D,
) -> Result<Option<bool>, D::Error>
where
    D: Deserializer<'de>,
{
    Ok(match Option::<OptionalBool>::deserialize(deserializer)? {
        Some(OptionalBool::Bool(value)) => Some(value),
        Some(OptionalBool::Other(_)) | None => None,
    })
}

pub(in crate::calibration) fn deserialize_optional_js_truthy_bool<'de, D>(
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

pub(in crate::calibration) fn deserialize_js_truthy_bool_or_false<'de, D>(
    deserializer: D,
) -> Result<bool, D::Error>
where
    D: Deserializer<'de>,
{
    Ok(deserialize_optional_js_truthy_bool(deserializer)?.unwrap_or(false))
}

pub(in crate::calibration) fn deserialize_optional_usize<'de, D>(
    deserializer: D,
) -> Result<Option<usize>, D::Error>
where
    D: Deserializer<'de>,
{
    Ok(match Option::<OptionalUsize>::deserialize(deserializer)? {
        Some(OptionalUsize::Integer(value)) => Some(value),
        Some(OptionalUsize::Other(_)) | None => None,
    })
}

pub(in crate::calibration) fn deserialize_optional_i64<'de, D>(
    deserializer: D,
) -> Result<Option<i64>, D::Error>
where
    D: Deserializer<'de>,
{
    Ok(match Option::<OptionalI64>::deserialize(deserializer)? {
        Some(OptionalI64::Integer(value)) => Some(value),
        Some(OptionalI64::Other(_)) | None => None,
    })
}
