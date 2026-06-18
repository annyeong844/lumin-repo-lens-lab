use std::fs;
use std::path::{Component, Path, PathBuf};

use anyhow::{bail, Context, Result};
use sha2::{Digest, Sha256};
use time::{format_description::well_known::Rfc3339, OffsetDateTime};

use crate::protocol::{
    HealthRequest, InputMeta, ParserRequest, PathPolicy, RequestFile, RuntimeRequest, SidecarMeta,
    SkippedFile, DEFAULT_EXCLUDE, DEFAULT_INCLUDE, DEFAULT_WORKER_STACK_BYTES, PARSER_EDITION,
    PARSER_EDITION_POLICY, PARSER_EDITION_SOURCE, SCHEMA_VERSION,
};
use crate::{analyze_request, FinalMeta};

pub fn run_cli(args: Vec<String>) -> Result<()> {
    let options = WrapperOptions::parse(args)?;
    let output = options.output.clone();
    let response = analyze_root(RustSourceHealthOptions {
        root: options.root,
        source_commit: options.source_commit,
        thread_count: options.thread_count,
        worker_stack_bytes: options.worker_stack_bytes,
    })?;
    write_json_atomic(&output, &response)?;
    println!("[rust-source-health] wrote {}", output.display());
    println!(
        "[rust-source-health] files={} skipped={} signals={}",
        response.summary.files, response.summary.skipped_files, response.summary.signals
    );
    Ok(())
}

#[derive(Debug)]
pub struct RustSourceHealthOptions {
    pub root: PathBuf,
    pub source_commit: String,
    pub thread_count: Option<usize>,
    pub worker_stack_bytes: usize,
}

pub fn analyze_root(options: RustSourceHealthOptions) -> Result<crate::protocol::HealthResponse> {
    let root = absolute_existing_dir(&options.root)?;
    let (files, skipped_files) = collect_rust_files(&root)?;
    let path_policy = default_path_policy();
    let request = HealthRequest {
        schema_version: SCHEMA_VERSION,
        root: root.to_string_lossy().to_string(),
        files,
        path_policy: path_policy.clone(),
        parser: ParserRequest {
            edition_policy: PARSER_EDITION_POLICY.to_string(),
            edition: PARSER_EDITION.to_string(),
            edition_source: PARSER_EDITION_SOURCE.to_string(),
        },
        runtime: RuntimeRequest {
            thread_count: options.thread_count,
            worker_stack_bytes: options.worker_stack_bytes,
        },
    };
    let binary_sha256 = hash_file_sha256(
        &std::env::current_exe().context("failed to read current executable path")?,
    )?;
    analyze_request(
        request,
        skipped_files,
        Some(FinalMeta {
            generated: OffsetDateTime::now_utc().format(&Rfc3339)?,
            sidecar: SidecarMeta {
                source_commit: options.source_commit,
                binary_sha256,
            },
            input: InputMeta { path_policy },
        }),
    )
}

#[derive(Debug)]
struct WrapperOptions {
    root: PathBuf,
    output: PathBuf,
    source_commit: String,
    thread_count: Option<usize>,
    worker_stack_bytes: usize,
}

impl WrapperOptions {
    fn parse(args: Vec<String>) -> Result<Self> {
        let mut root = None;
        let mut output = None;
        let mut source_commit = None;
        let mut thread_count = None;
        let mut worker_stack_bytes = DEFAULT_WORKER_STACK_BYTES;

        let mut iter = args.into_iter();
        while let Some(arg) = iter.next() {
            match arg.as_str() {
                "--root" => root = Some(PathBuf::from(value(&mut iter, "--root")?)),
                "--output" => output = Some(PathBuf::from(value(&mut iter, "--output")?)),
                "--source-commit" | "--sidecar-source-commit" => {
                    source_commit = Some(value(&mut iter, "--source-commit")?)
                }
                "--threads" => {
                    let raw = value(&mut iter, "--threads")?;
                    let parsed = raw
                        .parse::<usize>()
                        .with_context(|| format!("invalid --threads value: {raw}"))?;
                    if parsed == 0 {
                        bail!("--threads must be greater than zero");
                    }
                    thread_count = Some(parsed);
                }
                "--worker-stack-bytes" => {
                    let raw = value(&mut iter, "--worker-stack-bytes")?;
                    worker_stack_bytes = raw
                        .parse::<usize>()
                        .with_context(|| format!("invalid --worker-stack-bytes value: {raw}"))?;
                    if worker_stack_bytes < DEFAULT_WORKER_STACK_BYTES {
                        bail!(
                            "--worker-stack-bytes must be at least {}",
                            DEFAULT_WORKER_STACK_BYTES
                        );
                    }
                }
                "--help" | "-h" => {
                    print_usage();
                    std::process::exit(0);
                }
                unknown => bail!("unknown argument: {unknown}"),
            }
        }

        Ok(Self {
            root: root.context("--root is required")?,
            output: output.context("--output is required")?,
            source_commit: source_commit.context("--source-commit is required")?,
            thread_count,
            worker_stack_bytes,
        })
    }
}

