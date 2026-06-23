use std::str::FromStr;

use serde::Serialize;

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
