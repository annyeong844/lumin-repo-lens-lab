use anyhow::{bail, Context, Result};
use serde::Serialize;
use serde_json::Value;
use std::fs;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};

use lumin_audit_core::artifact_measurement::measure_artifact_sizes;
use lumin_audit_core::artifact_registry::{
    collect_produced_artifacts, collect_produced_artifacts_for_manifest,
};
use lumin_audit_core::artifact_summaries::{summarize_artifact, ArtifactSummaryKind};
use lumin_audit_core::blind_zones::{summarize_blind_zones, BlindZoneInput, BlindZoneSummary};
use lumin_audit_core::canon_draft_lifecycle::{
    execute_canon_draft_lifecycle, CanonDraftLifecycleRequest,
};
use lumin_audit_core::check_canon_lifecycle::{
    execute_check_canon_lifecycle, CheckCanonLifecycleRequest,
};
use lumin_audit_core::generated_artifacts::{
    summarize_generated_artifacts, GeneratedArtifactsMode, GeneratedArtifactsOptions,
};
use lumin_audit_core::lifecycle::summarize_lifecycle;
use lumin_audit_core::living_audit::summarize_living_audit;
use lumin_audit_core::manifest_core::{summarize_manifest_core, ManifestCoreOptions};
use lumin_audit_core::manifest_evidence::{
    summarize_manifest_evidence, ManifestEvidenceArtifacts, ManifestEvidenceOptions,
};
use lumin_audit_core::manifest_final::{
    build_manifest_final_summary_update, build_manifest_final_summary_update_for_rust_analysis,
};
use lumin_audit_core::manifest_meta::{build_manifest_meta, ManifestMetaInput};
use lumin_audit_core::manifest_root::{
    build_manifest_evidence_update, build_manifest_root, ManifestEvidenceUpdateInput,
    ManifestRootInput,
};
use lumin_audit_core::orchestration_events::{
    build_producer_performance_artifact, build_producer_performance_artifact_from_runtime,
    OrchestrationLedger, ProducerPerformanceRuntimeInput,
};
use lumin_audit_core::orchestration_executor::{execute_base_plan, ExecutorRequest};
use lumin_audit_core::orchestration_plan::{
    build_orchestration_plan, AuditProfile, OrchestrationPlanOptions,
};
use lumin_audit_core::orchestration_result::summarize_orchestration_result;
use lumin_audit_core::post_write_lifecycle::{
    execute_post_write_lifecycle, PostWriteLifecycleRequest,
};
use lumin_audit_core::producer_performance::summarize_producer_performance;
use lumin_audit_core::resolver_diagnostics::summarize_resolver_diagnostics;
use lumin_audit_core::rust_analysis::{
    merge_rust_analysis_run, summarize_rust_analysis_artifact, RustAnalysisRunMergeInput,
};

const USAGE: &str = "usage: lumin-audit-core artifact-registry --output <dir> [--rust-analysis-ran|--rust-analysis-block <path|->]\n       lumin-audit-core artifact-size-summary --output <dir> --input <path|->\n       lumin-audit-core rust-analysis-summary --root <repo> --artifact <path>\n       lumin-audit-core rust-analysis-run-merge --input <path|->\n       lumin-audit-core generated-artifacts-summary --root <repo> [--symbols <path>] [--generated-artifacts <default|present|prepared>] [--include-tests|--no-include-tests] [--exclude <path> ...]\n       lumin-audit-core artifact-summary --artifact-kind <framework-resource-surfaces|unused-deps|block-clones> --artifact <path>\n       lumin-audit-core resolver-diagnostics-summary [--symbols <path>] [--resolver-capabilities <path>] [--resolver-diagnostics <path>]\n       lumin-audit-core blind-zones-summary [--input <fixture.json>|--cases <cases.json>]\n       lumin-audit-core lifecycle-summary --input <path|->\n       lumin-audit-core manifest-meta --generated <iso> --profile <quick|full|ci> --root <repo> --output <dir>\n       lumin-audit-core manifest-root --input <path|->\n       lumin-audit-core manifest-evidence-update --input <path|->\n       lumin-audit-core manifest-final-summary-update --output <dir> --producer-performance <path> [--rust-analysis-ran|--rust-analysis-block <path|->]\n       lumin-audit-core manifest-core-summary --root <repo> [--triage <path>] [--symbols <path>] [--include-tests|--no-include-tests] [--production|--no-production] [--exclude <path> ...] [--auto-exclude <path> ...]\n       lumin-audit-core manifest-evidence-summary --root <repo> --output <dir> [--generated-artifacts <default|present|prepared>] [--include-tests|--no-include-tests] [--production|--no-production] [--exclude <path> ...] [--auto-exclude <path> ...]\n       lumin-audit-core orchestration-plan [--profile <quick|full|ci>] [--sarif] [--pre-write] [--post-write] [--canon-draft] [--check-canon] [--rust-analyzer]\n       lumin-audit-core execute-base-plan --input <path|->\n       lumin-audit-core execute-canon-draft --input <path|->\n       lumin-audit-core execute-check-canon --input <path|->\n       lumin-audit-core execute-post-write --input <path|->\n       lumin-audit-core orchestration-result-summary --artifact <path>\n       lumin-audit-core producer-performance-summary --artifact <path>\n       lumin-audit-core producer-performance-artifact --input <path|->\n       lumin-audit-core producer-performance-runtime-artifact --input <path|->\n       lumin-audit-core living-audit-summary --root <repo>";

