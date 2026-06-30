use serde::{Deserialize, Serialize};

use super::super::Location;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AstDefinition {
    pub kind: AstDefinitionKind,
    pub name: String,
    pub visibility: AstVisibility,
    pub owner: AstDefinitionOwner,
    pub test_context: bool,
    pub attributes: Vec<AstDefinitionAttribute>,
    pub location: Location,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AstDefinitionAttribute {
    pub kind: AstDefinitionAttributeKind,
    pub text: String,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AstDefinitionAttributeKind {
    Cfg,
    Derive,
    FfiLinker,
    Test,
    Other,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AstDefinitionOwner {
    Module,
    Trait,
    TraitImpl,
    InherentImpl,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AstVisibility {
    Public,
    Crate,
    Restricted,
    Private,
    Unknown,
}
