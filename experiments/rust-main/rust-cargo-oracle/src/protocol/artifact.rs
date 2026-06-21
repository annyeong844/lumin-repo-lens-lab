use serde::Serialize;

use super::{
    ArtifactProfile, CacheReusePolicy, CacheReuseReason, CargoCheckMode, CargoTargetDirMode,
    CargoTargetKind, CoverageEntry, DiagnosticEvidence, Finding, MissingInfluenceKind, OraclePlan,
    RustcCommandSource, SemanticArtifactMode, SemanticArtifactProducer, Summary,
};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SemanticHealthArtifact {
    pub schema_version: &'static str,
    pub policy_version: &'static str,
    pub oracle_registry_version: &'static str,
    pub meta: ArtifactMeta,
    pub findings: Vec<Finding>,
    pub diagnostics: Vec<DiagnosticEvidence>,
    pub coverage: Vec<CoverageEntry>,
    pub oracle_plan: OraclePlan,
    pub summary: Summary,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArtifactMeta {
    pub producer: SemanticArtifactProducer,
    pub mode: SemanticArtifactMode,
    pub generated: String,
    pub oracle_registry_version: &'static str,
    pub evidence_policy_version: &'static str,
    pub diagnostic_policy_version: &'static str,
    pub registry_content_hash: String,
    pub analysis_input_set_hash: String,
    pub analysis_input_set_complete: bool,
    pub missing_influence_kinds: Vec<MissingInfluenceKind>,
    pub toolchain: ToolchainMeta,
    pub cache_reuse_policy: CacheReusePolicy,
    pub cache_reuse: CacheReuse,
    pub input: InputMeta,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolchainMeta {
    pub cargo_version: Option<String>,
    pub rustc_version_verbose: Option<String>,
    pub rustc_bin: String,
    pub rustc_source: RustcCommandSource,
    pub host: Option<String>,
    pub profile: ArtifactProfile,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InputMeta {
    pub root: String,
    pub package_name: Option<String>,
    pub features: Option<String>,
    pub cargo_check_mode: CargoCheckMode,
    pub cargo_target_dir_mode: CargoTargetDirMode,
    pub cargo_target_dir: String,
    pub cargo_bin: String,
    pub cargo_args: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CacheReuse {
    pub policy: CacheReusePolicy,
    pub reason: CacheReuseReason,
    pub blocking_targets: Vec<BlockingTarget>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BlockingTarget {
    pub package_id: String,
    pub package_name: String,
    pub target_name: String,
    pub target_kinds: Vec<CargoTargetKind>,
}
