use anyhow::{Context, Result};
use std::path::Path;

use lumin_audit_core::artifact_read_metrics::ArtifactReadObservation;
use lumin_audit_core::audit_review_pack::{
    render_audit_review_pack_request, AuditReviewPackRenderRequest,
    AUDIT_REVIEW_PACK_RENDER_REQUEST_SCHEMA_VERSION,
};
use lumin_audit_core::audit_summary::{
    render_audit_summary_request, AuditSummaryRenderRequest,
    AUDIT_SUMMARY_RENDER_REQUEST_SCHEMA_VERSION,
};
use lumin_audit_core::manifest_final::build_manifest_artifacts_produced_update;
use lumin_audit_core::topology_mermaid::{
    render_topology_mermaid_request, TopologyMermaidOptions, TopologyMermaidRenderRequest,
    TOPOLOGY_MERMAID_RENDER_REQUEST_SCHEMA_VERSION,
};

use super::protocol::FinalizeAuditRunCompanionPlan;
use crate::cli::io_support::{read_optional_output_json_tolerant_observed, write_text_file};

pub(super) struct RenderedCompanions {
    pub(super) topology_mermaid_path: Option<String>,
    pub(super) audit_summary_path: Option<String>,
    pub(super) review_pack_path: Option<String>,
    pub(super) audit_summary_preview: Option<String>,
}

pub(super) fn render_companions(
    output: &Path,
    manifest: &mut serde_json::Value,
    rust_analysis: Option<&serde_json::Value>,
    companions: &FinalizeAuditRunCompanionPlan,
    base_pipeline_planned: bool,
    artifact_reads: &mut Vec<ArtifactReadObservation>,
) -> Result<RenderedCompanions> {
    let topology = if base_pipeline_planned {
        companion_artifact(output, "topology.json", artifact_reads)
    } else {
        serde_json::Value::Null
    };
    let module_reachability = if base_pipeline_planned {
        companion_artifact(output, "module-reachability.json", artifact_reads)
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

        let update = build_manifest_artifacts_produced_update(output, rust_analysis)?;
        apply_artifacts_produced_update(manifest, update.artifacts_produced)?;
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
                output,
                "checklist-facts.json",
                artifact_reads,
            ),
            fix_plan: base_companion_artifact(
                base_pipeline_planned,
                output,
                "fix-plan.json",
                artifact_reads,
            ),
            topology: topology.clone(),
            discipline: base_companion_artifact(
                base_pipeline_planned,
                output,
                "discipline.json",
                artifact_reads,
            ),
            call_graph: base_companion_artifact(
                base_pipeline_planned,
                output,
                "call-graph.json",
                artifact_reads,
            ),
            function_clones: base_companion_artifact(
                base_pipeline_planned,
                output,
                "function-clones.json",
                artifact_reads,
            ),
            symbols: base_companion_artifact(
                base_pipeline_planned,
                output,
                "symbols.json",
                artifact_reads,
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
            checklist_facts: companion_artifact(output, "checklist-facts.json", artifact_reads),
            fix_plan: companion_artifact(output, "fix-plan.json", artifact_reads),
            topology,
            discipline: companion_artifact(output, "discipline.json", artifact_reads),
            call_graph: companion_artifact(output, "call-graph.json", artifact_reads),
            function_clones: companion_artifact(output, "function-clones.json", artifact_reads),
            barrels: companion_artifact(output, "barrels.json", artifact_reads),
            shape_index: companion_artifact(output, "shape-index.json", artifact_reads),
            dead_classify: companion_artifact(output, "dead-classify.json", artifact_reads),
            symbols: companion_artifact(output, "symbols.json", artifact_reads),
            module_reachability,
            output_path: output_path.to_string_lossy().to_string(),
        };
        let (markdown, _result) = render_audit_review_pack_request(&render_request)?;
        write_text_file(&output_path, &markdown)?;
        review_pack_path = Some(output_path.to_string_lossy().to_string());
    }

    Ok(RenderedCompanions {
        topology_mermaid_path,
        audit_summary_path,
        review_pack_path,
        audit_summary_preview,
    })
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
