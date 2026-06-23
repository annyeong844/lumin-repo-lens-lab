use serde::Serialize;

use super::super::Location;

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
