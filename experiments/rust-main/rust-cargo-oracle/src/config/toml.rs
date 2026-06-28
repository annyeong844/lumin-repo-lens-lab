use std::fmt;
use std::fs;
use std::path::Path;

use serde::{
    de::{DeserializeSeed, IgnoredAny, MapAccess, SeqAccess, Visitor},
    Deserialize, Deserializer,
};

pub(crate) fn read_build_target_from_config(path: &Path) -> Option<Vec<String>> {
    let config = read_config(path)?;
    config
        .build
        .target
        .map(ConfigStringList::into_target_values)
}

pub(crate) fn read_build_rustflags_from_config(path: &Path) -> Vec<String> {
    read_config(path)
        .and_then(|config| config.build.rustflags)
        .map(ConfigStringList::into_rustflags)
        .unwrap_or_default()
}

fn read_config(path: &Path) -> Option<CargoConfig> {
    toml::from_str(&fs::read_to_string(path).ok()?).ok()
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct CargoConfig {
    build: CargoBuildConfig,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct CargoBuildConfig {
    #[serde(default, deserialize_with = "deserialize_string_list_option")]
    target: Option<ConfigStringList>,
    #[serde(default, deserialize_with = "deserialize_string_list_option")]
    rustflags: Option<ConfigStringList>,
}

#[derive(Debug)]
enum ConfigStringList {
    Text(String),
    Items(Vec<String>),
}

impl ConfigStringList {
    fn into_target_values(self) -> Vec<String> {
        match self {
            Self::Text(value) => vec![value],
            Self::Items(values) => values,
        }
    }

    fn into_rustflags(self) -> Vec<String> {
        match self {
            Self::Text(flags) => flags.split_whitespace().map(str::to_string).collect(),
            Self::Items(values) => values,
        }
    }
}

fn deserialize_string_list_option<'de, D>(
    deserializer: D,
) -> Result<Option<ConfigStringList>, D::Error>
where
    D: Deserializer<'de>,
{
    deserializer.deserialize_any(ConfigStringListVisitor)
}

struct ConfigStringListVisitor;

impl<'de> Visitor<'de> for ConfigStringListVisitor {
    type Value = Option<ConfigStringList>;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("a string or array of strings")
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E> {
        Ok(Some(ConfigStringList::Text(value.to_string())))
    }

    fn visit_string<E>(self, value: String) -> Result<Self::Value, E> {
        Ok(Some(ConfigStringList::Text(value)))
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut values = Vec::new();
        while let Some(value) = seq.next_element_seed(OptionalStringSeed)? {
            if let Some(value) = value {
                values.push(value);
            }
        }
        Ok(Some(ConfigStringList::Items(values)))
    }

    fn visit_unit<E>(self) -> Result<Self::Value, E> {
        Ok(None)
    }

    fn visit_bool<E>(self, _value: bool) -> Result<Self::Value, E> {
        Ok(None)
    }

    fn visit_i64<E>(self, _value: i64) -> Result<Self::Value, E> {
        Ok(None)
    }

    fn visit_u64<E>(self, _value: u64) -> Result<Self::Value, E> {
        Ok(None)
    }

    fn visit_f64<E>(self, _value: f64) -> Result<Self::Value, E> {
        Ok(None)
    }
}

struct OptionalStringSeed;

impl<'de> DeserializeSeed<'de> for OptionalStringSeed {
    type Value = Option<String>;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(OptionalStringVisitor)
    }
}

struct OptionalStringVisitor;

impl<'de> Visitor<'de> for OptionalStringVisitor {
    type Value = Option<String>;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("an optional string")
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E> {
        Ok(Some(value.to_string()))
    }

    fn visit_string<E>(self, value: String) -> Result<Self::Value, E> {
        Ok(Some(value))
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        while seq.next_element::<IgnoredAny>()?.is_some() {}
        Ok(None)
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        while map.next_entry::<IgnoredAny, IgnoredAny>()?.is_some() {}
        Ok(None)
    }

    fn visit_unit<E>(self) -> Result<Self::Value, E> {
        Ok(None)
    }

    fn visit_bool<E>(self, _value: bool) -> Result<Self::Value, E> {
        Ok(None)
    }

    fn visit_i64<E>(self, _value: i64) -> Result<Self::Value, E> {
        Ok(None)
    }

    fn visit_u64<E>(self, _value: u64) -> Result<Self::Value, E> {
        Ok(None)
    }

    fn visit_f64<E>(self, _value: f64) -> Result<Self::Value, E> {
        Ok(None)
    }
}
