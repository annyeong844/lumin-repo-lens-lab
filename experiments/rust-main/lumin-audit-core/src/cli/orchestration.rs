use anyhow::{bail, Context, Result};

use super::args::OrchestrationPlanArgs;
use super::io_support::{
    read_json_input, read_required_json, take_path, take_string, write_stdout_json,
};
use super::usage::USAGE;
use lumin_audit_core::generated_artifacts::GeneratedArtifactsMode;
use lumin_audit_core::living_audit::summarize_living_audit;
use lumin_audit_core::orchestration_events::{
    build_producer_performance_artifact,
    build_producer_performance_artifact_for_audit_run_from_output,
    build_producer_performance_artifact_from_runtime, OrchestrationLedger,
    ProducerPerformanceAuditRunContext, ProducerPerformanceRuntimeInput,
    ProducerPerformanceRuntimeObservations,
};
use lumin_audit_core::orchestration_executor::{
    execute_base_plan, execute_runtime_request, ExecutorRequest, RuntimeExecutorRequest,
};
use lumin_audit_core::orchestration_plan::{
    build_orchestration_plan, AuditProfile, OrchestrationPlanOptions,
};
use lumin_audit_core::orchestration_result::summarize_orchestration_result;
use lumin_audit_core::producer_performance::summarize_producer_performance;

pub(super) fn run_producer_performance_summary(args: Vec<String>) -> Result<()> {
    let mut artifact = None;
    let mut args = args.into_iter();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--artifact" => artifact = Some(take_path(&mut args, "--artifact")?),
            _ => bail!("producer-performance-summary: unknown argument '{arg}'\n{USAGE}"),
        }
    }

    let artifact = artifact.context("producer-performance-summary: missing --artifact <path>")?;
    let artifact_json = read_required_json(&artifact, "producer-performance-summary")?;
    let summary = summarize_producer_performance(&artifact_json);
    write_stdout_json(&summary)
}

pub(super) fn run_producer_performance_artifact(args: Vec<String>) -> Result<()> {
    let mut input = None;
    let mut args = args.into_iter();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--input" => input = Some(take_string(&mut args, "--input")?),
            _ => bail!("producer-performance-artifact: unknown argument '{arg}'\n{USAGE}"),
        }
    }

    let input = input.context("producer-performance-artifact: missing --input <path|->")?;
    let ledger_json = read_json_input(&input, "producer-performance-artifact")?;
    let ledger = serde_json::from_value::<OrchestrationLedger>(ledger_json)
        .context("producer-performance-artifact: invalid ledger shape")?;
    let artifact = build_producer_performance_artifact(ledger)?;
    write_stdout_json(&artifact)
}

pub(super) fn run_producer_performance_runtime_artifact(args: Vec<String>) -> Result<()> {
    let mut input = None;
    let mut args = args.into_iter();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--input" => input = Some(take_string(&mut args, "--input")?),
            _ => bail!("producer-performance-runtime-artifact: unknown argument '{arg}'\n{USAGE}"),
        }
    }

    let input = input.context("producer-performance-runtime-artifact: missing --input <path|->")?;
    let input_json = read_json_input(&input, "producer-performance-runtime-artifact")?;
    let input = serde_json::from_value::<ProducerPerformanceRuntimeInput>(input_json)
        .context("producer-performance-runtime-artifact: invalid input shape")?;
    let artifact = build_producer_performance_artifact_from_runtime(input)?;
    write_stdout_json(&artifact)
}

pub(super) fn run_producer_performance_audit_run_artifact(args: Vec<String>) -> Result<()> {
    let mut input = None;
    let mut generated = None;
    let mut root = None;
    let mut output = None;
    let mut profile = None;
    let mut include_tests = true;
    let mut production = false;
    let mut excludes = Vec::new();
    let mut auto_excludes = Vec::new();
    let mut no_incremental = false;
    let mut cache_root = None;
    let mut clear_incremental_cache = false;
    let mut generated_artifacts_mode = GeneratedArtifactsMode::Default;

    let mut args = args.into_iter();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--input" => input = Some(take_string(&mut args, "--input")?),
            "--generated" => generated = Some(take_string(&mut args, "--generated")?),
            "--root" => root = Some(take_string(&mut args, "--root")?),
            "--output" => output = Some(take_string(&mut args, "--output")?),
            "--profile" => profile = Some(take_string(&mut args, "--profile")?),
            "--include-tests" => include_tests = true,
            "--no-include-tests" => include_tests = false,
            "--production" => production = true,
            "--no-production" => production = false,
            "--exclude" => excludes.push(take_string(&mut args, "--exclude")?),
            "--auto-exclude" => auto_excludes.push(take_string(&mut args, "--auto-exclude")?),
            "--no-incremental" => no_incremental = true,
            "--cache-root" => cache_root = Some(take_string(&mut args, "--cache-root")?),
            "--clear-incremental-cache" => clear_incremental_cache = true,
            "--generated-artifacts" => {
                let mode = take_string(&mut args, "--generated-artifacts")?;
                generated_artifacts_mode = GeneratedArtifactsMode::parse(&mode)?;
            }
            _ => {
                bail!("producer-performance-audit-run-artifact: unknown argument '{arg}'\n{USAGE}")
            }
        }
    }

    let input =
        input.context("producer-performance-audit-run-artifact: missing --input <path|->")?;
    let observations_json = read_json_input(&input, "producer-performance-audit-run-artifact")?;
    let observations =
        serde_json::from_value::<ProducerPerformanceRuntimeObservations>(observations_json)
            .context(
                "producer-performance-audit-run-artifact: invalid runtime observation shape",
            )?;
    let context = ProducerPerformanceAuditRunContext {
        generated: generated
            .context("producer-performance-audit-run-artifact: missing --generated <iso>")?,
        root: root.context("producer-performance-audit-run-artifact: missing --root <repo>")?,
        output: output
            .context("producer-performance-audit-run-artifact: missing --output <dir>")?,
        profile: profile.context(
            "producer-performance-audit-run-artifact: missing --profile <quick|full|ci>",
        )?,
        include_tests,
        production,
        excludes,
        auto_excludes,
        no_incremental,
        cache_root: cache_root
            .context("producer-performance-audit-run-artifact: missing --cache-root <dir>")?,
        clear_incremental_cache,
        generated_artifacts_mode: generated_artifacts_mode.as_str().to_string(),
    };
    let artifact =
        build_producer_performance_artifact_for_audit_run_from_output(context, observations)?;
    write_stdout_json(&artifact)
}

