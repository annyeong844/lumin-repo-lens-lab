use serde::{Deserialize, Deserializer};

use super::model::{DependencyDeclaration, NameDeclaration, PlannedTypeEscape, RefactorSource};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct RawIntent {
    #[serde(default)]
    pub(super) names: Present<Vec<NameInput>>,
    #[serde(default)]
    pub(super) shapes: Present<Vec<ShapeIntentInput>>,
    #[serde(default)]
    pub(super) files: Present<Vec<String>>,
    #[serde(default)]
    pub(super) dependencies: Present<Vec<DependencyInput>>,
    #[serde(default)]
    pub(super) planned_type_escapes: Present<Vec<PlannedTypeEscape>>,
    #[serde(default)]
    pub(super) refactor_sources: Present<Vec<RefactorSource>>,
    #[serde(default)]
    pub(super) task_id: Present<String>,
}

#[derive(Debug)]
pub(super) struct Present<T>(pub(super) Option<T>);

impl<T> Default for Present<T> {
    fn default() -> Self {
        Self(None)
    }
}

impl<'de, T> Deserialize<'de> for Present<T>
where
    T: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        T::deserialize(deserializer).map(|value| Self(Some(value)))
    }
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub(super) enum NameInput {
    Name(String),
    Declaration(NameDeclaration),
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct ShapeIntentInput {
    pub(super) fields: Option<Vec<String>>,
    pub(super) hash: Option<String>,
    pub(super) type_literal: Option<String>,
    pub(super) name: Option<String>,
    pub(super) why: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub(super) enum DependencyInput {
    Specifier(String),
    Declaration(DependencyDeclaration),
}
