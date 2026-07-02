use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

pub const CHECK_CANON_LIFECYCLE_REQUEST_SCHEMA_VERSION: &str =
    "lumin-check-canon-lifecycle-request.v1";
pub const CHECK_CANON_LIFECYCLE_RESULT_SCHEMA_VERSION: &str =
    "lumin-check-canon-lifecycle-result.v1";

const CHECK_CANON_SOURCES: &[&str] = &["type-ownership", "helper-registry", "topology", "naming"];

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CheckCanonLifecycleRequest {
    pub schema_version: String,
    #[serde(default)]
    pub sources_value: Option<String>,
    #[serde(default)]
    pub strict: bool,
    pub root: PathBuf,
    #[serde(alias = "outDir")]
    pub output: PathBuf,
    pub scripts_dir: PathBuf,
    pub node_executable: String,
    #[serde(default)]
    pub scan_args: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CheckCanonLifecycleResult {
    pub schema_version: &'static str,
    pub block: CheckCanonBlock,
    pub exit_code: i32,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CheckCanonBlock {
    pub requested: bool,
    pub ran: bool,
    pub strict: bool,
    pub execution_owner: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub requested_sources: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub execution_mode: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub child_invocations: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<CheckCanonSummary>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub drift_counts: Option<BTreeMap<String, u64>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub per_source: Option<BTreeMap<String, CheckCanonSourceEntry>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CheckCanonSummary {
    pub drift_count: u64,
    pub sources_requested: usize,
    pub sources_checked: usize,
    pub sources_skipped: usize,
    pub sources_failed: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CheckCanonSourceEntry {
    pub ran: bool,
    pub exit_code: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub drift_count: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub report_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub diagnostics: Option<Vec<Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CanonDriftArtifact {
    #[serde(default)]
    per_source: BTreeMap<String, CanonDriftSourceEntry>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CanonDriftSourceEntry {
    #[serde(default)]
    status: Option<String>,
    #[serde(default)]
    drift_count: Option<u64>,
    #[serde(default)]
    report_path: Option<String>,
    #[serde(default)]
    diagnostics: Option<Vec<Value>>,
}

#[derive(Debug, Clone, PartialEq)]
struct ParsedSources {
    requested_sources: Vec<String>,
    unknown: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
struct ChildResult {
    exit_code: Option<i32>,
    spawn_failure_reason: Option<String>,
}

impl ChildResult {
    fn completed(exit_code: i32) -> Self {
        Self {
            exit_code: Some(exit_code),
            spawn_failure_reason: None,
        }
    }

    fn spawn_failed(reason: String) -> Self {
        Self {
            exit_code: None,
            spawn_failure_reason: Some(reason),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
struct ChildInvocation {
    source_name: String,
}

pub fn execute_check_canon_lifecycle(
    request: CheckCanonLifecycleRequest,
) -> Result<CheckCanonLifecycleResult> {
    execute_check_canon_lifecycle_with_runner(request, |invocation, request| {
        run_check_canon_child(invocation, request)
    })
}

fn execute_check_canon_lifecycle_with_runner<F>(
    request: CheckCanonLifecycleRequest,
    mut run_child: F,
) -> Result<CheckCanonLifecycleResult>
where
    F: FnMut(&ChildInvocation, &CheckCanonLifecycleRequest) -> ChildResult,
{
    validate_request(&request)?;
    let parsed = parse_requested_sources(request.sources_value.as_deref());
    if !parsed.unknown.is_empty() {
        let unknown_text = parsed.unknown.join(", ");
        return Ok(CheckCanonLifecycleResult {
            schema_version: CHECK_CANON_LIFECYCLE_RESULT_SCHEMA_VERSION,
            block: CheckCanonBlock {
                requested: true,
                ran: false,
                strict: request.strict,
                execution_owner: "lumin-audit-core",
                requested_sources: None,
                execution_mode: None,
                child_invocations: None,
                summary: None,
                drift_counts: None,
                per_source: None,
                reason: Some(format!("unknown --sources values: {unknown_text}")),
            },
            exit_code: 1,
        });
    }

    let run = run_check_canon_children(&request, &parsed.requested_sources, &mut run_child);
    let summary = summarize_check_canon(&run.per_source, &parsed.requested_sources);
    let entries = run.per_source.values().collect::<Vec<_>>();
    let ran = !entries.is_empty() && entries.iter().any(|entry| entry.ran);
    let exit_code = strict_exit_code(request.strict, summary.sources_checked, summary.drift_count);
    Ok(CheckCanonLifecycleResult {
        schema_version: CHECK_CANON_LIFECYCLE_RESULT_SCHEMA_VERSION,
        block: CheckCanonBlock {
            requested: true,
            ran,
            strict: request.strict,
            execution_owner: "lumin-audit-core",
            requested_sources: Some(parsed.requested_sources),
            execution_mode: Some(run.execution_mode),
            child_invocations: Some(run.child_invocations),
            summary: Some(summary),
            drift_counts: Some(run.drift_counts),
            per_source: Some(run.per_source),
            reason: None,
        },
        exit_code,
    })
}

fn validate_request(request: &CheckCanonLifecycleRequest) -> Result<()> {
    if request.schema_version != CHECK_CANON_LIFECYCLE_REQUEST_SCHEMA_VERSION {
        bail!(
            "execute-check-canon: unsupported schemaVersion '{}'",
            request.schema_version
        );
    }
    validate_nonempty("root", &request.root)?;
    validate_nonempty("output", &request.output)?;
    validate_nonempty("scriptsDir", &request.scripts_dir)?;
    if request.node_executable.trim().is_empty() {
        bail!("execute-check-canon: nodeExecutable must be a non-empty string");
    }
    Ok(())
}

fn validate_nonempty(field: &str, path: &Path) -> Result<()> {
    if path.as_os_str().is_empty() {
        bail!("execute-check-canon: {field} must be provided");
    }
    Ok(())
}

fn parse_requested_sources(sources_value: Option<&str>) -> ParsedSources {
    if sources_value.is_none_or(str::is_empty) {
        return ParsedSources {
            requested_sources: CHECK_CANON_SOURCES
                .iter()
                .map(|source| (*source).to_string())
                .collect(),
            unknown: Vec::new(),
        };
    }

    let mut expanded = Vec::new();
    for source in sources_value
        .unwrap_or_default()
        .split(',')
        .map(str::trim)
        .filter(|source| !source.is_empty())
    {
        if source == "all" {
            expanded.extend(
                CHECK_CANON_SOURCES
                    .iter()
                    .map(|source| (*source).to_string()),
            );
        } else {
            expanded.push(source.to_string());
        }
    }

    let unknown = expanded
        .iter()
        .filter(|source| !CHECK_CANON_SOURCES.contains(&source.as_str()))
        .cloned()
        .collect::<Vec<_>>();
    if !unknown.is_empty() {
        return ParsedSources {
            requested_sources: Vec::new(),
            unknown,
        };
    }

    let mut requested_sources = Vec::new();
    for source in expanded {
        if !requested_sources.contains(&source) {
            requested_sources.push(source);
        }
    }
    ParsedSources {
        requested_sources,
        unknown: Vec::new(),
    }
}

struct CheckCanonChildrenRun {
    per_source: BTreeMap<String, CheckCanonSourceEntry>,
    drift_counts: BTreeMap<String, u64>,
    execution_mode: &'static str,
    child_invocations: usize,
}

fn run_check_canon_children<F>(
    request: &CheckCanonLifecycleRequest,
    requested_sources: &[String],
    run_child: &mut F,
) -> CheckCanonChildrenRun
where
    F: FnMut(&ChildInvocation, &CheckCanonLifecycleRequest) -> ChildResult,
{
    let requested_all_sources = all_sources_requested(requested_sources);
    let use_all_source_child = requested_all_sources && primary_artifacts_ready(&request.output);
    let mut per_source = BTreeMap::new();

    if use_all_source_child {
        let invocation = ChildInvocation {
            source_name: "all".to_string(),
        };
        let result = run_child(&invocation, request);
        if let Some(reason) = result.spawn_failure_reason {
            copy_failure_for_sources(&mut per_source, requested_sources, reason);
        } else {
            let fallback_exit_code = result.exit_code.unwrap_or(1);
            let canon_drift = read_canon_drift(&request.output);
            for source_name in requested_sources {
                let child_entry = canon_drift
                    .as_ref()
                    .and_then(|artifact| artifact.per_source.get(source_name));
                per_source.insert(
                    source_name.clone(),
                    copy_child_entry(child_entry, fallback_exit_code),
                );
            }
        }
        let drift_counts = drift_counts_for_sources(&per_source, requested_sources);
        return CheckCanonChildrenRun {
            per_source,
            drift_counts,
            execution_mode: "single-invocation-all",
            child_invocations: 1,
        };
    }

    for source_name in requested_sources {
        let invocation = ChildInvocation {
            source_name: source_name.clone(),
        };
        let result = run_child(&invocation, request);
        if let Some(reason) = result.spawn_failure_reason {
            per_source.insert(source_name.clone(), child_failure_entry(reason));
            continue;
        }

        let fallback_exit_code = result.exit_code.unwrap_or(1);
        let canon_drift = read_canon_drift(&request.output);
        let child_entry = canon_drift
            .as_ref()
            .and_then(|artifact| artifact.per_source.get(source_name));
        per_source.insert(
            source_name.clone(),
            copy_child_entry(child_entry, fallback_exit_code),
        );
    }
    let drift_counts = drift_counts_for_sources(&per_source, requested_sources);
    CheckCanonChildrenRun {
        per_source,
        drift_counts,
        execution_mode: if requested_all_sources {
            "per-source-artifact-fallback"
        } else {
            "per-source"
        },
        child_invocations: requested_sources.len(),
    }
}

fn run_check_canon_child(
    invocation: &ChildInvocation,
    request: &CheckCanonLifecycleRequest,
) -> ChildResult {
    let check_canon_cli = request.scripts_dir.join("check-canon.mjs");
    let mut args = vec![
        path_string(&check_canon_cli),
        "--root".to_string(),
        path_string(&request.root),
        "--output".to_string(),
        path_string(&request.output),
        "--source".to_string(),
        invocation.source_name.clone(),
    ];
    args.extend(request.scan_args.clone());

    let output = match Command::new(&request.node_executable).args(args).output() {
        Ok(output) => output,
        Err(error) => return ChildResult::spawn_failed(error.to_string()),
    };
    ChildResult::completed(output.status.code().unwrap_or(1))
}

fn all_sources_requested(requested_sources: &[String]) -> bool {
    requested_sources.len() == CHECK_CANON_SOURCES.len()
        && CHECK_CANON_SOURCES.iter().all(|source| {
            requested_sources
                .iter()
                .any(|requested| requested == source)
        })
}

fn primary_artifacts_ready(out_dir: &Path) -> bool {
    out_dir.join("symbols.json").exists() && out_dir.join("topology.json").exists()
}

fn read_canon_drift(out_dir: &Path) -> Option<CanonDriftArtifact> {
    let bytes = fs::read(out_dir.join("canon-drift.json")).ok()?;
    serde_json::from_slice::<CanonDriftArtifact>(&bytes).ok()
}

fn copy_failure_for_sources(
    per_source: &mut BTreeMap<String, CheckCanonSourceEntry>,
    source_names: &[String],
    reason: String,
) {
    for source_name in source_names {
        per_source.insert(source_name.clone(), child_failure_entry(reason.clone()));
    }
}

fn child_failure_entry(reason: String) -> CheckCanonSourceEntry {
    CheckCanonSourceEntry {
        ran: false,
        exit_code: -1,
        status: None,
        drift_count: None,
        report_path: None,
        diagnostics: None,
        reason: Some(reason),
    }
}

fn copy_child_entry(
    child_entry: Option<&CanonDriftSourceEntry>,
    fallback_exit_code: i32,
) -> CheckCanonSourceEntry {
    let status = child_entry
        .and_then(|entry| entry.status.clone())
        .unwrap_or_else(|| "unknown".to_string());
    let drift_count = child_entry.and_then(|entry| entry.drift_count).unwrap_or(0);
    let diagnostics = child_entry.and_then(|entry| {
        entry
            .diagnostics
            .as_ref()
            .filter(|diagnostics| !diagnostics.is_empty())
            .cloned()
    });
    CheckCanonSourceEntry {
        ran: true,
        exit_code: logical_exit_for_status(&status, fallback_exit_code),
        status: Some(status),
        drift_count: Some(drift_count),
        report_path: child_entry.and_then(|entry| entry.report_path.clone()),
        diagnostics,
        reason: None,
    }
}

fn logical_exit_for_status(status: &str, fallback: i32) -> i32 {
    match status {
        "clean" => 0,
        "drift" => 1,
        "parse-error" | "skipped-unrecognized-schema" | "skipped-missing-canon" => 2,
        _ => fallback,
    }
}

fn drift_counts_for_sources(
    per_source: &BTreeMap<String, CheckCanonSourceEntry>,
    requested_sources: &[String],
) -> BTreeMap<String, u64> {
    requested_sources
        .iter()
        .map(|source_name| {
            (
                source_name.clone(),
                per_source
                    .get(source_name)
                    .and_then(|entry| entry.drift_count)
                    .unwrap_or(0),
            )
        })
        .collect()
}

fn summarize_check_canon(
    per_source: &BTreeMap<String, CheckCanonSourceEntry>,
    requested_sources: &[String],
) -> CheckCanonSummary {
    let mut sources_checked = 0;
    let mut sources_skipped = 0;
    let mut sources_failed = 0;
    let mut drift_count = 0;
    for entry in per_source.values() {
        let status = entry.status.as_deref();
        if matches!(status, Some("skipped-missing-canon")) {
            sources_skipped += 1;
        }
        if matches!(status, Some("parse-error" | "skipped-unrecognized-schema")) {
            sources_failed += 1;
        }
        if matches!(status, Some("clean" | "drift")) {
            sources_checked += 1;
        }
        drift_count += entry.drift_count.unwrap_or(0);
    }
    CheckCanonSummary {
        drift_count,
        sources_requested: requested_sources.len(),
        sources_checked,
        sources_skipped,
        sources_failed,
    }
}

fn strict_exit_code(strict: bool, checked_count: usize, drift_count: u64) -> i32 {
    if !strict {
        return 0;
    }
    if checked_count == 0 {
        return 2;
    }
    if drift_count > 0 {
        return 1;
    }
    0
}

fn path_string(path: &Path) -> String {
    path.to_string_lossy().to_string()
}
