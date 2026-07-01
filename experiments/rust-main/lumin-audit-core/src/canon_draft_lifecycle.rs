use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;

pub const CANON_DRAFT_LIFECYCLE_REQUEST_SCHEMA_VERSION: &str =
    "lumin-canon-draft-lifecycle-request.v1";
pub const CANON_DRAFT_LIFECYCLE_RESULT_SCHEMA_VERSION: &str =
    "lumin-canon-draft-lifecycle-result.v1";

const CANON_DRAFT_SOURCES: &[&str] = &["type-ownership", "helper-registry", "topology", "naming"];

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CanonDraftLifecycleRequest {
    pub schema_version: String,
    #[serde(default)]
    pub sources_value: Option<String>,
    pub root: PathBuf,
    pub output: PathBuf,
    #[serde(default)]
    pub canon_output: Option<PathBuf>,
    pub scripts_dir: PathBuf,
    pub node_executable: String,
    #[serde(default)]
    pub scan_args: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CanonDraftLifecycleResult {
    pub schema_version: &'static str,
    pub block: CanonDraftBlock,
    pub exit_code: i32,
    pub force_exit_code: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CanonDraftBlock {
    pub requested: bool,
    pub ran: bool,
    pub execution_owner: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub requested_sources: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub per_source: Option<BTreeMap<String, CanonDraftSourceEntry>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub draft_paths: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CanonDraftSourceEntry {
    pub ran: bool,
    pub exit_code: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub draft_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

pub fn execute_canon_draft_lifecycle(
    request: CanonDraftLifecycleRequest,
) -> Result<CanonDraftLifecycleResult> {
    validate_request(&request)?;
    let parsed = parse_requested_sources(request.sources_value.as_deref());
    if !parsed.unknown.is_empty() {
        let unknown_text = parsed.unknown.join(", ");
        return Ok(CanonDraftLifecycleResult {
            schema_version: CANON_DRAFT_LIFECYCLE_RESULT_SCHEMA_VERSION,
            block: CanonDraftBlock {
                requested: true,
                ran: false,
                execution_owner: "lumin-audit-core",
                requested_sources: None,
                per_source: None,
                draft_paths: None,
                reason: Some(format!("unknown --sources values: {unknown_text}")),
            },
            exit_code: 1,
            force_exit_code: true,
        });
    }

    let canon_cli_path = request.scripts_dir.join("generate-canon-draft.mjs");
    let canon_output_dir = request
        .canon_output
        .clone()
        .unwrap_or_else(|| request.root.join("canonical-draft"));
    let mut per_source = BTreeMap::new();
    let mut draft_paths = Vec::new();

    for source_name in &parsed.requested_sources {
        let mut args = vec![
            path_string(&canon_cli_path),
            "--root".to_string(),
            path_string(&request.root),
            "--output".to_string(),
            path_string(&request.output),
            "--source".to_string(),
            source_name.clone(),
        ];
        args.extend(request.scan_args.clone());
        args.extend(["--canon-output".to_string(), path_string(&canon_output_dir)]);

        let entry = run_canon_draft_child(
            &request.node_executable,
            &args,
            &canon_output_dir,
            source_name,
        );
        if let Some(draft_path) = entry.draft_path.clone() {
            draft_paths.push(draft_path);
        }
        per_source.insert(source_name.clone(), entry);
    }

    let ran = per_source.values().any(|entry| entry.ran);
    let block = CanonDraftBlock {
        requested: true,
        ran,
        execution_owner: "lumin-audit-core",
        requested_sources: Some(parsed.requested_sources),
        per_source: Some(per_source),
        draft_paths: Some(draft_paths),
        reason: (!ran).then(|| "all requested sources failed".to_string()),
    };

    Ok(CanonDraftLifecycleResult {
        schema_version: CANON_DRAFT_LIFECYCLE_RESULT_SCHEMA_VERSION,
        block,
        exit_code: if ran { 0 } else { 1 },
        force_exit_code: false,
    })
}

fn validate_request(request: &CanonDraftLifecycleRequest) -> Result<()> {
    if request.schema_version != CANON_DRAFT_LIFECYCLE_REQUEST_SCHEMA_VERSION {
        bail!(
            "execute-canon-draft: unsupported schemaVersion '{}'",
            request.schema_version
        );
    }
    validate_nonempty("root", &request.root)?;
    validate_nonempty("output", &request.output)?;
    validate_nonempty("scriptsDir", &request.scripts_dir)?;
    if request.node_executable.trim().is_empty() {
        bail!("execute-canon-draft: nodeExecutable must be a non-empty string");
    }
    Ok(())
}

fn validate_nonempty(field: &str, path: &Path) -> Result<()> {
    if path.as_os_str().is_empty() {
        bail!("execute-canon-draft: {field} must be provided");
    }
    Ok(())
}

#[derive(Debug, Clone, PartialEq)]
struct ParsedSources {
    requested_sources: Vec<String>,
    unknown: Vec<String>,
}

fn parse_requested_sources(sources_value: Option<&str>) -> ParsedSources {
    if sources_value.is_none_or(str::is_empty) {
        return ParsedSources {
            requested_sources: CANON_DRAFT_SOURCES
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
                CANON_DRAFT_SOURCES
                    .iter()
                    .map(|source| (*source).to_string()),
            );
        } else {
            expanded.push(source.to_string());
        }
    }

    let unknown = expanded
        .iter()
        .filter(|source| !CANON_DRAFT_SOURCES.contains(&source.as_str()))
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

fn run_canon_draft_child(
    node_executable: &str,
    args: &[String],
    fallback_dir: &Path,
    source_name: &str,
) -> CanonDraftSourceEntry {
    let output = match Command::new(node_executable).args(args).output() {
        Ok(output) => output,
        Err(error) => {
            return CanonDraftSourceEntry {
                ran: false,
                exit_code: -1,
                draft_path: None,
                reason: Some(error.to_string()),
            };
        }
    };

    let Some(child_exit_code) = output.status.code() else {
        return CanonDraftSourceEntry {
            ran: false,
            exit_code: -1,
            draft_path: None,
            reason: Some("spawn failed (signal: unknown)".to_string()),
        };
    };
    if child_exit_code == 0 {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let draft_path = draft_path_from_stderr(&stderr, fallback_dir, source_name);
        return CanonDraftSourceEntry {
            ran: true,
            exit_code: 0,
            draft_path: Some(draft_path),
            reason: None,
        };
    }

    CanonDraftSourceEntry {
        ran: false,
        exit_code: child_exit_code,
        draft_path: None,
        reason: Some(if child_exit_code == 2 {
            "required producer artifact absent (see stderr of child process)".to_string()
        } else {
            format!("generate-canon-draft.mjs exited {child_exit_code}")
        }),
    }
}

fn draft_path_from_stderr(stderr: &str, fallback_dir: &Path, source_name: &str) -> String {
    let saved_line = stderr
        .lines()
        .find(|line| line.starts_with("[canon-draft] saved "));
    if let Some((_, path)) = saved_line.and_then(|line| line.split_once('\u{2192}')) {
        return path.trim().to_string();
    }
    path_string(&fallback_dir.join(format!("{source_name}.md")))
}

fn path_string(path: &Path) -> String {
    path.to_string_lossy().to_string()
}
