use serde::de::{value::MapAccessDeserializer, IgnoredAny, MapAccess, SeqAccess, Visitor};
use serde::Deserialize;

use super::fields::{optional_string, optional_struct};
use crate::rustc_span::raw::model::{RustcExpansion, RustcSpan};

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

#[derive(Debug, Default)]
pub(super) struct PresentExpansion {
    pub(super) present_non_null: bool,
    pub(super) value: Option<RustcExpansion>,
}

pub(super) fn expansion_field<'de, D>(deserializer: D) -> Result<PresentExpansion, D::Error>
where
    D: serde::Deserializer<'de>,
{
    PresentExpansion::deserialize(deserializer)
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
        Ok(non_null_unknown_expansion())
    }

    fn visit_i64<E>(self, _value: i64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(non_null_unknown_expansion())
    }

    fn visit_u64<E>(self, _value: u64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(non_null_unknown_expansion())
    }

    fn visit_f64<E>(self, _value: f64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(non_null_unknown_expansion())
    }

    fn visit_str<E>(self, _value: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(non_null_unknown_expansion())
    }

    fn visit_string<E>(self, _value: String) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(non_null_unknown_expansion())
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        while seq.next_element::<IgnoredAny>()?.is_some() {}
        Ok(non_null_unknown_expansion())
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

fn non_null_unknown_expansion() -> PresentExpansion {
    PresentExpansion {
        present_non_null: true,
        value: None,
    }
}
