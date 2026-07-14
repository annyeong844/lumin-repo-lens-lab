use anyhow::{bail, Context, Result};
use std::path::Path;

use lumin_audit_core::artifact_read_metrics::summarize_artifact_read_events;
use lumin_audit_core::audit_summary::format_blind_zones_console_summary;
use lumin_audit_core::manifest_final::{
    apply_manifest_closeout_update, build_manifest_closeout_update, ManifestCloseoutCompanionInput,
};
use lumin_audit_core::orchestration_events::{
    build_producer_performance_artifact_for_audit_run,
    build_producer_performance_artifact_for_audit_run_from_output,
    ProducerPerformanceAuditRunContext, ProducerPerformanceRuntimeObservations,
};

use super::super::{
    mark_base_evidence_not_refreshed, required_base_pipeline_skip_reason, write_json_result,
};
use super::lifecycle_artifacts::current_lifecycle_artifacts;
use super::protocol::{
    FinalizeAuditRunCompanionPlan, FinalizeAuditRunCompanionPolicy,
    FinalizeAuditRunWithCompanionsCliInput, FinalizeAuditRunWithCompanionsResult,
};
use super::render::render_companions;
use crate::cli::io_support::{read_json_input, take_path, take_string, write_pretty_json_file};
use crate::cli::usage::USAGE;

pub(in crate::cli) fn run_finalize_audit_run_with_companions(args: Vec<String>) -> Result<()> {
    let mut input = None;
    let mut result_output = None;
    let mut args = args.into_iter();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--input" => input = Some(take_string(&mut args, "--input")?),
            "--result-output" => result_output = Some(take_path(&mut args, "--result-output")?),
            _ => bail!("finalize-audit-run-with-companions: unknown argument '{arg}'\n{USAGE}"),
        }
    }

    let input = input.context("finalize-audit-run-with-companions: missing --input <path|->")?;
    let json = read_json_input(&input, "finalize-audit-run-with-companions")?;
    let request = serde_json::from_value::<FinalizeAuditRunWithCompanionsCliInput>(json)
        .context("finalize-audit-run-with-companions: invalid request shape")?;
    let output = Path::new(&request.context.output).to_path_buf();
    let base_pipeline_planned = request
        .companion_policy
        .as_ref()
        .is_none_or(|policy| policy.base_pipeline_planned);
    let base_pipeline_skip_reason = request
        .companion_policy
        .as_ref()
        .and_then(|policy| policy.base_pipeline_skip_reason.clone());
    if !base_pipeline_planned {
        required_base_pipeline_skip_reason(base_pipeline_skip_reason.as_deref())?;
    }
    let companions = request
        .companion_policy
        .as_ref()
        .map(|policy| build_finalize_companion_plan(policy, &request.context, &request.manifest))
        .unwrap_or_else(|| request.companions.clone());
    let mut manifest = request.manifest;
    let mut artifact_read_events = request.artifact_read_events;

    let rendered = render_companions(
        &output,
        &mut manifest,
        request.rust_analysis.as_ref(),
        &companions,
        base_pipeline_planned,
        &mut artifact_read_events.reads,
    )?;

    let artifact_reads = summarize_artifact_read_events(artifact_read_events)
        .context("finalize-audit-run-with-companions: invalid artifact read events")?;
    let scoped_artifacts = if base_pipeline_planned {
        None
    } else {
        Some(current_lifecycle_artifacts(
            &output,
            &manifest,
            &ManifestCloseoutCompanionInput {
                topology_mermaid_path: rendered.topology_mermaid_path.clone(),
                audit_summary_path: rendered.audit_summary_path.clone(),
                review_pack_path: rendered.review_pack_path.clone(),
            },
            false,
        ))
    };
    let observations = ProducerPerformanceRuntimeObservations {
        artifact_reads,
        artifacts_produced: scoped_artifacts.unwrap_or_default(),
        rust_analysis: request.rust_analysis.clone(),
        commands_run: request.commands_run,
        skipped: request.skipped,
    };
    let producer_performance = if base_pipeline_planned {
        build_producer_performance_artifact_for_audit_run_from_output(
            request.context,
            observations,
        )?
    } else {
        build_producer_performance_artifact_for_audit_run(request.context, observations)?
    };
    let producer_performance_path = output.join("producer-performance.json");
    write_pretty_json_file(&producer_performance_path, &producer_performance)?;

    let producer_performance_json = serde_json::to_value(&producer_performance)
        .context("finalize-audit-run-with-companions: invalid producer-performance shape")?;
    let companion = ManifestCloseoutCompanionInput {
        topology_mermaid_path: rendered.topology_mermaid_path.clone(),
        audit_summary_path: rendered.audit_summary_path.clone(),
        review_pack_path: rendered.review_pack_path.clone(),
    };
    let mut update = build_manifest_closeout_update(
        &output,
        &producer_performance_json,
        request.rust_analysis.as_ref(),
        companion,
    )?;
    if !base_pipeline_planned {
        update.artifacts_produced = current_lifecycle_artifacts(
            &output,
            &manifest,
            &ManifestCloseoutCompanionInput {
                topology_mermaid_path: rendered.topology_mermaid_path.clone(),
                audit_summary_path: rendered.audit_summary_path.clone(),
                review_pack_path: rendered.review_pack_path.clone(),
            },
            true,
        );
    }
    apply_manifest_closeout_update(&mut manifest, update.clone())?;
    if !base_pipeline_planned {
        let reason = required_base_pipeline_skip_reason(base_pipeline_skip_reason.as_deref())?;
        mark_base_evidence_not_refreshed(&mut manifest, reason, &update.artifacts_produced)?;
    }
    let manifest_path = output.join("manifest.json");
    write_pretty_json_file(&manifest_path, &manifest)?;

    let blind_zones = manifest
        .get("blindZones")
        .and_then(serde_json::Value::as_array)
        .cloned()
        .unwrap_or_default();
    let artifacts_produced_count = manifest
        .get("artifactsProduced")
        .and_then(serde_json::Value::as_array)
        .map_or(0, Vec::len);
    let result = FinalizeAuditRunWithCompanionsResult {
        producer_performance_path: producer_performance_path.to_string_lossy().to_string(),
        manifest_path: manifest_path.to_string_lossy().to_string(),
        topology_mermaid_path: rendered.topology_mermaid_path,
        audit_summary_path: rendered.audit_summary_path,
        review_pack_path: rendered.review_pack_path,
        audit_summary_preview: rendered.audit_summary_preview,
        artifacts_produced_count,
        blind_zones_summary: format_blind_zones_console_summary(&blind_zones).unwrap_or_default(),
        blind_zones,
        closeout_update: update,
    };
    write_json_result(result_output, &result)
}

