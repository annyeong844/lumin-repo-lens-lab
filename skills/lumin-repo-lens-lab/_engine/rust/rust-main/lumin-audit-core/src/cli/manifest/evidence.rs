use anyhow::{bail, Context, Result};
use std::path::PathBuf;

use lumin_audit_core::artifact_read_metrics::{
    ArtifactReadObservation, ARTIFACT_READ_EVENTS_SCHEMA_VERSION,
};
use lumin_audit_core::generated_artifacts::GeneratedArtifactsMode;
use lumin_audit_core::manifest_evidence::{
    summarize_manifest_evidence, ManifestEvidenceArtifacts, ManifestEvidenceOptions,
    ManifestEvidenceSummary,
};
use lumin_audit_core::manifest_root::{
    build_manifest_evidence_update, ManifestEvidenceUpdateFields, ManifestEvidenceUpdateInput,
};
use lumin_audit_core::rust_analysis::RustAnalysisRunObservation;

use super::protocol::{
    ManifestEvidenceArtifactReadEvents, ManifestEvidenceSummaryWithReads,
    ManifestEvidenceWithArtifactReads,
};
use super::write_json_result;
use crate::cli::args::ManifestEvidenceSummaryArgs;
use crate::cli::io_support::{
    read_optional_json_input, read_optional_output_json_observed,
    read_optional_output_json_tolerant_observed, take_path, take_string, write_stdout_json,
    OptionalOutputJsonRead,
};
use crate::cli::usage::USAGE;

const MANIFEST_EVIDENCE_WITH_READS_SCHEMA_VERSION: &str =
    "lumin-manifest-evidence-with-artifact-reads.v1";

pub(super) struct ManifestEvidenceReadRequest {
    pub(super) root: String,
    pub(super) output: PathBuf,
    pub(super) include_tests: bool,
    pub(super) production: bool,
    pub(super) excludes: Vec<String>,
    pub(super) auto_excludes: Vec<String>,
    pub(super) generated_artifacts_mode: GeneratedArtifactsMode,
    pub(super) rust_analysis_ran: bool,
    pub(super) rust_analysis_run: Option<RustAnalysisRunObservation>,
    pub(super) label: String,
}

pub(in crate::cli) fn run_manifest_evidence_summary(args: Vec<String>) -> Result<()> {
    let summary = read_manifest_evidence_summary(args, "manifest-evidence-summary")?;
    write_stdout_json(&summary)
}

pub(in crate::cli) fn run_manifest_evidence_summary_with_reads(args: Vec<String>) -> Result<()> {
    let summary =
        read_manifest_evidence_summary_with_reads(args, "manifest-evidence-summary-with-reads")?;
    write_json_result(
        summary.result_output,
        &ManifestEvidenceWithArtifactReads {
            schema_version: MANIFEST_EVIDENCE_WITH_READS_SCHEMA_VERSION,
            evidence: summary.summary,
            artifact_reads: summary.artifact_reads,
        },
    )
}

pub(in crate::cli) fn run_manifest_evidence_refresh(args: Vec<String>) -> Result<()> {
    let summary = read_manifest_evidence_summary(args, "manifest-evidence-refresh")?;
    let evidence = serde_json::from_value::<ManifestEvidenceUpdateFields>(
        serde_json::to_value(summary)
            .context("manifest-evidence-refresh: invalid summary shape")?,
    )
    .context("manifest-evidence-refresh: invalid evidence update shape")?;
    let update = build_manifest_evidence_update(ManifestEvidenceUpdateInput { evidence });
    write_stdout_json(&update)
}

pub(in crate::cli) fn run_manifest_evidence_refresh_with_reads(args: Vec<String>) -> Result<()> {
    let summary =
        read_manifest_evidence_summary_with_reads(args, "manifest-evidence-refresh-with-reads")?;
    let evidence = serde_json::from_value::<ManifestEvidenceUpdateFields>(
        serde_json::to_value(summary.summary)
            .context("manifest-evidence-refresh-with-reads: invalid summary shape")?,
    )
    .context("manifest-evidence-refresh-with-reads: invalid evidence update shape")?;
    let update = build_manifest_evidence_update(ManifestEvidenceUpdateInput { evidence });
    write_json_result(
        summary.result_output,
        &ManifestEvidenceWithArtifactReads {
            schema_version: MANIFEST_EVIDENCE_WITH_READS_SCHEMA_VERSION,
            evidence: update,
            artifact_reads: summary.artifact_reads,
        },
    )
}

fn read_manifest_evidence_summary(
    args: Vec<String>,
    label: &str,
) -> Result<ManifestEvidenceSummary> {
    let result = read_manifest_evidence_summary_with_reads(args, label)?;
    if result.result_output.is_some() {
        bail!("{label}: --result-output is only supported by with-reads commands");
    }
    Ok(result.summary)
}

