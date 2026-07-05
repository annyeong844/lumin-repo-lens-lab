use anyhow::{bail, Context, Result};
use serde::Deserialize;
use serde_json::{json, Value};
use std::io::{self, Write};
use std::path::PathBuf;

use super::io_support::{
    read_json_input, take_path, take_string, write_json_file, write_stdout_json,
};
use super::usage::USAGE;
use lumin_audit_core::canon_draft_lifecycle::{
    execute_canon_draft_lifecycle, CanonDraftLifecycleRequest,
};
use lumin_audit_core::check_canon_lifecycle::{
    execute_check_canon_lifecycle, CheckCanonLifecycleRequest,
};
use lumin_audit_core::lifecycle::{
    build_manifest_lifecycle_update, summarize_lifecycle, ManifestLifecycleUpdateInput,
};
use lumin_audit_core::lifecycle_exit_policy::{
    apply_lifecycle_exit_policy, LifecycleExitPolicyRequest,
};
use lumin_audit_core::lifecycle_request::{
    evaluate_lifecycle_request_guard, LifecycleRequestGuardInput, LifecycleRequestGuardStatus,
};
use lumin_audit_core::post_write_lifecycle::{
    execute_post_write_lifecycle, execute_post_write_lifecycle_streaming, PostWriteLifecycleRequest,
};
use lumin_audit_core::pre_write_lifecycle::{
    execute_js_pre_write_lifecycle, execute_js_pre_write_lifecycle_streaming,
    execute_rust_pre_write_lifecycle, execute_rust_pre_write_lifecycle_streaming,
    AnalyzerInvocationRequest, JsPreWriteLifecycleRequest, RustPreWriteLifecycleRequest,
};
use lumin_audit_core::pre_write_routing::{
    resolve_pre_write_route, PreWriteRoutingRequest, PreWriteRoutingResult,
};

const AUDIT_LIFECYCLE_EXECUTION_REQUEST_SCHEMA_VERSION: &str =
    "lumin-audit-lifecycle-execution-request.v1";
const AUDIT_LIFECYCLE_EXECUTION_RESULT_SCHEMA_VERSION: &str =
    "lumin-audit-lifecycle-execution-result.v1";

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AuditLifecycleExecutionRequest {
    schema_version: String,
    base_exit_code: i32,
    lifecycle_request_guard: LifecycleRequestGuardInput,
    #[serde(default)]
    pre_write: Option<AuditLifecyclePreWriteRequest>,
    #[serde(default)]
    post_write: Option<AuditLifecycleStep<PostWriteLifecycleRequest>>,
    #[serde(default)]
    canon_draft: Option<AuditLifecycleStep<CanonDraftLifecycleRequest>>,
    #[serde(default)]
    check_canon: Option<AuditLifecycleStep<CheckCanonLifecycleRequest>>,
    exit_policy: AuditLifecycleExitPolicyOptions,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AuditLifecyclePreWriteRequest {
    #[serde(default)]
    requested: bool,
    #[serde(default)]
    routing: Option<PreWriteRoutingRequest>,
    #[serde(default)]
    routing_failure: Option<String>,
    rust: RustPreWriteLifecycleTemplate,
    js: JsPreWriteLifecycleTemplate,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RustPreWriteLifecycleTemplate {
    root: PathBuf,
    output: PathBuf,
    #[serde(rename = "invocationId")]
    advisory_invocation_id: String,
    rust_native_artifact_path: PathBuf,
    rust_native_latest_path: PathBuf,
    #[serde(default)]
    analyzer: Option<AnalyzerInvocationRequest>,
    include_tests: bool,
    #[serde(default)]
    production: bool,
    #[serde(default)]
    excludes: Vec<String>,
    file_inventory: Value,
    #[serde(default)]
    failures: Vec<Value>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct JsPreWriteLifecycleTemplate {
    root: PathBuf,
    output: PathBuf,
    scripts_dir: PathBuf,
    node_executable: String,
    #[serde(default)]
    no_fresh_audit: bool,
    #[serde(default)]
    scan_args: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AuditLifecycleStep<T> {
    #[serde(default)]
    requested: bool,
    request: T,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AuditLifecycleExitPolicyOptions {
    #[serde(default)]
    strict_post_write: bool,
    #[serde(default)]
    strict_post_write_confidence: bool,
}

pub(super) fn run_lifecycle_summary(args: Vec<String>) -> Result<()> {
    let mut input = None;
    let mut args = args.into_iter();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--input" => input = Some(take_string(&mut args, "--input")?),
            _ => bail!("lifecycle-summary: unknown argument '{arg}'\n{USAGE}"),
        }
    }

    let input = input.context("lifecycle-summary: missing --input <path|->")?;
    let lifecycle_json = read_json_input(&input, "lifecycle-summary")?;
    let summary = summarize_lifecycle(&lifecycle_json);
    write_stdout_json(&summary)
}

pub(super) fn run_manifest_lifecycle_update(args: Vec<String>) -> Result<()> {
    let mut input = None;
    let mut args = args.into_iter();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--input" => input = Some(take_string(&mut args, "--input")?),
            _ => bail!("manifest-lifecycle-update: unknown argument '{arg}'\n{USAGE}"),
        }
    }

    let input = input.context("manifest-lifecycle-update: missing --input <path|->")?;
    let json = read_json_input(&input, "manifest-lifecycle-update")?;
    let request = serde_json::from_value::<ManifestLifecycleUpdateInput>(json)
        .context("manifest-lifecycle-update: invalid request shape")?;
    let update = build_manifest_lifecycle_update(request);
    write_stdout_json(&update)
}

pub(super) fn run_lifecycle_exit_policy(args: Vec<String>) -> Result<()> {
    let mut input = None;
    let mut args = args.into_iter();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--input" => input = Some(take_string(&mut args, "--input")?),
            _ => bail!("lifecycle-exit-policy: unknown argument '{arg}'\n{USAGE}"),
        }
    }

    let input = input.context("lifecycle-exit-policy: missing --input <path|->")?;
    let json = read_json_input(&input, "lifecycle-exit-policy")?;
    let request = serde_json::from_value::<LifecycleExitPolicyRequest>(json)
        .context("lifecycle-exit-policy: invalid request shape")?;
    let result = apply_lifecycle_exit_policy(request)?;
    write_stdout_json(&result)
}

