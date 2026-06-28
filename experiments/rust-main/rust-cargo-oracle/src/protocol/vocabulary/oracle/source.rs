use serde::{Serialize, Serializer};

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
