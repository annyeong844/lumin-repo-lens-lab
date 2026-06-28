use serde::Serialize;

use super::super::Location;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AstDefinition {
    pub kind: AstDefinitionKind,
    pub name: String,
    pub visibility: AstVisibility,
    pub location: Location,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum AstDefinitionKind {
    Function,
    Struct,
    Enum,
    Trait,
    Module,
    Const,
    Static,
    TypeAlias,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum AstVisibility {
    Public,
    Crate,
    Restricted,
    Private,
    Unknown,
}