pub(super) fn run_lifecycle_request_guard(args: Vec<String>) -> Result<()> {
    let mut input = None;
    let mut args = args.into_iter();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--input" => input = Some(take_string(&mut args, "--input")?),
            _ => bail!("lifecycle-request-guard: unknown argument '{arg}'\n{USAGE}"),
        }
    }

    let input = input.context("lifecycle-request-guard: missing --input <path|->")?;
    let json = read_json_input(&input, "lifecycle-request-guard")?;
    let request = serde_json::from_value::<LifecycleRequestGuardInput>(json)
        .context("lifecycle-request-guard: invalid request shape")?;
    let result = evaluate_lifecycle_request_guard(request)?;
    write_stdout_json(&result)
}

pub(super) fn run_execute_audit_lifecycle(args: Vec<String>) -> Result<()> {
    let mut input = None;
    let mut result_output = None;
    let mut args = args.into_iter();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--input" => input = Some(take_string(&mut args, "--input")?),
            "--result-output" => result_output = Some(take_path(&mut args, "--result-output")?),
            _ => bail!("execute-audit-lifecycle: unknown argument '{arg}'\n{USAGE}"),
        }
    }

    let input = input.context("execute-audit-lifecycle: missing --input <path|->")?;
    let result_output =
        result_output.context("execute-audit-lifecycle: missing --result-output <path>")?;
    let json = read_json_input(&input, "execute-audit-lifecycle")?;
    let request = serde_json::from_value::<AuditLifecycleExecutionRequest>(json)
        .context("execute-audit-lifecycle: invalid request shape")?;
    let result = execute_audit_lifecycle_request(request)?;
    write_json_file(&result_output, &result)
}

