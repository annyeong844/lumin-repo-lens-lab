use serde::de::{IgnoredAny, MapAccess, SeqAccess, Visitor};
use serde::Deserialize;

use crate::protocol::{CodePresence, DiagnosticCode, DiagnosticCodeDetail};

#[derive(Debug, Clone, Default)]
pub(super) enum RustcDiagnosticCode {
    Detail {
        code: Option<String>,
        explanation: Option<String>,
    },
    Text(String),
    Null,
    #[default]
    Omitted,
    Other,
}

impl RustcDiagnosticCode {
    pub(super) fn presence(&self) -> CodePresence {
        match self {
            Self::Null => CodePresence::PresentNull,
            Self::Omitted => CodePresence::Omitted,
            _ => CodePresence::PresentValue,
        }
    }

    pub(super) fn text(&self) -> Option<&str> {
        match self {
            Self::Detail { code, .. } => code.as_deref(),
            Self::Text(value) => Some(value),
            _ => None,
        }
    }

    pub(super) fn to_protocol(&self) -> DiagnosticCode {
        match self {
            Self::Detail { code, explanation } => DiagnosticCode::Detail(DiagnosticCodeDetail {
                code: code.clone(),
                explanation: explanation.clone(),
            }),
            Self::Text(value) => DiagnosticCode::Text(value.clone()),
            _ => DiagnosticCode::Null,
        }
    }
}

impl<'de> Deserialize<'de> for RustcDiagnosticCode {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_any(RustcDiagnosticCodeVisitor)
    }
}

struct RustcDiagnosticCodeVisitor;

impl<'de> Visitor<'de> for RustcDiagnosticCodeVisitor {
    type Value = RustcDiagnosticCode;

    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("a rustc diagnostic code object, string, null, or future value")
    }

    fn visit_unit<E>(self) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(RustcDiagnosticCode::Null)
    }

    fn visit_none<E>(self) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(RustcDiagnosticCode::Null)
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(RustcDiagnosticCode::Text(value.to_string()))
    }

    fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(RustcDiagnosticCode::Text(value))
    }

    fn visit_bool<E>(self, _value: bool) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(RustcDiagnosticCode::Other)
    }

    fn visit_i64<E>(self, _value: i64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(RustcDiagnosticCode::Other)
    }

    fn visit_u64<E>(self, _value: u64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(RustcDiagnosticCode::Other)
    }

    fn visit_f64<E>(self, _value: f64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(RustcDiagnosticCode::Other)
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        while seq.next_element::<IgnoredAny>()?.is_some() {}
        Ok(RustcDiagnosticCode::Other)
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut code = None;
        let mut explanation = None;

        while let Some(key) = map.next_key::<String>()? {
            match key.as_str() {
                "code" => code = map.next_value::<LossyOptionalString>()?.0,
                "explanation" => explanation = map.next_value::<LossyOptionalString>()?.0,
                _ => {
                    let _ = map.next_value::<IgnoredAny>()?;
                }
            }
        }

        Ok(RustcDiagnosticCode::Detail { code, explanation })
    }
}

#[derive(Debug, Deserialize)]
struct LossyOptionalString(#[serde(deserialize_with = "lossy_optional_string")] Option<String>);

fn lossy_optional_string<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    Ok(Option::<String>::deserialize(deserializer).ok().flatten())
}
