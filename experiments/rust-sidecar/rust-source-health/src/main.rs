mod analyzer;
mod locations;
mod parallel;
mod protocol;
mod signals;
mod summary;

use std::io::{self, Read};

use analyzer::analyze_files;
use anyhow::{bail, Context, Result};
use parallel::{build_pool, RuntimeConfig};
use protocol::{
    HealthRequest, HealthResponse, ParserMeta, PolicyMeta, ResponseMeta, RuntimeMeta, Thresholds,
    DEFAULT_WORKER_STACK_BYTES, PARSER_EDITION, PARSER_EDITION_POLICY, PARSER_EDITION_SOURCE,
    PARSER_KIND, PARSER_VERSION, POLICY_VERSION, SCHEMA_VERSION,
};
use summary::summarize;

const MAX_FUNCTION_LINES: usize = 80;
const MAX_IMPL_LINES: usize = 200;

fn main() -> Result<()> {
    let mut input = String::new();
    io::stdin()
        .read_to_string(&mut input)
        .context("failed to read stdin")?;
    if input.trim().is_empty() {
        bail!("stdin request JSON is required");
    }

    let request: HealthRequest =
        serde_json::from_str(&input).context("failed to parse request JSON")?;
    validate_request(&request)?;

    let thresholds = Thresholds {
        max_function_lines: MAX_FUNCTION_LINES,
        max_impl_lines: MAX_IMPL_LINES,
    };
    let runtime_config = RuntimeConfig {
        thread_count: request.runtime.thread_count,
        worker_stack_bytes: request.runtime.worker_stack_bytes,
    };
    let pool = build_pool(runtime_config)?;
    let files = pool.install(|| analyze_files(&request.files, &thresholds))?;
    let summary = summarize(&files);
    let response = HealthResponse {
        schema_version: SCHEMA_VERSION,
        meta: ResponseMeta {
            producer: "rust-source-health".to_string(),
            mode: "syntax-only".to_string(),
            parser: ParserMeta {
                kind: PARSER_KIND.to_string(),
                version: PARSER_VERSION.to_string(),
                edition_policy: PARSER_EDITION_POLICY.to_string(),
                edition: PARSER_EDITION.to_string(),
                edition_source: PARSER_EDITION_SOURCE.to_string(),
            },
            policy: PolicyMeta {
                version: POLICY_VERSION.to_string(),
                thresholds,
            },
            runtime: RuntimeMeta {
                thread_count: pool.current_num_threads(),
                worker_stack_bytes: request.runtime.worker_stack_bytes,
            },
            limits: vec![
                "syntax-only".to_string(),
                "no-type-info".to_string(),
                "no-trait-solving".to_string(),
                "no-borrow-check".to_string(),
            ],
        },
        summary,
        skipped_files: Vec::new(),
        files,
    };

    println!("{}", serde_json::to_string(&response)?);
    Ok(())
}

fn validate_request(request: &HealthRequest) -> Result<()> {
    if request.schema_version != SCHEMA_VERSION {
        bail!("unsupported schemaVersion {}", request.schema_version);
    }
    validate_root(&request.root)?;
    if request.parser.edition_policy != PARSER_EDITION_POLICY
        || request.parser.edition != PARSER_EDITION
        || request.parser.edition_source != PARSER_EDITION_SOURCE
    {
        bail!("unsupported parser edition policy");
    }
    if matches!(request.runtime.thread_count, Some(0)) {
        bail!("runtime.threadCount must be greater than zero when provided");
    }
    if request.runtime.worker_stack_bytes < DEFAULT_WORKER_STACK_BYTES {
        bail!(
            "runtime.workerStackBytes must be at least {}",
            DEFAULT_WORKER_STACK_BYTES
        );
    }
    if request.path_policy.include.is_empty() {
        bail!("pathPolicy.include must not be empty");
    }
    if request.path_policy.exclude.is_empty() {
        bail!("pathPolicy.exclude must not be empty");
    }
    for file in &request.files {
        validate_file_path(&file.path)?;
        validate_sha256(&file.sha256, &file.path)?;
    }
    Ok(())
}

fn validate_root(root: &str) -> Result<()> {
    if root.trim().is_empty() {
        bail!("root is required");
    }
    if !is_absoluteish_root(root) {
        bail!("root must be absolute");
    }
    Ok(())
}

fn is_absoluteish_root(root: &str) -> bool {
    root.starts_with('/') || root.as_bytes().get(1) == Some(&b':')
}

fn validate_file_path(path: &str) -> Result<()> {
    if path.is_empty() {
        bail!("file path is required");
    }
    if path.starts_with('/') || path.starts_with('\\') {
        bail!("file path must be root-relative: {}", path);
    }
    if path.contains('\\') {
        bail!("file path must use POSIX slash separators: {}", path);
    }
    if path.contains(':') {
        bail!(
            "file path must not contain drive prefixes or colons: {}",
            path
        );
    }
    if path
        .split('/')
        .any(|part| part.is_empty() || part == "." || part == "..")
    {
        bail!(
            "file path must not contain empty, ., or .. segments: {}",
            path
        );
    }
    Ok(())
}

fn validate_sha256(value: &str, path: &str) -> Result<()> {
    let hex = value
        .strip_prefix("sha256:")
        .ok_or_else(|| anyhow::anyhow!("invalid sha256 for {}", path))?;
    if hex.len() != 64 || !hex.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        bail!("invalid sha256 for {}", path);
    }
    Ok(())
}
