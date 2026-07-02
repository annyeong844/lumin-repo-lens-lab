use anyhow::{bail, Context, Result};
use serde::Serialize;

use super::args::{ManifestCoreSummaryArgs, ManifestEvidenceSummaryArgs};
use super::io_support::{
    read_json_input, read_optional_json, read_optional_json_input,
    read_optional_output_json_observed, read_optional_output_json_tolerant_observed,
    read_required_json, take_path, take_string, write_stdout_json, OptionalOutputJsonRead,
};
use super::usage::USAGE;
use lumin_audit_core::artifact_read_metrics::{
    ArtifactReadObservation, ARTIFACT_READ_EVENTS_SCHEMA_VERSION,
};
use lumin_audit_core::generated_artifacts::GeneratedArtifactsMode;
use lumin_audit_core::manifest_companion::{
    build_manifest_companion_update, ManifestCompanionUpdateInput,
};
use lumin_audit_core::manifest_core::{summarize_manifest_core, ManifestCoreOptions};
use lumin_audit_core::manifest_evidence::{
    summarize_manifest_evidence, ManifestEvidenceArtifacts, ManifestEvidenceOptions,
    ManifestEvidenceSummary,
};
use lumin_audit_core::manifest_final::{
    build_manifest_artifacts_produced_update, build_manifest_final_summary_update,
    build_manifest_final_summary_update_for_rust_analysis,
};
use lumin_audit_core::manifest_meta::{build_manifest_meta, ManifestMetaInput};
use lumin_audit_core::manifest_root::{
    build_manifest_evidence_update, build_manifest_root, ManifestEvidenceUpdateFields,
    ManifestEvidenceUpdateInput, ManifestRootInput,
};
use lumin_audit_core::rust_analysis::RustAnalysisRunObservation;

const MANIFEST_EVIDENCE_WITH_READS_SCHEMA_VERSION: &str =
    "lumin-manifest-evidence-with-artifact-reads.v1";

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ManifestEvidenceWithArtifactReads<T: Serialize> {
    schema_version: &'static str,
    evidence: T,
    artifact_reads: ManifestEvidenceArtifactReadEvents,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ManifestEvidenceArtifactReadEvents {
    schema_version: &'static str,
    root_dir: String,
    reads: Vec<ArtifactReadObservation>,
}

struct ManifestEvidenceSummaryWithReads {
    summary: ManifestEvidenceSummary,
    artifact_reads: ManifestEvidenceArtifactReadEvents,
}

pub(super) fn run_manifest_meta(args: Vec<String>) -> Result<()> {
    let mut generated = None;
    let mut profile = None;
    let mut root = None;
    let mut output = None;
    let mut args = args.into_iter();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--generated" => generated = Some(take_string(&mut args, "--generated")?),
            "--profile" => profile = Some(take_string(&mut args, "--profile")?),
            "--root" => root = Some(take_string(&mut args, "--root")?),
            "--output" => output = Some(take_string(&mut args, "--output")?),
            _ => bail!("manifest-meta: unknown argument '{arg}'\n{USAGE}"),
        }
    }

    let meta = build_manifest_meta(ManifestMetaInput {
        generated: generated.context("manifest-meta: missing --generated <iso>")?,
        profile: profile.context("manifest-meta: missing --profile <quick|full|ci>")?,
        root: root.context("manifest-meta: missing --root <repo>")?,
        output: output.context("manifest-meta: missing --output <dir>")?,
    })?;
    write_stdout_json(&meta)
}

pub(super) fn run_manifest_root(args: Vec<String>) -> Result<()> {
    let mut input = None;
    let mut args = args.into_iter();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--input" => input = Some(take_string(&mut args, "--input")?),
            _ => bail!("manifest-root: unknown argument '{arg}'\n{USAGE}"),
        }
    }

    let input = input.context("manifest-root: missing --input <path|->")?;
    let json = read_json_input(&input, "manifest-root")?;
    let request = serde_json::from_value::<ManifestRootInput>(json)
        .context("manifest-root: invalid request shape")?;
    let manifest = build_manifest_root(request)?;
    write_stdout_json(&manifest)
}