pub fn run() -> Result<()> {
    let mut args = std::env::args().skip(1);
    match args.next().as_deref() {
        Some("artifact-registry") => run_artifact_registry(args.collect()),
        Some("artifact-size-summary") => run_artifact_size_summary(args.collect()),
        Some("rust-analysis-summary") => run_rust_analysis_summary(args.collect()),
        Some("rust-analysis-run-merge") => run_rust_analysis_run_merge(args.collect()),
        Some("generated-artifacts-summary") => run_generated_artifacts_summary(args.collect()),
        Some("artifact-summary") => run_artifact_summary(args.collect()),
        Some("resolver-diagnostics-summary") => run_resolver_diagnostics_summary(args.collect()),
        Some("blind-zones-summary") => run_blind_zones_summary(args.collect()),
        Some("lifecycle-summary") => run_lifecycle_summary(args.collect()),
        Some("manifest-meta") => run_manifest_meta(args.collect()),
        Some("manifest-root") => run_manifest_root(args.collect()),
        Some("manifest-evidence-update") => run_manifest_evidence_update(args.collect()),
        Some("manifest-final-summary-update") => run_manifest_final_summary_update(args.collect()),
        Some("manifest-core-summary") => run_manifest_core_summary(args.collect()),
        Some("manifest-evidence-summary") => run_manifest_evidence_summary(args.collect()),
        Some("orchestration-plan") => run_orchestration_plan(args.collect()),
        Some("execute-base-plan") => run_execute_base_plan(args.collect()),
        Some("execute-canon-draft") => run_execute_canon_draft(args.collect()),
        Some("execute-check-canon") => run_execute_check_canon(args.collect()),
        Some("execute-post-write") => run_execute_post_write(args.collect()),
        Some("orchestration-result-summary") => run_orchestration_result_summary(args.collect()),
        Some("producer-performance-summary") => run_producer_performance_summary(args.collect()),
        Some("producer-performance-artifact") => run_producer_performance_artifact(args.collect()),
        Some("producer-performance-runtime-artifact") => {
            run_producer_performance_runtime_artifact(args.collect())
        }
        Some("living-audit-summary") => run_living_audit_summary(args.collect()),
        _ => bail!(USAGE),
    }
}

fn run_artifact_registry(args: Vec<String>) -> Result<()> {
    let mut parsed = ArtifactRegistryArgs::default();
    let mut args = args.into_iter();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--output" => parsed.output = Some(take_path(&mut args, "--output")?),
            "--rust-analysis-ran" => parsed.rust_analysis_ran = true,
            "--rust-analysis-block" => {
                parsed.rust_analysis_block = Some(take_string(&mut args, "--rust-analysis-block")?)
            }
            _ => bail!("artifact-registry: unknown argument '{arg}'\n{USAGE}"),
        }
    }

    let output = parsed
        .output
        .context("artifact-registry: missing --output <dir>")?;
    if parsed.rust_analysis_ran && parsed.rust_analysis_block.is_some() {
        bail!(
            "artifact-registry: use either --rust-analysis-ran or --rust-analysis-block, not both"
        );
    }
    let rust_analysis_block =
        read_optional_json_input(parsed.rust_analysis_block, "artifact-registry")?;
    let artifacts = match rust_analysis_block.as_ref() {
        Some(rust_analysis) => {
            collect_produced_artifacts_for_manifest(&output, Some(rust_analysis))?
        }
        None => collect_produced_artifacts(&output, parsed.rust_analysis_ran)?,
    };
    write_stdout_json(&artifacts)
}

