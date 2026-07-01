use anyhow::{bail, Context, Result};
use serde::Serialize;
use serde_json::Value;
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;

use lumin_audit_core::artifact_registry::collect_produced_artifacts;
use lumin_audit_core::rust_analysis::summarize_rust_analysis_artifact;

const USAGE: &str = "usage: lumin-audit-core artifact-registry --output <dir> [--rust-analysis-ran]\n       lumin-audit-core rust-analysis-summary --root <repo> --artifact <path>";

pub fn run() -> Result<()> {
    let mut args = std::env::args().skip(1);
    match args.next().as_deref() {
        Some("artifact-registry") => run_artifact_registry(args.collect()),
        Some("rust-analysis-summary") => run_rust_analysis_summary(args.collect()),
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
            _ => bail!("artifact-registry: unknown argument '{arg}'\n{USAGE}"),
        }
    }

    let output = parsed
        .output
        .context("artifact-registry: missing --output <dir>")?;
    let artifacts = collect_produced_artifacts(&output, parsed.rust_analysis_ran)?;
    write_stdout_json(&artifacts)
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

fn take_path(args: &mut impl Iterator<Item = String>, flag: &str) -> Result<PathBuf> {
    let Some(value) = args.next() else {
        bail!("{flag} requires a value");
    };
    if value.starts_with("--") {
        bail!("{flag} requires a value");
    }
    Ok(PathBuf::from(value))
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
}

#[derive(Default)]
struct RustAnalysisSummaryArgs {
    root: Option<PathBuf>,
    artifact: Option<PathBuf>,
}
