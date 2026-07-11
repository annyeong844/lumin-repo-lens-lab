use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use super::args::{ManifestCoreSummaryArgs, ManifestEvidenceSummaryArgs};
use super::io_support::{
    read_json_input, read_optional_json, read_optional_json_input,
    read_optional_output_json_observed, read_optional_output_json_tolerant_observed,
    read_required_json, take_path, take_string, write_json_file, write_pretty_json_file,
    write_stdout_json, write_text_file, OptionalOutputJsonRead,
};
use super::usage::USAGE;
use lumin_audit_core::artifact_read_metrics::{
    summarize_artifact_read_events, ArtifactReadMetricsRequest, ArtifactReadObservation,
    ARTIFACT_READ_EVENTS_SCHEMA_VERSION,
};
use lumin_audit_core::audit_review_pack::{
    render_audit_review_pack_request, AuditReviewPackRenderRequest,
    AUDIT_REVIEW_PACK_RENDER_REQUEST_SCHEMA_VERSION,
};
use lumin_audit_core::audit_summary::{
    render_audit_summary_request, AuditSummaryRenderRequest,
    AUDIT_SUMMARY_RENDER_REQUEST_SCHEMA_VERSION,
};
use lumin_audit_core::generated_artifacts::GeneratedArtifactsMode;
use lumin_audit_core::lifecycle::{
    apply_manifest_lifecycle_update, build_manifest_lifecycle_update, ManifestLifecycleUpdateInput,
};
use lumin_audit_core::manifest_companion::{
    build_manifest_companion_update, ManifestCompanionUpdateInput,
};
use lumin_audit_core::manifest_core::{summarize_manifest_core, ManifestCoreOptions};
use lumin_audit_core::manifest_evidence::{
    summarize_manifest_evidence, ManifestEvidenceArtifacts, ManifestEvidenceOptions,
    ManifestEvidenceSummary,
};
use lumin_audit_core::manifest_final::{
    apply_manifest_closeout_update, build_manifest_artifacts_produced_update,
    build_manifest_closeout_update, build_manifest_final_summary_update,
    build_manifest_final_summary_update_for_rust_analysis, ManifestCloseoutCompanionInput,
    ManifestCloseoutUpdate,
};
use lumin_audit_core::manifest_meta::{build_manifest_meta, ManifestMetaInput};
use lumin_audit_core::manifest_root::{
    apply_manifest_evidence_update, build_manifest_evidence_update, build_manifest_root,
    ManifestEvidenceUpdateFields, ManifestEvidenceUpdateInput, ManifestRootInput,
};
use lumin_audit_core::orchestration_events::{
    build_producer_performance_artifact_for_audit_run,
    build_producer_performance_artifact_for_audit_run_from_output,
    ProducerPerformanceAuditRunContext, ProducerPerformanceRuntimeObservations, RuntimeCommandRun,
    RuntimeSkippedRun,
};
use lumin_audit_core::rust_analysis::RustAnalysisRunObservation;
use lumin_audit_core::topology_mermaid::{
    render_topology_mermaid_request, TopologyMermaidOptions, TopologyMermaidRenderRequest,
    TOPOLOGY_MERMAID_RENDER_REQUEST_SCHEMA_VERSION,
};