fn run_artifact_size_summary(args: Vec<String>) -> Result<()> {
    let mut output = None;
    let mut input = None;
    let mut args = args.into_iter();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--output" => output = Some(take_path(&mut args, "--output")?),
            "--input" => input = Some(take_string(&mut args, "--input")?),
            _ => bail!("artifact-size-summary: unknown argument '{arg}'\n{USAGE}"),
        }
    }

    let output = output.context("artifact-size-summary: missing --output <dir>")?;
    let input = input.context("artifact-size-summary: missing --input <path|->")?;
    let json = read_json_input(&input, "artifact-size-summary")?;
    let artifacts = serde_json::from_value::<Vec<String>>(json)
        .context("artifact-size-summary: invalid artifact list shape")?;
    let summary = measure_artifact_sizes(&output, &artifacts);
    write_stdout_json(&summary)
}

fn run_rust_analysis_summary(args: Vec<String>) -> Result<()> {
    let mut parsed = RustAnalysisSummaryArgs::default();
    let mut args = args.into_iter();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--root" => parsed.root = Some(take_path(&mut args, "--root")?),
            "--artifact" => parsed.artifact = Some(take_path(&mut args, "--artifact")?),
            _ => bail!("rust-analysis-summary: unknown argument '{arg}'\n{USAGE}"),
        }
    }

    let root = parsed
        .root
        .context("rust-analysis-summary: missing --root <repo>")?;
    let artifact = parsed
        .artifact
        .context("rust-analysis-summary: missing --artifact <path>")?;
    let artifact_text = fs::read_to_string(&artifact).with_context(|| {
        format!(
            "rust-analysis-summary: failed to read {}",
            artifact.display()
        )
    })?;
    let artifact_json = serde_json::from_str::<Value>(&artifact_text).with_context(|| {
        format!(
            "rust-analysis-summary: invalid JSON in {}",
            artifact.display()
        )
    })?;
    let summary = summarize_rust_analysis_artifact(&root, &artifact_json);
    write_stdout_json(&summary)
}

fn run_rust_analysis_run_merge(args: Vec<String>) -> Result<()> {
    let mut input = None;
    let mut args = args.into_iter();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--input" => input = Some(take_string(&mut args, "--input")?),
            _ => bail!("rust-analysis-run-merge: unknown argument '{arg}'\n{USAGE}"),
        }
    }

    let input = input.context("rust-analysis-run-merge: missing --input <path|->")?;
    let json = read_json_input(&input, "rust-analysis-run-merge")?;
    let request = serde_json::from_value::<RustAnalysisRunMergeInput>(json)
        .context("rust-analysis-run-merge: invalid request shape")?;
    let merged = merge_rust_analysis_run(request)?;
    write_stdout_json(&merged)
}

fn run_generated_artifacts_summary(args: Vec<String>) -> Result<()> {
    let mut parsed = GeneratedArtifactsSummaryArgs {
        include_tests: true,
        generated_artifacts_mode: GeneratedArtifactsMode::Default,
        ..GeneratedArtifactsSummaryArgs::default()
    };
    let mut args = args.into_iter();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--root" => parsed.root = Some(take_path(&mut args, "--root")?),
            "--symbols" => parsed.symbols = Some(take_path(&mut args, "--symbols")?),
            "--generated-artifacts" => {
                let mode = take_string(&mut args, "--generated-artifacts")?;
                parsed.generated_artifacts_mode = GeneratedArtifactsMode::parse(&mode)?;
            }
            "--include-tests" => parsed.include_tests = true,
            "--no-include-tests" => parsed.include_tests = false,
            "--exclude" => parsed.excludes.push(take_string(&mut args, "--exclude")?),
            _ => bail!("generated-artifacts-summary: unknown argument '{arg}'\n{USAGE}"),
        }
    }

    let root = parsed
        .root
        .context("generated-artifacts-summary: missing --root <repo>")?;
    let symbols_json = match parsed.symbols {
        Some(symbols_path) if symbols_path.exists() => {
            let symbols_text = fs::read_to_string(&symbols_path).with_context(|| {
                format!(
                    "generated-artifacts-summary: failed to read {}",
                    symbols_path.display()
                )
            })?;
            Some(
                serde_json::from_str::<Value>(&symbols_text).with_context(|| {
                    format!(
                        "generated-artifacts-summary: invalid JSON in {}",
                        symbols_path.display()
                    )
                })?,
            )
        }
        _ => None,
    };
    let summary = summarize_generated_artifacts(
        &root,
        symbols_json.as_ref(),
        &GeneratedArtifactsOptions {
            include_tests: parsed.include_tests,
            excludes: parsed.excludes,
            mode: parsed.generated_artifacts_mode,
        },
    );
    write_stdout_json(&summary)
}