fn execute_audit_lifecycle_request(request: AuditLifecycleExecutionRequest) -> Result<Value> {
    if request.schema_version != AUDIT_LIFECYCLE_EXECUTION_REQUEST_SCHEMA_VERSION {
        bail!(
            "execute-audit-lifecycle: unsupported schemaVersion '{}'",
            request.schema_version
        );
    }

    let mut pre_write_block = Value::Null;
    let mut post_write_block = Value::Null;
    let mut canon_draft_block = Value::Null;
    let mut check_canon_block = Value::Null;
    let mut final_exit_code = request.base_exit_code;

    let lifecycle_request_guard =
        evaluate_lifecycle_request_guard(request.lifecycle_request_guard.clone())?;
    if let Some(stderr) = lifecycle_request_guard.stderr.as_deref() {
        io::stderr()
            .write_all(stderr.as_bytes())
            .context("failed to write lifecycle request guard stderr")?;
    }

    if lifecycle_request_guard.status == LifecycleRequestGuardStatus::Blocked {
        pre_write_block = option_to_json(lifecycle_request_guard.pre_write)?;
        post_write_block = option_to_json(lifecycle_request_guard.post_write)?;
        final_exit_code = i32::from(lifecycle_request_guard.exit_code);
    } else if request
        .pre_write
        .as_ref()
        .is_some_and(|pre_write| pre_write.requested)
    {
        let pre_write = request.pre_write.context(
            "execute-audit-lifecycle: preWrite requested but preWrite request is missing",
        )?;
        let route_result = if let Some(reason) = pre_write.routing_failure.as_deref() {
            Err(anyhow::anyhow!("{reason}"))
        } else {
            let routing = pre_write.routing.clone().context(
                "execute-audit-lifecycle: preWrite requested but routing request is missing",
            )?;
            resolve_pre_write_route(routing)
        };
        match route_result {
            Ok(route) if route.engine == "rust" => {
                let result = execute_rust_pre_write_lifecycle_streaming(
                    build_rust_pre_write_request(pre_write.rust, &route)?,
                )?;
                pre_write_block = serde_json::to_value(result.block)?;
                if final_exit_code == 0 {
                    final_exit_code = result.exit_code;
                }
            }
            Ok(route) if route.engine == "js" => {
                let result = execute_js_pre_write_lifecycle_streaming(build_js_pre_write_request(
                    pre_write.js,
                    &route,
                )?)?;
                pre_write_block = serde_json::to_value(result.block)?;
                if final_exit_code == 0 {
                    final_exit_code = result.exit_code;
                }
            }
            Ok(route) => {
                bail!(
                    "execute-audit-lifecycle: unsupported pre-write route engine '{}'",
                    route.engine
                );
            }
            Err(error) => {
                pre_write_block = json!({
                    "requested": true,
                    "ran": false,
                    "engine": lifecycle_request_guard
                        .pre_write
                        .as_ref()
                        .and_then(|block| block.engine.clone())
                        .or_else(|| {
                            pre_write
                                .routing
                                .as_ref()
                                .map(|routing| routing.requested_engine.clone())
                        })
                        .unwrap_or_else(|| "auto".to_string()),
                    "reason": format!("pre-write engine selection failed: {error}"),
                });
                final_exit_code = 2;
            }
        }
    } else if let Some(post_write) = request.post_write.filter(|step| step.requested) {
        let result = execute_post_write_lifecycle_streaming(post_write.request)?;
        post_write_block = serde_json::to_value(result.block)?;
        if final_exit_code == 0 {
            final_exit_code = result.exit_code;
        }
    }

    if let Some(canon_draft) = request.canon_draft.filter(|step| step.requested) {
        let result = execute_canon_draft_lifecycle(canon_draft.request)?;
        canon_draft_block = serde_json::to_value(result.block)?;
        if result.force_exit_code || final_exit_code == 0 {
            final_exit_code = result.exit_code;
        }
    }

    if let Some(check_canon) = request.check_canon.filter(|step| step.requested) {
        let result = execute_check_canon_lifecycle(check_canon.request)?;
        check_canon_block = serde_json::to_value(result.block)?;
        if final_exit_code == 0 {
            final_exit_code = result.exit_code;
        }
    }

    let exit_policy = apply_lifecycle_exit_policy(LifecycleExitPolicyRequest {
        schema_version: "lumin-lifecycle-exit-policy-request.v1".to_string(),
        current_exit_code: final_exit_code,
        strict_post_write: request.exit_policy.strict_post_write,
        strict_post_write_confidence: request.exit_policy.strict_post_write_confidence,
        post_write: if post_write_block.is_null() {
            None
        } else {
            Some(
                serde_json::from_value(post_write_block.clone())
                    .context("execute-audit-lifecycle: invalid postWrite block for exit policy")?,
            )
        },
    })?;
    if let Some(stderr) = exit_policy.stderr.as_deref() {
        io::stderr()
            .write_all(stderr.as_bytes())
            .context("failed to write lifecycle exit policy stderr")?;
    }
    final_exit_code = exit_policy.exit_code;

    Ok(json!({
        "schemaVersion": AUDIT_LIFECYCLE_EXECUTION_RESULT_SCHEMA_VERSION,
        "preWrite": pre_write_block,
        "postWrite": post_write_block,
        "canonDraft": canon_draft_block,
        "checkCanon": check_canon_block,
        "finalExitCode": final_exit_code,
    }))
}