const MANIFEST_EVIDENCE_WITH_READS_SCHEMA_VERSION: &str =
    "lumin-manifest-evidence-with-artifact-reads.v1";

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ManifestEvidenceWithArtifactReads<T: Serialize> {
    schema_version: &'static str,
    evidence: T,
    artifact_reads: ManifestEvidenceArtifactReadEvents,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ManifestEvidenceArtifactReadEvents {
    schema_version: &'static str,
    root_dir: String,
    reads: Vec<ArtifactReadObservation>,
}

struct ManifestEvidenceSummaryWithReads {
    summary: ManifestEvidenceSummary,
    artifact_reads: ManifestEvidenceArtifactReadEvents,
    result_output: Option<PathBuf>,
}

struct ManifestEvidenceReadRequest {
    root: String,
    output: PathBuf,
    include_tests: bool,
    production: bool,
    excludes: Vec<String>,
    auto_excludes: Vec<String>,
    generated_artifacts_mode: GeneratedArtifactsMode,
    rust_analysis_ran: bool,
    rust_analysis_run: Option<RustAnalysisRunObservation>,
    label: String,
}

pub(super) fn run_manifest_meta(args: Vec<String>) -> Result<()> {
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

pub(super) fn run_manifest_root(args: Vec<String>) -> Result<()> {
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

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ManifestLifecycleEvidenceRefreshCliInput {
    manifest: serde_json::Value,
    lifecycle: ManifestLifecycleUpdateInput,
    evidence: ManifestLifecycleEvidenceRefreshInput,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ManifestLifecycleEvidenceRefreshInput {
    root: String,
    output: PathBuf,
    #[serde(default = "default_true")]
    include_tests: bool,
    #[serde(default)]
    production: bool,
    #[serde(default = "default_generated_artifacts_mode")]
    generated_artifacts_mode: String,
    #[serde(default)]
    excludes: Vec<String>,
    #[serde(default)]
    auto_excludes: Vec<String>,
    #[serde(default)]
    rust_analysis_ran: bool,
    #[serde(default)]
    rust_analysis_run: Option<RustAnalysisRunObservation>,
    #[serde(default = "default_true")]
    base_pipeline_planned: bool,
    #[serde(default)]
    base_pipeline_skip_reason: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ManifestLifecycleEvidenceRefreshResult {
    manifest: serde_json::Value,
    artifact_reads: ManifestEvidenceArtifactReadEvents,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ManifestRootWithEvidenceCliInput {
    generated: String,
    profile: String,
    root: String,
    output: String,
    #[serde(default)]
    commands_run: Vec<lumin_audit_core::manifest_root::ManifestCommandRun>,
    #[serde(default)]
    skipped: Vec<lumin_audit_core::manifest_root::ManifestSkippedStep>,
    #[serde(default = "default_true")]
    include_tests: bool,
    #[serde(default)]
    production: bool,
    #[serde(default = "default_generated_artifacts_mode")]
    generated_artifacts_mode: String,
    #[serde(default)]
    excludes: Vec<String>,
    #[serde(default)]
    auto_excludes: Vec<String>,
    #[serde(default)]
    rust_analysis_ran: bool,
    #[serde(default)]
    rust_analysis_run: Option<RustAnalysisRunObservation>,
    #[serde(default = "default_true")]
    base_pipeline_planned: bool,
    #[serde(default)]
    base_pipeline_skip_reason: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ManifestRootWithEvidenceResult {
    manifest: serde_json::Value,
    artifact_reads: ManifestEvidenceArtifactReadEvents,
}

fn default_true() -> bool {
    true
}

fn default_generated_artifacts_mode() -> String {
    "default".to_string()
}

struct BasePipelineEvidenceRequest {
    label: &'static str,
    root: String,
    output: PathBuf,
    include_tests: bool,
    production: bool,
    excludes: Vec<String>,
    auto_excludes: Vec<String>,
    generated_artifacts_mode: String,
    rust_analysis_ran: bool,
    rust_analysis_run: Option<RustAnalysisRunObservation>,
    planned: bool,
    skip_reason: Option<String>,
}

fn manifest_evidence_for_base_pipeline(
    request: BasePipelineEvidenceRequest,
) -> Result<(
    ManifestEvidenceUpdateFields,
    ManifestEvidenceArtifactReadEvents,
)> {
    if !request.planned {
        let reason = required_base_pipeline_skip_reason(request.skip_reason.as_deref())?;
        let unavailable = || {
            serde_json::json!({
                "status": "unavailable",
                "reason": reason,
            })
        };
        return Ok((
            ManifestEvidenceUpdateFields {
                scan_range: serde_json::json!({
                    "status": "unavailable",
                    "reason": reason,
                    "root": request.root,
                    "includeTests": request.include_tests,
                    "production": request.production,
                    "excludes": request.excludes,
                    "autoExcludes": request.auto_excludes,
                }),
                confidence: unavailable(),
                resolver_diagnostics: unavailable(),
                blind_zones: vec![serde_json::json!({
                    "area": "base-audit",
                    "severity": "scan-gap",
                    "effect": "Base audit evidence was not refreshed for this lifecycle-only run; absence and freshness claims are unavailable.",
                    "reason": reason,
                })],
                rust_analysis: unavailable(),
                generated_artifacts: unavailable(),
                framework_resource_surfaces: unavailable(),
                unused_dependencies: unavailable(),
                block_clones: unavailable(),
                sfc_evidence: unavailable(),
                living_audit: unavailable(),
            },
            ManifestEvidenceArtifactReadEvents {
                schema_version: ARTIFACT_READ_EVENTS_SCHEMA_VERSION,
                root_dir: request.output.to_string_lossy().to_string(),
                reads: Vec::new(),
            },
        ));
    }

    let generated_artifacts_mode =
        GeneratedArtifactsMode::parse(&request.generated_artifacts_mode)?;
    let summary = build_manifest_evidence_summary_with_reads(ManifestEvidenceReadRequest {
        root: request.root,
        output: request.output,
        include_tests: request.include_tests,
        production: request.production,
        excludes: request.excludes,
        auto_excludes: request.auto_excludes,
        generated_artifacts_mode,
        rust_analysis_ran: request.rust_analysis_ran,
        rust_analysis_run: request.rust_analysis_run,
        label: request.label.to_string(),
    })?;
    let evidence = serde_json::from_value::<ManifestEvidenceUpdateFields>(
        serde_json::to_value(summary.summary)
            .with_context(|| format!("{}: invalid summary shape", request.label))?,
    )
    .with_context(|| format!("{}: invalid evidence update shape", request.label))?;
    Ok((evidence, summary.artifact_reads))
}

fn required_base_pipeline_skip_reason(reason: Option<&str>) -> Result<&str> {
    reason
        .filter(|reason| !reason.trim().is_empty())
        .context("basePipelineSkipReason is required when basePipelinePlanned is false")
}

fn mark_base_evidence_not_refreshed(
    manifest: &mut serde_json::Value,
    reason: &str,
    artifacts_produced: &[String],
) -> Result<()> {
    let object = manifest
        .as_object_mut()
        .context("manifest-root-with-evidence: manifest must be an object")?;
    object.insert(
        "baseEvidence".to_string(),
        serde_json::json!({
            "status": "not-refreshed",
            "reason": reason,
            "artifactsProducedStatus": "current-lifecycle-only",
        }),
    );
    object.insert(
        "artifactsProduced".to_string(),
        serde_json::to_value(artifacts_produced)
            .context("manifest-root-with-evidence: invalid lifecycle artifact inventory")?,
    );
    Ok(())
}

pub(super) fn run_manifest_write(args: Vec<String>) -> Result<()> {
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

pub(super) fn run_manifest_closeout_write(args: Vec<String>) -> Result<()> {
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

pub(super) fn run_finalize_audit_run(args: Vec<String>) -> Result<()> {
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

pub(super) fn run_finalize_audit_run_with_companions(args: Vec<String>) -> Result<()> {
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
        blind_zones_summary: format_blind_zones_summary(&blind_zones).unwrap_or_default(),
        blind_zones,
        closeout_update: update,
    };
    write_json_result(result_output, &result)
}

pub(super) fn run_manifest_lifecycle_evidence_refresh(args: Vec<String>) -> Result<()> {
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

pub(super) fn run_manifest_root_with_evidence(args: Vec<String>) -> Result<()> {
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

pub(super) fn run_manifest_evidence_update(args: Vec<String>) -> Result<()> {
    let mut input = None;
    let mut args = args.into_iter();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--input" => input = Some(take_string(&mut args, "--input")?),
            _ => bail!("manifest-evidence-update: unknown argument '{arg}'\n{USAGE}"),
        }
    }

    let input = input.context("manifest-evidence-update: missing --input <path|->")?;
    let json = read_json_input(&input, "manifest-evidence-update")?;
    let request = serde_json::from_value::<ManifestEvidenceUpdateInput>(json)
        .context("manifest-evidence-update: invalid request shape")?;
    let update = build_manifest_evidence_update(request);
    write_stdout_json(&update)
}

pub(super) fn run_manifest_companion_update(args: Vec<String>) -> Result<()> {
    let mut input = None;
    let mut args = args.into_iter();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--input" => input = Some(take_string(&mut args, "--input")?),
            _ => bail!("manifest-companion-update: unknown argument '{arg}'\n{USAGE}"),
        }
    }

    let input = input.context("manifest-companion-update: missing --input <path|->")?;
    let json = read_json_input(&input, "manifest-companion-update")?;
    if !json.is_object() {
        bail!("manifest-companion-update: invalid request shape");
    }
    let request = serde_json::from_value::<ManifestCompanionUpdateInput>(json)
        .context("manifest-companion-update: invalid request shape")?;
    let update = build_manifest_companion_update(request)?;
    write_stdout_json(&update)
}

pub(super) fn run_manifest_artifacts_produced_update(args: Vec<String>) -> Result<()> {
    let mut output = None;
    let mut rust_analysis_block = None;
    let mut args = args.into_iter();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--output" => output = Some(take_path(&mut args, "--output")?),
            "--rust-analysis-block" => {
                rust_analysis_block = Some(take_string(&mut args, "--rust-analysis-block")?)
            }
            _ => bail!("manifest-artifacts-produced-update: unknown argument '{arg}'\n{USAGE}"),
        }
    }

    let output = output.context("manifest-artifacts-produced-update: missing --output <dir>")?;
    let rust_analysis_block =
        read_optional_json_input(rust_analysis_block, "manifest-artifacts-produced-update")?;
    let update = build_manifest_artifacts_produced_update(&output, rust_analysis_block.as_ref())?;
    write_stdout_json(&update)
}

pub(super) fn run_manifest_final_summary_update(args: Vec<String>) -> Result<()> {
    let mut output = None;
    let mut producer_performance = None;
    let mut rust_analysis_ran = false;
    let mut rust_analysis_block = None;
    let mut args = args.into_iter();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--output" => output = Some(take_path(&mut args, "--output")?),
            "--producer-performance" => {
                producer_performance = Some(take_path(&mut args, "--producer-performance")?)
            }
            "--rust-analysis-ran" => rust_analysis_ran = true,
            "--rust-analysis-block" => {
                rust_analysis_block = Some(take_string(&mut args, "--rust-analysis-block")?)
            }
            _ => bail!("manifest-final-summary-update: unknown argument '{arg}'\n{USAGE}"),
        }
    }

    let output = output.context("manifest-final-summary-update: missing --output <dir>")?;
    let producer_performance = producer_performance
        .context("manifest-final-summary-update: missing --producer-performance <path>")?;
    if rust_analysis_ran && rust_analysis_block.is_some() {
        bail!(
            "manifest-final-summary-update: use either --rust-analysis-ran or --rust-analysis-block, not both"
        );
    }
    let artifact = read_required_json(&producer_performance, "manifest-final-summary-update")?;
    let rust_analysis_block =
        read_optional_json_input(rust_analysis_block, "manifest-final-summary-update")?;
    let update = match rust_analysis_block.as_ref() {
        Some(rust_analysis) => build_manifest_final_summary_update_for_rust_analysis(
            &output,
            &artifact,
            Some(rust_analysis),
        )?,
        None => build_manifest_final_summary_update(&output, &artifact, rust_analysis_ran)?,
    };
    write_stdout_json(&update)
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct ManifestCloseoutUpdateCliInput {
    output: String,
    producer_performance_path: String,
    #[serde(default)]
    rust_analysis: Option<serde_json::Value>,
    #[serde(default)]
    companion: ManifestCloseoutCompanionInput,
}

pub(super) fn run_manifest_closeout_update(args: Vec<String>) -> Result<()> {
    let mut input = None;
    let mut args = args.into_iter();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--input" => input = Some(take_string(&mut args, "--input")?),
            _ => bail!("manifest-closeout-update: unknown argument '{arg}'\n{USAGE}"),
        }
    }

    let input = input.context("manifest-closeout-update: missing --input <path|->")?;
    let json = read_json_input(&input, "manifest-closeout-update")?;
    let request = serde_json::from_value::<ManifestCloseoutUpdateCliInput>(json)
        .context("manifest-closeout-update: invalid request shape")?;
    let producer_performance = read_required_json(
        Path::new(&request.producer_performance_path),
        "manifest-closeout-update",
    )?;
    let update = build_manifest_closeout_update(
        Path::new(&request.output),
        &producer_performance,
        request.rust_analysis.as_ref(),
        request.companion,
    )?;
    write_stdout_json(&update)
}

pub(super) fn run_manifest_core_summary(args: Vec<String>) -> Result<()> {
    let mut parsed = ManifestCoreSummaryArgs {
        include_tests: true,
        production: false,
        ..ManifestCoreSummaryArgs::default()
    };
    let mut args = args.into_iter();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--root" => parsed.root = Some(take_string(&mut args, "--root")?),
            "--triage" => parsed.triage = Some(take_path(&mut args, "--triage")?),
            "--symbols" => parsed.symbols = Some(take_path(&mut args, "--symbols")?),
            "--include-tests" => parsed.include_tests = true,
            "--no-include-tests" => parsed.include_tests = false,
            "--production" => parsed.production = true,
            "--no-production" => parsed.production = false,
            "--exclude" => parsed.excludes.push(take_string(&mut args, "--exclude")?),
            "--auto-exclude" => parsed
                .auto_excludes
                .push(take_string(&mut args, "--auto-exclude")?),
            _ => bail!("manifest-core-summary: unknown argument '{arg}'\n{USAGE}"),
        }
    }

    let root = parsed
        .root
        .context("manifest-core-summary: missing --root <repo>")?;
    let triage = read_optional_json(parsed.triage, "manifest-core-summary")?;
    let symbols = read_optional_json(parsed.symbols, "manifest-core-summary")?;
    let summary = summarize_manifest_core(
        ManifestCoreOptions {
            root,
            include_tests: parsed.include_tests,
            production: parsed.production,
            excludes: parsed.excludes,
            auto_excludes: parsed.auto_excludes,
        },
        triage.as_ref(),
        symbols.as_ref(),
    );
    write_stdout_json(&summary)
}