fn run_artifact_summary(args: Vec<String>) -> Result<()> {
    let mut parsed = ArtifactSummaryArgs::default();
    let mut args = args.into_iter();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--artifact-kind" => {
                let kind = take_string(&mut args, "--artifact-kind")?;
                parsed.kind = Some(ArtifactSummaryKind::parse(&kind)?);
            }
            "--artifact" => parsed.artifact = Some(take_path(&mut args, "--artifact")?),
            _ => bail!("artifact-summary: unknown argument '{arg}'\n{USAGE}"),
        }
    }

    let kind = parsed
        .kind
        .context("artifact-summary: missing --artifact-kind <kind>")?;
    let artifact = parsed
        .artifact
        .context("artifact-summary: missing --artifact <path>")?;
    let artifact_text = fs::read_to_string(&artifact)
        .with_context(|| format!("artifact-summary: failed to read {}", artifact.display()))?;
    let artifact_json = serde_json::from_str::<Value>(&artifact_text)
        .with_context(|| format!("artifact-summary: invalid JSON in {}", artifact.display()))?;
    let summary = summarize_artifact(kind, &artifact_json);
    write_stdout_json(&summary)
}

fn run_resolver_diagnostics_summary(args: Vec<String>) -> Result<()> {
    let mut parsed = ResolverDiagnosticsSummaryArgs::default();
    let mut args = args.into_iter();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--symbols" => parsed.symbols = Some(take_path(&mut args, "--symbols")?),
            "--resolver-capabilities" => {
                parsed.resolver_capabilities =
                    Some(take_path(&mut args, "--resolver-capabilities")?);
            }
            "--resolver-diagnostics" => {
                parsed.resolver_diagnostics = Some(take_path(&mut args, "--resolver-diagnostics")?);
            }
            _ => bail!("resolver-diagnostics-summary: unknown argument '{arg}'\n{USAGE}"),
        }
    }

    let symbols = read_optional_json(parsed.symbols, "resolver-diagnostics-summary")?;
    let resolver_capabilities =
        read_optional_json(parsed.resolver_capabilities, "resolver-diagnostics-summary")?;
    let resolver_diagnostics =
        read_optional_json(parsed.resolver_diagnostics, "resolver-diagnostics-summary")?;
    let summary = summarize_resolver_diagnostics(
        symbols.as_ref(),
        resolver_capabilities.as_ref(),
        resolver_diagnostics.as_ref(),
    );
    write_stdout_json(&summary)
}

fn run_blind_zones_summary(args: Vec<String>) -> Result<()> {
    let mut parsed = BlindZonesSummaryArgs::default();
    let mut args = args.into_iter();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--input" => parsed.input = Some(take_path(&mut args, "--input")?),
            "--cases" => parsed.cases = Some(take_path(&mut args, "--cases")?),
            _ => bail!("blind-zones-summary: unknown argument '{arg}'\n{USAGE}"),
        }
    }

    match (parsed.input, parsed.cases) {
        (Some(input), None) => {
            let fixture = read_required_json(&input, "blind-zones-summary")?;
            let summary = summarize_blind_zone_fixture(&fixture);
            write_stdout_json(&summary)
        }
        (None, Some(cases_path)) => {
            let cases_json = read_required_json(&cases_path, "blind-zones-summary")?;
            let cases = cases_json
                .as_array()
                .context("blind-zones-summary: --cases fixture must be an array")?;
            let mut summaries = Vec::new();
            for case in cases {
                summaries.push(summarize_blind_zone_case(case)?);
            }
            write_stdout_json(&summaries)
        }
        (None, None) => {
            bail!("blind-zones-summary: missing --input <fixture.json> or --cases <cases.json>")
        }
        (Some(_), Some(_)) => bail!(
            "blind-zones-summary: use either --input <fixture.json> or --cases <cases.json>, not both"
        ),
    }
}

