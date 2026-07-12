use serde::{Deserialize, Serialize};

use super::Location;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Signal {
    pub kind: SignalKind,
    pub severity: Severity,
    pub claim: Claim,
    #[serde(flatten)]
    pub visibility: SignalVisibilityState,
    pub location: Location,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SignalKind {
    UnsafeBlock,
    UnwrapCall,
    ExpectCall,
    CloneCall,
    PanicMacro,
    TodoMacro,
    UnimplementedMacro,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Severity {
    Review,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Claim {
    SyntaxOnly,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SignalVisibility {
    Review,
    Muted,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Serialize)]
#[serde(tag = "visibility", rename_all = "kebab-case")]
pub enum SignalVisibilityState {
    Review,
    Muted {
        #[serde(rename = "muteReason")]
        mute_reason: SignalMuteReason,
    },
}

impl SignalVisibilityState {
    pub fn visibility(self) -> SignalVisibility {
        match self {
            Self::Review => SignalVisibility::Review,
            Self::Muted { .. } => SignalVisibility::Muted,
        }
    }

    pub fn mute_reason(self) -> Option<SignalMuteReason> {
        match self {
            Self::Review => None,
            Self::Muted { mute_reason } => Some(mute_reason),
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SignalMuteReason {
    TestPath,
    GeneratedPath,
    TestAttribute,
    CfgTest,
}
