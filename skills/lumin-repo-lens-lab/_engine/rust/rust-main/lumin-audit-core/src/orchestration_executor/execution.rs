use super::child_process::{
    clear_producer_phase_timing, command_status, remove_file_if_present, run_child,
};
use super::observations::{append_planned_skips, push_command, push_skip, result_from_parts};
use super::protocol::{
    CommandRun, ExecutorBasePipelineInput, ExecutorPlanInput, ExecutorPlannedSkipInput,
    ExecutorRequest, ExecutorResult, ExecutorStepInput, RuntimeExecutorRequest,
    RuntimeExecutorResult,
};
use super::rust_analyzer::{execute_rust_analyzer_step, not_requested_rust_analysis};
use super::validation::{validate_executor_request, validate_runtime_executor_request};
use super::{
    EXECUTOR_REQUEST_SCHEMA_VERSION, INCREMENTAL_PRODUCER_STEPS,
    RUNTIME_EXECUTOR_RESULT_SCHEMA_VERSION, RUST_ANALYZER_STEP, RUST_ONLY_SKIP_REASON, TRIAGE_STEP,
};
use crate::orchestration_plan::{
    build_orchestration_plan, AuditProfile, OrchestrationPlan, OrchestrationPlanOptions,
};
use crate::source_inventory::{
    load_source_inventory, ValidatedSourceInventory, SOURCE_INVENTORY_FILE_NAME,
};
use anyhow::{bail, Context, Result};
use std::path::Path;
use std::process::Command;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PreconditionOutcome {
    Met,
    Unmet,
}

pub fn execute_runtime_request(request: RuntimeExecutorRequest) -> Result<RuntimeExecutorResult> {
    validate_runtime_executor_request(&request)?;
    let profile = AuditProfile::parse(&request.profile)?;
    let plan = build_orchestration_plan(OrchestrationPlanOptions {
        profile,
        profile_explicit: request.profile_explicit,
        sarif: request.sarif,
        pre_write: request.pre_write,
        post_write: request.post_write,
        canon_draft: request.canon_draft,
        check_canon: request.check_canon,
        rust_analyzer: request.rust_analyzer.requested,
    });
    let executor_request = ExecutorRequest {
        schema_version: EXECUTOR_REQUEST_SCHEMA_VERSION.to_string(),
        run_id: request.run_id,
        plan: executor_plan_input_from_plan(&plan),
        root: request.root,
        output: request.output,
        scripts_dir: request.scripts_dir,
        node_executable: request.node_executable,
        verbose: request.verbose,
        scan_range: request.scan_range,
        cache: request.cache,
        generated_artifacts: request.generated_artifacts,
        rust_analyzer: request.rust_analyzer,
    };
    let result = execute_base_plan(executor_request)?;
    Ok(RuntimeExecutorResult {
        schema_version: RUNTIME_EXECUTOR_RESULT_SCHEMA_VERSION,
        plan,
        events: result.events,
        commands_run: result.commands_run,
        skipped: result.skipped,
        rust_analysis_run: result.rust_analysis_run,
        exit_policy: result.exit_policy,
    })
}

pub fn execute_base_plan(request: ExecutorRequest) -> Result<ExecutorResult> {
    validate_executor_request(&request)?;
    let mut commands_run = Vec::new();
    let mut skipped = Vec::new();
    let mut events = Vec::new();
    let mut failed_required = false;
    let mut rust_analysis_run = not_requested_rust_analysis(&request);
    let mut source_inventory: Option<ValidatedSourceInventory> = None;

    if request.plan.base_pipeline.status != "planned" {
        append_planned_skips(&request, &mut skipped, &mut events);
        return Ok(result_from_parts(
            commands_run,
            skipped,
            events,
            rust_analysis_run,
            false,
        ));
    }

    let inventory_path = request.output.join(SOURCE_INVENTORY_FILE_NAME);
    for step in request.plan.steps.clone() {
        if step.step == RUST_ANALYZER_STEP {
            let inventory = source_inventory.as_ref().context(
                "execute-base-plan: Rust analyzer cannot run before current-run source inventory",
            )?;
            let observed = execute_rust_analyzer_step(&request, inventory)?;
            rust_analysis_run = observed.rust_analysis_run;
            commands_run.extend(observed.commands_run);
            skipped.extend(observed.skipped);
            events.extend(observed.events);
            continue;
        }

        if step.script != TRIAGE_STEP
            && source_inventory
                .as_ref()
                .is_some_and(ValidatedSourceInventory::is_rust_only)
        {
            push_skip(&mut skipped, &mut events, &step.step, RUST_ONLY_SKIP_REASON);
            continue;
        }

        if precondition_outcome(&request, &step.step)? == PreconditionOutcome::Unmet {
            let reason = step
                .skip_reason_when_unmet
                .as_deref()
                .unwrap_or("precondition unmet");
            push_skip(&mut skipped, &mut events, &step.step, reason);
            continue;
        }

        if step.script != TRIAGE_STEP && source_inventory.is_none() {
            bail!(
                "execute-base-plan: producer '{}' cannot run before current-run source inventory",
                step.step
            );
        }
        if step.script == TRIAGE_STEP {
            if source_inventory.is_some() {
                bail!("execute-base-plan: triage/source inventory step may run only once");
            }
            remove_file_if_present(&inventory_path)?;
        }

        let argv = argv_for_js_step(
            &request,
            &step.script,
            source_inventory
                .as_ref()
                .map(ValidatedSourceInventory::path),
            &request.run_id,
        )?;
        clear_producer_phase_timing(&request.output, &step.step)?;
        let observed = run_child(&request.node_executable, &argv, request.verbose)?;
        let status = command_status(&observed, step.required);
        if step.script == TRIAGE_STEP && status == "ok" {
            source_inventory = Some(
                load_source_inventory(
                    &inventory_path,
                    &request.run_id,
                    &request.root,
                    request.scan_range.include_tests,
                    &request.scan_range.excludes,
                )
                .with_context(|| {
                    format!(
                        "execute-base-plan: successful triage did not produce a valid {}",
                        inventory_path.display()
                    )
                })?,
            );
        }
        if status == "failed-required" {
            failed_required = true;
        }
        push_command(
            &mut commands_run,
            &mut events,
            CommandRun {
                step: step.step.clone(),
                status,
                ms: observed.ms,
                artifact: None,
                rust_files: None,
                analyzer_invocation: None,
                stderr: observed.stderr_snippet,
                memory: observed.memory,
            },
        );
        if failed_required {
            break;
        }
    }

    append_planned_skips(&request, &mut skipped, &mut events);

    Ok(result_from_parts(
        commands_run,
        skipped,
        events,
        rust_analysis_run,
        failed_required,
    ))
}

