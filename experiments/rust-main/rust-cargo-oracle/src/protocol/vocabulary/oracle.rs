use std::str::FromStr;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

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