fn build_rust_pre_write_request(
    template: RustPreWriteLifecycleTemplate,
    route: &PreWriteRoutingResult,
) -> Result<RustPreWriteLifecycleRequest> {
    Ok(RustPreWriteLifecycleRequest {
        schema_version: "lumin-rust-pre-write-lifecycle-request.v1".to_string(),
        root: template.root,
        output: template.output,
        source_commit: None,
        advisory_invocation_id: template.advisory_invocation_id,
        rust_native_artifact_path: template.rust_native_artifact_path,
        rust_native_latest_path: template.rust_native_latest_path,
        analyzer_invocation: template.analyzer.context(
            "execute-audit-lifecycle: rust pre-write selected but analyzer invocation is missing",
        )?,
        intent_input: route.child_intent_input.clone().unwrap_or_default(),
        include_tests: template.include_tests,
        production: template.production,
        excludes: template.excludes,
        engine_selection: serde_json::to_value(&route.engine_selection)?,
        file_inventory: template.file_inventory,
        failures: template.failures,
    })
}

fn build_js_pre_write_request(
    template: JsPreWriteLifecycleTemplate,
    route: &PreWriteRoutingResult,
) -> Result<JsPreWriteLifecycleRequest> {
    Ok(JsPreWriteLifecycleRequest {
        schema_version: "lumin-js-pre-write-lifecycle-request.v1".to_string(),
        root: template.root,
        output: template.output,
        scripts_dir: template.scripts_dir,
        node_executable: template.node_executable,
        child_intent_flag: route.child_intent_flag.clone(),
        child_intent_input: route.child_intent_input.clone(),
        engine_selection: serde_json::to_value(&route.engine_selection)?,
        no_fresh_audit: template.no_fresh_audit,
        scan_args: template.scan_args,
    })
}

fn option_to_json<T: serde::Serialize>(value: Option<T>) -> Result<Value> {
    Ok(value
        .map(serde_json::to_value)
        .transpose()?
        .unwrap_or(Value::Null))
}

pub(super) fn run_execute_canon_draft(args: Vec<String>) -> Result<()> {
    let mut input = None;
    let mut args = args.into_iter();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--input" => input = Some(take_string(&mut args, "--input")?),
            _ => bail!("execute-canon-draft: unknown argument '{arg}'\n{USAGE}"),
        }
    }

    let input = input.context("execute-canon-draft: missing --input <path|->")?;
    let json = read_json_input(&input, "execute-canon-draft")?;
    let request = serde_json::from_value::<CanonDraftLifecycleRequest>(json)
        .context("execute-canon-draft: invalid request shape")?;
    let result = execute_canon_draft_lifecycle(request)?;
    write_stdout_json(&result)
}

pub(super) fn run_execute_check_canon(args: Vec<String>) -> Result<()> {
    let mut input = None;
    let mut args = args.into_iter();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--input" => input = Some(take_string(&mut args, "--input")?),
            _ => bail!("execute-check-canon: unknown argument '{arg}'\n{USAGE}"),
        }
    }

    let input = input.context("execute-check-canon: missing --input <path|->")?;
    let json = read_json_input(&input, "execute-check-canon")?;
    let request = serde_json::from_value::<CheckCanonLifecycleRequest>(json)
        .context("execute-check-canon: invalid request shape")?;
    let result = execute_check_canon_lifecycle(request)?;
    write_stdout_json(&result)
}

pub(super) fn run_pre_write_route(args: Vec<String>) -> Result<()> {
    let mut input = None;
    let mut args = args.into_iter();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--input" => input = Some(take_string(&mut args, "--input")?),
            _ => bail!("pre-write-route: unknown argument '{arg}'\n{USAGE}"),
        }
    }

    let input = input.context("pre-write-route: missing --input <path|->")?;
    let json = read_json_input(&input, "pre-write-route")?;
    let request = serde_json::from_value::<PreWriteRoutingRequest>(json)
        .context("pre-write-route: invalid request shape")?;
    let result = resolve_pre_write_route(request)?;
    write_stdout_json(&result)
}

