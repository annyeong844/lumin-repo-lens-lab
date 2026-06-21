use std::borrow::Cow;
use std::str::FromStr;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

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

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize)]
pub enum SemanticArtifactProducer {
    #[serde(rename = "rust-cargo-oracle")]
    RustCargoOracle,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize)]
pub enum RustcCommandSource {
    #[serde(rename = "env:CARGO_BUILD_RUSTC")]
    CargoBuildRustc,
    #[serde(rename = "env:RUSTC")]
    RustcEnv,
    #[serde(rename = "default:rustc")]
    DefaultRustc,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ArtifactProfile {
    Dev,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum SemanticArtifactMode {
    SemanticOracle,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum MissingInfluenceKind {
    BuildScriptRuntimeInputs,
    ProcMacroRuntimeInputs,
    IncludeStrNonRustFiles,
    GeneratedFiles,
    TargetSpecificCargoConfigExpanded,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum CacheReusePolicy {
    NoReuseUnlessCompleteInfluenceSetIsCaptured,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum CacheReuseReason {
    AnalysisInputSetIncompleteForCacheReuse,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Disposition {
    Finding,
    NonFinding,
    CoverageUnavailable,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ConfidenceTier {
    Verified,
    RuleBacked,
    Candidate,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Serialize)]
pub enum ClaimKind {
    #[serde(rename = "verified.rust.rustc-error-diagnostic")]
    RustcErrorDiagnostic,
    #[serde(rename = "verified.rust.rustc-codeless-error-diagnostic")]
    RustcCodelessErrorDiagnostic,
    #[serde(rename = "rule-backed.rust.rustc-lint-diagnostic")]
    RustcLintDiagnostic,
    #[serde(rename = "candidate.rust.unclassified-cargo-diagnostic")]
    UnclassifiedCargoDiagnostic,
}

impl ClaimKind {
    pub const EMITTED_BY_CLASSIFIER: [Self; 4] = [
        Self::RustcErrorDiagnostic,
        Self::RustcCodelessErrorDiagnostic,
        Self::RustcLintDiagnostic,
        Self::UnclassifiedCargoDiagnostic,
    ];

    pub const ABSENCE_CLEAN_CLAIM_KINDS: [Self; 2] = [
        Self::RustcErrorDiagnostic,
        Self::RustcCodelessErrorDiagnostic,
    ];

    pub const ABSENCE_CLEAN_CONCURRENT_CLAIM_KINDS: [Self; 2] =
        [Self::RustcLintDiagnostic, Self::UnclassifiedCargoDiagnostic];

    pub fn tier(self) -> ConfidenceTier {
        match self {
            Self::RustcErrorDiagnostic | Self::RustcCodelessErrorDiagnostic => {
                ConfidenceTier::Verified
            }
            Self::RustcLintDiagnostic => ConfidenceTier::RuleBacked,
            Self::UnclassifiedCargoDiagnostic => ConfidenceTier::Candidate,
        }
    }

    pub fn authority_ids(self) -> Vec<&'static str> {
        match self {
            Self::RustcErrorDiagnostic => vec!["rust.rustc.error-diagnostic"],
            Self::RustcCodelessErrorDiagnostic => vec!["rust.rustc.codeless-error-diagnostic"],
            _ => Vec::new(),
        }
    }

    pub fn rule_ids(self) -> Vec<&'static str> {
        match self {
            Self::RustcLintDiagnostic => vec!["rust.rustc.lint-diagnostic"],
            _ => Vec::new(),
        }
    }

    pub fn is_verified_rustc_error(self) -> bool {
        matches!(
            self,
            Self::RustcErrorDiagnostic | Self::RustcCodelessErrorDiagnostic
        )
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ClassificationRule {
    NoteHelpFailureNoteAreNotFindings,
    NonUserPrimaryErrorMakesAbsenceCleanUnavailable,
    NonUserPrimaryDiagnosticsAreNotUserFacingFindings,
    NonEcodeCodeNameTreatedAsRuleBackedBeforeLevel,
    EcodeErrorUserCodePrimary,
    CodelessErrorUserCodePrimary,
    FallbackRealWarningOrErrorNeverVerified,
}

impl ClassificationRule {
    pub const EMITTED_BY_CLASSIFIER: [Self; 7] = [
        Self::NoteHelpFailureNoteAreNotFindings,
        Self::NonUserPrimaryErrorMakesAbsenceCleanUnavailable,
        Self::NonUserPrimaryDiagnosticsAreNotUserFacingFindings,
        Self::NonEcodeCodeNameTreatedAsRuleBackedBeforeLevel,
        Self::EcodeErrorUserCodePrimary,
        Self::CodelessErrorUserCodePrimary,
        Self::FallbackRealWarningOrErrorNeverVerified,
    ];
}

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

#[derive(Debug, Clone, Eq, PartialEq)]
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

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum CodePresence {
    PresentNull,
    Omitted,
    PresentValue,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum CodeNamespace {
    RustcCodeless,
    RustcError,
    RustcNonEcode,
    Unknown,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum CodeKind {
    NullErrorCode,
    RustcErrorCode,
    NonEcodeName,
    Unknown,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize)]
pub enum OracleId {
    #[serde(rename = "rust.cargo-check")]
    RustCargoCheck,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum FindingSourceKind {
    SemanticOracle,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize)]
pub enum FindingSourceVersion {
    #[serde(rename = "cargo-check-json.v1")]
    CargoCheckJsonV1,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum OraclePlanStatus {
    NotRun,
    Timeout,
    Ran,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum OraclePlanReason {
    MetadataOnlyFastPath,
    ExplicitCargoCheckMode,
    TargetedCargoCheckSelectedNoPackages,
    ReviewSyntaxEvidencePackageScope,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum CargoCheckMode {
    MetadataOnly,
    CargoCheck,
    TargetedCargoCheck,
}

impl CargoCheckMode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::MetadataOnly => "metadata-only",
            Self::CargoCheck => "cargo-check",
            Self::TargetedCargoCheck => "targeted-cargo-check",
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct ParseCargoCheckModeError;

impl FromStr for CargoCheckMode {
    type Err = ParseCargoCheckModeError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "metadata-only" => Ok(Self::MetadataOnly),
            "cargo-check" => Ok(Self::CargoCheck),
            "targeted-cargo-check" => Ok(Self::TargetedCargoCheck),
            _ => Err(ParseCargoCheckModeError),
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum CargoTargetDirMode {
    IsolatedTemp,
    ReusableTemp,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct ParseCargoTargetDirModeError;

impl FromStr for CargoTargetDirMode {
    type Err = ParseCargoTargetDirModeError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "isolated-temp" => Ok(Self::IsolatedTemp),
            "reusable-temp" => Ok(Self::ReusableTemp),
            _ => Err(ParseCargoTargetDirModeError),
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum OracleScopeKind {
    CrateTargetConfiguration,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum OracleScopeProfile {
    Dev,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum OracleScopeTargetSource {
    CargoJsonMessage,
    CargoMetadataDefaultSelection,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum OracleTargetTripleSource {
    EnvCargoBuildTarget,
    CargoConfig(String),
    DefaultHost,
    NotResolved,
}

impl OracleTargetTripleSource {
    pub(crate) fn cargo_config(path: String) -> Self {
        Self::CargoConfig(path)
    }

    fn serialized(&self) -> String {
        match self {
            Self::EnvCargoBuildTarget => "env:CARGO_BUILD_TARGET".to_string(),
            Self::CargoConfig(path) => format!("cargo-config:{path}"),
            Self::DefaultHost => "default-host".to_string(),
            Self::NotResolved => "not-resolved".to_string(),
        }
    }
}

impl Serialize for OracleTargetTripleSource {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.serialized())
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum OracleCfgSetSource {
    EnvRustflagsBestEffort,
    CargoConfig(String),
    CargoConfigMergedBestEffort,
    NotResolved,
}

impl OracleCfgSetSource {
    pub(crate) fn cargo_config(path: String) -> Self {
        Self::CargoConfig(path)
    }

    fn serialized(&self) -> String {
        match self {
            Self::EnvRustflagsBestEffort => "env-rustflags-best-effort".to_string(),
            Self::CargoConfig(path) => format!("cargo-config:{path}"),
            Self::CargoConfigMergedBestEffort => "cargo-config-merged-best-effort".to_string(),
            Self::NotResolved => "not-resolved".to_string(),
        }
    }
}

impl Serialize for OracleCfgSetSource {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.serialized())
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum PrimarySpanClass {
    UserCode,
    Dependency,
    Generated,
    Unknown,
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

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum CacheReuseSummaryStatus {
    NotReusable,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum CargoTargetKind {
    Bin,
    Lib,
    Rlib,
    Dylib,
    Cdylib,
    Staticlib,
    ProcMacro,
    Example,
    Test,
    Bench,
    CustomBuild,
    Unknown(String),
}

impl CargoTargetKind {
    pub(crate) fn is_default_checked(&self) -> bool {
        matches!(self, Self::Lib | Self::Bin)
    }

    pub(crate) fn blocks_cache_reuse(&self) -> bool {
        matches!(self, Self::CustomBuild | Self::ProcMacro)
    }

    fn from_cargo_str(value: String) -> Self {
        match value.as_str() {
            "bin" => Self::Bin,
            "lib" => Self::Lib,
            "rlib" => Self::Rlib,
            "dylib" => Self::Dylib,
            "cdylib" => Self::Cdylib,
            "staticlib" => Self::Staticlib,
            "proc-macro" => Self::ProcMacro,
            "example" => Self::Example,
            "test" => Self::Test,
            "bench" => Self::Bench,
            "custom-build" => Self::CustomBuild,
            _ => Self::Unknown(value),
        }
    }

    fn as_str(&self) -> &str {
        match self {
            Self::Bin => "bin",
            Self::Lib => "lib",
            Self::Rlib => "rlib",
            Self::Dylib => "dylib",
            Self::Cdylib => "cdylib",
            Self::Staticlib => "staticlib",
            Self::ProcMacro => "proc-macro",
            Self::Example => "example",
            Self::Test => "test",
            Self::Bench => "bench",
            Self::CustomBuild => "custom-build",
            Self::Unknown(value) => value,
        }
    }
}

impl Serialize for CargoTargetKind {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for CargoTargetKind {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        String::deserialize(deserializer).map(Self::from_cargo_str)
    }
}
