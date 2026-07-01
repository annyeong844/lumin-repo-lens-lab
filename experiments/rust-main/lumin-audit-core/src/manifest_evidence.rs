use serde::Serialize;
use serde_json::Value;
use std::path::PathBuf;

use crate::artifact_summaries::{summarize_artifact, ArtifactSummary, ArtifactSummaryKind};
use crate::blind_zones::{summarize_blind_zones, BlindZoneInput, BlindZoneSummary};
use crate::generated_artifacts::{
    summarize_generated_artifacts, GeneratedArtifactsMode, GeneratedArtifactsOptions,
    GeneratedArtifactsSummary,
};
use crate::living_audit::{summarize_living_audit, LivingAuditSummary};
use crate::manifest_core::{
    summarize_manifest_core, ConfidenceSummary, ManifestCoreOptions, ScanRangeSummary,
    SfcEvidenceSummary,
};
use crate::resolver_diagnostics::{summarize_resolver_diagnostics, ResolverDiagnosticsSummary};
use crate::rust_analysis::{summarize_rust_analysis_artifact, RustAnalysisSummary};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ManifestEvidenceOptions {
    pub root: String,
    pub include_tests: bool,
    pub production: bool,
    pub excludes: Vec<String>,
    pub auto_excludes: Vec<String>,
    pub generated_artifacts_mode: GeneratedArtifactsMode,
    pub rust_analysis_ran: bool,
}

pub struct ManifestEvidenceArtifacts<'a> {
    pub triage: Option<&'a Value>,
    pub symbols: Option<&'a Value>,
    pub resolver_capabilities: Option<&'a Value>,
    pub resolver_diagnostics: Option<&'a Value>,
    pub framework_resource_surfaces: Option<&'a Value>,
    pub unused_deps: Option<&'a Value>,
    pub block_clones: Option<&'a Value>,
    pub dead_classify: Option<&'a Value>,
    pub entry_surface: Option<&'a Value>,
    pub rust_analysis: Option<&'a Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ManifestEvidenceSummary {
    pub scan_range: ScanRangeSummary,
    pub confidence: ConfidenceSummary,
    pub resolver_diagnostics: ResolverDiagnosticsSummary,
    pub blind_zones: Vec<BlindZoneSummary>,
    pub rust_analysis: Option<RustAnalysisSummary>,
    pub generated_artifacts: GeneratedArtifactsSummary,
    pub framework_resource_surfaces: Option<ArtifactSummary>,
    pub unused_dependencies: Option<ArtifactSummary>,
    pub block_clones: Option<ArtifactSummary>,
    pub sfc_evidence: Option<SfcEvidenceSummary>,
    pub living_audit: LivingAuditSummary,
}

pub fn summarize_manifest_evidence(
    options: ManifestEvidenceOptions,
    artifacts: ManifestEvidenceArtifacts<'_>,
) -> ManifestEvidenceSummary {
    let root_path = PathBuf::from(&options.root);
    let rust_analysis_summary = artifacts
        .rust_analysis
        .and_then(|artifact| summarize_rust_analysis_artifact(&root_path, artifact));
    let rust_analysis_value = rust_analysis_summary
        .as_ref()
        .and_then(|summary| serde_json::to_value(summary).ok());
    let manifest_core = summarize_manifest_core(
        ManifestCoreOptions {
            root: options.root.clone(),
            include_tests: options.include_tests,
            production: options.production,
            excludes: options.excludes.clone(),
            auto_excludes: options.auto_excludes,
        },
        artifacts.triage,
        artifacts.symbols,
    );

    ManifestEvidenceSummary {
        scan_range: manifest_core.scan_range,
        confidence: manifest_core.confidence,
        resolver_diagnostics: summarize_resolver_diagnostics(
            artifacts.symbols,
            artifacts.resolver_capabilities,
            artifacts.resolver_diagnostics,
        ),
        blind_zones: summarize_blind_zones(BlindZoneInput {
            triage: artifacts.triage,
            symbols: artifacts.symbols,
            dead_classify: artifacts.dead_classify,
            entry_surface: artifacts.entry_surface,
            resolver_diagnostics: artifacts.resolver_diagnostics,
            rust_analysis: options
                .rust_analysis_ran
                .then_some(rust_analysis_value.as_ref())
                .flatten(),
        }),
        rust_analysis: rust_analysis_summary,
        generated_artifacts: summarize_generated_artifacts(
            &root_path,
            artifacts.symbols,
            &GeneratedArtifactsOptions {
                include_tests: options.include_tests,
                excludes: options.excludes,
                mode: options.generated_artifacts_mode,
            },
        ),
        framework_resource_surfaces: artifacts.framework_resource_surfaces.and_then(|artifact| {
            summarize_artifact(ArtifactSummaryKind::FrameworkResourceSurfaces, artifact)
        }),
        unused_dependencies: artifacts
            .unused_deps
            .and_then(|artifact| summarize_artifact(ArtifactSummaryKind::UnusedDeps, artifact)),
        block_clones: artifacts
            .block_clones
            .and_then(|artifact| summarize_artifact(ArtifactSummaryKind::BlockClones, artifact)),
        sfc_evidence: manifest_core.sfc_evidence,
        living_audit: summarize_living_audit(&root_path),
    }
}
