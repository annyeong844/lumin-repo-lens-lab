use serde::{Serialize, Serializer};

#[derive(Debug, Clone, Eq, Hash, PartialEq)]
pub enum RustcDiagnosticLevel {
    Error,
    Warning,
    Note,
    Help,
    FailureNote,
    Other(String),
}

impl RustcDiagnosticLevel {
    pub(crate) fn from_rustc_str(value: String) -> Self {
        match value.as_str() {
            "error" => Self::Error,
            "warning" => Self::Warning,
            "note" => Self::Note,
            "help" => Self::Help,
            "failure-note" => Self::FailureNote,
            _ => Self::Other(value),
        }
    }

    pub(crate) fn is_error(&self) -> bool {
        matches!(self, Self::Error)
    }

    pub(crate) fn is_warning(&self) -> bool {
        matches!(self, Self::Warning)
    }

    fn as_str(&self) -> &str {
        match self {
            Self::Error => "error",
            Self::Warning => "warning",
            Self::Note => "note",
            Self::Help => "help",
            Self::FailureNote => "failure-note",
            Self::Other(value) => value,
        }
    }
}

impl Serialize for RustcDiagnosticLevel {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

#[derive(Debug, Copy, Clone, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum CodePresence {
    PresentNull,
    Omitted,
    PresentValue,
}

#[derive(Debug, Copy, Clone, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum CodeNamespace {
    RustcCodeless,
    RustcError,
    RustcNonEcode,
    Unknown,
}

#[derive(Debug, Copy, Clone, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum CodeKind {
    NullErrorCode,
    RustcErrorCode,
    NonEcodeName,
    Unknown,
}