pub(super) fn run_manifest_evidence_summary(args: Vec<String>) -> Result<()> {
    let summary = read_manifest_evidence_summary(args, "manifest-evidence-summary")?;
    write_stdout_json(&summary)
}

pub(super) fn run_manifest_evidence_summary_with_reads(args: Vec<String>) -> Result<()> {
    let summary =
        read_manifest_evidence_summary_with_reads(args, "manifest-evidence-summary-with-reads")?;
    write_json_result(
        summary.result_output,
        &ManifestEvidenceWithArtifactReads {
            schema_version: MANIFEST_EVIDENCE_WITH_READS_SCHEMA_VERSION,
            evidence: summary.summary,
            artifact_reads: summary.artifact_reads,
        },
    )
}

pub(super) fn run_manifest_evidence_refresh(args: Vec<String>) -> Result<()> {
    let summary = read_manifest_evidence_summary(args, "manifest-evidence-refresh")?;
    let evidence = serde_json::from_value::<ManifestEvidenceUpdateFields>(
        serde_json::to_value(summary)
            .context("manifest-evidence-refresh: invalid summary shape")?,
    )
    .context("manifest-evidence-refresh: invalid evidence update shape")?;
    let update = build_manifest_evidence_update(ManifestEvidenceUpdateInput { evidence });
    write_stdout_json(&update)
}

