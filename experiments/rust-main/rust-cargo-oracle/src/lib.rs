use anyhow::{bail, Context, Result};
use serde_json::Value;
use std::path::{Path, PathBuf};

mod artifact;
mod classify;
mod command;
mod config;
mod input_hash;
mod metadata;
mod ownership;
mod path_util;
pub mod protocol;
mod scope;
mod toolchain;
mod util;

use artifact::{build_artifact, BuildArtifactInput};
use classify::parse_cargo_jsonl;
use command::{cargo_check_args, run_cargo_check, run_cargo_metadata};
use input_hash::analysis_input_set_hash;
use metadata::selected_packages;
use toolchain::collect_toolchain;
use util::{atomic_write_json, sha256_file};

pub const SEMANTIC_HEALTH_SCHEMA_VERSION: &str = "semantic-health.v1";
pub const EVIDENCE_POLICY_VERSION: &str = "evidence-ladder.v1";
pub const ORACLE_REGISTRY_VERSION: &str = "oracle-registry.v1";
pub const DIAGNOSTIC_POLICY_VERSION: &str = "m7-cargo-diagnostic-classifier.v1";

const EVENT_STREAM_COVERAGE_ID: &str = "cov.cargo-check.cargo-event-stream";
const ABSENCE_CLEAN_COVERAGE_ID: &str = "cov.cargo-check.absence-clean";

#[derive(Debug, Clone)]
pub struct OracleOptions {
    pub root: PathBuf,
    pub output: Option<PathBuf>,
    pub cargo_bin: String,
    pub timeout_ms: u64,
    pub features: Option<String>,
    pub package_name: Option<String>,
    pub repo_root: PathBuf,
}

pub fn run_oracle(options: OracleOptions) -> Result<Value> {
    let root = canonical_existing_dir(&options.root)
        .with_context(|| format!("invalid root {}", options.root.display()))?;
    validate_package_name(options.package_name.as_deref())?;

    let registry_path = options.repo_root.join("canonical/oracle-registry.json");
    let registry_hash = sha256_file(&registry_path)
        .with_context(|| format!("failed to hash {}", registry_path.display()))?;

    let toolchain = collect_toolchain(&root, &options.cargo_bin, options.timeout_ms);

    let metadata_result = run_cargo_metadata(
        &root,
        &options.cargo_bin,
        options.timeout_ms,
        options.features.as_deref(),
    );
    let (metadata, metadata_unavailable_reason) = match metadata_result {
        Ok(metadata) => (Some(metadata), None),
        Err(error) => (None, Some(error.to_string())),
    };

    let check_output = run_cargo_check(&root, &options)?;
    let parsed = parse_cargo_jsonl(&check_output.stdout, check_output.timed_out);
    let cargo_args = cargo_check_args(options.features.as_deref(), options.package_name.as_deref());
    let selected = selected_packages(metadata.as_ref(), options.package_name.as_deref(), &root);
    let input_hash = analysis_input_set_hash(
        &root,
        metadata.as_ref(),
        &cargo_args,
        &selected,
        &registry_hash,
        options.features.as_deref(),
        options.package_name.as_deref(),
        &toolchain,
    );

    let artifact = build_artifact(BuildArtifactInput {
        root: &root,
        output: options.output.as_deref(),
        metadata: metadata.as_ref(),
        messages: &parsed.messages,
        stream_parse_status: parsed.stream_parse_status,
        invalid_json_line_count: parsed.invalid_json_line_count,
        check_output: &check_output,
        cargo_bin: &options.cargo_bin,
        cargo_args: &cargo_args,
        selected_packages: &selected,
        metadata_unavailable_reason,
        registry_hash: &registry_hash,
        input_hash: &input_hash,
        features: options.features.as_deref(),
        package_name: options.package_name.as_deref(),
        toolchain: &toolchain,
    });

    if let Some(output) = options.output {
        atomic_write_json(&output, &artifact)
            .with_context(|| format!("failed to write {}", output.display()))?;
    }

    serde_json::to_value(artifact).context("failed to serialize semantic health artifact")
}

fn validate_package_name(value: Option<&str>) -> Result<()> {
    if let Some(value) = value {
        if value.is_empty()
            || !value
                .chars()
                .all(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '-')
        {
            bail!("--package currently supports exact package names only");
        }
    }
    Ok(())
}

fn canonical_existing_dir(path: &Path) -> Result<PathBuf> {
    let path = path.canonicalize()?;
    if !path.is_dir() {
        bail!("not a directory");
    }
    Ok(path)
}
