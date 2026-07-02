use anyhow::Result;
use serde_json::json;
use std::process::Command;

use lumin_audit_core::orchestration_plan::{
    build_orchestration_plan, AuditProfile, BasePipelineStatus, OrchestrationPlanOptions,
};

#[test]
fn quick_plan_matches_default_audit_profile_without_full_or_sarif_steps() {
    let plan = build_orchestration_plan(OrchestrationPlanOptions::default());

    assert_eq!(plan.profile, AuditProfile::Quick);
    assert!(!plan.emit_sarif);
    assert_eq!(plan.base_pipeline.status, BasePipelineStatus::Planned);
    assert_eq!(
        step_names(&plan),
        vec![
            "triage-repo.mjs",
            "build-framework-resource-surfaces.mjs",
            "measure-topology.mjs",
            "measure-discipline.mjs",
            "build-symbol-graph.mjs",
            "build-unused-deps.mjs",
            "build-resolver-diagnostics.mjs",
            "build-entry-surface.mjs",
            "build-module-reachability.mjs",
            "classify-dead-exports.mjs",
            "export-action-safety.mjs",
            "rank-fixes.mjs",
            "checklist-facts.mjs",
        ]
    );
    assert_eq!(plan.summary.required_step_count, 2);
    assert!(plan
        .skipped
        .iter()
        .any(|skip| skip.step == "emit-sarif.mjs"));
    assert!(!plan
        .skipped
        .iter()
        .any(|skip| skip.step == "lumin-rust-analyzer"));
}

#[test]
fn full_plan_adds_structural_runtime_and_staleness_steps() {
    let plan = build_orchestration_plan(OrchestrationPlanOptions {
        profile: AuditProfile::Full,
        rust_analyzer: true,
        ..OrchestrationPlanOptions::default()
    });
    let steps = step_names(&plan);

    assert!(steps.contains(&"lumin-rust-analyzer"));
    assert!(steps.contains(&"build-call-graph.mjs"));
    assert!(steps.contains(&"check-barrel-discipline.mjs"));
    assert!(steps.contains(&"build-shape-index.mjs"));
    assert!(steps.contains(&"build-function-clone-index.mjs"));
    assert!(steps.contains(&"build-block-clone-index.mjs"));
    assert!(steps.contains(&"merge-runtime-evidence.mjs"));
    assert!(steps.contains(&"measure-staleness.mjs"));
    assert!(!steps.contains(&"emit-sarif.mjs"));
    assert_eq!(plan.summary.rust_owned_step_count, 1);
}

#[test]
fn ci_or_explicit_sarif_plans_emit_sarif_without_forcing_ci_profile() {
    let ci_plan = build_orchestration_plan(OrchestrationPlanOptions {
        profile: AuditProfile::Ci,
        ..OrchestrationPlanOptions::default()
    });
    let forced_plan = build_orchestration_plan(OrchestrationPlanOptions {
        profile: AuditProfile::Quick,
        sarif: true,
        ..OrchestrationPlanOptions::default()
    });

    assert!(ci_plan.emit_sarif);
    assert!(step_names(&ci_plan).contains(&"emit-sarif.mjs"));
    assert!(forced_plan.emit_sarif);
    assert!(step_names(&forced_plan).contains(&"emit-sarif.mjs"));
    assert!(!step_names(&forced_plan).contains(&"build-call-graph.mjs"));
}

#[test]
fn pre_write_only_plan_skips_base_profile_without_losing_lifecycle_request() {
    let plan = build_orchestration_plan(OrchestrationPlanOptions {
        pre_write: true,
        ..OrchestrationPlanOptions::default()
    });

    assert_eq!(plan.base_pipeline.status, BasePipelineStatus::Skipped);
    assert!(plan.steps.is_empty());
    assert_eq!(plan.summary.planned_step_count, 0);
    assert!(plan.lifecycle.pre_write.requested);
    assert!(!plan.lifecycle.post_write.requested);
    assert_eq!(
        plan.skipped
            .iter()
            .find(|skip| skip.step == "base-audit-profile")
            .map(|skip| skip.reason),
        Some("pre-write-only mode uses intent-shaped cold-cache instead of full quick audit")
    );
}

