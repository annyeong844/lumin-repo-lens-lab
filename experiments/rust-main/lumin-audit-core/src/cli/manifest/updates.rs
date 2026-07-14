use anyhow::{bail, Context, Result};
use std::path::Path;

use lumin_audit_core::manifest_companion::{
    build_manifest_companion_update, ManifestCompanionUpdateInput,
};
use lumin_audit_core::manifest_core::{summarize_manifest_core, ManifestCoreOptions};
use lumin_audit_core::manifest_final::{
    build_manifest_artifacts_produced_update, build_manifest_closeout_update,
    build_manifest_final_summary_update, build_manifest_final_summary_update_for_rust_analysis,
};
use lumin_audit_core::manifest_root::{
    build_manifest_evidence_update, ManifestEvidenceUpdateInput,
};

use super::protocol::ManifestCloseoutUpdateCliInput;
use crate::cli::args::ManifestCoreSummaryArgs;
use crate::cli::io_support::{
    read_json_input, read_optional_json, read_optional_json_input, read_required_json, take_path,
    take_string, write_stdout_json,
};
use crate::cli::usage::USAGE;

pub(in crate::cli) fn run_manifest_evidence_update(args: Vec<String>) -> Result<()> {
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

pub(in crate::cli) fn run_manifest_companion_update(args: Vec<String>) -> Result<()> {
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

pub(in crate::cli) fn run_manifest_artifacts_produced_update(args: Vec<String>) -> Result<()> {
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

pub(in crate::cli) fn run_manifest_final_summary_update(args: Vec<String>) -> Result<()> {
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

pub(in crate::cli) fn run_manifest_closeout_update(args: Vec<String>) -> Result<()> {
    let mut input = None;
    let mut args = args.into_iter();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--input" => input = Some(take_string(&mut args, "--input")?),
            _ => bail!("manifest-closeout-update: unknown argument '{arg}'\n{USAGE}"),
        }
    }

    let input = input.context("manifest-closeout-update: missing --input <path|->")?;
    let json = read_json_input(&input, "manifest-closeout-update")?;
    let request = serde_json::from_value::<ManifestCloseoutUpdateCliInput>(json)
        .context("manifest-closeout-update: invalid request shape")?;
    let producer_performance = read_required_json(
        Path::new(&request.producer_performance_path),
        "manifest-closeout-update",
    )?;
    let update = build_manifest_closeout_update(
        Path::new(&request.output),
        &producer_performance,
        request.rust_analysis.as_ref(),
        request.companion,
    )?;
    write_stdout_json(&update)
}

pub(in crate::cli) fn run_manifest_core_summary(args: Vec<String>) -> Result<()> {
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
