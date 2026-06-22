use std::path::PathBuf;
use std::process;
use std::time::Instant;

use anyhow::{Context, Result};
use lumin_rust_cargo_oracle::{run_oracle, OracleOptions};
use lumin_rust_common::{
    atomic_write_json, canonical_existing_dir_usage, is_usage_error, CliAction,
};
use lumin_rust_source_health::{analyze_root, RustSourceHealthOptions};
use product_artifact::{unified_artifact, PhaseTimings};

mod calibration;
mod cli;
mod oracle_targeting;
mod policy;
mod prewrite;
mod product_artifact;
mod product_files;
mod product_summary;

fn main() {
    match cli::parse_args() {
        Ok(CliAction::Run(command)) => match run_command(command) {
            Ok(result) => print_run_result(result),
            Err(error) => exit_with_error(error),
        },
        Ok(CliAction::Help) => {}
        Err(error) => exit_with_error(error),
    }
}

fn run_command(command: cli::Command) -> Result<RunResult> {
    match command {
        cli::Command::Analyze(options) => run_unified_analyzer(options),
        cli::Command::PreWrite(options) => run_pre_write(options),
    }
}

fn print_run_result(result: RunResult) {
    if let Some(output) = result.output {
        println!("[lumin-rust-analyzer] wrote {}", output.display());
    } else if let Some(stdout) = result.stdout {
        println!("{stdout}");
    }
}

fn exit_with_error(error: anyhow::Error) -> ! {
    eprintln!("{error:#}");
    process::exit(if is_usage_error(&error) { 2 } else { 1 });
}

struct RunResult {
    output: Option<PathBuf>,
    stdout: Option<String>,
}

fn run_unified_analyzer(options: cli::Options) -> Result<RunResult> {
    let analyzer_started = Instant::now();
    let root = canonical_existing_dir_usage(&options.root, "--root")?;
    let syntax_started = Instant::now();
    let syntax = analyze_root(RustSourceHealthOptions {
        root: root.clone(),
        source_commit: options.source_commit.clone(),
        thread_count: options.thread_count,
        worker_stack_bytes: options.worker_stack_bytes,
    })?;
    let syntax_ms = syntax_started.elapsed().as_millis();
    let target_paths = oracle_targeting::targeted_oracle_paths(options.semantic_mode, &syntax);
    let semantic_started = Instant::now();
    let semantic_artifact = run_oracle(OracleOptions {
        root: root.clone(),
        output: None,
        cargo_bin: options.cargo_bin.clone(),
        features: options.features.clone(),
        package_name: options.package_name.clone(),
        repo_root: options.repo_root.clone(),
        cargo_check_mode: options.semantic_mode,
        cargo_target_dir_mode: options.cargo_target_dir_mode,
        target_paths,
    })?;
    let semantic_ms = semantic_started.elapsed().as_millis();
    let timings = PhaseTimings {
        syntax_ms,
        semantic_ms,
        analyzer_ms: analyzer_started.elapsed().as_millis(),
    };
    let calibration_adjudication =
        calibration::load_adjudication(options.calibration_adjudication.as_deref())?;
    let artifact = unified_artifact(
        &options,
        &root,
        &syntax,
        &semantic_artifact,
        calibration_adjudication.as_ref(),
        timings,
    )?;

    let output = options.output.clone();
    if let Some(output) = &output {
        atomic_write_json(output, &artifact)
            .with_context(|| format!("failed to write {}", output.display()))?;
    }
    let stdout = if output.is_none() {
        Some(
            artifact
                .to_pretty_string()
                .context("failed to serialize rust analyzer artifact")?,
        )
    } else {
        None
    };

    Ok(RunResult { output, stdout })
}

fn run_pre_write(options: cli::PreWriteOptions) -> Result<RunResult> {
    let artifact = prewrite::run(&options)?;
    let output = options.output;
    if let Some(output) = &output {
        atomic_write_json(output, &artifact)
            .with_context(|| format!("failed to write {}", output.display()))?;
    }
    let stdout = if output.is_none() {
        Some(
            artifact
                .to_pretty_string()
                .context("failed to serialize rust pre-write artifact")?,
        )
    } else {
        None
    };

    Ok(RunResult { output, stdout })
}
