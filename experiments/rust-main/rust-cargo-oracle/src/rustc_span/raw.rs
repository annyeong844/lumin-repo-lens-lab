use serde::de::{value::MapAccessDeserializer, IgnoredAny, MapAccess, SeqAccess, Visitor};
use serde::Deserialize;

use crate::protocol::RustcSuggestionApplicability;

#[derive(Debug, Clone)]
pub(crate) struct RustcSpan {
    file_name: Option<String>,
    line_start: Option<i64>,
    line_end: Option<i64>,
    column_start: Option<i64>,
    column_end: Option<i64>,
    is_primary: bool,
    suggestion_applicability: Option<RustcSuggestionApplicability>,
    suggested_replacement: Option<String>,
    has_suggestion_applicability_field: bool,
    has_suggested_replacement_field: bool,
    has_expansion: bool,
    expansion: Option<Box<RustcExpansion>>,
}

#[derive(Debug, Clone)]
pub(crate) struct RustcExpansion {
    macro_decl_name: Option<String>,
    span: Option<RustcSpan>,
    def_site_span: Option<RustcSpan>,
}

impl RustcSpan {
    pub(crate) fn file_name(&self) -> Option<&str> {
        self.file_name.as_deref()
    }

    pub(crate) fn line_start(&self) -> Option<i64> {
        self.line_start
    }

    pub(crate) fn line_end(&self) -> Option<i64> {
        self.line_end
    }

    pub(crate) fn column_start(&self) -> Option<i64> {
        self.column_start
    }

    pub(crate) fn column_end(&self) -> Option<i64> {
        self.column_end
    }

    pub(crate) fn is_primary(&self) -> bool {
        self.is_primary
    }

    pub(crate) fn suggestion_applicability(&self) -> Option<RustcSuggestionApplicability> {
        self.suggestion_applicability
    }

    pub(crate) fn suggested_replacement(&self) -> Option<&str> {
        self.suggested_replacement.as_deref()
    }

    pub(crate) fn has_suggestion_payload(&self) -> bool {
        self.has_suggestion_applicability_field || self.has_suggested_replacement_field
    }

    pub(super) fn expansion(&self) -> Option<&RustcExpansion> {
        self.expansion.as_deref()
    }

    pub(crate) fn has_expansion(&self) -> bool {
        self.has_expansion
    }

    pub(crate) fn expansion_callsite_file_names(&self) -> Vec<String> {
        let mut out = Vec::new();
        let mut current = self.expansion();
        while let Some(expansion) = current {
            let Some(callsite) = expansion.span() else {
                break;
            };
            if let Some(file_name) = callsite.file_name() {
                out.push(file_name.to_string());
            }
            current = callsite.expansion();
        }
        out
    }
}

impl RustcExpansion {
    pub(super) fn macro_decl_name(&self) -> Option<&str> {
        self.macro_decl_name.as_deref()
    }

    pub(super) fn span(&self) -> Option<&RustcSpan> {
        self.span.as_ref()
    }

    pub(super) fn def_site_span(&self) -> Option<&RustcSpan> {
        self.def_site_span.as_ref()
    }
}

#[derive(Debug, Deserialize)]
struct RawRustcSpan {
    #[serde(default, deserialize_with = "optional_file_name")]
    file_name: Option<String>,
    #[serde(default, deserialize_with = "optional_i64")]
    line_start: Option<i64>,
    #[serde(default, deserialize_with = "optional_i64")]
    line_end: Option<i64>,
    #[serde(default, deserialize_with = "optional_i64")]
    column_start: Option<i64>,
    #[serde(default, deserialize_with = "optional_i64")]
    column_end: Option<i64>,
    #[serde(default, deserialize_with = "bool_or_false")]
    is_primary: bool,
    #[serde(default, deserialize_with = "presence_applicability")]
    suggestion_applicability: PresentApplicability,
    #[serde(default, deserialize_with = "presence_string")]
    suggested_replacement: PresentString,
    #[serde(default, deserialize_with = "expansion_field")]
    expansion: PresentExpansion,
}

impl From<RawRustcSpan> for RustcSpan {
    fn from(raw: RawRustcSpan) -> Self {
        Self {
            file_name: raw.file_name,
            line_start: raw.line_start,
            line_end: raw.line_end,
            column_start: raw.column_start,
            column_end: raw.column_end,
            is_primary: raw.is_primary,
            suggestion_applicability: raw.suggestion_applicability.value,
            suggested_replacement: raw.suggested_replacement.value,
            has_suggestion_applicability_field: raw.suggestion_applicability.present,
            has_suggested_replacement_field: raw.suggested_replacement.present,
            has_expansion: raw.expansion.present_non_null,
            expansion: raw.expansion.value.map(Box::new),
        }
    }
}

