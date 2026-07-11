use anyhow::Result;
use serde_json::json;
use std::process::Command;

use lumin_audit_core::orchestration_plan::{
    build_orchestration_plan, AuditProfile, BasePipelineStatus, OrchestrationPlanOptions,
    ProducerOwner,
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
    assert_eq!(plan.summary.rust_owned_step_count, 12);
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
    assert_eq!(plan.summary.rust_owned_step_count, 20);
    assert!(plan.steps.iter().any(|step| {
        step.step == "merge-runtime-evidence.mjs" && step.producer_owner == ProducerOwner::Rust
    }));
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
    assert!(ci_plan.steps.iter().any(|step| {
        step.step == "emit-sarif.mjs" && step.producer_owner == ProducerOwner::Rust
    }));
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
fn post_write_only_plan_skips_base_profile_without_losing_lifecycle_request() {
    let plan = build_orchestration_plan(OrchestrationPlanOptions {
        post_write: true,
        ..OrchestrationPlanOptions::default()
    });

    assert_eq!(plan.base_pipeline.status, BasePipelineStatus::Skipped);
    assert!(plan.steps.is_empty());
    assert_eq!(plan.summary.planned_step_count, 0);
    assert!(!plan.lifecycle.pre_write.requested);
    assert!(plan.lifecycle.post_write.requested);
    assert_eq!(
        plan.skipped
            .iter()
            .find(|skip| skip.step == "base-audit-profile")
            .map(|skip| skip.reason),
        Some(
            "post-write-only mode refreshes delta-required inventory instead of running the full quick audit"
        )
    );
}

#[test]
fn post_write_with_sarif_keeps_the_base_profile() {
    let plan = build_orchestration_plan(OrchestrationPlanOptions {
        post_write: true,
        sarif: true,
        ..OrchestrationPlanOptions::default()
    });

    assert_eq!(plan.base_pipeline.status, BasePipelineStatus::Planned);
    assert!(plan.lifecycle.post_write.requested);
    assert!(step_names(&plan).contains(&"emit-sarif.mjs"));
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
    assert_eq!(plan["summary"]["rustOwnedStepCount"], 21);
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
    let framework_surfaces = steps
        .iter()
        .find(|step| step["step"] == "build-framework-resource-surfaces.mjs")
        .ok_or_else(|| anyhow::anyhow!("framework-resource-surfaces step should be planned"))?;
    assert_eq!(framework_surfaces["producerOwner"], "rust");
    assert_eq!(framework_surfaces["executionOwner"], "lumin-audit-core");

    let unused_deps = steps
        .iter()
        .find(|step| step["step"] == "build-unused-deps.mjs")
        .ok_or_else(|| anyhow::anyhow!("unused-deps step should be planned"))?;
    assert_eq!(unused_deps["producerOwner"], "rust");
    assert_eq!(unused_deps["executionOwner"], "lumin-audit-core");

    let discipline = steps
        .iter()
        .find(|step| step["step"] == "measure-discipline.mjs")
        .ok_or_else(|| anyhow::anyhow!("discipline step should be planned"))?;
    assert_eq!(discipline["producerOwner"], "rust");
    assert_eq!(discipline["executionOwner"], "lumin-audit-core");

    let topology = steps
        .iter()
        .find(|step| step["step"] == "measure-topology.mjs")
        .ok_or_else(|| anyhow::anyhow!("topology step should be planned"))?;
    assert_eq!(topology["producerOwner"], "rust");
    assert_eq!(topology["executionOwner"], "lumin-audit-core");

    let entry_surface = steps
        .iter()
        .find(|step| step["step"] == "build-entry-surface.mjs")
        .ok_or_else(|| anyhow::anyhow!("entry-surface step should be planned"))?;
    assert_eq!(entry_surface["producerOwner"], "rust");
    assert_eq!(entry_surface["executionOwner"], "lumin-audit-core");

    let full_plan = build_orchestration_plan(OrchestrationPlanOptions {
        profile: AuditProfile::Full,
        ..OrchestrationPlanOptions::default()
    });
    let full_value = serde_json::to_value(full_plan)?;
    let barrel = full_value
        .get("steps")
        .and_then(|steps| steps.as_array())
        .and_then(|steps| {
            steps
                .iter()
                .find(|step| step["step"] == "check-barrel-discipline.mjs")
        })
        .cloned()
        .ok_or_else(|| anyhow::anyhow!("barrel-discipline step should be planned"))?;
    assert_eq!(barrel["producerOwner"], "rust");
    assert_eq!(barrel["executionOwner"], "lumin-audit-core");

    let module_reachability = steps
        .iter()
        .find(|step| step["step"] == "build-module-reachability.mjs")
        .ok_or_else(|| anyhow::anyhow!("module-reachability step should be planned"))?;
    assert_eq!(module_reachability["producerOwner"], "rust");
    assert_eq!(module_reachability["executionOwner"], "lumin-audit-core");

    let export_action_safety = steps
        .iter()
        .find(|step| step["step"] == "export-action-safety.mjs")
        .ok_or_else(|| anyhow::anyhow!("export-action-safety step should be planned"))?;
    assert_eq!(export_action_safety["producerOwner"], "rust");
    assert_eq!(export_action_safety["executionOwner"], "lumin-audit-core");

    let rank_fixes = steps
        .iter()
        .find(|step| step["step"] == "rank-fixes.mjs")
        .ok_or_else(|| anyhow::anyhow!("rank-fixes step should be planned"))?;
    assert_eq!(rank_fixes["producerOwner"], "rust");
    assert_eq!(rank_fixes["executionOwner"], "lumin-audit-core");

    let checklist_facts = steps
        .iter()
        .find(|step| step["step"] == "checklist-facts.mjs")
        .ok_or_else(|| anyhow::anyhow!("checklist-facts step should be planned"))?;
    assert_eq!(checklist_facts["producerOwner"], "rust");
    assert_eq!(checklist_facts["executionOwner"], "lumin-audit-core");

    let sarif_plan = build_orchestration_plan(OrchestrationPlanOptions {
        sarif: true,
        ..OrchestrationPlanOptions::default()
    });
    let sarif = serde_json::to_value(sarif_plan)?
        .get("steps")
        .and_then(|steps| steps.as_array())
        .and_then(|steps| steps.iter().find(|step| step["step"] == "emit-sarif.mjs"))
        .cloned()
        .ok_or_else(|| anyhow::anyhow!("sarif step should be planned"))?;
    assert_eq!(sarif["producerOwner"], "rust");
    assert_eq!(sarif["executionOwner"], "lumin-audit-core");

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
            "producerOwner": "rust",
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
