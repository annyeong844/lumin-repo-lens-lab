use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process;

use anyhow::{bail, Context, Result};
use lumin_rust_cargo_oracle::{run_oracle, OracleOptions};
use lumin_rust_source_health::protocol::DEFAULT_WORKER_STACK_BYTES;
use lumin_rust_source_health::wrapper::{analyze_root, RustSourceHealthOptions};
use serde_json::{json, Value};
use time::{format_description::well_known::Rfc3339, OffsetDateTime};

const SCHEMA_VERSION: &str = "rust-analyzer-health.v1";
const POLICY_VERSION: &str = "rust-unified-analyzer.v1";

fn main() {
    match parse_args().and_then(run_unified_analyzer) {
        Ok(artifact) => {
            if let Some(output) = artifact
                .get("meta")
                .and_then(|meta| meta.get("output"))
                .and_then(Value::as_str)
            {
                println!("[lumin-rust-analyzer] wrote {output}");
            } else {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&artifact)
                        .unwrap_or_else(|_| artifact.to_string())
                );
            }
        }
        Err(error) => {
            eprintln!("{error:#}");
            process::exit(if is_usage_error(&error) { 2 } else { 1 });
        }
    }
}

#[derive(Debug)]
struct Options {
    root: PathBuf,
    output: Option<PathBuf>,
    source_commit: String,
    cargo_bin: String,
    timeout_ms: u64,
    features: Option<String>,
    package_name: Option<String>,
    repo_root: PathBuf,
    thread_count: Option<usize>,
    worker_stack_bytes: usize,
}

fn parse_args() -> Result<Options> {
    let mut root: Option<PathBuf> = None;
    let mut output: Option<PathBuf> = None;
    let mut source_commit: Option<String> = None;
    let mut cargo_bin = "cargo".to_string();
    let mut timeout_ms = 60_000_u64;
    let mut features: Option<String> = None;
    let mut package_name: Option<String> = None;
    let mut repo_root: Option<PathBuf> = None;
    let mut thread_count: Option<usize> = None;
    let mut worker_stack_bytes = DEFAULT_WORKER_STACK_BYTES;

    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--root" => root = Some(value_path(&mut args, "--root")?),
            "--output" => output = Some(value_path(&mut args, "--output")?),
            "--source-commit" | "--sidecar-source-commit" => {
                source_commit = Some(value_string(&mut args, "--source-commit")?)
            }
            "--cargo-bin" => cargo_bin = value_string(&mut args, "--cargo-bin")?,
            "--timeout-ms" => {
                let value = value_string(&mut args, "--timeout-ms")?;
                timeout_ms = value
                    .parse::<u64>()
                    .with_context(|| format!("invalid --timeout-ms value: {value}"))?;
            }
            "--features" => features = Some(value_string(&mut args, "--features")?),
            "--package" => package_name = Some(value_string(&mut args, "--package")?),
            "--repo-root" => repo_root = Some(value_path(&mut args, "--repo-root")?),
            "--threads" => {
                let value = value_string(&mut args, "--threads")?;
                let parsed = value
                    .parse::<usize>()
                    .with_context(|| format!("invalid --threads value: {value}"))?;
                if parsed == 0 {
                    bail!("--threads must be greater than zero");
                }
                thread_count = Some(parsed);
            }
            "--worker-stack-bytes" => {
                let value = value_string(&mut args, "--worker-stack-bytes")?;
                worker_stack_bytes = value
                    .parse::<usize>()
                    .with_context(|| format!("invalid --worker-stack-bytes value: {value}"))?;
                if worker_stack_bytes < DEFAULT_WORKER_STACK_BYTES {
                    bail!(
                        "--worker-stack-bytes must be at least {}",
                        DEFAULT_WORKER_STACK_BYTES
                    );
                }
            }
            "--help" | "-h" => {
                print_usage();
                process::exit(0);
            }
            unknown => bail!("unknown argument: {unknown}"),
        }
    }

    let root = root.unwrap_or(env::current_dir().context("failed to read current directory")?);
    let output = output.unwrap_or_else(|| root.join("rust-analyzer-health.json"));
    let repo_root = match repo_root {
        Some(path) => path,
        None => find_repo_root(&root).unwrap_or_else(|| PathBuf::from(".")),
    };

    Ok(Options {
        root,
        output: Some(output),
        source_commit: source_commit.context("--source-commit is required")?,
        cargo_bin,
        timeout_ms,
        features,
        package_name,
        repo_root,
        thread_count,
        worker_stack_bytes,
    })
}

