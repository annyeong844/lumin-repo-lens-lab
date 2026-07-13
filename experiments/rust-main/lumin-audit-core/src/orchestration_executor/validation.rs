use super::protocol::{ExecutorRequest, ExecutorStepInput, RuntimeExecutorRequest};
use super::{EXECUTOR_REQUEST_SCHEMA_VERSION, RUNTIME_EXECUTOR_REQUEST_SCHEMA_VERSION};
use crate::orchestration_plan::{AuditProfile, ORCHESTRATION_PLAN_SCHEMA_VERSION};
use anyhow::{bail, Result};
use std::path::Path;

pub fn validate_executor_request(request: &ExecutorRequest) -> Result<()> {
    if request.schema_version != EXECUTOR_REQUEST_SCHEMA_VERSION {
        bail!(
            "execute-base-plan: unsupported schemaVersion '{}'",
            request.schema_version
        );
    }
    if request.plan.schema_version != ORCHESTRATION_PLAN_SCHEMA_VERSION {
        bail!(
            "execute-base-plan: unsupported plan.schemaVersion '{}'",
            request.plan.schema_version
        );
    }
    AuditProfile::parse(&request.plan.profile)?;
    validate_source_inventory_run_id("execute-base-plan", &request.run_id)?;
    validate_non_empty("nodeExecutable", &request.node_executable)?;
    validate_path("root", &request.root)?;
    validate_path("output", &request.output)?;
    validate_path("scriptsDir", &request.scripts_dir)?;
    validate_base_pipeline_status(&request.plan.base_pipeline.status)?;
    for step in &request.plan.steps {
        validate_step(step)?;
    }
    for skip in &request.plan.skipped {
        validate_non_empty("skipped.step", &skip.step)?;
        validate_non_empty("skipped.reason", &skip.reason)?;
    }
    Ok(())
}

pub(super) fn validate_runtime_executor_request(request: &RuntimeExecutorRequest) -> Result<()> {
    if request.schema_version != RUNTIME_EXECUTOR_REQUEST_SCHEMA_VERSION {
        bail!(
            "execute-base-runtime: unsupported schemaVersion '{}'",
            request.schema_version
        );
    }
    AuditProfile::parse(&request.profile)?;
    validate_source_inventory_run_id("execute-base-runtime", &request.run_id)?;
    validate_non_empty_for(
        "execute-base-runtime",
        "nodeExecutable",
        &request.node_executable,
    )?;
    validate_path_for("execute-base-runtime", "root", &request.root)?;
    validate_path_for("execute-base-runtime", "output", &request.output)?;
    validate_path_for("execute-base-runtime", "scriptsDir", &request.scripts_dir)?;
    Ok(())
}

fn validate_non_empty(field: &str, value: &str) -> Result<()> {
    validate_non_empty_for("execute-base-plan", field, value)
}

fn validate_non_empty_for(label: &str, field: &str, value: &str) -> Result<()> {
    if value.trim().is_empty() {
        bail!("{label}: {field} must be a non-empty string");
    }
    Ok(())
}

fn validate_source_inventory_run_id(label: &str, value: &str) -> Result<()> {
    validate_non_empty_for(label, "runId", value)?;
    if value.len() > 128
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-'))
    {
        bail!("{label}: runId must contain 1-128 safe identifier characters");
    }
    Ok(())
}

fn validate_path(field: &str, value: &Path) -> Result<()> {
    validate_path_for("execute-base-plan", field, value)
}

fn validate_path_for(label: &str, field: &str, value: &Path) -> Result<()> {
    if value.as_os_str().is_empty() {
        bail!("{label}: {field} must be provided");
    }
    Ok(())
}

fn validate_base_pipeline_status(status: &str) -> Result<()> {
    if !matches!(status, "planned" | "skipped") {
        bail!("execute-base-plan: unsupported basePipeline.status '{status}'");
    }
    Ok(())
}

fn validate_step(step: &ExecutorStepInput) -> Result<()> {
    validate_non_empty("step.step", &step.step)?;
    validate_non_empty("step.script", &step.script)?;
    if !matches!(step.producer_owner.as_str(), "js-mjs" | "rust") {
        bail!(
            "execute-base-plan: unsupported producerOwner '{}' for step '{}'",
            step.producer_owner,
            step.step
        );
    }
    if !matches!(
        step.execution_owner.as_str(),
        "audit-repo.mjs" | "lumin-audit-core"
    ) {
        bail!(
            "execute-base-plan: unsupported executionOwner '{}' for step '{}'",
            step.execution_owner,
            step.step
        );
    }
    Ok(())
}