pub(super) fn run_manifest_evidence_refresh_with_reads(args: Vec<String>) -> Result<()> {
    let summary =
        read_manifest_evidence_summary_with_reads(args, "manifest-evidence-refresh-with-reads")?;
    let evidence = serde_json::from_value::<ManifestEvidenceUpdateFields>(
        serde_json::to_value(summary.summary)
            .context("manifest-evidence-refresh-with-reads: invalid summary shape")?,
    )
    .context("manifest-evidence-refresh-with-reads: invalid evidence update shape")?;
    let update = build_manifest_evidence_update(ManifestEvidenceUpdateInput { evidence });
    write_json_result(
        summary.result_output,
        &ManifestEvidenceWithArtifactReads {
            schema_version: MANIFEST_EVIDENCE_WITH_READS_SCHEMA_VERSION,
            evidence: update,
            artifact_reads: summary.artifact_reads,
        },
    )
}

fn read_manifest_evidence_summary(
    args: Vec<String>,
    label: &str,
) -> Result<ManifestEvidenceSummary> {
    let result = read_manifest_evidence_summary_with_reads(args, label)?;
    if result.result_output.is_some() {
        bail!("{label}: --result-output is only supported by with-reads commands");
    }
    Ok(result.summary)
}

fn read_manifest_evidence_summary_with_reads(
    args: Vec<String>,
    label: &str,
) -> Result<ManifestEvidenceSummaryWithReads> {
    let mut parsed = ManifestEvidenceSummaryArgs {
        include_tests: true,
        production: false,
        generated_artifacts_mode: GeneratedArtifactsMode::Default,
        ..ManifestEvidenceSummaryArgs::default()
    };
    let mut args = args.into_iter();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--root" => parsed.root = Some(take_string(&mut args, "--root")?),
            "--output" => parsed.output = Some(take_path(&mut args, "--output")?),
            "--result-output" => {
                parsed.result_output = Some(take_path(&mut args, "--result-output")?)
            }
            "--generated-artifacts" => {
                let mode = take_string(&mut args, "--generated-artifacts")?;
                parsed.generated_artifacts_mode = GeneratedArtifactsMode::parse(&mode)?;
            }
            "--include-tests" => parsed.include_tests = true,
            "--no-include-tests" => parsed.include_tests = false,
            "--production" => parsed.production = true,
            "--no-production" => parsed.production = false,
            "--rust-analysis-ran" => parsed.rust_analysis_ran = true,
            "--rust-analysis-run-block" => {
                parsed.rust_analysis_run_block =
                    Some(take_string(&mut args, "--rust-analysis-run-block")?)
            }
            "--exclude" => parsed.excludes.push(take_string(&mut args, "--exclude")?),
            "--auto-exclude" => parsed
                .auto_excludes
                .push(take_string(&mut args, "--auto-exclude")?),
            _ => bail!("{label}: unknown argument '{arg}'\n{USAGE}"),
        }
    }

    let root = parsed
        .root
        .with_context(|| format!("{label}: missing --root <repo>"))?;
    let output = parsed
        .output
        .with_context(|| format!("{label}: missing --output <dir>"))?;
    let rust_analysis_run = read_optional_json_input(parsed.rust_analysis_run_block, label)?
        .map(serde_json::from_value::<RustAnalysisRunObservation>)
        .transpose()
        .with_context(|| format!("{label}: invalid --rust-analysis-run-block shape"))?;
    let mut summary = build_manifest_evidence_summary_with_reads(ManifestEvidenceReadRequest {
        root,
        output,
        include_tests: parsed.include_tests,
        production: parsed.production,
        excludes: parsed.excludes,
        auto_excludes: parsed.auto_excludes,
        generated_artifacts_mode: parsed.generated_artifacts_mode,
        rust_analysis_ran: parsed.rust_analysis_ran,
        rust_analysis_run,
        label: label.to_string(),
    })?;
    summary.result_output = parsed.result_output;
    Ok(summary)
}