pub(super) fn run_orchestration_result_summary(args: Vec<String>) -> Result<()> {
    let mut artifact = None;
    let mut args = args.into_iter();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--artifact" => artifact = Some(take_path(&mut args, "--artifact")?),
            _ => bail!("orchestration-result-summary: unknown argument '{arg}'\n{USAGE}"),
        }
    }

    let artifact = artifact.context("orchestration-result-summary: missing --artifact <path>")?;
    let artifact_json = read_required_json(&artifact, "orchestration-result-summary")?;
    let summary = summarize_orchestration_result(&artifact_json);
    write_stdout_json(&summary)
}

pub(super) fn run_execute_base_plan(args: Vec<String>) -> Result<()> {
    let mut input = None;
    let mut args = args.into_iter();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--input" => input = Some(take_string(&mut args, "--input")?),
            _ => bail!("execute-base-plan: unknown argument '{arg}'\n{USAGE}"),
        }
    }

    let input = input.context("execute-base-plan: missing --input <path|->")?;
    let json = read_json_input(&input, "execute-base-plan")?;
    let request = serde_json::from_value::<ExecutorRequest>(json)
        .context("execute-base-plan: invalid request shape")?;
    let result = execute_base_plan(request)?;
    write_stdout_json(&result)
}

pub(super) fn run_execute_base_runtime(args: Vec<String>) -> Result<()> {
    let mut input = None;
    let mut args = args.into_iter();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--input" => input = Some(take_string(&mut args, "--input")?),
            _ => bail!("execute-base-runtime: unknown argument '{arg}'\n{USAGE}"),
        }
    }

    let input = input.context("execute-base-runtime: missing --input <path|->")?;
    let json = read_json_input(&input, "execute-base-runtime")?;
    let request = serde_json::from_value::<RuntimeExecutorRequest>(json)
        .context("execute-base-runtime: invalid request shape")?;
    let result = execute_runtime_request(request)?;
    write_stdout_json(&result)
}

pub(super) fn run_orchestration_plan(args: Vec<String>) -> Result<()> {
    let mut parsed = OrchestrationPlanArgs {
        profile: AuditProfile::Quick,
        ..OrchestrationPlanArgs::default()
    };
    let mut args = args.into_iter();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--profile" => {
                let profile = take_string(&mut args, "--profile")?;
                parsed.profile = AuditProfile::parse(&profile)?;
            }
            "--sarif" => parsed.sarif = true,
            "--pre-write" => parsed.pre_write = true,
            "--post-write" => parsed.post_write = true,
            "--canon-draft" => parsed.canon_draft = true,
            "--check-canon" => parsed.check_canon = true,
            "--rust-analyzer" => parsed.rust_analyzer = true,
            _ => bail!("orchestration-plan: unknown argument '{arg}'\n{USAGE}"),
        }
    }

    let plan = build_orchestration_plan(OrchestrationPlanOptions {
        profile: parsed.profile,
        sarif: parsed.sarif,
        pre_write: parsed.pre_write,
        post_write: parsed.post_write,
        canon_draft: parsed.canon_draft,
        check_canon: parsed.check_canon,
        rust_analyzer: parsed.rust_analyzer,
    });
    write_stdout_json(&plan)
}

pub(super) fn run_living_audit_summary(args: Vec<String>) -> Result<()> {
    let mut root = None;
    let mut args = args.into_iter();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--root" => root = Some(take_path(&mut args, "--root")?),
            _ => bail!("living-audit-summary: unknown argument '{arg}'\n{USAGE}"),
        }
    }

    let root = root.context("living-audit-summary: missing --root <repo>")?;
    let summary = summarize_living_audit(&root);
    write_stdout_json(&summary)
}
