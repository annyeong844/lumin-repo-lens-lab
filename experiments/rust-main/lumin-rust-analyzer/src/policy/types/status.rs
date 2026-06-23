use lumin_rust_cargo_oracle::protocol::CoverageStatus;
use serde::Serialize;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum FileParseStatus {
    Ok,
    Error,
    Missing,
}

impl FileParseStatus {
    pub(crate) fn is_ok(self) -> bool {
        self == Self::Ok
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum CoverageRunStatus {
    Ran,
    Unavailable,
    Missing,
}

impl CoverageRunStatus {
    pub(crate) fn from_coverage_status(status: Option<CoverageStatus>) -> Self {
        match status {
            Some(CoverageStatus::Ran) => Self::Ran,
            Some(CoverageStatus::Unavailable) => Self::Unavailable,
            None => Self::Missing,
        }
    }

    pub(crate) fn is_ran(self) -> bool {
        self == Self::Ran
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize)]
pub(crate) enum OracleBridgeStatus {
    #[serde(rename = "oracle-covered")]
    Covered,
    #[serde(rename = "oracle-partial")]
    Partial,
    #[serde(rename = "oracle-unavailable")]
    Unavailable,
    #[serde(rename = "oracle-missing")]
    Missing,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum OracleConfidence {
    Low,
    Medium,
    High,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum CalibrationStatus {
    Pending,
    Measured,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum ActionTier {
    SafeFix,
    ReviewFix,
    Degraded,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum DegradedReason {
    SemanticCandidateFinding,
    CoverageUnavailableEntry,
}