fn summarize_blind_zone_fixture(fixture: &Value) -> Vec<BlindZoneSummary> {
    summarize_blind_zones(BlindZoneInput {
        triage: fixture.get("triage"),
        symbols: fixture.get("symbols"),
        dead_classify: fixture.get("deadClassify"),
        entry_surface: fixture.get("entrySurface"),
        resolver_diagnostics: fixture.get("resolverDiagnostics"),
        rust_analysis: fixture.get("rustAnalysis"),
    })
}

fn summarize_blind_zone_case(case: &Value) -> Result<BlindZoneCaseSummary> {
    let name = case
        .get("name")
        .and_then(Value::as_str)
        .unwrap_or("<unnamed>")
        .to_string();
    let input = case
        .get("input")
        .with_context(|| format!("blind-zones-summary: case '{name}' missing input"))?;
    Ok(BlindZoneCaseSummary {
        name,
        blind_zones: summarize_blind_zone_fixture(input),
    })
}

fn run_lifecycle_summary(args: Vec<String>) -> Result<()> {
    let mut input = None;
    let mut args = args.into_iter();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--input" => input = Some(take_string(&mut args, "--input")?),
            _ => bail!("lifecycle-summary: unknown argument '{arg}'\n{USAGE}"),
        }
    }

    let input = input.context("lifecycle-summary: missing --input <path|->")?;
    let lifecycle_json = read_json_input(&input, "lifecycle-summary")?;
    let summary = summarize_lifecycle(&lifecycle_json);
    write_stdout_json(&summary)
}

