use std::borrow::Cow;

use serde::{Serialize, Serializer};

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum CoverageEffect {
    AbsenceCleanUnavailable,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize)]
pub enum CoverageId {
    #[serde(rename = "cov.cargo-check.cargo-event-stream")]
    CargoCheckEventStream,
    #[serde(rename = "cov.cargo-check.absence-clean")]
    AbsenceClean,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum CoverageKind {
    CargoEventStream,
    AbsenceClean,
}

impl CoverageKind {
    pub const EMITTED_BY_ORACLE: [Self; 2] = [Self::CargoEventStream, Self::AbsenceClean];
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum CleanKind {
    VerifiedRustcErrorAbsence,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize)]
pub enum CleanScope {
    #[serde(rename = "verified rustc error diagnostics for the declared cargo-check scope")]
    DeclaredCargoCheckRustcErrorDiagnostics,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum CoverageStatus {
    Ran,
    Unavailable,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct CoverageUnavailableReasons {
    reasons: Vec<CoverageUnavailableReason>,
}

impl CoverageUnavailableReasons {
    pub fn one(reason: CoverageUnavailableReason) -> Self {
        Self {
            reasons: vec![reason],
        }
    }

    pub fn from_reasons(reasons: Vec<CoverageUnavailableReason>) -> Option<Self> {
        (!reasons.is_empty()).then_some(Self { reasons })
    }

    pub fn message(&self) -> String {
        self.reasons
            .iter()
            .map(CoverageUnavailableReason::message)
            .collect::<Vec<_>>()
            .join("; ")
    }
}

impl Serialize for CoverageUnavailableReasons {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.message())
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum CoverageUnavailableReason {
    CargoCheckOracleNotRunInMetadataOnlyMode,
    TargetedCargoCheckSelectedNoPackages,
    CargoCheckOracleNotRun,
    CargoJsonStreamContainedNoEvents,
    CargoJsonStreamDidNotParseCompletely,
    CargoJsonStreamContainedInvalidJsonLines,
    MissingBuildFinishedEvent,
    BuildFinishedSuccessWasFalse,
    BuildFinishedSuccessWasNotTrue,
    NonUserCodePrimaryErrorDiagnosticEncountered,
    CargoMetadataUnavailable(String),
    CargoJsonStreamUnavailableOrIncomplete,
    AbsenceCleanCoverageEntryMissing,
}

impl CoverageUnavailableReason {
    fn message(&self) -> Cow<'_, str> {
        match self {
            Self::CargoCheckOracleNotRunInMetadataOnlyMode => {
                Cow::Borrowed("cargo check oracle not run in metadata-only mode")
            }
            Self::TargetedCargoCheckSelectedNoPackages => {
                Cow::Borrowed("targeted cargo check selected no packages")
            }
            Self::CargoCheckOracleNotRun => Cow::Borrowed("cargo check oracle not run"),
            Self::CargoJsonStreamContainedNoEvents => {
                Cow::Borrowed("cargo JSON stream contained no events")
            }
            Self::CargoJsonStreamDidNotParseCompletely => {
                Cow::Borrowed("cargo JSON stream did not parse completely")
            }
            Self::CargoJsonStreamContainedInvalidJsonLines => {
                Cow::Borrowed("cargo JSON stream contained invalid JSON lines")
            }
            Self::MissingBuildFinishedEvent => Cow::Borrowed("missing build-finished event"),
            Self::BuildFinishedSuccessWasFalse => Cow::Borrowed("build-finished success was false"),
            Self::BuildFinishedSuccessWasNotTrue => {
                Cow::Borrowed("build-finished success was not true")
            }
            Self::NonUserCodePrimaryErrorDiagnosticEncountered => {
                Cow::Borrowed("non-user-code primary error diagnostic encountered")
            }
            Self::CargoMetadataUnavailable(reason) => {
                Cow::Owned(format!("cargo metadata unavailable: {reason}"))
            }
            Self::CargoJsonStreamUnavailableOrIncomplete => {
                Cow::Borrowed("cargo JSON stream unavailable or incomplete")
            }
            Self::AbsenceCleanCoverageEntryMissing => {
                Cow::Borrowed("absence-clean coverage entry missing")
            }
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum StreamParseStatus {
    Complete,
    Timeout,
    NoJsonEvents,
    InvalidJson,
    NotRun,
}

impl StreamParseStatus {
    pub(crate) fn is_complete(self) -> bool {
        matches!(self, Self::Complete)
    }
}
