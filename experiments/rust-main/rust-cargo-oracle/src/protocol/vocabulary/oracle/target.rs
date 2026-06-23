use serde::{Deserialize, Deserializer, Serialize, Serializer};

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