fn build_manifest_evidence_summary_with_reads(
    request: ManifestEvidenceReadRequest,
) -> Result<ManifestEvidenceSummaryWithReads> {
    let ManifestEvidenceReadRequest {
        root,
        output,
        include_tests,
        production,
        excludes,
        auto_excludes,
        generated_artifacts_mode,
        rust_analysis_ran,
        rust_analysis_run,
        label,
    } = request;
    let label = label.as_str();
    let mut artifact_reads = Vec::new();
    let triage = artifact_value(
        read_optional_output_json_observed(&output, "triage.json", label)?,
        &mut artifact_reads,
    );
    let symbols = artifact_value(
        read_optional_output_json_observed(&output, "symbols.json", label)?,
        &mut artifact_reads,
    );
    let resolver_capabilities = artifact_value(
        read_optional_output_json_tolerant_observed(&output, "resolver-capabilities.json"),
        &mut artifact_reads,
    );
    let resolver_diagnostics = artifact_value(
        read_optional_output_json_tolerant_observed(&output, "resolver-diagnostics.json"),
        &mut artifact_reads,
    );
    let framework_resource_surfaces = artifact_value(
        read_optional_output_json_tolerant_observed(&output, "framework-resource-surfaces.json"),
        &mut artifact_reads,
    );
    let unused_deps = artifact_value(
        read_optional_output_json_tolerant_observed(&output, "unused-deps.json"),
        &mut artifact_reads,
    );
    let block_clones = artifact_value(
        read_optional_output_json_tolerant_observed(&output, "block-clones.json"),
        &mut artifact_reads,
    );
    let dead_classify = artifact_value(
        read_optional_output_json_tolerant_observed(&output, "dead-classify.json"),
        &mut artifact_reads,
    );
    let entry_surface = artifact_value(
        read_optional_output_json_tolerant_observed(&output, "entry-surface.json"),
        &mut artifact_reads,
    );
    let rust_analysis = artifact_value(
        read_optional_output_json_tolerant_observed(&output, "rust-analyzer-health.latest.json"),
        &mut artifact_reads,
    );
    let summary = summarize_manifest_evidence(
        ManifestEvidenceOptions {
            root,
            include_tests,
            production,
            excludes,
            auto_excludes,
            generated_artifacts_mode,
            rust_analysis_ran,
            rust_analysis_run,
        },
        ManifestEvidenceArtifacts {
            triage: triage.as_ref(),
            symbols: symbols.as_ref(),
            resolver_capabilities: resolver_capabilities.as_ref(),
            resolver_diagnostics: resolver_diagnostics.as_ref(),
            framework_resource_surfaces: framework_resource_surfaces.as_ref(),
            unused_deps: unused_deps.as_ref(),
            block_clones: block_clones.as_ref(),
            dead_classify: dead_classify.as_ref(),
            entry_surface: entry_surface.as_ref(),
            rust_analysis: rust_analysis.as_ref(),
        },
    )?;
    Ok(ManifestEvidenceSummaryWithReads {
        summary,
        artifact_reads: ManifestEvidenceArtifactReadEvents {
            schema_version: ARTIFACT_READ_EVENTS_SCHEMA_VERSION,
            root_dir: output.to_string_lossy().to_string(),
            reads: artifact_reads,
        },
        result_output: None,
    })
}