fn run_manifest_meta(args: Vec<String>) -> Result<()> {
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

fn run_manifest_root(args: Vec<String>) -> Result<()> {
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

fn run_manifest_evidence_update(args: Vec<String>) -> Result<()> {
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

fn run_manifest_final_summary_update(args: Vec<String>) -> Result<()> {
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

fn run_manifest_core_summary(args: Vec<String>) -> Result<()> {
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

fn run_manifest_evidence_summary(args: Vec<String>) -> Result<()> {
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
            "--generated-artifacts" => {
                let mode = take_string(&mut args, "--generated-artifacts")?;
                parsed.generated_artifacts_mode = GeneratedArtifactsMode::parse(&mode)?;
            }
            "--include-tests" => parsed.include_tests = true,
            "--no-include-tests" => parsed.include_tests = false,
            "--production" => parsed.production = true,
            "--no-production" => parsed.production = false,
            "--exclude" => parsed.excludes.push(take_string(&mut args, "--exclude")?),
            "--auto-exclude" => parsed
                .auto_excludes
                .push(take_string(&mut args, "--auto-exclude")?),
            _ => bail!("manifest-evidence-summary: unknown argument '{arg}'\n{USAGE}"),
        }
    }

    let root = parsed
        .root
        .context("manifest-evidence-summary: missing --root <repo>")?;
    let output = parsed
        .output
        .context("manifest-evidence-summary: missing --output <dir>")?;
    let triage = read_optional_output_json(&output, "triage.json", "manifest-evidence-summary")?;
    let symbols = read_optional_output_json(&output, "symbols.json", "manifest-evidence-summary")?;
    let resolver_capabilities = read_optional_output_json(
        &output,
        "resolver-capabilities.json",
        "manifest-evidence-summary",
    )?;
    let resolver_diagnostics = read_optional_output_json(
        &output,
        "resolver-diagnostics.json",
        "manifest-evidence-summary",
    )?;
    let framework_resource_surfaces = read_optional_output_json(
        &output,
        "framework-resource-surfaces.json",
        "manifest-evidence-summary",
    )?;
    let unused_deps =
        read_optional_output_json(&output, "unused-deps.json", "manifest-evidence-summary")?;
    let block_clones =
        read_optional_output_json(&output, "block-clones.json", "manifest-evidence-summary")?;
    let rust_analysis = read_optional_output_json(
        &output,
        "rust-analyzer-health.latest.json",
        "manifest-evidence-summary",
    )?;

    let summary = summarize_manifest_evidence(
        ManifestEvidenceOptions {
            root,
            include_tests: parsed.include_tests,
            production: parsed.production,
            excludes: parsed.excludes,
            auto_excludes: parsed.auto_excludes,
            generated_artifacts_mode: parsed.generated_artifacts_mode,
        },
        ManifestEvidenceArtifacts {
            triage: triage.as_ref(),
            symbols: symbols.as_ref(),
            resolver_capabilities: resolver_capabilities.as_ref(),
            resolver_diagnostics: resolver_diagnostics.as_ref(),
            framework_resource_surfaces: framework_resource_surfaces.as_ref(),
            unused_deps: unused_deps.as_ref(),
            block_clones: block_clones.as_ref(),
            rust_analysis: rust_analysis.as_ref(),
        },
    );
    write_stdout_json(&summary)
}

fn run_producer_performance_summary(args: Vec<String>) -> Result<()> {
    let mut artifact = None;
    let mut args = args.into_iter();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--artifact" => artifact = Some(take_path(&mut args, "--artifact")?),
            _ => bail!("producer-performance-summary: unknown argument '{arg}'\n{USAGE}"),
        }
    }

    let artifact = artifact.context("producer-performance-summary: missing --artifact <path>")?;
    let artifact_json = read_required_json(&artifact, "producer-performance-summary")?;
    let summary = summarize_producer_performance(&artifact_json);
    write_stdout_json(&summary)
}

fn run_producer_performance_artifact(args: Vec<String>) -> Result<()> {
    let mut input = None;
    let mut args = args.into_iter();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--input" => input = Some(take_string(&mut args, "--input")?),
            _ => bail!("producer-performance-artifact: unknown argument '{arg}'\n{USAGE}"),
        }
    }

    let input = input.context("producer-performance-artifact: missing --input <path|->")?;
    let ledger_json = read_json_input(&input, "producer-performance-artifact")?;
    let ledger = serde_json::from_value::<OrchestrationLedger>(ledger_json)
        .context("producer-performance-artifact: invalid ledger shape")?;
    let artifact = build_producer_performance_artifact(ledger)?;
    write_stdout_json(&artifact)
}

fn run_producer_performance_runtime_artifact(args: Vec<String>) -> Result<()> {
    let mut input = None;
    let mut args = args.into_iter();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--input" => input = Some(take_string(&mut args, "--input")?),
            _ => bail!("producer-performance-runtime-artifact: unknown argument '{arg}'\n{USAGE}"),
        }
    }

    let input = input.context("producer-performance-runtime-artifact: missing --input <path|->")?;
    let input_json = read_json_input(&input, "producer-performance-runtime-artifact")?;
    let input = serde_json::from_value::<ProducerPerformanceRuntimeInput>(input_json)
        .context("producer-performance-runtime-artifact: invalid input shape")?;
    let artifact = build_producer_performance_artifact_from_runtime(input)?;
    write_stdout_json(&artifact)
}

fn run_orchestration_result_summary(args: Vec<String>) -> Result<()> {
    let mut artifact = None;
    let mut args = args.into_iter();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--artifact" => artifact = Some(take_path(&mut args, "--artifact")?),
            _ => bail!("orchestration-result-summary: unknown argument '{arg}'\n{USAGE}"),
        }
    }

    let artifact = artifact.context("orchestration-result-summary: missing --artifact <path>")?;
    let artifact_json = read_required_json(&artifact, "orchestration-result-summary")?;
    let summary = summarize_orchestration_result(&artifact_json);
    write_stdout_json(&summary)
}

fn run_execute_base_plan(args: Vec<String>) -> Result<()> {
    let mut input = None;
    let mut args = args.into_iter();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--input" => input = Some(take_string(&mut args, "--input")?),
            _ => bail!("execute-base-plan: unknown argument '{arg}'\n{USAGE}"),
        }
    }

    let input = input.context("execute-base-plan: missing --input <path|->")?;
    let json = read_json_input(&input, "execute-base-plan")?;
    let request = serde_json::from_value::<ExecutorRequest>(json)
        .context("execute-base-plan: invalid request shape")?;
    let result = execute_base_plan(request)?;
    write_stdout_json(&result)
}

