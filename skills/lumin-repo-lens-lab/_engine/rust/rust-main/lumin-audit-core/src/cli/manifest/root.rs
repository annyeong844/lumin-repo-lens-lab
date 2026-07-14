use anyhow::{bail, Context, Result};
use std::path::PathBuf;

use lumin_audit_core::lifecycle::{
    apply_manifest_lifecycle_update, build_manifest_lifecycle_update,
};
use lumin_audit_core::manifest_meta::{build_manifest_meta, ManifestMetaInput};
use lumin_audit_core::manifest_root::{
    apply_manifest_evidence_update, build_manifest_root, ManifestRootInput,
};

use super::base_evidence::{
    manifest_evidence_for_base_pipeline, mark_base_evidence_not_refreshed,
    required_base_pipeline_skip_reason, BasePipelineEvidenceRequest,
};
use super::protocol::{
    ManifestLifecycleEvidenceRefreshCliInput, ManifestLifecycleEvidenceRefreshResult,
    ManifestRootWithEvidenceCliInput, ManifestRootWithEvidenceResult,
};
use super::write_json_result;
use crate::cli::io_support::{read_json_input, take_path, take_string, write_stdout_json};
use crate::cli::usage::USAGE;

pub(in crate::cli) fn run_manifest_meta(args: Vec<String>) -> Result<()> {
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

pub(in crate::cli) fn run_manifest_root(args: Vec<String>) -> Result<()> {
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

pub(in crate::cli) fn run_manifest_lifecycle_evidence_refresh(args: Vec<String>) -> Result<()> {
    let mut input = None;
    let mut result_output = None;
    let mut args = args.into_iter();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--input" => input = Some(take_string(&mut args, "--input")?),
            "--result-output" => result_output = Some(take_path(&mut args, "--result-output")?),
            _ => bail!("manifest-lifecycle-evidence-refresh: unknown argument '{arg}'\n{USAGE}"),
        }
    }

    let input = input.context("manifest-lifecycle-evidence-refresh: missing --input <path|->")?;
    let json = read_json_input(&input, "manifest-lifecycle-evidence-refresh")?;
    let request = serde_json::from_value::<ManifestLifecycleEvidenceRefreshCliInput>(json)
        .context("manifest-lifecycle-evidence-refresh: invalid request shape")?;
    let mut manifest = request.manifest;
    let lifecycle_update = build_manifest_lifecycle_update(request.lifecycle);
    apply_manifest_lifecycle_update(&mut manifest, lifecycle_update)?;
    let base_pipeline_planned = request.evidence.base_pipeline_planned;
    let base_pipeline_skip_reason = request.evidence.base_pipeline_skip_reason.clone();
    let (evidence, artifact_reads) =
        manifest_evidence_for_base_pipeline(BasePipelineEvidenceRequest {
            label: "manifest-lifecycle-evidence-refresh",
            root: request.evidence.root,
            output: request.evidence.output,
            include_tests: request.evidence.include_tests,
            production: request.evidence.production,
            excludes: request.evidence.excludes,
            auto_excludes: request.evidence.auto_excludes,
            generated_artifacts_mode: request.evidence.generated_artifacts_mode,
            rust_analysis_ran: request.evidence.rust_analysis_ran,
            rust_analysis_run: request.evidence.rust_analysis_run,
            planned: base_pipeline_planned,
            skip_reason: base_pipeline_skip_reason.clone(),
        })?;
    apply_manifest_evidence_update(&mut manifest, evidence)?;
    if !base_pipeline_planned {
        let reason = required_base_pipeline_skip_reason(base_pipeline_skip_reason.as_deref())?;
        mark_base_evidence_not_refreshed(&mut manifest, reason, &[])?;
    }
    write_json_result(
        result_output,
        &ManifestLifecycleEvidenceRefreshResult {
            manifest,
            artifact_reads,
        },
    )
}

pub(in crate::cli) fn run_manifest_root_with_evidence(args: Vec<String>) -> Result<()> {
    let mut input = None;
    let mut result_output = None;
    let mut args = args.into_iter();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--input" => input = Some(take_string(&mut args, "--input")?),
            "--result-output" => result_output = Some(take_path(&mut args, "--result-output")?),
            _ => bail!("manifest-root-with-evidence: unknown argument '{arg}'\n{USAGE}"),
        }
    }

    let input = input.context("manifest-root-with-evidence: missing --input <path|->")?;
    let json = read_json_input(&input, "manifest-root-with-evidence")?;
    let request = serde_json::from_value::<ManifestRootWithEvidenceCliInput>(json)
        .context("manifest-root-with-evidence: invalid request shape")?;
    let base_pipeline_planned = request.base_pipeline_planned;
    let base_pipeline_skip_reason = request.base_pipeline_skip_reason.clone();
    let (evidence, artifact_reads) =
        manifest_evidence_for_base_pipeline(BasePipelineEvidenceRequest {
            label: "manifest-root-with-evidence",
            root: request.root.clone(),
            output: PathBuf::from(&request.output),
            include_tests: request.include_tests,
            production: request.production,
            excludes: request.excludes,
            auto_excludes: request.auto_excludes,
            generated_artifacts_mode: request.generated_artifacts_mode,
            rust_analysis_ran: request.rust_analysis_ran,
            rust_analysis_run: request.rust_analysis_run,
            planned: base_pipeline_planned,
            skip_reason: base_pipeline_skip_reason.clone(),
        })?;
    let mut manifest = serde_json::to_value(build_manifest_root(ManifestRootInput {
        generated: request.generated,
        profile: request.profile,
        root: request.root,
        output: request.output,
        commands_run: request.commands_run,
        skipped: request.skipped,
        evidence,
    })?)
    .context("manifest-root-with-evidence: invalid manifest shape")?;
    if !base_pipeline_planned {
        let reason = required_base_pipeline_skip_reason(base_pipeline_skip_reason.as_deref())?;
        mark_base_evidence_not_refreshed(&mut manifest, reason, &[])?;
    }
    write_json_result(
        result_output,
        &ManifestRootWithEvidenceResult {
            manifest,
            artifact_reads,
        },
    )
}
