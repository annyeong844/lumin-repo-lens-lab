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
    pub cargo_target_dir_policy: CargoTargetDirPolicy,
    pub cargo_target_dir: String,
    pub cargo_bin: String,
    pub cargo_args: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CargoTargetDirPolicy {
    pub repo_target_dir_used: bool,
    pub owned_temp_target_dir: bool,
    pub incremental_disabled: bool,
    pub debug_symbols_disabled: bool,
    pub stale_cleanup_owned_temp_target_dirs: bool,
    pub stale_isolated_target_dir_max_age_seconds: u64,
    pub stale_reusable_target_dir_max_age_seconds: u64,
}

impl CargoTargetDirPolicy {
    pub(crate) const STALE_ISOLATED_TARGET_DIR_MAX_AGE_SECONDS: u64 = 24 * 60 * 60;
    pub(crate) const STALE_REUSABLE_TARGET_DIR_MAX_AGE_SECONDS: u64 = 7 * 24 * 60 * 60;

    pub(crate) fn from_mode(mode: CargoTargetDirMode) -> Self {
        match mode {
            CargoTargetDirMode::IsolatedTemp | CargoTargetDirMode::ReusableTemp => Self {
                repo_target_dir_used: false,
                owned_temp_target_dir: true,
                incremental_disabled: true,
                debug_symbols_disabled: true,
                stale_cleanup_owned_temp_target_dirs: true,
                stale_isolated_target_dir_max_age_seconds:
                    Self::STALE_ISOLATED_TARGET_DIR_MAX_AGE_SECONDS,
                stale_reusable_target_dir_max_age_seconds:
                    Self::STALE_REUSABLE_TARGET_DIR_MAX_AGE_SECONDS,
            },
        }
    }
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