fn run_execute_canon_draft(args: Vec<String>) -> Result<()> {
    let mut input = None;
    let mut args = args.into_iter();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--input" => input = Some(take_string(&mut args, "--input")?),
            _ => bail!("execute-canon-draft: unknown argument '{arg}'\n{USAGE}"),
        }
    }

    let input = input.context("execute-canon-draft: missing --input <path|->")?;
    let json = read_json_input(&input, "execute-canon-draft")?;
    let request = serde_json::from_value::<CanonDraftLifecycleRequest>(json)
        .context("execute-canon-draft: invalid request shape")?;
    let result = execute_canon_draft_lifecycle(request)?;
    write_stdout_json(&result)
}

fn run_execute_check_canon(args: Vec<String>) -> Result<()> {
    let mut input = None;
    let mut args = args.into_iter();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--input" => input = Some(take_string(&mut args, "--input")?),
            _ => bail!("execute-check-canon: unknown argument '{arg}'\n{USAGE}"),
        }
    }

    let input = input.context("execute-check-canon: missing --input <path|->")?;
    let json = read_json_input(&input, "execute-check-canon")?;
    let request = serde_json::from_value::<CheckCanonLifecycleRequest>(json)
        .context("execute-check-canon: invalid request shape")?;
    let result = execute_check_canon_lifecycle(request)?;
    write_stdout_json(&result)
}

fn run_execute_post_write(args: Vec<String>) -> Result<()> {
    let mut input = None;
    let mut args = args.into_iter();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--input" => input = Some(take_string(&mut args, "--input")?),
            _ => bail!("execute-post-write: unknown argument '{arg}'\n{USAGE}"),
        }
    }

    let input = input.context("execute-post-write: missing --input <path|->")?;
    let json = read_json_input(&input, "execute-post-write")?;
    let request = serde_json::from_value::<PostWriteLifecycleRequest>(json)
        .context("execute-post-write: invalid request shape")?;
    let result = execute_post_write_lifecycle(request)?;
    write_stdout_json(&result)
}

fn run_orchestration_plan(args: Vec<String>) -> Result<()> {
    let mut parsed = OrchestrationPlanArgs {
        profile: AuditProfile::Quick,
        ..OrchestrationPlanArgs::default()
    };
    let mut args = args.into_iter();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--profile" => {
                let profile = take_string(&mut args, "--profile")?;
                parsed.profile = AuditProfile::parse(&profile)?;
            }
            "--sarif" => parsed.sarif = true,
            "--pre-write" => parsed.pre_write = true,
            "--post-write" => parsed.post_write = true,
            "--canon-draft" => parsed.canon_draft = true,
            "--check-canon" => parsed.check_canon = true,
            "--rust-analyzer" => parsed.rust_analyzer = true,
            _ => bail!("orchestration-plan: unknown argument '{arg}'\n{USAGE}"),
        }
    }

    let plan = build_orchestration_plan(OrchestrationPlanOptions {
        profile: parsed.profile,
        sarif: parsed.sarif,
        pre_write: parsed.pre_write,
        post_write: parsed.post_write,
        canon_draft: parsed.canon_draft,
        check_canon: parsed.check_canon,
        rust_analyzer: parsed.rust_analyzer,
    });
    write_stdout_json(&plan)
}

fn run_living_audit_summary(args: Vec<String>) -> Result<()> {
    let mut root = None;
    let mut args = args.into_iter();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--root" => root = Some(take_path(&mut args, "--root")?),
            _ => bail!("living-audit-summary: unknown argument '{arg}'\n{USAGE}"),
        }
    }

    let root = root.context("living-audit-summary: missing --root <repo>")?;
    let summary = summarize_living_audit(&root);
    write_stdout_json(&summary)
}

fn read_optional_json(path: Option<PathBuf>, label: &str) -> Result<Option<Value>> {
    let Some(path) = path else {
        return Ok(None);
    };
    let text = fs::read_to_string(&path)
        .with_context(|| format!("{label}: failed to read {}", path.display()))?;
    let json = serde_json::from_str::<Value>(&text)
        .with_context(|| format!("{label}: invalid JSON in {}", path.display()))?;
    Ok(Some(json))
}

