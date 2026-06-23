use serde::Serialize;

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
pub enum PrimarySpanClass {
    UserCode,
    Dependency,
    Generated,
    Unknown,
}