fn run_unified_analyzer(options: Options) -> Result<Value> {
    let root = canonical_existing_dir(&options.root)
        .with_context(|| format!("invalid root {}", options.root.display()))?;
    let syntax = analyze_root(RustSourceHealthOptions {
        root: root.clone(),
        source_commit: options.source_commit.clone(),
        thread_count: options.thread_count,
        worker_stack_bytes: options.worker_stack_bytes,
    })?;
    let semantic = run_oracle(OracleOptions {
        root: root.clone(),
        output: None,
        cargo_bin: options.cargo_bin.clone(),
        timeout_ms: options.timeout_ms,
        features: options.features.clone(),
        package_name: options.package_name.clone(),
        repo_root: options.repo_root.clone(),
    })?;
    let syntax_value = serde_json::to_value(syntax).context("failed to serialize syntax phase")?;
    let artifact = unified_artifact(&options, &root, syntax_value, semantic)?;

    if let Some(output) = &options.output {
        write_json_atomic(output, &artifact)
            .with_context(|| format!("failed to write {}", output.display()))?;
    }

    Ok(artifact)
}

fn unified_artifact(
    options: &Options,
    root: &Path,
    syntax: Value,
    semantic: Value,
) -> Result<Value> {
    let semantic_summary = &semantic["summary"];
    let syntax_summary = &syntax["summary"];
    Ok(json!({
        "schemaVersion": SCHEMA_VERSION,
        "policyVersion": POLICY_VERSION,
        "meta": {
            "producer": "lumin-rust-analyzer",
            "mode": "rust-main",
            "generated": OffsetDateTime::now_utc().format(&Rfc3339)?,
            "input": {
                "root": root.display().to_string(),
                "packageName": options.package_name,
                "features": options.features,
                "cargoBin": options.cargo_bin,
            },
            "output": options.output.as_ref().map(|path| path.display().to_string()),
        },
        "summary": {
            "files": syntax_summary["files"],
            "syntaxReviewSignals": syntax_summary["reviewSignals"],
            "syntaxMutedSignals": syntax_summary["mutedSignals"],
            "syntaxMutedSignalsByReason": syntax_summary["mutedSignalsByReason"],
            "verifiedSemanticFindings": semantic_summary["verifiedFindings"],
            "ruleBackedSemanticFindings": semantic_summary["ruleBackedFindings"],
            "candidateSemanticFindings": semantic_summary["candidateFindings"],
            "semanticClean": semantic_summary["semanticClean"],
            "cacheReuse": semantic_summary["cacheReuse"],
        },
        "phases": {
            "syntax": syntax,
            "semantic": semantic,
        },
    }))
}

fn value_string(args: &mut impl Iterator<Item = String>, name: &str) -> Result<String> {
    args.next()
        .filter(|value| !value.trim().is_empty())
        .with_context(|| format!("{name} requires a value"))
}

fn value_path(args: &mut impl Iterator<Item = String>, name: &str) -> Result<PathBuf> {
    Ok(PathBuf::from(value_string(args, name)?))
}

fn canonical_existing_dir(path: &Path) -> Result<PathBuf> {
    let path = path.canonicalize()?;
    if !path.is_dir() {
        bail!("not a directory");
    }
    Ok(path)
}

fn find_repo_root(start: &Path) -> Option<PathBuf> {
    let mut cursor = if start.is_file() {
        start.parent()?.to_path_buf()
    } else {
        start.to_path_buf()
    };
    loop {
        if cursor
            .join("canonical")
            .join("oracle-registry.json")
            .is_file()
        {
            return Some(cursor);
        }
        if !cursor.pop() {
            return None;
        }
    }
}

fn write_json_atomic(path: &Path, value: &Value) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let temp = path.with_extension("tmp");
    fs::write(&temp, format!("{}\n", serde_json::to_string_pretty(value)?))?;
    fs::rename(&temp, path)?;
    Ok(())
}

fn is_usage_error(error: &anyhow::Error) -> bool {
    let message = error.to_string();
    message.starts_with("unknown argument:")
        || message.contains(" requires a value")
        || message.starts_with("invalid --")
        || message.starts_with("--threads must be greater than zero")
        || message.starts_with("--worker-stack-bytes must be at least ")
        || message == "--source-commit is required"
        || message.starts_with("--package currently supports")
}

fn print_usage() {
    eprintln!(
        "Usage: lumin-rust-analyzer --root <path> --source-commit <sha> [--output <path>] [--cargo-bin <path>] [--timeout-ms <ms>] [--features <csv>] [--package <name>] [--repo-root <path>] [--threads <n>] [--worker-stack-bytes <bytes>]"
    );
}