#[derive(Debug, Deserialize)]
struct RawRustcExpansion {
    #[serde(default, deserialize_with = "optional_string")]
    macro_decl_name: Option<String>,
    #[serde(default, deserialize_with = "optional_struct")]
    span: Option<RustcSpan>,
    #[serde(default, deserialize_with = "optional_struct")]
    def_site_span: Option<RustcSpan>,
}

impl From<RawRustcExpansion> for RustcExpansion {
    fn from(raw: RawRustcExpansion) -> Self {
        Self {
            macro_decl_name: raw.macro_decl_name,
            span: raw.span,
            def_site_span: raw.def_site_span,
        }
    }
}

impl<'de> Deserialize<'de> for RustcSpan {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        RawRustcSpan::deserialize(deserializer).map(Into::into)
    }
}

#[derive(Debug, Default)]
struct PresentString {
    present: bool,
    value: Option<String>,
}

#[derive(Debug, Default)]
struct PresentApplicability {
    present: bool,
    value: Option<RustcSuggestionApplicability>,
}

#[derive(Debug, Default)]
struct PresentExpansion {
    present_non_null: bool,
    value: Option<RustcExpansion>,
}

fn optional_string<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    Ok(Option::<String>::deserialize(deserializer).ok().flatten())
}

fn optional_file_name<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    Ok(optional_string(deserializer)?.map(|value| value.replace('\\', "/")))
}

fn optional_i64<'de, D>(deserializer: D) -> Result<Option<i64>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    Ok(Option::<i64>::deserialize(deserializer).ok().flatten())
}

fn bool_or_false<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: serde::Deserializer<'de>,
{
    Ok(bool::deserialize(deserializer).unwrap_or(false))
}

fn presence_applicability<'de, D>(deserializer: D) -> Result<PresentApplicability, D::Error>
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

fn presence_string<'de, D>(deserializer: D) -> Result<PresentString, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value = Option::<String>::deserialize(deserializer).ok().flatten();
    Ok(PresentString {
        present: true,
        value,
    })
}

fn expansion_field<'de, D>(deserializer: D) -> Result<PresentExpansion, D::Error>
where
    D: serde::Deserializer<'de>,
{
    PresentExpansion::deserialize(deserializer)
}

fn optional_struct<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: serde::Deserializer<'de>,
    T: Deserialize<'de>,
{
    Ok(Option::<Lossy<T>>::deserialize(deserializer)
        .ok()
        .flatten()
        .and_then(|value| value.0))
}

impl<'de> Deserialize<'de> for PresentExpansion {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_any(PresentExpansionVisitor)
    }
}

struct PresentExpansionVisitor;

impl<'de> Visitor<'de> for PresentExpansionVisitor {
    type Value = PresentExpansion;

    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("a rustc expansion object, null, or future value")
    }

    fn visit_unit<E>(self) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(PresentExpansion::default())
    }

    fn visit_none<E>(self) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(PresentExpansion::default())
    }

    fn visit_bool<E>(self, _value: bool) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(PresentExpansion {
            present_non_null: true,
            value: None,
        })
    }

    fn visit_i64<E>(self, _value: i64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(PresentExpansion {
            present_non_null: true,
            value: None,
        })
    }

    fn visit_u64<E>(self, _value: u64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(PresentExpansion {
            present_non_null: true,
            value: None,
        })
    }

    fn visit_f64<E>(self, _value: f64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(PresentExpansion {
            present_non_null: true,
            value: None,
        })
    }

    fn visit_str<E>(self, _value: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(PresentExpansion {
            present_non_null: true,
            value: None,
        })
    }

    fn visit_string<E>(self, _value: String) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(PresentExpansion {
            present_non_null: true,
            value: None,
        })
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        while seq.next_element::<IgnoredAny>()?.is_some() {}
        Ok(PresentExpansion {
            present_non_null: true,
            value: None,
        })
    }

    fn visit_map<A>(self, map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let value = RawRustcExpansion::deserialize(MapAccessDeserializer::new(map))
            .ok()
            .map(Into::into);
        Ok(PresentExpansion {
            present_non_null: true,
            value,
        })
    }
}

struct Lossy<T>(Option<T>);

impl<'de, T> Deserialize<'de> for Lossy<T>
where
    T: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Ok(Self(T::deserialize(deserializer).ok()))
    }
}
