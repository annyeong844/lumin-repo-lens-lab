use serde::Serialize;
use std::collections::BTreeMap;

use super::Location;

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AstFacts {
    pub definitions: Vec<AstDefinition>,
    pub shape_hashes: Vec<AstShapeHash>,
    pub impls: Vec<AstImplBlock>,
    pub use_trees: Vec<AstUseTree>,
    pub path_refs: Vec<AstPathRef>,
    pub method_call_counts: BTreeMap<String, usize>,
    pub method_calls: Vec<AstMethodCall>,
    pub macro_calls: Vec<AstMacroCall>,
    pub cfg_gates: Vec<AstCfgGate>,
    pub opaque_surfaces: Vec<AstOpaqueSurface>,
}

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

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AstShapeHash {
    pub kind: AstShapeHashKind,
    pub hash: String,
    pub name: String,
    pub visibility: AstVisibility,
    pub shape_kind: AstShapeKind,
    pub normalized_version: &'static str,
    pub confidence: AstShapeConfidence,
    pub fields: Vec<AstShapeField>,
    pub location: Location,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum AstShapeHashKind {
    ShapeHash,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum AstShapeKind {
    RecordStruct,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum AstShapeConfidence {
    High,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AstShapeField {
    pub kind: AstShapeFieldKind,
    pub name: String,
    #[serde(rename = "type")]
    pub type_text: String,
    pub visibility: AstVisibility,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum AstShapeFieldKind {
    Property,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AstImplBlock {
    pub target: String,
    #[serde(rename = "trait", skip_serializing_if = "Option::is_none")]
    pub trait_path: Option<String>,
    pub methods: Vec<AstImplMethod>,
    pub location: Location,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AstImplMethod {
    pub name: String,
    pub visibility: AstVisibility,
    pub has_receiver: bool,
    pub location: Location,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AstUseTree {
    pub tree: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alias: Option<String>,
    pub glob: bool,
    pub visibility: AstVisibility,
    pub location: Location,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AstPathRef {
    pub path: String,
    pub name: String,
    pub location: Location,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AstMethodCall {
    pub method: String,
    pub receiver: String,
    pub location: Location,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AstMacroCall {
    pub path: String,
    pub name: String,
    pub location: Location,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AstCfgGate {
    pub expr: String,
    pub location: Location,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AstOpaqueSurface {
    pub kind: AstOpaqueSurfaceKind,
    pub reason: AstOpaqueReason,
    #[serde(flatten)]
    pub visibility: AstOpaqueSurfaceVisibility,
    pub detail: String,
    pub location: Location,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum AstOpaqueSurfaceKind {
    MacroExpansion,
    CfgGate,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum AstOpaqueReason {
    MacroExpansionNotEvaluated,
    CfgConditionNotEvaluated,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum AstOpaqueVisibility {
    Review,
    Muted,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Serialize)]
#[serde(tag = "visibility", rename_all = "kebab-case")]
pub enum AstOpaqueSurfaceVisibility {
    Review,
    Muted {
        #[serde(rename = "muteReason")]
        mute_reason: AstOpaqueMuteReason,
    },
}

impl AstOpaqueSurfaceVisibility {
    pub fn visibility(self) -> AstOpaqueVisibility {
        match self {
            Self::Review => AstOpaqueVisibility::Review,
            Self::Muted { .. } => AstOpaqueVisibility::Muted,
        }
    }

    pub fn mute_reason(self) -> Option<AstOpaqueMuteReason> {
        match self {
            Self::Review => None,
            Self::Muted { mute_reason } => Some(mute_reason),
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum AstOpaqueMuteReason {
    TestPath,
    GeneratedPath,
    TestAttribute,
    CfgTest,
    AssertionMacro,
    CollectionMacro,
    DataLiteralMacro,
    FormattingMacro,
    IoFormattingMacro,
    LoggingMacro,
    BuiltinDeriveMacro,
    KnownDataDeriveMacro,
}