fn build_finalize_companion_plan(
    policy: &FinalizeAuditRunCompanionPolicy,
    context: &ProducerPerformanceAuditRunContext,
    manifest: &serde_json::Value,
) -> FinalizeAuditRunCompanionPlan {
    let non_post_write_lifecycle_requested = lifecycle_block_requested(manifest, "preWrite")
        || lifecycle_block_requested(manifest, "canonDraft")
        || lifecycle_block_requested(manifest, "checkCanon");
    FinalizeAuditRunCompanionPlan {
        topology_mermaid: policy.base_pipeline_planned,
        audit_summary: policy.base_pipeline_planned || non_post_write_lifecycle_requested,
        review_pack: policy.base_pipeline_planned && context.profile != "quick",
    }
}

fn lifecycle_block_requested(manifest: &serde_json::Value, field: &str) -> bool {
    manifest
        .get(field)
        .and_then(|block| block.get("requested"))
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lifecycle_summary_is_kept_for_pre_write_but_not_post_write_only() {
        let policy = FinalizeAuditRunCompanionPolicy {
            base_pipeline_planned: false,
            base_pipeline_skip_reason: Some("lifecycle-only".to_string()),
        };
        let context = ProducerPerformanceAuditRunContext {
            generated: "2026-07-11T00:00:00.000Z".to_string(),
            root: "C:/repo".to_string(),
            output: "C:/repo/.audit".to_string(),
            profile: "quick".to_string(),
            include_tests: true,
            production: false,
            excludes: Vec::new(),
            auto_excludes: Vec::new(),
            no_incremental: false,
            cache_root: "C:/repo/.audit/.cache".to_string(),
            clear_incremental_cache: false,
            generated_artifacts_mode: "default".to_string(),
        };
        let pre_write = build_finalize_companion_plan(
            &policy,
            &context,
            &serde_json::json!({ "preWrite": { "requested": true } }),
        );
        let post_write = build_finalize_companion_plan(
            &policy,
            &context,
            &serde_json::json!({ "postWrite": { "requested": true } }),
        );
        assert!(pre_write.audit_summary);
        assert!(!pre_write.topology_mermaid);
        assert!(!post_write.audit_summary);
    }
}