fn executor_plan_input_from_plan(plan: &OrchestrationPlan) -> ExecutorPlanInput {
    ExecutorPlanInput {
        schema_version: plan.schema_version.to_string(),
        profile: plan.profile.as_str().to_string(),
        emit_sarif: plan.emit_sarif,
        base_pipeline: ExecutorBasePipelineInput {
            status: plan.base_pipeline.status.as_str().to_string(),
        },
        steps: plan
            .steps
            .iter()
            .map(|step| ExecutorStepInput {
                step: step.step.to_string(),
                script: step.script.to_string(),
                required: step.required,
                producer_owner: step.producer_owner.as_str().to_string(),
                execution_owner: step.execution_owner.to_string(),
                skip_reason_when_unmet: step.skip_reason_when_unmet.map(str::to_string),
            })
            .collect(),
        skipped: plan
            .skipped
            .iter()
            .map(|skip| ExecutorPlannedSkipInput {
                step: skip.step.to_string(),
                reason: skip.reason.to_string(),
            })
            .collect(),
    }
}

fn argv_for_js_step(
    request: &ExecutorRequest,
    script: &str,
    source_inventory: Option<&Path>,
    source_inventory_run_id: &str,
) -> Result<Vec<String>> {
    let mut argv = vec![
        request
            .scripts_dir
            .join(script)
            .to_string_lossy()
            .to_string(),
        "--root".to_string(),
        request.root.to_string_lossy().to_string(),
        "--output".to_string(),
        request.output.to_string_lossy().to_string(),
    ];

    if !request.scan_range.include_tests {
        argv.push("--production".to_string());
    }
    for exclude in &request.scan_range.excludes {
        argv.push("--exclude".to_string());
        argv.push(exclude.clone());
    }
    argv.push("--source-inventory-run-id".to_string());
    argv.push(source_inventory_run_id.to_string());
    if script != TRIAGE_STEP {
        let source_inventory = source_inventory
            .context("execute-base-plan: non-triage argv requires validated source inventory")?;
        argv.push("--source-inventory".to_string());
        argv.push(source_inventory.to_string_lossy().to_string());
    }
    if is_incremental_step(script) {
        if request.cache.no_incremental {
            argv.push("--no-incremental".to_string());
        }
        if !request.cache.cache_root.trim().is_empty() {
            argv.push("--cache-root".to_string());
            argv.push(request.cache.cache_root.clone());
        }
    }
    if script == "build-symbol-graph.mjs" {
        argv.push("--generated-artifacts".to_string());
        argv.push(request.generated_artifacts.mode.clone());
    }
    Ok(argv)
}

fn is_incremental_step(step: &str) -> bool {
    INCREMENTAL_PRODUCER_STEPS.contains(&step)
}

fn precondition_outcome(request: &ExecutorRequest, step: &str) -> Result<PreconditionOutcome> {
    let output = &request.output;
    let root = &request.root;
    let exists_in_output = |name: &str| output.join(name).is_file();

    let met = match step {
        "build-resolver-diagnostics.mjs" | "build-entry-surface.mjs" => {
            exists_in_output("symbols.json")
        }
        "build-module-reachability.mjs" => {
            exists_in_output("symbols.json") && exists_in_output("entry-surface.json")
        }
        "export-action-safety.mjs" | "rank-fixes.mjs" => exists_in_output("dead-classify.json"),
        "merge-runtime-evidence.mjs" => {
            root.join("coverage").join("coverage-final.json").is_file()
                || root
                    .join(".nyc_output")
                    .join("coverage-final.json")
                    .is_file()
        }
        "measure-staleness.mjs" => is_git_work_tree(root)?,
        _ => true,
    };

    Ok(if met {
        PreconditionOutcome::Met
    } else {
        PreconditionOutcome::Unmet
    })
}

fn is_git_work_tree(root: &Path) -> Result<bool> {
    let output = Command::new("git")
        .arg("rev-parse")
        .arg("--is-inside-work-tree")
        .current_dir(root)
        .output();
    match output {
        Ok(output) if output.status.success() => {
            Ok(String::from_utf8_lossy(&output.stdout).trim() == "true")
        }
        Ok(_) => Ok(false),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(error.into()),
    }
}