#[test]
fn pre_post_mutex_plan_records_base_skip() {
    let plan = build_orchestration_plan(OrchestrationPlanOptions {
        pre_write: true,
        post_write: true,
        ..OrchestrationPlanOptions::default()
    });

    assert_eq!(plan.base_pipeline.status, BasePipelineStatus::Skipped);
    assert!(plan.steps.is_empty());
    assert!(plan.lifecycle.pre_write.requested);
    assert!(plan.lifecycle.post_write.requested);
    assert_eq!(
        plan.skipped
            .iter()
            .find(|skip| skip.step == "base-audit-profile")
            .map(|skip| skip.reason),
        Some("--pre-write and --post-write are mutually exclusive")
    );
}

#[test]
fn cli_orchestration_plan_emits_typed_json() -> Result<()> {
    let output = Command::new(audit_core_bin())
        .arg("orchestration-plan")
        .arg("--profile")
        .arg("ci")
        .arg("--rust-analyzer")
        .output()?;

    assert!(output.status.success());
    let plan = serde_json::from_slice::<serde_json::Value>(&output.stdout)?;
    assert_eq!(plan["schemaVersion"], "lumin-audit-orchestration-plan.v1");
    assert_eq!(plan["planOwner"], "lumin-audit-core");
    assert_eq!(plan["executionOwner"], "lumin-audit-core");
    assert_eq!(
        plan["lifecycle"]["preWrite"]["executionOwner"],
        "lumin-audit-core"
    );
    assert_eq!(
        plan["lifecycle"]["postWrite"]["executionOwner"],
        "lumin-audit-core"
    );
    assert_eq!(
        plan["lifecycle"]["canonDraft"]["executionOwner"],
        "lumin-audit-core"
    );
    assert_eq!(
        plan["lifecycle"]["checkCanon"]["executionOwner"],
        "lumin-audit-core"
    );
    assert_eq!(plan["profile"], "ci");
    assert_eq!(plan["emitSarif"], true);
    assert_eq!(plan["summary"]["rustOwnedStepCount"], 1);
    assert!(plan["steps"]
        .as_array()
        .is_some_and(|steps| steps.iter().any(|step| step["step"] == "emit-sarif.mjs")));
    Ok(())
}

#[test]
fn cli_orchestration_plan_rejects_unknown_profile() -> Result<()> {
    let output = Command::new(audit_core_bin())
        .arg("orchestration-plan")
        .arg("--profile")
        .arg("slow")
        .output()?;

    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("quick|full|ci"));
    Ok(())
}

#[test]
fn serialized_step_preconditions_keep_js_producer_with_rust_executor_contract() -> Result<()> {
    let plan = build_orchestration_plan(OrchestrationPlanOptions::default());
    let value = serde_json::to_value(plan)?;
    let steps = value["steps"]
        .as_array()
        .ok_or_else(|| anyhow::anyhow!("steps must serialize as an array"))?;
    let resolver = steps
        .iter()
        .find(|step| step["step"] == "build-resolver-diagnostics.mjs")
        .ok_or_else(|| anyhow::anyhow!("resolver step should be planned"))?;

    assert_eq!(
        resolver,
        &json!({
            "order": 7,
            "step": "build-resolver-diagnostics.mjs",
            "script": "build-resolver-diagnostics.mjs",
            "phase": "resolver-diagnostics",
            "required": false,
            "producerOwner": "js-mjs",
            "executionOwner": "lumin-audit-core",
            "mode": "precondition",
            "requiresArtifacts": ["symbols.json"],
            "precondition": "symbols.json exists",
            "skipReasonWhenUnmet": "symbols.json missing (symbol graph step failed or was skipped)"
        })
    );
    Ok(())
}

fn step_names(plan: &lumin_audit_core::orchestration_plan::OrchestrationPlan) -> Vec<&str> {
    plan.steps.iter().map(|step| step.step).collect()
}

fn audit_core_bin() -> &'static str {
    env!("CARGO_BIN_EXE_lumin-audit-core")
}
