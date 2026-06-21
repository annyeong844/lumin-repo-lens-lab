use serde::Serialize;

use super::Location;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Signal {
    pub kind: SignalKind,
    pub severity: Severity,
    pub claim: Claim,
    pub visibility: SignalVisibility,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mute_reason: Option<SignalMuteReason>,
    pub location: Location,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum SignalKind {
    OversizedFunction,
    OversizedImpl,
    UnsafeBlock,
    UnwrapCall,
    ExpectCall,
    CloneCall,
    PanicMacro,
    TodoMacro,
    UnimplementedMacro,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Severity {
    Review,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Claim {
    SyntaxOnly,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum SignalVisibility {
    Review,
    Muted,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum SignalMuteReason {
    TestPath,
    GeneratedPath,
    TestAttribute,
    CfgTest,
}
