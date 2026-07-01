use serde::Serialize;
use std::path::PathBuf;

use lumin_audit_core::artifact_summaries::ArtifactSummaryKind;
use lumin_audit_core::blind_zones::BlindZoneSummary;
use lumin_audit_core::generated_artifacts::GeneratedArtifactsMode;
use lumin_audit_core::orchestration_plan::AuditProfile;

#[derive(Default)]
pub(super) struct ArtifactRegistryArgs {
    pub(super) output: Option<PathBuf>,
    pub(super) rust_analysis_ran: bool,
    pub(super) rust_analysis_block: Option<String>,
}

#[derive(Default)]
pub(super) struct RustAnalysisSummaryArgs {
    pub(super) root: Option<PathBuf>,
    pub(super) artifact: Option<PathBuf>,
}

#[derive(Default)]
pub(super) struct GeneratedArtifactsSummaryArgs {
    pub(super) root: Option<PathBuf>,
    pub(super) symbols: Option<PathBuf>,
    pub(super) include_tests: bool,
    pub(super) excludes: Vec<String>,
    pub(super) generated_artifacts_mode: GeneratedArtifactsMode,
}

#[derive(Default)]
pub(super) struct ArtifactSummaryArgs {
    pub(super) kind: Option<ArtifactSummaryKind>,
    pub(super) artifact: Option<PathBuf>,
}

#[derive(Default)]
pub(super) struct ResolverDiagnosticsSummaryArgs {
    pub(super) symbols: Option<PathBuf>,
    pub(super) resolver_capabilities: Option<PathBuf>,
    pub(super) resolver_diagnostics: Option<PathBuf>,
}

#[derive(Default)]
pub(super) struct BlindZonesSummaryArgs {
    pub(super) input: Option<PathBuf>,
    pub(super) cases: Option<PathBuf>,
    pub(super) root: Option<PathBuf>,
    pub(super) output: Option<PathBuf>,
    pub(super) rust_analysis_ran: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct BlindZoneCaseSummary {
    pub(super) name: String,
    pub(super) blind_zones: Vec<BlindZoneSummary>,
}

#[derive(Default)]
pub(super) struct ManifestCoreSummaryArgs {
    pub(super) root: Option<String>,
    pub(super) triage: Option<PathBuf>,
    pub(super) symbols: Option<PathBuf>,
    pub(super) include_tests: bool,
    pub(super) production: bool,
    pub(super) excludes: Vec<String>,
    pub(super) auto_excludes: Vec<String>,
}

#[derive(Default)]
pub(super) struct ManifestEvidenceSummaryArgs {
    pub(super) root: Option<String>,
    pub(super) output: Option<PathBuf>,
    pub(super) include_tests: bool,
    pub(super) production: bool,
    pub(super) excludes: Vec<String>,
    pub(super) auto_excludes: Vec<String>,
    pub(super) generated_artifacts_mode: GeneratedArtifactsMode,
}

#[derive(Default)]
pub(super) struct OrchestrationPlanArgs {
    pub(super) profile: AuditProfile,
    pub(super) sarif: bool,
    pub(super) pre_write: bool,
    pub(super) post_write: bool,
    pub(super) canon_draft: bool,
    pub(super) check_canon: bool,
    pub(super) rust_analyzer: bool,
}
