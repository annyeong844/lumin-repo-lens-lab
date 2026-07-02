use anyhow::{bail, Result};

mod args;
mod artifact;
mod io_support;
mod lifecycle;
mod manifest;
mod orchestration;
mod usage;

use artifact::*;
use lifecycle::*;
use manifest::*;
use orchestration::*;
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
        Some("resolver-diagnostics-summary") => run_resolver_diagnostics_summary(args.collect()),
        Some("blind-zones-summary") => run_blind_zones_summary(args.collect()),
        Some("lifecycle-summary") => run_lifecycle_summary(args.collect()),
        Some("manifest-lifecycle-update") => run_manifest_lifecycle_update(args.collect()),
        Some("lifecycle-exit-policy") => run_lifecycle_exit_policy(args.collect()),
        Some("lifecycle-request-guard") => run_lifecycle_request_guard(args.collect()),
        Some("manifest-meta") => run_manifest_meta(args.collect()),
        Some("manifest-root") => run_manifest_root(args.collect()),
        Some("manifest-write") => run_manifest_write(args.collect()),
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
