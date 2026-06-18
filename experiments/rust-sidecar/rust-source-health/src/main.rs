mod analyzer;
mod locations;
mod parallel;
mod protocol;
mod signals;
mod summary;
mod wrapper;

use std::io::{self, Read};
use std::process;

use analyzer::analyze_files;
use anyhow::{bail, Context, Result};
use parallel::{build_pool, RuntimeConfig};
use protocol::{
    HealthRequest, HealthResponse, InputMeta, ParserMeta, PolicyMeta, ResponseMeta, RuntimeMeta,
    SidecarMeta, SkippedFile, Thresholds, DEFAULT_EXCLUDE, DEFAULT_INCLUDE, PARSER_EDITION,
    PARSER_EDITION_POLICY, PARSER_EDITION_SOURCE, PARSER_KIND, PARSER_VERSION, POLICY_VERSION,
    SCHEMA_VERSION, SIGNAL_POLICY_ID, SIGNAL_POLICY_VERSION,
};
use sha2::{Digest, Sha256};
use summary::summarize;

const MAX_FUNCTION_LINES: usize = 80;
const MAX_IMPL_LINES: usize = 200;

pub(crate) struct FinalMeta {
    pub generated: String,
    pub sidecar: SidecarMeta,
    pub input: InputMeta,
}

fn main() {
    if let Err(error) = real_main() {
        eprintln!("{error:#}");
        process::exit(if is_usage_error(&error) { 2 } else { 1 });
    }
}

fn real_main() -> Result<()> {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    if !args.is_empty() {
        return wrapper::run_cli(args);
    }

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
    let response = analyze_request(request, Vec::new(), None)?;
    println!("{}", serde_json::to_string(&response)?);
    Ok(())
}

fn is_usage_error(error: &anyhow::Error) -> bool {
    let message = error.to_string();
    message == "stdin request JSON is required"
        || message.starts_with("failed to parse request JSON")
        || message.starts_with("unsupported schemaVersion")
        || message.starts_with("unsupported pathPolicy.")
        || message.starts_with("unsupported parser edition policy")
        || message == "root is required"
        || message == "root must be absolute"
        || message.starts_with("file path ")
        || message.starts_with("invalid sha256 for ")
        || message.starts_with("sha256/text mismatch for ")
        || message.starts_with("duplicate file path: ")
        || message.starts_with("runtime.threadCount must be greater than zero")
        || message.starts_with("runtime.workerStackBytes must be at least ")
        || message.starts_with("unknown argument:")
        || message.contains(" requires a value")
        || message.starts_with("invalid --threads value:")
        || message.starts_with("invalid --worker-stack-bytes value:")
        || message.starts_with("--threads must be greater than zero")
        || message.starts_with("--worker-stack-bytes must be at least ")
        || message == "--root is required"
        || message == "--output is required"
        || message == "--source-commit is required"
        || message.starts_with("rust source health root not found:")
        || message.starts_with("rust source health root is not a directory:")
}

pub(crate) fn analyze_request(
    request: HealthRequest,
    skipped_files: Vec<SkippedFile>,
    final_meta: Option<FinalMeta>,
) -> Result<HealthResponse> {
    let thresholds = Thresholds {
        max_function_lines: MAX_FUNCTION_LINES,
        max_impl_lines: MAX_IMPL_LINES,
    };
    let runtime_config = RuntimeConfig::try_from(request.runtime)?;
    let pool = build_pool(runtime_config)?;
    let files = pool.install(|| analyze_files(&request.files, &thresholds, &request.parser))?;
    let mut summary = summarize(&files);
    summary.skipped_files = skipped_files.len();
    let (generated, sidecar, input) = final_meta
        .map(|meta| (Some(meta.generated), Some(meta.sidecar), Some(meta.input)))
        .unwrap_or((None, None, None));
    Ok(HealthResponse {
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
                signal_policy: protocol::SignalPolicyMeta {
                    id: SIGNAL_POLICY_ID.to_string(),
                    version: SIGNAL_POLICY_VERSION.to_string(),
                },
            },
            runtime: RuntimeMeta {
                thread_count: pool.current_num_threads(),
                worker_stack_bytes: runtime_config.worker_stack_bytes,
            },
            limits: vec![
                "syntax-only".to_string(),
                "no-type-info".to_string(),
                "no-trait-solving".to_string(),
                "no-borrow-check".to_string(),
            ],
            generated,
            sidecar,
            input,
        },
        summary,
        skipped_files,
        files,
    })
}

fn validate_request(request: &HealthRequest) -> Result<()> {
    if request.schema_version != SCHEMA_VERSION {
        bail!("unsupported schemaVersion {}", request.schema_version);
    }
    validate_root(&request.root)?;
    if request.path_policy.include != DEFAULT_INCLUDE {
        bail!("unsupported pathPolicy.include");
    }
    if request.path_policy.exclude != DEFAULT_EXCLUDE {
        bail!("unsupported pathPolicy.exclude");
    }
    let mut seen_paths = std::collections::BTreeSet::<&str>::new();
    for file in &request.files {
        validate_file_path(&file.path)?;
        if !seen_paths.insert(file.path.as_str()) {
            bail!("duplicate file path: {}", file.path);
        }
        validate_sha256(&file.sha256, &file.path)?;
        validate_text_sha256(&file.text, &file.sha256, &file.path)?;
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
    root.starts_with('/')
        || (root.len() >= 3
            && root.as_bytes().get(1) == Some(&b':')
            && matches!(root.as_bytes().get(2), Some(b'/') | Some(b'\\')))
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

fn validate_text_sha256(text: &str, expected: &str, path: &str) -> Result<()> {
    let actual = format!("sha256:{:x}", Sha256::digest(text.as_bytes()));
    if actual != expected {
        bail!("sha256/text mismatch for {}", path);
    }
    Ok(())
}
