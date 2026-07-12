use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use super::super::io_support::{
    read_json_input, read_optional_output_json_tolerant_observed, read_required_json, take_path,
    take_string, write_pretty_json_file, write_stdout_json, write_text_file,
};
use super::super::usage::USAGE;
use super::{
    mark_base_evidence_not_refreshed, required_base_pipeline_skip_reason, write_json_result,
};
use lumin_audit_core::artifact_read_metrics::{
    summarize_artifact_read_events, ArtifactReadMetricsRequest, ArtifactReadObservation,
};
use lumin_audit_core::audit_review_pack::{
    render_audit_review_pack_request, AuditReviewPackRenderRequest,
    AUDIT_REVIEW_PACK_RENDER_REQUEST_SCHEMA_VERSION,
};
use lumin_audit_core::audit_summary::{
    format_blind_zones_console_summary, render_audit_summary_request, AuditSummaryRenderRequest,
    AUDIT_SUMMARY_RENDER_REQUEST_SCHEMA_VERSION,
};
use lumin_audit_core::manifest_final::{
    apply_manifest_closeout_update, build_manifest_artifacts_produced_update,
    build_manifest_closeout_update, ManifestCloseoutCompanionInput, ManifestCloseoutUpdate,
};
use lumin_audit_core::orchestration_events::{
    build_producer_performance_artifact_for_audit_run,
    build_producer_performance_artifact_for_audit_run_from_output,
    ProducerPerformanceAuditRunContext, ProducerPerformanceRuntimeObservations, RuntimeCommandRun,
    RuntimeSkippedRun,
};
use lumin_audit_core::topology_mermaid::{
    render_topology_mermaid_request, TopologyMermaidOptions, TopologyMermaidRenderRequest,
    TOPOLOGY_MERMAID_RENDER_REQUEST_SCHEMA_VERSION,
};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ManifestWriteCliInput {
    manifest: serde_json::Value,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ManifestWriteResult {
    manifest_path: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ManifestCloseoutWriteCliInput {
    manifest: serde_json::Value,
    output: String,
    producer_performance_path: String,
    #[serde(default)]
    rust_analysis: Option<serde_json::Value>,
    #[serde(default)]
    companion: ManifestCloseoutCompanionInput,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ManifestCloseoutWriteResult {
    manifest_path: String,
    closeout_update: ManifestCloseoutUpdate,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct FinalizeAuditRunCliInput {
    manifest: serde_json::Value,
    context: ProducerPerformanceAuditRunContext,
    observations: ProducerPerformanceRuntimeObservations,
    #[serde(default)]
    rust_analysis: Option<serde_json::Value>,
    #[serde(default)]
    companion: ManifestCloseoutCompanionInput,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct FinalizeAuditRunResult {
    producer_performance_path: String,
    manifest_path: String,
    closeout_update: ManifestCloseoutUpdate,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct FinalizeAuditRunWithCompanionsCliInput {
    manifest: serde_json::Value,
    context: ProducerPerformanceAuditRunContext,
    artifact_read_events: ArtifactReadMetricsRequest,
    #[serde(default)]
    commands_run: Vec<RuntimeCommandRun>,
    #[serde(default)]
    skipped: Vec<RuntimeSkippedRun>,
    #[serde(default)]
    rust_analysis: Option<serde_json::Value>,
    #[serde(default)]
    companions: FinalizeAuditRunCompanionPlan,
    #[serde(default)]
    companion_policy: Option<FinalizeAuditRunCompanionPolicy>,
}

#[derive(Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct FinalizeAuditRunCompanionPlan {
    #[serde(default)]
    topology_mermaid: bool,
    #[serde(default)]
    audit_summary: bool,
    #[serde(default)]
    review_pack: bool,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct FinalizeAuditRunCompanionPolicy {
    #[serde(default)]
    base_pipeline_planned: bool,
    #[serde(default)]
    base_pipeline_skip_reason: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct FinalizeAuditRunWithCompanionsResult {
    producer_performance_path: String,
    manifest_path: String,
    topology_mermaid_path: Option<String>,
    audit_summary_path: Option<String>,
    review_pack_path: Option<String>,
    audit_summary_preview: Option<String>,
    artifacts_produced_count: usize,
    blind_zones: Vec<serde_json::Value>,
    blind_zones_summary: String,
    closeout_update: ManifestCloseoutUpdate,
}

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

    let topology = if base_pipeline_planned {
        companion_artifact(&output, "topology.json", &mut artifact_read_events.reads)
    } else {
        serde_json::Value::Null
    };
    let module_reachability = if base_pipeline_planned {
        companion_artifact(
            &output,
            "module-reachability.json",
            &mut artifact_read_events.reads,
        )
    } else {
        serde_json::Value::Null
    };

    let mut topology_mermaid_path = None;
    if companions.topology_mermaid && !topology.is_null() {
        let output_path = output.join("topology.mermaid.md");
        let render_request = TopologyMermaidRenderRequest {
            schema_version: TOPOLOGY_MERMAID_RENDER_REQUEST_SCHEMA_VERSION.to_string(),
            topology: topology.clone(),
            output_path: output_path.to_string_lossy().to_string(),
            options: TopologyMermaidOptions::default(),
        };
        let (markdown, _result) = render_topology_mermaid_request(&render_request)?;
        write_text_file(&output_path, &markdown)?;
        topology_mermaid_path = Some(output_path.to_string_lossy().to_string());

        let update =
            build_manifest_artifacts_produced_update(&output, request.rust_analysis.as_ref())?;
        apply_artifacts_produced_update(&mut manifest, update.artifacts_produced)?;
    }

    let mut audit_summary_path = None;
    let mut audit_summary_preview = None;
    if companions.audit_summary {
        let output_path = output.join("audit-summary.latest.md");
        let render_request = AuditSummaryRenderRequest {
            schema_version: AUDIT_SUMMARY_RENDER_REQUEST_SCHEMA_VERSION.to_string(),
            manifest: manifest.clone(),
            checklist_facts: base_companion_artifact(
                base_pipeline_planned,
                &output,
                "checklist-facts.json",
                &mut artifact_read_events.reads,
            ),
            fix_plan: base_companion_artifact(
                base_pipeline_planned,
                &output,
                "fix-plan.json",
                &mut artifact_read_events.reads,
            ),
            topology: topology.clone(),
            discipline: base_companion_artifact(
                base_pipeline_planned,
                &output,
                "discipline.json",
                &mut artifact_read_events.reads,
            ),
            call_graph: base_companion_artifact(
                base_pipeline_planned,
                &output,
                "call-graph.json",
                &mut artifact_read_events.reads,
            ),
            function_clones: base_companion_artifact(
                base_pipeline_planned,
                &output,
                "function-clones.json",
                &mut artifact_read_events.reads,
            ),
            symbols: base_companion_artifact(
                base_pipeline_planned,
                &output,
                "symbols.json",
                &mut artifact_read_events.reads,
            ),
            module_reachability: module_reachability.clone(),
            output_path: output_path.to_string_lossy().to_string(),
        };
        let (markdown, result) = render_audit_summary_request(&render_request)?;
        write_text_file(&output_path, &markdown)?;
        audit_summary_preview = result.preview;
        audit_summary_path = Some(output_path.to_string_lossy().to_string());
    }

    let mut review_pack_path = None;
    if companions.review_pack {
        let output_path = output.join("audit-review-pack.latest.md");
        let render_request = AuditReviewPackRenderRequest {
            schema_version: AUDIT_REVIEW_PACK_RENDER_REQUEST_SCHEMA_VERSION.to_string(),
            manifest: manifest.clone(),
            checklist_facts: companion_artifact(
                &output,
                "checklist-facts.json",
                &mut artifact_read_events.reads,
            ),
            fix_plan: companion_artifact(&output, "fix-plan.json", &mut artifact_read_events.reads),
            topology,
            discipline: companion_artifact(
                &output,
                "discipline.json",
                &mut artifact_read_events.reads,
            ),
            call_graph: companion_artifact(
                &output,
                "call-graph.json",
                &mut artifact_read_events.reads,
            ),
            function_clones: companion_artifact(
                &output,
                "function-clones.json",
                &mut artifact_read_events.reads,
            ),
            barrels: companion_artifact(&output, "barrels.json", &mut artifact_read_events.reads),
            shape_index: companion_artifact(
                &output,
                "shape-index.json",
                &mut artifact_read_events.reads,
            ),
            dead_classify: companion_artifact(
                &output,
                "dead-classify.json",
                &mut artifact_read_events.reads,
            ),
            symbols: companion_artifact(&output, "symbols.json", &mut artifact_read_events.reads),
            module_reachability,
            output_path: output_path.to_string_lossy().to_string(),
        };
        let (markdown, _result) = render_audit_review_pack_request(&render_request)?;
        write_text_file(&output_path, &markdown)?;
        review_pack_path = Some(output_path.to_string_lossy().to_string());
    }

    let artifact_reads = summarize_artifact_read_events(artifact_read_events)
        .context("finalize-audit-run-with-companions: invalid artifact read events")?;
    let scoped_artifacts = if base_pipeline_planned {
        None
    } else {
        Some(current_lifecycle_artifacts(
            &output,
            &manifest,
            &ManifestCloseoutCompanionInput {
                topology_mermaid_path: topology_mermaid_path.clone(),
                audit_summary_path: audit_summary_path.clone(),
                review_pack_path: review_pack_path.clone(),
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
        topology_mermaid_path: topology_mermaid_path.clone(),
        audit_summary_path: audit_summary_path.clone(),
        review_pack_path: review_pack_path.clone(),
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
                topology_mermaid_path: topology_mermaid_path.clone(),
                audit_summary_path: audit_summary_path.clone(),
                review_pack_path: review_pack_path.clone(),
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
        topology_mermaid_path,
        audit_summary_path,
        review_pack_path,
        audit_summary_preview,
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

fn base_companion_artifact(
    base_pipeline_planned: bool,
    output: &Path,
    artifact_name: &str,
    artifact_reads: &mut Vec<ArtifactReadObservation>,
) -> serde_json::Value {
    if base_pipeline_planned {
        companion_artifact(output, artifact_name, artifact_reads)
    } else {
        serde_json::Value::Null
    }
}

fn companion_artifact(
    output: &Path,
    artifact_name: &str,
    artifact_reads: &mut Vec<ArtifactReadObservation>,
) -> serde_json::Value {
    let observed = read_optional_output_json_tolerant_observed(output, artifact_name);
    if let Some(observation) = observed.observation {
        artifact_reads.push(observation);
    }
    match observed.value {
        Some(value) if !is_malformed_optional_artifact(&value) => value,
        _ => serde_json::Value::Null,
    }
}

fn current_lifecycle_artifacts(
    output: &Path,
    manifest: &serde_json::Value,
    companion: &ManifestCloseoutCompanionInput,
    include_producer_performance: bool,
) -> Vec<String> {
    let mut artifacts = BTreeSet::new();

    for pointer in [
        "/preWrite/advisoryPath",
        "/preWrite/latestAdvisoryPath",
        "/preWrite/rustEvidencePath",
        "/preWrite/anyInventoryPath",
        "/preWrite/rustNativeArtifactPath",
        "/preWrite/rustNativeLatestPath",
        "/postWrite/deltaPath",
    ] {
        add_current_output_path(
            &mut artifacts,
            output,
            manifest
                .pointer(pointer)
                .and_then(serde_json::Value::as_str),
        );
    }

    let fresh_rust_evidence = manifest
        .pointer("/preWrite/rustEvidencePath")
        .and_then(serde_json::Value::as_str)
        .is_some();
    if fresh_rust_evidence {
        add_current_output_file(&mut artifacts, output, "pre-write-evidence.latest.json");
    }

    if manifest
        .pointer("/postWrite/ran")
        .and_then(serde_json::Value::as_bool)
        == Some(true)
    {
        let pre_write_id = manifest
            .pointer("/postWrite/preWriteInvocationId")
            .and_then(serde_json::Value::as_str);
        let delta_id = manifest
            .pointer("/postWrite/deltaInvocationId")
            .and_then(serde_json::Value::as_str);
        if let (Some(pre_write_id), Some(delta_id)) = (pre_write_id, delta_id) {
            add_current_output_file(
                &mut artifacts,
                output,
                &format!("post-write-delta.{pre_write_id}.{delta_id}.json"),
            );
        }
    }

    if let Some(paths) = manifest
        .pointer("/canonDraft/draftPaths")
        .and_then(serde_json::Value::as_array)
    {
        for path in paths {
            add_current_output_path(&mut artifacts, output, path.as_str());
        }
    }

    if manifest
        .pointer("/checkCanon/ran")
        .and_then(serde_json::Value::as_bool)
        == Some(true)
    {
        add_current_output_file(&mut artifacts, output, "canon-drift.json");
        if let Some(per_source) = manifest
            .pointer("/checkCanon/perSource")
            .and_then(serde_json::Value::as_object)
        {
            for entry in per_source.values() {
                add_current_output_path(
                    &mut artifacts,
                    output,
                    entry.get("reportPath").and_then(serde_json::Value::as_str),
                );
            }
        }
    }

    for path in [
        companion.topology_mermaid_path.as_deref(),
        companion.audit_summary_path.as_deref(),
        companion.review_pack_path.as_deref(),
    ]
    .into_iter()
    .flatten()
    {
        add_current_output_path(&mut artifacts, output, Some(path));
    }
    if include_producer_performance {
        add_current_output_file(&mut artifacts, output, "producer-performance.json");
    }

    artifacts.into_iter().collect()
}

fn add_current_output_file(artifacts: &mut BTreeSet<String>, output: &Path, name: &str) {
    add_current_output_path(artifacts, output, Some(name));
}

fn add_current_output_path(artifacts: &mut BTreeSet<String>, output: &Path, value: Option<&str>) {
    let Some(value) = value else {
        return;
    };
    let path = PathBuf::from(value);
    let path = if path.is_absolute() {
        path
    } else {
        output.join(path)
    };
    if !path.is_file() {
        return;
    }
    let Ok(relative) = path.strip_prefix(output) else {
        return;
    };
    if relative.components().count() != 1 {
        return;
    }
    artifacts.insert(relative.to_string_lossy().to_string());
}

fn is_malformed_optional_artifact(value: &serde_json::Value) -> bool {
    value
        .get("reason")
        .and_then(|reason| reason.get("kind"))
        .and_then(serde_json::Value::as_str)
        .is_some_and(|kind| kind == "read-error" || kind == "malformed-json")
}

fn apply_artifacts_produced_update(
    manifest: &mut serde_json::Value,
    artifacts_produced: Vec<String>,
) -> Result<()> {
    let manifest = manifest
        .as_object_mut()
        .context("finalize-audit-run-with-companions: manifest must be a JSON object")?;
    manifest.insert(
        "artifactsProduced".to_string(),
        serde_json::to_value(artifacts_produced)
            .context("finalize-audit-run-with-companions: invalid artifactsProduced update")?,
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lifecycle_artifact_scope_excludes_reused_base_outputs() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let output = temp.path();
        for name in [
            "symbols.json",
            "producer-performance.json",
            "pre-write-evidence.PRE.json",
            "pre-write-evidence.latest.json",
            "pre-write-advisory.latest.json",
            "pre-write-advisory.PRE.json",
            "any-inventory.pre.PRE.json",
        ] {
            std::fs::write(output.join(name), "{}")?;
        }
        let manifest = serde_json::json!({
            "preWrite": {
                "requested": true,
                "ran": true,
                "advisoryPath": output.join("pre-write-advisory.PRE.json"),
                "latestAdvisoryPath": output.join("pre-write-advisory.latest.json"),
                "advisoryInvocationId": "PRE",
                "rustEvidencePath": "pre-write-evidence.PRE.json",
                "anyInventoryPath": "any-inventory.pre.PRE.json"
            }
        });
        let artifacts = current_lifecycle_artifacts(
            output,
            &manifest,
            &ManifestCloseoutCompanionInput::default(),
            false,
        );
        assert_eq!(
            artifacts,
            vec![
                "any-inventory.pre.PRE.json",
                "pre-write-advisory.PRE.json",
                "pre-write-advisory.latest.json",
                "pre-write-evidence.PRE.json",
                "pre-write-evidence.latest.json",
            ]
        );
        Ok(())
    }

    #[test]
    fn lifecycle_artifact_scope_excludes_stale_pre_write_evidence() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let output = temp.path();
        for name in [
            "pre-write-advisory.latest.json",
            "pre-write-advisory.PRE.json",
            "pre-write-evidence.latest.json",
            "any-inventory.pre.PRE.json",
        ] {
            std::fs::write(output.join(name), "{}")?;
        }
        let manifest = serde_json::json!({
            "preWrite": {
                "requested": true,
                "ran": true,
                "advisoryPath": output.join("pre-write-advisory.PRE.json"),
                "latestAdvisoryPath": output.join("pre-write-advisory.latest.json"),
                "advisoryInvocationId": "PRE"
            }
        });

        let artifacts = current_lifecycle_artifacts(
            output,
            &manifest,
            &ManifestCloseoutCompanionInput::default(),
            false,
        );
        assert_eq!(
            artifacts,
            vec![
                "pre-write-advisory.PRE.json",
                "pre-write-advisory.latest.json",
            ]
        );
        Ok(())
    }

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
