use serde::Serialize;

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