fn write_json_result<T: Serialize>(result_output: Option<PathBuf>, value: &T) -> Result<()> {
    if let Some(result_output) = result_output {
        write_json_file(&result_output, value)
    } else {
        write_stdout_json(value)
    }
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

    if manifest
        .pointer("/preWrite/ran")
        .and_then(serde_json::Value::as_bool)
        == Some(true)
    {
        if let Some(invocation_id) = manifest
            .pointer("/preWrite/advisoryInvocationId")
            .and_then(serde_json::Value::as_str)
        {
            add_current_output_file(
                &mut artifacts,
                output,
                &format!("any-inventory.pre.{invocation_id}.json"),
            );
            add_current_output_file(&mut artifacts, output, "pre-write-evidence.latest.json");
        }
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

fn format_blind_zones_summary(zones: &[serde_json::Value]) -> Option<String> {
    if zones.is_empty() {
        return None;
    }
    let scan_gap = severity_count(zones, "scan-gap");
    let precision_gap = severity_count(zones, "precision-gap");
    let confidence_gap = severity_count(zones, "confidence-gap");
    let mut parts = Vec::new();
    if scan_gap > 0 {
        parts.push(format!("{scan_gap} scan-gap"));
    }
    if precision_gap > 0 {
        parts.push(format!("{precision_gap} precision-gap"));
    }
    if confidence_gap > 0 {
        parts.push(format!("{confidence_gap} confidence-gap"));
    }
    let resolver_reasons = zones
        .iter()
        .find(|zone| {
            zone.get("area")
                .and_then(serde_json::Value::as_str)
                .is_some_and(|area| area == "resolver")
        })
        .and_then(|zone| zone.pointer("/details/topUnresolvedReasons"))
        .and_then(format_unresolved_reason_counts);
    Some(format!(
        "blindZones: {}{}",
        parts.join(", "),
        resolver_reasons
            .map(|reasons| format!("; resolver reasons: {reasons}"))
            .unwrap_or_default()
    ))
}

fn severity_count(zones: &[serde_json::Value], severity: &str) -> usize {
    zones
        .iter()
        .filter(|zone| {
            zone.get("severity")
                .and_then(serde_json::Value::as_str)
                .is_some_and(|value| value == severity)
        })
        .count()
}

fn format_unresolved_reason_counts(reasons: &serde_json::Value) -> Option<String> {
    let reasons = reasons.as_array()?;
    let parts = reasons
        .iter()
        .take(3)
        .filter_map(|item| {
            let reason = item.get("reason")?.as_str()?;
            let count = item.get("count")?.as_i64()?;
            Some(format!("{reason} {count}"))
        })
        .collect::<Vec<_>>();
    if parts.is_empty() {
        None
    } else {
        Some(parts.join(", "))
    }
}

fn artifact_value(
    observed: OptionalOutputJsonRead,
    artifact_reads: &mut Vec<ArtifactReadObservation>,
) -> Option<serde_json::Value> {
    if let Some(observation) = observed.observation {
        artifact_reads.push(observation);
    }
    observed.value
}

#[cfg(test)]
mod post_write_base_evidence_tests {
    use super::*;

    #[test]
    fn skipped_base_pipeline_uses_the_plan_reason_and_standard_gap_severity() -> Result<()> {
        let output = PathBuf::from("C:/tmp/lumin-pre-write-output");
        let reason =
            "pre-write-only mode uses intent-shaped cold-cache instead of full quick audit";
        let (evidence, reads) = manifest_evidence_for_base_pipeline(BasePipelineEvidenceRequest {
            label: "test",
            root: "C:/repo".to_string(),
            output: output.clone(),
            include_tests: true,
            production: false,
            excludes: Vec::new(),
            auto_excludes: Vec::new(),
            generated_artifacts_mode: "default".to_string(),
            rust_analysis_ran: false,
            rust_analysis_run: None,
            planned: false,
            skip_reason: Some(reason.to_string()),
        })?;
        assert_eq!(evidence.scan_range["status"], "unavailable");
        assert_eq!(evidence.confidence["reason"], reason);
        assert_eq!(evidence.blind_zones[0]["severity"], "scan-gap");
        assert_eq!(reads.root_dir, output.to_string_lossy());
        assert!(reads.reads.is_empty());
        Ok(())
    }

    #[test]
    fn skipped_base_pipeline_preserves_scoped_lifecycle_artifacts() -> Result<()> {
        let mut manifest = serde_json::json!({
            "artifactsProduced": ["symbols.json", "triage.json"]
        });
        let artifacts = vec!["pre-write-advisory.latest.json".to_string()];
        mark_base_evidence_not_refreshed(&mut manifest, "pre-write-only", &artifacts)?;
        assert_eq!(manifest["baseEvidence"]["status"], "not-refreshed");
        assert_eq!(manifest["baseEvidence"]["reason"], "pre-write-only");
        assert_eq!(manifest["artifactsProduced"], serde_json::json!(artifacts));
        Ok(())
    }

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
                "rustEvidencePath": "pre-write-evidence.PRE.json"
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