pub(super) fn run_execute_rust_pre_write(args: Vec<String>) -> Result<()> {
    let mut input = None;
    let mut result_output = None;
    let mut args = args.into_iter();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--input" => input = Some(take_string(&mut args, "--input")?),
            "--result-output" => result_output = Some(take_path(&mut args, "--result-output")?),
            _ => bail!("execute-rust-pre-write: unknown argument '{arg}'\n{USAGE}"),
        }
    }

    let input = input.context("execute-rust-pre-write: missing --input <path|->")?;
    let json = read_json_input(&input, "execute-rust-pre-write")?;
    let request = serde_json::from_value::<RustPreWriteLifecycleRequest>(json)
        .context("execute-rust-pre-write: invalid request shape")?;
    if let Some(result_output) = result_output {
        let mut result = execute_rust_pre_write_lifecycle_streaming(request)?;
        if let Some(stdout) = result.stdout.as_deref() {
            io::stdout()
                .write_all(stdout.as_bytes())
                .context("failed to replay rust pre-write stdout")?;
        }
        if let Some(stderr) = result.stderr.as_deref() {
            io::stderr()
                .write_all(stderr.as_bytes())
                .context("failed to replay rust pre-write stderr")?;
        }
        result.stdout = None;
        result.stderr = None;
        write_json_file(&result_output, &result)
    } else {
        let result = execute_rust_pre_write_lifecycle(request)?;
        write_stdout_json(&result)
    }
}

pub(super) fn run_execute_js_pre_write(args: Vec<String>) -> Result<()> {
    let mut input = None;
    let mut result_output = None;
    let mut args = args.into_iter();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--input" => input = Some(take_string(&mut args, "--input")?),
            "--result-output" => result_output = Some(take_path(&mut args, "--result-output")?),
            _ => bail!("execute-js-pre-write: unknown argument '{arg}'\n{USAGE}"),
        }
    }

    let input = input.context("execute-js-pre-write: missing --input <path|->")?;
    let json = read_json_input(&input, "execute-js-pre-write")?;
    let request = serde_json::from_value::<JsPreWriteLifecycleRequest>(json)
        .context("execute-js-pre-write: invalid request shape")?;
    if let Some(result_output) = result_output {
        let mut result = execute_js_pre_write_lifecycle_streaming(request)?;
        if let Some(stdout) = result.stdout.as_deref() {
            io::stdout()
                .write_all(stdout.as_bytes())
                .context("failed to replay js pre-write stdout")?;
        }
        if let Some(stderr) = result.stderr.as_deref() {
            io::stderr()
                .write_all(stderr.as_bytes())
                .context("failed to replay js pre-write stderr")?;
        }
        result.stdout = None;
        result.stderr = None;
        write_json_file(&result_output, &result)
    } else {
        let result = execute_js_pre_write_lifecycle(request)?;
        write_stdout_json(&result)
    }
}

pub(super) fn run_execute_post_write(args: Vec<String>) -> Result<()> {
    let mut input = None;
    let mut result_output = None;
    let mut args = args.into_iter();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--input" => input = Some(take_string(&mut args, "--input")?),
            "--result-output" => result_output = Some(take_path(&mut args, "--result-output")?),
            _ => bail!("execute-post-write: unknown argument '{arg}'\n{USAGE}"),
        }
    }

    let input = input.context("execute-post-write: missing --input <path|->")?;
    let json = read_json_input(&input, "execute-post-write")?;
    let request = serde_json::from_value::<PostWriteLifecycleRequest>(json)
        .context("execute-post-write: invalid request shape")?;
    if let Some(result_output) = result_output {
        let mut result = execute_post_write_lifecycle_streaming(request)?;
        if let Some(stdout) = result.stdout.as_deref() {
            io::stdout()
                .write_all(stdout.as_bytes())
                .context("failed to replay post-write stdout")?;
        }
        if let Some(stderr) = result.stderr.as_deref() {
            io::stderr()
                .write_all(stderr.as_bytes())
                .context("failed to replay post-write stderr")?;
        }
        result.stdout = None;
        result.stderr = None;
        write_json_file(&result_output, &result)
    } else {
        let result = execute_post_write_lifecycle(request)?;
        write_stdout_json(&result)
    }
}
