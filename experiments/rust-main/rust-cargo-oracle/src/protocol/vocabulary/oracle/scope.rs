use serde::Serialize;

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
