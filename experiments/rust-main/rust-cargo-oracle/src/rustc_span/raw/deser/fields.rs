use serde::Deserialize;

use crate::protocol::RustcSuggestionApplicability;

use super::lossy::Lossy;

#[derive(Debug, Default)]
pub(super) struct PresentString {
    pub(super) present: bool,
    pub(super) value: Option<String>,
}

#[derive(Debug, Default)]
pub(super) struct PresentApplicability {
    pub(super) present: bool,
    pub(super) value: Option<RustcSuggestionApplicability>,
}

pub(super) fn optional_string<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    Ok(Option::<String>::deserialize(deserializer).ok().flatten())
}

pub(super) fn optional_file_name<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    Ok(optional_string(deserializer)?.map(|value| value.replace('\\', "/")))
}

pub(super) fn optional_i64<'de, D>(deserializer: D) -> Result<Option<i64>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    Ok(Option::<i64>::deserialize(deserializer).ok().flatten())
}

pub(super) fn bool_or_false<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: serde::Deserializer<'de>,
{
    Ok(bool::deserialize(deserializer).unwrap_or(false))
}

pub(super) fn presence_applicability<'de, D>(
    deserializer: D,
) -> Result<PresentApplicability, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value = Option::<String>::deserialize(deserializer).ok().flatten();
    Ok(PresentApplicability {
        present: true,
        value: value
            .as_deref()
            .and_then(RustcSuggestionApplicability::from_rustc_str),
    })
}

pub(super) fn presence_string<'de, D>(deserializer: D) -> Result<PresentString, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value = Option::<String>::deserialize(deserializer).ok().flatten();
    Ok(PresentString {
        present: true,
        value,
    })
}

pub(super) fn optional_struct<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: serde::Deserializer<'de>,
    T: Deserialize<'de>,
{
    Ok(Option::<Lossy<T>>::deserialize(deserializer)
        .ok()
        .flatten()
        .and_then(|value| value.0))
}