pub(super) fn run_manifest_evidence_update(args: Vec<String>) -> Result<()> {
    let mut input = None;
    let mut args = args.into_iter();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--input" => input = Some(take_string(&mut args, "--input")?),
            _ => bail!("manifest-evidence-update: unknown argument '{arg}'\n{USAGE}"),
        }
    }

    let input = input.context("manifest-evidence-update: missing --input <path|->")?;
    let json = read_json_input(&input, "manifest-evidence-update")?;
    let request = serde_json::from_value::<ManifestEvidenceUpdateInput>(json)
        .context("manifest-evidence-update: invalid request shape")?;
    let update = build_manifest_evidence_update(request);
    write_stdout_json(&update)
}

pub(super) fn run_manifest_companion_update(args: Vec<String>) -> Result<()> {
    let mut input = None;
    let mut args = args.into_iter();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--input" => input = Some(take_string(&mut args, "--input")?),
            _ => bail!("manifest-companion-update: unknown argument '{arg}'\n{USAGE}"),
        }
    }

    let input = input.context("manifest-companion-update: missing --input <path|->")?;
    let json = read_json_input(&input, "manifest-companion-update")?;
    if !json.is_object() {
        bail!("manifest-companion-update: invalid request shape");
    }
    let request = serde_json::from_value::<ManifestCompanionUpdateInput>(json)
        .context("manifest-companion-update: invalid request shape")?;
    let update = build_manifest_companion_update(request)?;
    write_stdout_json(&update)
}

pub(super) fn run_manifest_artifacts_produced_update(args: Vec<String>) -> Result<()> {
    let mut output = None;
    let mut rust_analysis_block = None;
    let mut args = args.into_iter();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--output" => output = Some(take_path(&mut args, "--output")?),
            "--rust-analysis-block" => {
                rust_analysis_block = Some(take_string(&mut args, "--rust-analysis-block")?)
            }
            _ => bail!("manifest-artifacts-produced-update: unknown argument '{arg}'\n{USAGE}"),
        }
    }

    let output = output.context("manifest-artifacts-produced-update: missing --output <dir>")?;
    let rust_analysis_block =
        read_optional_json_input(rust_analysis_block, "manifest-artifacts-produced-update")?;
    let update = build_manifest_artifacts_produced_update(&output, rust_analysis_block.as_ref())?;
    write_stdout_json(&update)
}

pub(super) fn run_manifest_final_summary_update(args: Vec<String>) -> Result<()> {
    let mut output = None;
    let mut producer_performance = None;
    let mut rust_analysis_ran = false;
    let mut rust_analysis_block = None;
    let mut args = args.into_iter();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--output" => output = Some(take_path(&mut args, "--output")?),
            "--producer-performance" => {
                producer_performance = Some(take_path(&mut args, "--producer-performance")?)
            }
            "--rust-analysis-ran" => rust_analysis_ran = true,
            "--rust-analysis-block" => {
                rust_analysis_block = Some(take_string(&mut args, "--rust-analysis-block")?)
            }
            _ => bail!("manifest-final-summary-update: unknown argument '{arg}'\n{USAGE}"),
        }
    }

    let output = output.context("manifest-final-summary-update: missing --output <dir>")?;
    let producer_performance = producer_performance
        .context("manifest-final-summary-update: missing --producer-performance <path>")?;
    if rust_analysis_ran && rust_analysis_block.is_some() {
        bail!(
            "manifest-final-summary-update: use either --rust-analysis-ran or --rust-analysis-block, not both"
        );
    }
    let artifact = read_required_json(&producer_performance, "manifest-final-summary-update")?;
    let rust_analysis_block =
        read_optional_json_input(rust_analysis_block, "manifest-final-summary-update")?;
    let update = match rust_analysis_block.as_ref() {
        Some(rust_analysis) => build_manifest_final_summary_update_for_rust_analysis(
            &output,
            &artifact,
            Some(rust_analysis),
        )?,
        None => build_manifest_final_summary_update(&output, &artifact, rust_analysis_ran)?,
    };
    write_stdout_json(&update)
}

