use anyhow::{bail, Result};

mod args;
mod artifact;
mod audit_review_pack;
mod audit_summary;
mod barrel_discipline;
mod block_clones;
mod call_graph;
mod checklist_facts;
mod dead_classify;
mod discipline;
mod entry_surface;
mod export_action_safety;
mod framework_resource_surfaces;
mod function_clones;
mod io_support;
mod lifecycle;
mod manifest;
mod module_reachability;
mod orchestration;
mod rank_fixes;
mod resolver_diagnostics_artifacts;
mod runtime_evidence;
mod sarif;
mod shape_index;
mod staleness;
mod symbol_graph;
mod topology;
mod topology_mermaid;
mod unused_deps;
mod usage;

use artifact::*;
use audit_review_pack::*;
use audit_summary::*;
use barrel_discipline::*;
use block_clones::*;
use call_graph::*;
use checklist_facts::*;
use dead_classify::*;
use discipline::*;
use entry_surface::*;
use export_action_safety::*;
use framework_resource_surfaces::*;
use function_clones::*;
use lifecycle::*;
use manifest::*;
use module_reachability::*;
use orchestration::*;
use rank_fixes::*;
use resolver_diagnostics_artifacts::*;
use runtime_evidence::*;
use sarif::*;
use shape_index::*;
use staleness::*;
use symbol_graph::*;
use topology::*;
use topology_mermaid::*;
use unused_deps::*;
use usage::USAGE;

pub fn run() -> Result<()> {
    let mut args = std::env::args().skip(1);
    match args.next().as_deref() {
        Some("artifact-registry") => run_artifact_registry(args.collect()),
        Some("artifact-size-summary") => run_artifact_size_summary(args.collect()),
        Some("artifact-read-metrics-summary") => run_artifact_read_metrics_summary(args.collect()),
        Some("rust-analysis-summary") => run_rust_analysis_summary(args.collect()),
        Some("rust-analysis-run-merge") => run_rust_analysis_run_merge(args.collect()),
        Some("generated-artifacts-summary") => run_generated_artifacts_summary(args.collect()),
        Some("artifact-summary") => run_artifact_summary(args.collect()),
        Some("audit-review-pack-render") => run_audit_review_pack_render(args.collect()),
        Some("audit-summary-render") => run_audit_summary_render(args.collect()),
        Some("barrel-discipline-artifact") => run_barrel_discipline_artifact(args.collect()),
        Some("block-clones-artifact") => run_block_clones_artifact(args.collect()),
        Some("call-graph-artifact") => run_call_graph_artifact(args.collect()),
        Some("checklist-facts-artifact") => run_checklist_facts_artifact(args.collect()),
        Some("dead-classify-artifact") => run_dead_classify_artifact(args.collect()),
        Some("discipline-artifact") => run_discipline_artifact(args.collect()),
        Some("entry-surface-artifact") => run_entry_surface_artifact(args.collect()),
        Some("export-action-safety-artifact") => run_export_action_safety_artifact(args.collect()),
        Some("framework-resource-surfaces-artifact") => {
            run_framework_resource_surfaces_artifact(args.collect())
        }
        Some("function-clones-artifact") => run_function_clones_artifact(args.collect()),
        Some("module-reachability-artifact") => run_module_reachability_artifact(args.collect()),
        Some("rank-fixes-artifact") => run_rank_fixes_artifact(args.collect()),
        Some("resolver-diagnostics-artifacts") => {
            run_resolver_diagnostics_artifacts(args.collect())
        }
        Some("runtime-evidence-artifact") => run_runtime_evidence_artifact(args.collect()),
        Some("sarif-artifact") => run_sarif_artifact(args.collect()),
        Some("shape-index-artifact") => run_shape_index_artifact(args.collect()),
        Some("staleness-artifact") => run_staleness_artifact(args.collect()),
        Some("symbol-graph-artifact") => run_symbol_graph_artifact(args.collect()),
        Some("topology-artifact") => run_topology_artifact(args.collect()),
        Some("topology-mermaid-render") => run_topology_mermaid_render(args.collect()),
        Some("unused-deps-artifact") => run_unused_deps_artifact(args.collect()),
        Some("resolver-diagnostics-summary") => run_resolver_diagnostics_summary(args.collect()),
        Some("blind-zones-summary") => run_blind_zones_summary(args.collect()),
        Some("lifecycle-summary") => run_lifecycle_summary(args.collect()),
        Some("manifest-lifecycle-update") => run_manifest_lifecycle_update(args.collect()),
        Some("lifecycle-exit-policy") => run_lifecycle_exit_policy(args.collect()),
        Some("lifecycle-request-guard") => run_lifecycle_request_guard(args.collect()),
        Some("manifest-meta") => run_manifest_meta(args.collect()),
        Some("manifest-root") => run_manifest_root(args.collect()),
        Some("manifest-root-with-evidence") => run_manifest_root_with_evidence(args.collect()),
        Some("manifest-write") => run_manifest_write(args.collect()),
        Some("manifest-closeout-write") => run_manifest_closeout_write(args.collect()),
        Some("finalize-audit-run") => run_finalize_audit_run(args.collect()),
        Some("finalize-audit-run-with-companions") => {
            run_finalize_audit_run_with_companions(args.collect())
        }
        Some("manifest-lifecycle-evidence-refresh") => {
            run_manifest_lifecycle_evidence_refresh(args.collect())
        }
        Some("manifest-evidence-update") => run_manifest_evidence_update(args.collect()),
        Some("manifest-evidence-refresh") => run_manifest_evidence_refresh(args.collect()),
        Some("manifest-evidence-refresh-with-reads") => {
            run_manifest_evidence_refresh_with_reads(args.collect())
        }
        Some("manifest-companion-update") => run_manifest_companion_update(args.collect()),
        Some("manifest-artifacts-produced-update") => {
            run_manifest_artifacts_produced_update(args.collect())
        }
        Some("manifest-final-summary-update") => run_manifest_final_summary_update(args.collect()),
        Some("manifest-closeout-update") => run_manifest_closeout_update(args.collect()),
        Some("manifest-core-summary") => run_manifest_core_summary(args.collect()),
        Some("manifest-evidence-summary") => run_manifest_evidence_summary(args.collect()),
        Some("manifest-evidence-summary-with-reads") => {
            run_manifest_evidence_summary_with_reads(args.collect())
        }
        Some("orchestration-plan") => run_orchestration_plan(args.collect()),
        Some("execute-base-plan") => run_execute_base_plan(args.collect()),
        Some("execute-base-runtime") => run_execute_base_runtime(args.collect()),
        Some("execute-canon-draft") => run_execute_canon_draft(args.collect()),
        Some("execute-check-canon") => run_execute_check_canon(args.collect()),
        Some("pre-write-route") => run_pre_write_route(args.collect()),
        Some("execute-js-pre-write") => run_execute_js_pre_write(args.collect()),
        Some("execute-rust-pre-write") => run_execute_rust_pre_write(args.collect()),
        Some("execute-post-write") => run_execute_post_write(args.collect()),
        Some("orchestration-result-summary") => run_orchestration_result_summary(args.collect()),
        Some("producer-performance-summary") => run_producer_performance_summary(args.collect()),
        Some("producer-performance-artifact") => run_producer_performance_artifact(args.collect()),
        Some("producer-performance-runtime-artifact") => {
            run_producer_performance_runtime_artifact(args.collect())
        }
        Some("producer-performance-audit-run-artifact") => {
            run_producer_performance_audit_run_artifact(args.collect())
        }
        Some("living-audit-summary") => run_living_audit_summary(args.collect()),
        _ => bail!(USAGE),
    }
}
