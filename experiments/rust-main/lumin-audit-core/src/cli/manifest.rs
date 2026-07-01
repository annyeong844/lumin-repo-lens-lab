use anyhow::{bail, Context, Result};

use super::args::{ManifestCoreSummaryArgs, ManifestEvidenceSummaryArgs};
use super::io_support::{
    read_json_input, read_optional_json, read_optional_json_input, read_optional_output_json,
    read_optional_output_json_tolerant, read_required_json, take_path, take_string,
    write_stdout_json,
};
use super::usage::USAGE;
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

fn read_manifest_evidence_summary(
    args: Vec<String>,
    label: &str,
) -> Result<ManifestEvidenceSummary> {
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
    let triage = read_optional_output_json(&output, "triage.json", label)?;
    let symbols = read_optional_output_json(&output, "symbols.json", label)?;
    let resolver_capabilities =
        read_optional_output_json_tolerant(&output, "resolver-capabilities.json");
    let resolver_diagnostics =
        read_optional_output_json_tolerant(&output, "resolver-diagnostics.json");
    let framework_resource_surfaces =
        read_optional_output_json_tolerant(&output, "framework-resource-surfaces.json");
    let unused_deps = read_optional_output_json_tolerant(&output, "unused-deps.json");
    let block_clones = read_optional_output_json_tolerant(&output, "block-clones.json");
    let dead_classify = read_optional_output_json_tolerant(&output, "dead-classify.json");
    let entry_surface = read_optional_output_json_tolerant(&output, "entry-surface.json");
    let rust_analysis =
        read_optional_output_json_tolerant(&output, "rust-analyzer-health.latest.json");
    let rust_analysis_run = read_optional_json_input(parsed.rust_analysis_run_block, label)?
        .map(serde_json::from_value::<RustAnalysisRunObservation>)
        .transpose()
        .with_context(|| format!("{label}: invalid --rust-analysis-run-block shape"))?;

    summarize_manifest_evidence(
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
    )
}