pub(super) fn run_manifest_core_summary(args: Vec<String>) -> Result<()> {
    let mut parsed = ManifestCoreSummaryArgs {
        include_tests: true,
        production: false,
        ..ManifestCoreSummaryArgs::default()
    };
    let mut args = args.into_iter();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--root" => parsed.root = Some(take_string(&mut args, "--root")?),
            "--triage" => parsed.triage = Some(take_path(&mut args, "--triage")?),
            "--symbols" => parsed.symbols = Some(take_path(&mut args, "--symbols")?),
            "--include-tests" => parsed.include_tests = true,
            "--no-include-tests" => parsed.include_tests = false,
            "--production" => parsed.production = true,
            "--no-production" => parsed.production = false,
            "--exclude" => parsed.excludes.push(take_string(&mut args, "--exclude")?),
            "--auto-exclude" => parsed
                .auto_excludes
                .push(take_string(&mut args, "--auto-exclude")?),
            _ => bail!("manifest-core-summary: unknown argument '{arg}'\n{USAGE}"),
        }
    }

    let root = parsed
        .root
        .context("manifest-core-summary: missing --root <repo>")?;
    let triage = read_optional_json(parsed.triage, "manifest-core-summary")?;
    let symbols = read_optional_json(parsed.symbols, "manifest-core-summary")?;
    let summary = summarize_manifest_core(
        ManifestCoreOptions {
            root,
            include_tests: parsed.include_tests,
            production: parsed.production,
            excludes: parsed.excludes,
            auto_excludes: parsed.auto_excludes,
        },
        triage.as_ref(),
        symbols.as_ref(),
    );
    write_stdout_json(&summary)
}

pub(super) fn run_manifest_evidence_summary(args: Vec<String>) -> Result<()> {
    let summary = read_manifest_evidence_summary(args, "manifest-evidence-summary")?;
    write_stdout_json(&summary)
}

pub(super) fn run_manifest_evidence_summary_with_reads(args: Vec<String>) -> Result<()> {
    let summary =
        read_manifest_evidence_summary_with_reads(args, "manifest-evidence-summary-with-reads")?;
    write_stdout_json(&ManifestEvidenceWithArtifactReads {
        schema_version: MANIFEST_EVIDENCE_WITH_READS_SCHEMA_VERSION,
        evidence: summary.summary,
        artifact_reads: summary.artifact_reads,
    })
}

pub(super) fn run_manifest_evidence_refresh(args: Vec<String>) -> Result<()> {
    let summary = read_manifest_evidence_summary(args, "manifest-evidence-refresh")?;
    let evidence = serde_json::from_value::<ManifestEvidenceUpdateFields>(
        serde_json::to_value(summary)
            .context("manifest-evidence-refresh: invalid summary shape")?,
    )
    .context("manifest-evidence-refresh: invalid evidence update shape")?;
    let update = build_manifest_evidence_update(ManifestEvidenceUpdateInput { evidence });
    write_stdout_json(&update)
}

pub(super) fn run_manifest_evidence_refresh_with_reads(args: Vec<String>) -> Result<()> {
    let summary =
        read_manifest_evidence_summary_with_reads(args, "manifest-evidence-refresh-with-reads")?;
    let evidence = serde_json::from_value::<ManifestEvidenceUpdateFields>(
        serde_json::to_value(summary.summary)
            .context("manifest-evidence-refresh-with-reads: invalid summary shape")?,
    )
    .context("manifest-evidence-refresh-with-reads: invalid evidence update shape")?;
    let update = build_manifest_evidence_update(ManifestEvidenceUpdateInput { evidence });
    write_stdout_json(&ManifestEvidenceWithArtifactReads {
        schema_version: MANIFEST_EVIDENCE_WITH_READS_SCHEMA_VERSION,
        evidence: update,
        artifact_reads: summary.artifact_reads,
    })
}

fn read_manifest_evidence_summary(
    args: Vec<String>,
    label: &str,
) -> Result<ManifestEvidenceSummary> {
    Ok(read_manifest_evidence_summary_with_reads(args, label)?.summary)
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
    let rust_analysis_run = read_optional_json_input(parsed.rust_analysis_run_block, label)?
        .map(serde_json::from_value::<RustAnalysisRunObservation>)
        .transpose()
        .with_context(|| format!("{label}: invalid --rust-analysis-run-block shape"))?;

    let summary = summarize_manifest_evidence(
        ManifestEvidenceOptions {
            root,
            include_tests: parsed.include_tests,
            production: parsed.production,
            excludes: parsed.excludes,
            auto_excludes: parsed.auto_excludes,
            generated_artifacts_mode: parsed.generated_artifacts_mode,
            rust_analysis_ran: parsed.rust_analysis_ran,
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