fn read_required_json(path: &Path, label: &str) -> Result<Value> {
    let text = fs::read_to_string(path)
        .with_context(|| format!("{label}: failed to read {}", path.display()))?;
    serde_json::from_str::<Value>(&text)
        .with_context(|| format!("{label}: invalid JSON in {}", path.display()))
}

fn read_json_input(input: &str, label: &str) -> Result<Value> {
    if input == "-" {
        let mut text = String::new();
        io::stdin()
            .read_to_string(&mut text)
            .with_context(|| format!("{label}: failed to read stdin"))?;
        return serde_json::from_str::<Value>(&text)
            .with_context(|| format!("{label}: invalid JSON in stdin"));
    }
    read_required_json(Path::new(input), label)
}

fn read_optional_json_input(input: Option<String>, label: &str) -> Result<Option<Value>> {
    input
        .as_deref()
        .map(|input| read_json_input(input, label))
        .transpose()
}

fn read_optional_output_json(
    output: &Path,
    artifact_name: &str,
    label: &str,
) -> Result<Option<Value>> {
    let path = output.join(artifact_name);
    if !path.exists() {
        return Ok(None);
    }
    read_optional_json(Some(path), label)
}

fn take_path(args: &mut impl Iterator<Item = String>, flag: &str) -> Result<PathBuf> {
    let Some(value) = args.next() else {
        bail!("{flag} requires a value");
    };
    if value.starts_with("--") {
        bail!("{flag} requires a value");
    }
    Ok(PathBuf::from(value))
}

fn take_string(args: &mut impl Iterator<Item = String>, flag: &str) -> Result<String> {
    let Some(value) = args.next() else {
        bail!("{flag} requires a value");
    };
    if value.starts_with("--") {
        bail!("{flag} requires a value");
    }
    Ok(value)
}

fn write_stdout_json<T: Serialize>(value: &T) -> Result<()> {
    let stdout = io::stdout();
    let mut stdout = stdout.lock();
    serde_json::to_writer(&mut stdout, value).context("failed to write audit-core JSON stdout")?;
    stdout
        .write_all(b"\n")
        .context("failed to write audit-core JSON newline")
}

#[derive(Default)]
struct ArtifactRegistryArgs {
    output: Option<PathBuf>,
    rust_analysis_ran: bool,
    rust_analysis_block: Option<String>,
}

#[derive(Default)]
struct RustAnalysisSummaryArgs {
    root: Option<PathBuf>,
    artifact: Option<PathBuf>,
}

#[derive(Default)]
struct GeneratedArtifactsSummaryArgs {
    root: Option<PathBuf>,
    symbols: Option<PathBuf>,
    include_tests: bool,
    excludes: Vec<String>,
    generated_artifacts_mode: GeneratedArtifactsMode,
}

#[derive(Default)]
struct ArtifactSummaryArgs {
    kind: Option<ArtifactSummaryKind>,
    artifact: Option<PathBuf>,
}

#[derive(Default)]
struct ResolverDiagnosticsSummaryArgs {
    symbols: Option<PathBuf>,
    resolver_capabilities: Option<PathBuf>,
    resolver_diagnostics: Option<PathBuf>,
}

#[derive(Default)]
struct BlindZonesSummaryArgs {
    input: Option<PathBuf>,
    cases: Option<PathBuf>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct BlindZoneCaseSummary {
    name: String,
    blind_zones: Vec<BlindZoneSummary>,
}

#[derive(Default)]
struct ManifestCoreSummaryArgs {
    root: Option<String>,
    triage: Option<PathBuf>,
    symbols: Option<PathBuf>,
    include_tests: bool,
    production: bool,
    excludes: Vec<String>,
    auto_excludes: Vec<String>,
}

#[derive(Default)]
struct ManifestEvidenceSummaryArgs {
    root: Option<String>,
    output: Option<PathBuf>,
    include_tests: bool,
    production: bool,
    excludes: Vec<String>,
    auto_excludes: Vec<String>,
    generated_artifacts_mode: GeneratedArtifactsMode,
}

#[derive(Default)]
struct OrchestrationPlanArgs {
    profile: AuditProfile,
    sarif: bool,
    pre_write: bool,
    post_write: bool,
    canon_draft: bool,
    check_canon: bool,
    rust_analyzer: bool,
}