fn read_manifest_evidence_summary_with_reads(
    args: Vec<String>,
    label: &str,
) -> Result<ManifestEvidenceSummaryWithReads> {
    let mut parsed = ManifestEvidenceSummaryArgs {
        include_tests: true,
        production: false,
        generated_artifacts_mode: GeneratedArtifactsMode::Default,
        ..ManifestEvidenceSummaryArgs::default()
    };
    let mut args = args.into_iter();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--root" => parsed.root = Some(take_string(&mut args, "--root")?),
            "--output" => parsed.output = Some(take_path(&mut args, "--output")?),
            "--result-output" => {
                parsed.result_output = Some(take_path(&mut args, "--result-output")?)
            }
            "--generated-artifacts" => {
                let mode = take_string(&mut args, "--generated-artifacts")?;
                parsed.generated_artifacts_mode = GeneratedArtifactsMode::parse(&mode)?;
            }
            "--include-tests" => parsed.include_tests = true,
            "--no-include-tests" => parsed.include_tests = false,
            "--production" => parsed.production = true,
            "--no-production" => parsed.production = false,
            "--rust-analysis-ran" => parsed.rust_analysis_ran = true,
            "--rust-analysis-run-block" => {
                parsed.rust_analysis_run_block =
                    Some(take_string(&mut args, "--rust-analysis-run-block")?)
            }
            "--exclude" => parsed.excludes.push(take_string(&mut args, "--exclude")?),
            "--auto-exclude" => parsed
                .auto_excludes
                .push(take_string(&mut args, "--auto-exclude")?),
            _ => bail!("{label}: unknown argument '{arg}'\n{USAGE}"),
        }
    }

    let root = parsed
        .root
        .with_context(|| format!("{label}: missing --root <repo>"))?;
    let output = parsed
        .output
        .with_context(|| format!("{label}: missing --output <dir>"))?;
    let rust_analysis_run = read_optional_json_input(parsed.rust_analysis_run_block, label)?
        .map(serde_json::from_value::<RustAnalysisRunObservation>)
        .transpose()
        .with_context(|| format!("{label}: invalid --rust-analysis-run-block shape"))?;
    let mut summary = build_manifest_evidence_summary_with_reads(ManifestEvidenceReadRequest {
        root,
        output,
        include_tests: parsed.include_tests,
        production: parsed.production,
        excludes: parsed.excludes,
        auto_excludes: parsed.auto_excludes,
        generated_artifacts_mode: parsed.generated_artifacts_mode,
        rust_analysis_ran: parsed.rust_analysis_ran,
        rust_analysis_run,
        label: label.to_string(),
    })?;
    summary.result_output = parsed.result_output;
    Ok(summary)
}

pub(super) fn build_manifest_evidence_summary_with_reads(
    request: ManifestEvidenceReadRequest,
) -> Result<ManifestEvidenceSummaryWithReads> {
    let ManifestEvidenceReadRequest {
        root,
        output,
        include_tests,
        production,
        excludes,
        auto_excludes,
        generated_artifacts_mode,
        rust_analysis_ran,
        rust_analysis_run,
        label,
    } = request;
    let label = label.as_str();
    let mut artifact_reads = Vec::new();
    let triage = artifact_value(
        read_optional_output_json_observed(&output, "triage.json", label)?,
        &mut artifact_reads,
    );
    let symbols = artifact_value(
        read_optional_output_json_observed(&output, "symbols.json", label)?,
        &mut artifact_reads,
    );
    let resolver_capabilities = artifact_value(
        read_optional_output_json_tolerant_observed(&output, "resolver-capabilities.json"),
        &mut artifact_reads,
    );
    let resolver_diagnostics = artifact_value(
        read_optional_output_json_tolerant_observed(&output, "resolver-diagnostics.json"),
        &mut artifact_reads,
    );
    let framework_resource_surfaces = artifact_value(
        read_optional_output_json_tolerant_observed(&output, "framework-resource-surfaces.json"),
        &mut artifact_reads,
    );
    let unused_deps = artifact_value(
        read_optional_output_json_tolerant_observed(&output, "unused-deps.json"),
        &mut artifact_reads,
    );
    let block_clones = artifact_value(
        read_optional_output_json_tolerant_observed(&output, "block-clones.json"),
        &mut artifact_reads,
    );
    let dead_classify = artifact_value(
        read_optional_output_json_tolerant_observed(&output, "dead-classify.json"),
        &mut artifact_reads,
    );
    let entry_surface = artifact_value(
        read_optional_output_json_tolerant_observed(&output, "entry-surface.json"),
        &mut artifact_reads,
    );
    let rust_analysis = artifact_value(
        read_optional_output_json_tolerant_observed(&output, "rust-analyzer-health.latest.json"),
        &mut artifact_reads,
    );
    let summary = summarize_manifest_evidence(
        ManifestEvidenceOptions {
            root,
            include_tests,
            production,
            excludes,
            auto_excludes,
            generated_artifacts_mode,
            rust_analysis_ran,
            rust_analysis_run,
        },
        ManifestEvidenceArtifacts {
            triage: triage.as_ref(),
            symbols: symbols.as_ref(),
            resolver_capabilities: resolver_capabilities.as_ref(),
            resolver_diagnostics: resolver_diagnostics.as_ref(),
            framework_resource_surfaces: framework_resource_surfaces.as_ref(),
            unused_deps: unused_deps.as_ref(),
            block_clones: block_clones.as_ref(),
            dead_classify: dead_classify.as_ref(),
            entry_surface: entry_surface.as_ref(),
            rust_analysis: rust_analysis.as_ref(),
        },
    )?;
    Ok(ManifestEvidenceSummaryWithReads {
        summary,
        artifact_reads: ManifestEvidenceArtifactReadEvents {
            schema_version: ARTIFACT_READ_EVENTS_SCHEMA_VERSION,
            root_dir: output.to_string_lossy().to_string(),
            reads: artifact_reads,
        },
        result_output: None,
    })
}

fn artifact_value(
    observed: OptionalOutputJsonRead,
    artifact_reads: &mut Vec<ArtifactReadObservation>,
) -> Option<serde_json::Value> {
    if let Some(observation) = observed.observation {
        artifact_reads.push(observation);
    }
    observed.value
}
