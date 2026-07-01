use anyhow::{bail, Context, Result};
use serde_json::Value;
use std::fs;

use super::args::{
    ArtifactRegistryArgs, ArtifactSummaryArgs, BlindZoneCaseSummary, BlindZonesSummaryArgs,
    GeneratedArtifactsSummaryArgs, ResolverDiagnosticsSummaryArgs, RustAnalysisSummaryArgs,
};
use super::io_support::{
    read_json_input, read_optional_json, read_optional_json_input, read_required_json, take_path,
    take_string, write_stdout_json,
};
use super::usage::USAGE;
use lumin_audit_core::artifact_measurement::measure_artifact_sizes;
use lumin_audit_core::artifact_read_metrics::{
    summarize_artifact_read_events, ArtifactReadMetricsRequest,
};
use lumin_audit_core::artifact_registry::{
    collect_produced_artifacts, collect_produced_artifacts_for_manifest,
};
use lumin_audit_core::artifact_summaries::{summarize_artifact, ArtifactSummaryKind};
use lumin_audit_core::blind_zones::{summarize_blind_zones, BlindZoneInput, BlindZoneSummary};
use lumin_audit_core::generated_artifacts::{
    summarize_generated_artifacts, GeneratedArtifactsMode, GeneratedArtifactsOptions,
};
use lumin_audit_core::resolver_diagnostics::summarize_resolver_diagnostics;
use lumin_audit_core::rust_analysis::{
    merge_rust_analysis_run, summarize_rust_analysis_artifact, RustAnalysisRunMergeInput,
};

pub(super) fn run_artifact_registry(args: Vec<String>) -> Result<()> {
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

pub(super) fn run_artifact_size_summary(args: Vec<String>) -> Result<()> {
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

pub(super) fn run_artifact_read_metrics_summary(args: Vec<String>) -> Result<()> {
    let mut input = None;
    let mut args = args.into_iter();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--input" => input = Some(take_string(&mut args, "--input")?),
            _ => bail!("artifact-read-metrics-summary: unknown argument '{arg}'\n{USAGE}"),
        }
    }

    let input = input.context("artifact-read-metrics-summary: missing --input <path|->")?;
    let input_json = read_json_input(&input, "artifact-read-metrics-summary")?;
    let request = serde_json::from_value::<ArtifactReadMetricsRequest>(input_json)
        .context("artifact-read-metrics-summary: invalid request shape")?;
    let summary = summarize_artifact_read_events(request)?;
    write_stdout_json(&summary)
}

pub(super) fn run_rust_analysis_summary(args: Vec<String>) -> Result<()> {
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

pub(super) fn run_rust_analysis_run_merge(args: Vec<String>) -> Result<()> {
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

pub(super) fn run_generated_artifacts_summary(args: Vec<String>) -> Result<()> {
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

pub(super) fn run_artifact_summary(args: Vec<String>) -> Result<()> {
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

pub(super) fn run_resolver_diagnostics_summary(args: Vec<String>) -> Result<()> {
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

pub(super) fn run_blind_zones_summary(args: Vec<String>) -> Result<()> {
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
