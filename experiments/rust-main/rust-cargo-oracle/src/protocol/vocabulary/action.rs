use serde::Serialize;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ActionBlockerReason {
    DiagnosticLevelNotWarning,
    DiagnosticNotRuleBacked,
    InvalidEditRange,
    MacroExpansion,
    MissingMachineApplicableSuggestion,
    MissingSafeEdit,
    MissingSuggestedReplacement,
    NonUserCodePrimary,
    OverlappingEdits,
}

impl ActionBlockerReason {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::DiagnosticLevelNotWarning => "diagnostic-level-not-warning",
            Self::DiagnosticNotRuleBacked => "diagnostic-not-rule-backed",
            Self::InvalidEditRange => "invalid-edit-range",
            Self::MacroExpansion => "macro-expansion",
            Self::MissingMachineApplicableSuggestion => "missing-machine-applicable-suggestion",
            Self::MissingSafeEdit => "missing-safe-edit",
            Self::MissingSuggestedReplacement => "missing-suggested-replacement",
            Self::NonUserCodePrimary => "non-user-code-primary",
            Self::OverlappingEdits => "overlapping-edits",
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum SafeActionKind {
    ApplyRustcMachineApplicableSuggestion,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize)]
pub enum RustcSuggestionApplicability {
    #[serde(rename = "MachineApplicable")]
    MachineApplicable,
    #[serde(rename = "MaybeIncorrect")]
    MaybeIncorrect,
    #[serde(rename = "HasPlaceholders")]
    HasPlaceholders,
    #[serde(rename = "Unspecified")]
    Unspecified,
}

impl RustcSuggestionApplicability {
    pub(crate) fn from_rustc_str(value: &str) -> Option<Self> {
        match value {
            "MachineApplicable" => Some(Self::MachineApplicable),
            "MaybeIncorrect" => Some(Self::MaybeIncorrect),
            "HasPlaceholders" => Some(Self::HasPlaceholders),
            "Unspecified" => Some(Self::Unspecified),
            _ => None,
        }
    }
}
