use anyhow::{bail, Context, Result};
use std::path::Path;

use lumin_audit_core::manifest_final::{
    apply_manifest_closeout_update, build_manifest_closeout_update,
};
use lumin_audit_core::orchestration_events::build_producer_performance_artifact_for_audit_run_from_output;

use super::protocol::{
    FinalizeAuditRunCliInput, FinalizeAuditRunResult, ManifestCloseoutWriteCliInput,
    ManifestCloseoutWriteResult, ManifestWriteCliInput, ManifestWriteResult,
};
use crate::cli::io_support::{
    read_json_input, read_required_json, take_path, take_string, write_pretty_json_file,
    write_stdout_json,
};
use crate::cli::usage::USAGE;

pub(in crate::cli) fn run_manifest_write(args: Vec<String>) -> Result<()> {
    let mut output = None;
    let mut input = None;
    let mut args = args.into_iter();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--output" => output = Some(take_path(&mut args, "--output")?),
            "--input" => input = Some(take_string(&mut args, "--input")?),
            _ => bail!("manifest-write: unknown argument '{arg}'\n{USAGE}"),
        }
    }

    let output = output.context("manifest-write: missing --output <dir>")?;
    let input = input.context("manifest-write: missing --input <path|->")?;
    let json = read_json_input(&input, "manifest-write")?;
    let request = serde_json::from_value::<ManifestWriteCliInput>(json)
        .context("manifest-write: invalid request shape")?;
    if !request.manifest.is_object() {
        bail!("manifest-write: manifest must be a JSON object");
    }
    let manifest_path = output.join("manifest.json");
    write_pretty_json_file(&manifest_path, &request.manifest)?;
    write_stdout_json(&ManifestWriteResult {
        manifest_path: manifest_path.to_string_lossy().to_string(),
    })
}

pub(in crate::cli) fn run_manifest_closeout_write(args: Vec<String>) -> Result<()> {
    let mut input = None;
    let mut args = args.into_iter();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--input" => input = Some(take_string(&mut args, "--input")?),
            _ => bail!("manifest-closeout-write: unknown argument '{arg}'\n{USAGE}"),
        }
    }

    let input = input.context("manifest-closeout-write: missing --input <path|->")?;
    let json = read_json_input(&input, "manifest-closeout-write")?;
    let request = serde_json::from_value::<ManifestCloseoutWriteCliInput>(json)
        .context("manifest-closeout-write: invalid request shape")?;
    let producer_performance = read_required_json(
        Path::new(&request.producer_performance_path),
        "manifest-closeout-write",
    )?;
    let update = build_manifest_closeout_update(
        Path::new(&request.output),
        &producer_performance,
        request.rust_analysis.as_ref(),
        request.companion,
    )?;
    let mut manifest = request.manifest;
    apply_manifest_closeout_update(&mut manifest, update.clone())?;
    let manifest_path = Path::new(&request.output).join("manifest.json");
    write_pretty_json_file(&manifest_path, &manifest)?;
    write_stdout_json(&ManifestCloseoutWriteResult {
        manifest_path: manifest_path.to_string_lossy().to_string(),
        closeout_update: update,
    })
}

pub(in crate::cli) fn run_finalize_audit_run(args: Vec<String>) -> Result<()> {
    let mut input = None;
    let mut args = args.into_iter();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--input" => input = Some(take_string(&mut args, "--input")?),
            _ => bail!("finalize-audit-run: unknown argument '{arg}'\n{USAGE}"),
        }
    }

    let input = input.context("finalize-audit-run: missing --input <path|->")?;
    let json = read_json_input(&input, "finalize-audit-run")?;
    let request = serde_json::from_value::<FinalizeAuditRunCliInput>(json)
        .context("finalize-audit-run: invalid request shape")?;
    let output = Path::new(&request.context.output).to_path_buf();
    let producer_performance = build_producer_performance_artifact_for_audit_run_from_output(
        request.context,
        request.observations,
    )?;
    let producer_performance_path = output.join("producer-performance.json");
    write_pretty_json_file(&producer_performance_path, &producer_performance)?;

    let producer_performance_json = serde_json::to_value(&producer_performance)
        .context("finalize-audit-run: invalid producer-performance shape")?;
    let update = build_manifest_closeout_update(
        &output,
        &producer_performance_json,
        request.rust_analysis.as_ref(),
        request.companion,
    )?;
    let mut manifest = request.manifest;
    apply_manifest_closeout_update(&mut manifest, update.clone())?;
    let manifest_path = output.join("manifest.json");
    write_pretty_json_file(&manifest_path, &manifest)?;
    write_stdout_json(&FinalizeAuditRunResult {
        producer_performance_path: producer_performance_path.to_string_lossy().to_string(),
        manifest_path: manifest_path.to_string_lossy().to_string(),
        closeout_update: update,
    })
}