fn value(iter: &mut impl Iterator<Item = String>, name: &str) -> Result<String> {
    iter.next()
        .filter(|value| !value.trim().is_empty())
        .with_context(|| format!("{name} requires a value"))
}

fn print_usage() {
    eprintln!(
        "Usage: lumin-rust-source-health --root <path> --output <path> --source-commit <sha> [--threads <n>] [--worker-stack-bytes <bytes>]"
    );
}

fn default_path_policy() -> PathPolicy {
    PathPolicy {
        include: DEFAULT_INCLUDE
            .iter()
            .map(|value| value.to_string())
            .collect(),
        exclude: DEFAULT_EXCLUDE
            .iter()
            .map(|value| value.to_string())
            .collect(),
    }
}

fn absolute_existing_dir(path: &Path) -> Result<PathBuf> {
    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()?.join(path)
    };
    let metadata = fs::metadata(&absolute)
        .with_context(|| format!("rust source health root not found: {}", absolute.display()))?;
    if !metadata.is_dir() {
        bail!(
            "rust source health root is not a directory: {}",
            absolute.display()
        );
    }
    Ok(absolute)
}

fn collect_rust_files(root: &Path) -> Result<(Vec<RequestFile>, Vec<SkippedFile>)> {
    let mut files = Vec::new();
    let mut skipped = Vec::new();
    collect_rust_files_inner(root, root, &mut files, &mut skipped)?;
    files.sort_by(|left, right| left.path.cmp(&right.path));
    skipped.sort_by(|left, right| left.path.cmp(&right.path));
    Ok((files, skipped))
}

fn collect_rust_files_inner(
    root: &Path,
    dir: &Path,
    files: &mut Vec<RequestFile>,
    skipped: &mut Vec<SkippedFile>,
) -> Result<()> {
    let mut entries = fs::read_dir(dir)
        .with_context(|| format!("failed to read directory {}", dir.display()))?
        .collect::<std::result::Result<Vec<_>, _>>()?;
    entries.sort_by(|left, right| left.file_name().cmp(&right.file_name()));

    for entry in entries {
        let absolute = entry.path();
        let relative = relative_posix(root, &absolute)?;
        assert_safe_relative_path(&relative)?;
        let file_type = entry.file_type()?;
        if file_type.is_symlink() {
            continue;
        }
        if is_excluded_by_path_policy(&relative) {
            continue;
        }
        if file_type.is_dir() {
            collect_rust_files_inner(root, &absolute, files, skipped)?;
            continue;
        }
        if !file_type.is_file() || !relative.ends_with(".rs") {
            continue;
        }

        let raw = fs::read(&absolute)
            .with_context(|| format!("failed to read Rust source {}", absolute.display()))?;
        let sha256 = sha256_bytes(&raw);
        let text = match String::from_utf8(raw) {
            Ok(text) => text,
            Err(_) => {
                skipped.push(SkippedFile {
                    path: relative,
                    reason: "invalid-utf8".to_string(),
                });
                continue;
            }
        };
        files.push(RequestFile {
            path: relative,
            sha256,
            text,
        });
    }
    Ok(())
}

fn relative_posix(root: &Path, path: &Path) -> Result<String> {
    let relative = path
        .strip_prefix(root)
        .with_context(|| format!("failed to relativize {}", path.display()))?;
    let mut parts = Vec::new();
    for component in relative.components() {
        match component {
            Component::Normal(value) => parts.push(value.to_string_lossy().to_string()),
            _ => bail!("unsafe rust source health path: {}", relative.display()),
        }
    }
    Ok(parts.join("/"))
}

fn assert_safe_relative_path(path: &str) -> Result<()> {
    if path.is_empty()
        || path.starts_with('/')
        || path.starts_with('\\')
        || path.contains('\\')
        || path.contains(':')
        || path
            .split('/')
            .any(|segment| segment.is_empty() || segment == "." || segment == "..")
    {
        bail!("unsafe rust source health path: {path}");
    }
    Ok(())
}

fn is_excluded_by_path_policy(path: &str) -> bool {
    has_path_segment(path, "target") || has_path_segment(path, "vendor")
}

fn has_path_segment(path: &str, segment: &str) -> bool {
    path.split('/').any(|part| part == segment)
}

fn hash_file_sha256(path: &Path) -> Result<String> {
    let bytes = fs::read(path).with_context(|| format!("failed to hash {}", path.display()))?;
    Ok(sha256_bytes(&bytes))
}

fn sha256_bytes(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("sha256:{:x}", hasher.finalize())
}

fn write_json_atomic(path: &Path, value: &impl serde::Serialize) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let tmp = path.with_extension("tmp");
    fs::write(&tmp, format!("{}\n", serde_json::to_string_pretty(value)?))?;
    fs::rename(&tmp, path).with_context(|| format!("failed to replace {}", path.display()))?;
    Ok(())
}
